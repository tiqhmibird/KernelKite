# Changelog

All notable changes to kernelkite are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and the project adheres
to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Fixed

- Reject `--interval-ms 0`, which sampled in a busy loop instead of failing.
- Report a mistyped global flag as an unknown flag rather than an unknown command.


Nothing yet.

## [0.1.0] — 2026-08-31

Initial release: a working, honest local developer-loop process profiler.

### Added

- **Rust core (`kernelkite-core`)**
  - Serializable data model: `ProcessSample`, `Frame`, `NetworkSummary`,
    `CaptureMeta`, `ProfileBundle` (bundle schema **v1**).
  - `Sampler` trait with three backends:
    - `proc` — Linux `/proc` sampler reading CPU (`stat`), memory (`statm`),
      block I/O (`io`), open fds (`fd/`) and an optional host-wide network
      summary (`net/dev`). **Implemented.**
    - `fixture` — deterministic cross-platform replay of recorded timelines.
      **Implemented.**
    - `ebpf` — explicitly **not implemented**; returns `Error::Unsupported` and
      never fabricates data. Gated behind the `ebpf` cargo feature.
  - Robust `/proc/<pid>/stat` parser that handles `comm` values containing
    spaces and parentheses.
  - Pid filters: `All`, `Subtree(pid)`, `Set(pids)`.
  - Live command runner (`run_and_profile`) that spawns a child, profiles its
    process subtree on cadence, and records the exit code.
  - Capture loop (`capture`) with a safety frame bound.
  - Analysis: `summarize` (CPU seconds, peak RSS, I/O and network deltas) and
    `compare` (before/after `Delta`s with percentage change).
  - Deterministic JSON bundle I/O with schema-version validation.
  - Cross-platform compilation via `cfg` guards: on non-Linux hosts the `proc`
    backend is a compile-time shim that reports `Unsupported`.
- **CLI (`kernelkite`)**
  - Subcommands: `run`, `replay`, `summary`, `compare`, `tree`, `viewer`,
    `help`, `version`.
  - Global flags: `--interval-ms`, `--max-frames`, `--label`, `--network`,
    `--json`.
  - Dependency-light hand-rolled argument parser.
  - Human-readable and `--json` output for `summary` and `compare`.
- **TypeScript timeline viewer (`@kernelkite/viewer`)**
  - Strict, defensively-validated viewer-document parser.
  - Framework-free inline-SVG timeline renderer (per-process CPU sparklines +
    peak-memory annotations) and a standalone-HTML wrapper with no remote assets.
  - Node CLI `kernelkite-render` to turn a viewer document into SVG/HTML.
- **Fixtures & samples**
  - `fixtures/cargo-build-before.json`, `fixtures/cargo-build-after.json`.
  - Generated `samples/` bundles, comparison, viewer documents and a rendered
    timeline (reproducible via `make samples`).
- **Docs & assets**
  - `README.md` (kernel-city/kite themed).
  - `docs/PROFILE.md` — sampling methodology and bundle format reference.
  - Two hand-authored, animated, fully local SVGs:
    `docs/assets/kernel-kite.svg`, `docs/assets/devloop-skyline.svg`.
- **Tooling**
  - `Makefile` with build/test/fmt/clippy/samples/clean targets.
  - GitHub Actions CI running the Rust suite on Linux (with real `/proc` tests)
    and Windows (cfg-guard build), plus the TypeScript build and tests.

### Known limitations

- Sampling profiler: sub-interval events may be missed.
- Network is a host-wide summary; per-process attribution awaits the eBPF
  backend.
- The `ebpf` backend is a stub only.

[Unreleased]: https://github.com/kernelkite/kernelkite/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/kernelkite/kernelkite/releases/tag/v0.1.0
