//! Small hand-rolled argument parser. Kept dependency-free so the CLI has a
//! minimal, auditable footprint and deterministic behaviour.

pub const USAGE: &str = "\
kernelkite — Linux-first local developer-loop process profiler

USAGE:
    kernelkite [GLOBAL FLAGS] <COMMAND> [ARGS]

COMMANDS:
    run <bundle.json> -- <cmd> [args...]   Live-profile a command via /proc (Linux).
    replay <fixture.json> <bundle.json>    Deterministically replay a fixture.
    summary <bundle.json>                  Print derived metrics for a bundle.
    compare <before.json> <after.json>     Before/after comparison report.
    tree <bundle.json>                     Print the observed process tree.
    viewer <bundle.json> [out.json]        Emit timeline JSON for the TS viewer.
    help                                   Show this help.
    version                                Show version.

GLOBAL FLAGS:
    --interval-ms <N>   Sampling interval in milliseconds (default 100).
    --max-frames <N>    Maximum frames to capture (default 100).
    --label <TEXT>      Label stored in the bundle (default per command).
    --network           Include a host-wide network summary (Linux /proc).
    --json              Emit machine-readable JSON where supported.

NOTES:
    The /proc backend is implemented and Linux-only. On other platforms use
    `replay` with a fixture. An eBPF backend is planned but NOT implemented;
    kernelkite never fabricates eBPF data.
";

/// Parsed command-line invocation.
#[derive(Debug, Clone)]
pub struct Cli {
    pub interval_ms: u64,
    pub max_frames: usize,
    pub label: Option<String>,
    pub network: bool,
    pub json: bool,
    pub command: Command,
}

/// The selected subcommand and its positional arguments.
#[derive(Debug, Clone)]
pub enum Command {
    Help,
    Version,
    Run { out: String, command: Vec<String> },
    Replay { fixture: String, out: String },
    Summary { bundle: String },
    Compare { before: String, after: String },
    Tree { bundle: String },
    Viewer { bundle: String, out: Option<String> },
}

impl Cli {
    /// Parse an iterator of arguments (already skipping argv[0]).
    pub fn parse<I: IntoIterator<Item = String>>(args: I) -> Result<Cli, String> {
        let mut interval_ms = 100u64;
        let mut max_frames = 100usize;
        let mut label = None;
        let mut network = false;
        let mut json = false;

        let mut it = args.into_iter().peekable();
        let mut subcommand: Option<String> = None;

        // Consume global flags until we hit the subcommand token.
        while let Some(tok) = it.peek() {
            match tok.as_str() {
                "--interval-ms" => {
                    it.next();
                    interval_ms = parse_num(it.next(), "--interval-ms")?;
                }
                "--max-frames" => {
                    it.next();
                    max_frames = parse_num(it.next(), "--max-frames")?;
                }
                "--label" => {
                    it.next();
                    label = Some(
                        it.next()
                            .ok_or_else(|| "--label requires a value".to_string())?,
                    );
                }
                "--network" => {
                    it.next();
                    network = true;
                }
                "--json" => {
                    it.next();
                    json = true;
                }
                "-h" | "--help" => {
                    return Ok(Cli::simple(Command::Help));
                }
                "-V" | "--version" => {
                    return Ok(Cli::simple(Command::Version));
                }
                other if other.starts_with('-') && other != "-" => {
                    return Err(format!(
                        "unknown flag {other:?}; global flags go before the subcommand"
                    ));
                }
                _ => {
                    subcommand = it.next();
                    break;
                }
            }
        }

        let sub = subcommand.ok_or_else(|| "no command given".to_string())?;
        let rest: Vec<String> = it.collect();
        let command = parse_command(&sub, rest)?;

        if interval_ms == 0 {
            return Err(
                "--interval-ms must be greater than zero; a zero interval samples in a busy loop"
                    .to_string(),
            );
        }

        if max_frames == 0 {
            return Err(
                "--max-frames must be greater than zero; an empty bundle carries no frames"
                    .to_string(),
            );
        }

        Ok(Cli {
            interval_ms,
            max_frames,
            label,
            network,
            json,
            command,
        })
    }

    fn simple(command: Command) -> Cli {
        Cli {
            interval_ms: 100,
            max_frames: 100,
            label: None,
            network: false,
            json: false,
            command,
        }
    }
}

fn parse_num<T: std::str::FromStr>(v: Option<String>, flag: &str) -> Result<T, String> {
    v.ok_or_else(|| format!("{flag} requires a value"))?
        .parse()
        .map_err(|_| format!("{flag} expects a number"))
}

fn parse_command(sub: &str, rest: Vec<String>) -> Result<Command, String> {
    match sub {
        "help" => Ok(Command::Help),
        "version" => Ok(Command::Version),
        "run" => {
            // Layout: <bundle.json> -- <cmd> [args...]
            let mut parts = rest.into_iter();
            let out = parts
                .next()
                .ok_or_else(|| "run: missing output bundle path".to_string())?;
            let sep = parts.next();
            if sep.as_deref() != Some("--") {
                return Err("run: expected `--` before the command to profile".into());
            }
            let command: Vec<String> = parts.collect();
            if command.is_empty() {
                return Err("run: no command specified after `--`".into());
            }
            Ok(Command::Run { out, command })
        }
        "replay" => {
            let mut p = rest.into_iter();
            let fixture = p
                .next()
                .ok_or_else(|| "replay: missing fixture path".to_string())?;
            let out = p
                .next()
                .ok_or_else(|| "replay: missing output bundle path".to_string())?;
            Ok(Command::Replay { fixture, out })
        }
        "summary" => {
            let bundle = rest
                .into_iter()
                .next()
                .ok_or_else(|| "summary: missing bundle path".to_string())?;
            Ok(Command::Summary { bundle })
        }
        "compare" => {
            let mut p = rest.into_iter();
            let before = p
                .next()
                .ok_or_else(|| "compare: missing before bundle".to_string())?;
            let after = p
                .next()
                .ok_or_else(|| "compare: missing after bundle".to_string())?;
            Ok(Command::Compare { before, after })
        }
        "tree" => {
            let bundle = rest
                .into_iter()
                .next()
                .ok_or_else(|| "tree: missing bundle path".to_string())?;
            Ok(Command::Tree { bundle })
        }
        "viewer" => {
            let mut p = rest.into_iter();
            let bundle = p
                .next()
                .ok_or_else(|| "viewer: missing bundle path".to_string())?;
            let out = p.next();
            Ok(Command::Viewer { bundle, out })
        }
        other => Err(format!("unknown command `{other}`")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_zero_interval_is_rejected() {
        let err = Cli::parse(["--interval-ms", "0", "version"].map(String::from)).unwrap_err();
        assert!(
            err.contains("--interval-ms must be greater than zero"),
            "{err}"
        );
        assert!(Cli::parse(["--interval-ms", "1", "version"].map(String::from)).is_ok());
    }

    #[test]
    fn a_mistyped_global_flag_is_reported_as_a_flag() {
        let err = Cli::parse(["--netwrok", "version"].map(String::from)).unwrap_err();
        assert!(err.contains("unknown flag"), "{err}");
        assert!(err.contains("--netwrok"), "{err}");
        assert!(Cli::parse(["version"].map(String::from)).is_ok());
    }

    #[test]
    fn a_zero_frame_budget_is_rejected() {
        let err = Cli::parse(["--max-frames", "0", "version"].map(String::from)).unwrap_err();
        assert!(
            err.contains("--max-frames must be greater than zero"),
            "{err}"
        );
        assert!(Cli::parse(["--max-frames", "1", "version"].map(String::from)).is_ok());
    }
}
