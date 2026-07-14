# Nova — Project Review

A snapshot of the Nova codebase: what it is, how it's built, how healthy it is, and where the rough edges are. Findings are drawn from a full read of the workspace (`crates/*`, `cli/`, `tests/`, `examples/`) and a clean `cargo check` / `cargo test --no-run`.

## What Nova is

Nova is a **YAML-driven workflow runtime** with an embedded template engine. A workflow is a *manifest*: a named routine of ordered *steps*. Each step either runs shell (`shell:`), renders a [minijinja](https://docs.rs/minijinja) template for its side effects (`run:`), or calls another routine (`call:`). Manifests declare typed, schema-validated inputs (`args`), variables (`vars`), environment bindings (`env`), inline `templates`, and `on:` triggers. Execution streams an event pipeline into a live terminal board and a nested diagnostics tree. In short: a small, self-contained orchestration engine where the shell, templates, and reusable subroutines are all first-class.

## Architecture

The root `nova` crate (`src/lib.rs`) is a thin facade that re-exports `nova-core` plus feature-gated capability crates. `default = ["macros"]`; everything else is opt-in via Cargo features (`ai`, `fs`, `codec`, `http`, `schema`, `reflect`, `json`).

Dependency layering is clean and acyclic:

- **reflect** (leaf) — a bespoke reflection/type system (`Type`, `Value`, `Map`, derive macros) with optional serde / minijinja / json bridges.
- **schema** → reflect — JSON-Schema-style validators (`string`, `integer`, `object`, `oneof`, …) implementing a `Validate` trait against `reflect::Value`.
- **core** → reflect + schema — the runtime engine (below); everything else depends on it.
- **macros** (proc-macro) — the `call!/set!/get!/has!/del!/args!` scope macros and `info!/warn!/error!` diagnostic builders.
- **ai, codec, fs, http** — capability plugins that extend `nova::Builder` via extension traits (`.fs()`, `.json()`, `.http()`), registering `Reflect` objects (`http.get`, `fs.read`, `json.encode`, …) callable from templates.

**Core model** (`crates/core/src`):
- **Scope** (`state/scope.rs`) — an `Arc`-wrapped node with a parent chain, a `symbols` map (name→Ulid), a shared `Arena` of `Object`s, `args`/`kargs`, a minijinja `Environment`, and an event `Sender`. Scope *is* a minijinja object, so `{{ foo }}` resolves against live state and functions/routines are invoked directly from templates.
- **Object** = `Value | Func | Routine`. **Routine** (`object/routine.rs`) validates args against its schema, forks a child scope, then iterates steps emitting `step::start`/`step::end` events with `Ok`/`Error`/`Skipped` status and timing.
- **Diagnostics / events** — steps swallow errors into `scope.error(...)` diagnostics rather than aborting; a node's severity is the max of itself and its children. All activity flows as `Event`s to registered `Observer`s on a listener thread, drained at shutdown.

## User-facing surface

The CLI (package `nova-cli`, binary **`nova`**) is a `clap` wrapper exposing one command:

```
nova run <files...>   # files are paths or globs — quote globs so the shell doesn't expand them
```

`run` (`cli/src/cmd/run.rs`) globs and deserializes each `.yml` into a `Manifest`, selects manifests whose `on` includes a `run` trigger as entry points, sorts them by descending priority, registers all as routines with `.fs().json().yaml().http()` enabled, and executes while rendering a live board plus a final diagnostics dump.

Manifest top-level keys: `name`, `on`, `args`, `vars`, `env`, `templates`, `steps`. Triggers: `run`, `run(<priority>)`, `call`. `run` manifests are the entry graph; `call` manifests are reusable subroutines; numeric priority sequences independent `run` roots across files. Default template helpers: `info()/warn()/error()`, `fs.read/write`, `json.encode/decode`, `yaml.encode/decode`, `http.get`, plus minijinja filters (`| length`) and loop vars (`loop.index`).

The TUI (`cli/src/widgets/`) is custom-built on `ratatui`, drawn inline to scrollback (no alternate screen) and **TTY-aware** — ANSI styling is stripped when piped, so logs stay clean. Three widgets: **board** (step tree with spinner, per-step glyphs, timings, summary line), **diagnostic** (nested severity tree with rollup), **error** (friendly top-level error block per `Error` variant).

The four `examples/` dirs form a de-facto tutorial: **order-guard** (arg schema validation) → **build-pipeline** (steps, artifacts, `call:` subroutines) → **http-healthcheck** (modules + cross-file priority) → **batch-report** (loops, fan-out, yaml round-trip).

## Quality & maturity

- **Tests** — ~68 integration test functions in `tests/` (flagship `runtime.rs` alone has 18) plus ~163 inline unit-test attributes across crates. The key asset is a `Recorder` implementing the `Observer` trait (`tests/common/mod.rs`), used in a consistent build → `runtime.call(...)` → `drop(runtime)` → assert pattern (the drop flushes the async observer before assertions). `reflect` and `schema` are heavily unit-tested.
- **Tooling** — pinned `nightly` toolchain (`rust-toolchain.toml`) with rustfmt + clippy; `rustfmt.toml` (edition 2024, width 130, grouped imports); `.cargo/config.toml` with a `cargo lint` alias = `clippy --workspace --all-features -- -D warnings`.
- **Build health** — `cargo check --workspace` and `cargo test --workspace --all-features --no-run` both pass clean across all 11 crates.
- **Discipline** — near-zero TODO/FIXME/dead code, disciplined panics (mostly test assertions and type-invariant guards), active refactoring in recent history.

## Strengths

- Uniform `Reflect` integration: Rust functions, subroutines, and vars are all first-class inside templates.
- Capabilities are genuinely optional plugins added via ergonomic builder extension traits.
- Thoughtful diagnostics model (max-severity rollup, child nesting) and a real event/observer pipeline.
- Event-sourced test strategy via the `Recorder` harness; high code discipline for a pre-release project.

## Rough edges & recommendations

- **CI** — `.github/workflows/ci.yml` runs fmt + clippy (`-D warnings`) + build + test on every push and PR. It omits `--all-targets` from the clippy step, so benches go unlinted.
- **Duplicated guard evaluation** — the `if:` condition is evaluated in both `Step::invoke` (`manifest/step.rs:37`) and `Routine::invoke` (`object/routine.rs:117`). Evaluate once and thread the result through.
- **Locking friction** — `Scope` takes fine-grained mutexes on `symbols`/`Arena` per access, and `with_env` (`state/scope.rs:41`) `panic!`s if the scope `Arc` is shared. Worth revisiting the concurrency model.
- **Uneven coverage** — `reflect`/`schema` are well unit-tested, but `core` (the engine), `cli`, and the thin io crates (`http`/`fs`/`ai`/`codec`) lean almost entirely on integration tests, and the shipped `examples/*.yml` aren't wired into any test. **Add smoke tests that parse/execute each example manifest.**
- **Stringly-typed errors** — `Box<dyn Error>` throughout; steps convert errors to strings. A typed error enum would improve diagnostics.
- **`ai` crate** — heavy deps (candle + tokenizers + hf-hub) behind a thin surface (`Artifact`/`Span`/`Annotation`). Wired into `Builder` via the `AI` extension trait; each pipeline runs local (candle) or remote (OpenAI-compatible) off a single `ModelRef`.
- **Config cruft** — `.editorconfig` carries inherited-from-rustc entries referencing paths that don't exist in this repo.
- **Stub README** — no user-facing docs (addressed alongside this report); `docs.rs`/homepage URLs in `Cargo.toml` are aspirational placeholders.
