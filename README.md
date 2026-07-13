<div align="center">

# Nova

**A YAML-driven workflow runtime with an embedded template engine.**

[![status](https://img.shields.io/badge/status-pre--release-orange)](#project-status)
[![version](https://img.shields.io/badge/version-0.0.0-blue)](Cargo.toml)
[![edition](https://img.shields.io/badge/rust-2024%20nightly-informational)](rust-toolchain.toml)
[![license](https://img.shields.io/badge/license-MIT-green)](LICENSE)

</div>

Nova runs *workflows* written as YAML manifests. A workflow is a named routine of ordered *steps* — each one runs a shell command, renders a template, or calls another routine. Inputs are schema-validated, side effects and errors surface as structured diagnostics, and everything streams into a live terminal board as it runs.

```
✗ guard  1ms
  ✓ valid_order  0ms
  ✗ bad_sku  0ms
  ✗ qty_out_of_range  0ms
✓ place_order  0ms
  ✓ accept  0ms
2 tasks · 4 steps · 2 ok · 0 skipped · 2 failed · total 2ms
● info   accepted order ABC-123 x5
● error  invalid args for `place_order`: sku: must look like ABC-123
● error  invalid args for `place_order`: qty: must be between 1 and 100
```

---

## Contents

- [Install](#install)
- [Quick start](#quick-start)
- [Manifests](#manifests)
  - [Triggers](#triggers)
  - [Steps](#steps)
  - [Validated inputs](#validated-inputs)
  - [Environment](#environment)
- [Templating](#templating)
- [Recipes](#recipes)
- [Examples](#examples)
- [Architecture](#architecture)
- [Project status](#project-status)

---

## Install

Nova is built with a pinned nightly toolchain (see [`rust-toolchain.toml`](rust-toolchain.toml)):

```sh
cargo build --release
```

The CLI binary is named `nova` (from the `nova-cli` package). To run it without installing:

```sh
cargo run -p nova-cli -- run <files...>
```

Examples below use `nova run …` — substitute `cargo run -p nova-cli -- run …` if you haven't put the binary on your `PATH`.

---

## Quick start

Create `hello.yml`:

```yaml
name: hello
on: [run]

vars:
  who: world

steps:
  - name: greet
    shell: |
      echo "hello, {{ who }}!"

  - name: log
    run: |
      {{ info('greeted ' ~ who) }}
```

Run it — quote globs so your shell doesn't expand them first:

```sh
nova run 'hello.yml'
```

Nova loads the manifest, runs the `hello` routine (because it's triggered by `run`), renders a live board of the steps, and prints any diagnostics afterward. Output is TTY-aware: pipe it and the styling is stripped so logs stay clean.

---

## Manifests

A manifest is a single YAML file with the following top-level keys.

| Key         | Purpose                                                               |
|-------------|-----------------------------------------------------------------------|
| `name`      | The routine's name — used to call it and to identify it in the board. |
| `on`        | Triggers: `run`, `run(<priority>)`, or `call`.                        |
| `args`      | Schema for validating inputs passed by a caller.                      |
| `vars`      | Variables available to every step's templates.                        |
| `env`       | Bind process environment variables into the routine's scope.          |
| `templates` | Inline named templates.                                               |
| `steps`     | The ordered list of steps to execute.                                 |

### Triggers

| Trigger            | Meaning                                                                                          |
|--------------------|--------------------------------------------------------------------------------------------------|
| `run`              | An **entry point** the CLI invokes automatically.                                                |
| `run(<priority>)`  | An entry point with an ordering priority — higher priority runs first when multiple files match. |
| `call`             | A **subroutine**, invoked only from another step's `call:`.                                       |

### Steps

Every step may carry a `name:` and an optional `if:` guard (a condition expression), plus exactly one body.

| Body      | Runs                                                                            |
|-----------|---------------------------------------------------------------------------------|
| `shell:`  | A command executed via `sh -c` (templated first).                               |
| `run:`    | A [minijinja](https://docs.rs/minijinja) template, rendered for its side effects. |
| `call:`   | Another routine, invoked by name and passed data with `with:`.                  |

A guard and a subroutine call in practice:

```yaml
steps:
  - if: build.stages | length > 0
    name: run_stages
    shell: |
      {% for stage in build.stages %}
      echo "[{{ loop.index }}/{{ build.stages | length }}] {{ stage }}..."
      {% endfor %}

  - name: report
    call: report
    with:
      workspace: workspace
```

### Validated inputs

A `call`-triggered routine can validate its arguments with an `args` schema. Invalid inputs are rejected as diagnostics instead of running the steps:

```yaml
name: place_order
on: [call]

args:
  type: object
  fields:
    sku:
      type: string
      min: 3
      pattern: "^[A-Z]{3}-[0-9]+$"
      message: "must look like ABC-123"
    qty:
      type: integer
      min: 1
      max: 100
      message: "must be between 1 and 100"

steps:
  - name: accept
    run: |
      {{ info('accepted order ' ~ sku ~ ' x' ~ qty) }}
```

### Environment

The `env:` block pulls values from the process environment into the routine's scope. Each entry maps a **local variable name** to the **name of an OS environment variable**; when that variable is set at startup, its value is bound as a local you can reference in templates. Unset variables are simply skipped.

```yaml
name: deploy
on: [run]

env:
  token: DEPLOY_TOKEN     # {{ token }}  <- value of $DEPLOY_TOKEN
  region: AWS_REGION      # {{ region }} <- value of $AWS_REGION

steps:
  - name: check
    run: |
      {% if token %}
        {{ info('deploying to ' ~ region) }}
      {% else %}
        {{ error('DEPLOY_TOKEN is not set') }}
      {% endif %}
```

To read a variable inline instead of pre-binding it, use the `env()` helper — it takes an optional `default`:

```yaml
steps:
  - name: greet
    run: |
      {{ info('hello from ' ~ env('HOSTNAME', default='localhost')) }}
```

---

## Templating

`run:` blocks — and interpolations inside `shell:` — are minijinja templates. State from `vars`, `args`, and parent scopes is directly available, alongside these built-in functions and modules (enabled by default in the CLI).

| Helper                             | Purpose                                     |
|------------------------------------|---------------------------------------------|
| `info(msg)` `warn(msg)` `error(msg)` | Emit a diagnostic.                        |
| `env(name, default=…)`              | Read a process environment variable.       |
| `fs.read(path)` `fs.write(path, s)` | Read / write a file.                       |
| `json.encode(v)` `json.decode(s)`   | JSON encode / decode.                      |
| `yaml.encode(v)` `yaml.decode(s)`   | YAML encode / decode.                      |
| `http.get(url)`                     | HTTP request — returns `.status`, etc.     |
| `ai.sentiment(text)`                | Sentiment classification (annotations).    |
| `ai.entities.extract(text)`         | Named-entity extraction (annotations).     |
| `ai.keywords.extract(text)`         | Keyword extraction (annotations).          |
| `ai.pii.extract(text)`              | PII detection (annotations).               |
| `ai.embeddings(text)`               | Sentence embeddings (artifacts + vectors). |
| `ai.summarize(text)`                | Abstractive summarization (artifacts).     |

Standard minijinja filters (`| length`), loops with `loop.index`, `{% set %}`, and `{% if %}/{% else %}` all work.

### AI routines

The `ai` module wraps [rust-bert](https://github.com/guillaume-be/rust-bert) NLP pipelines (enabled by the `ai` feature, on by default in the CLI). Each routine accepts a single string **or** a list of strings, plus an optional `min_score` keyword to filter low-confidence results. Models are downloaded on first use and cached per worker thread.

`ai.sentiment`, `ai.entities.extract`, `ai.keywords.extract`, and `ai.pii.extract` return **annotations** (`name`, `label`, `text`, `score`, `spans`); `ai.embeddings` and `ai.summarize` return **artifacts** (`name`, `value`, and, for embeddings, a `vector`).

```yaml
name: analyze
on: [run]

vars:
  review: "The staff were incredibly helpful and the room was spotless."

steps:
  - name: score
    run: |
      {% set out = ai.sentiment(review, min_score=0.5) %}
      {% if out %}
        {{ info('sentiment: ' ~ out[0].label ~ ' (' ~ out[0].score ~ ')') }}
      {% endif %}
```

---

## Recipes

### Fetch over HTTP and persist the result

Probe a URL, then write a JSON status file — combining `http.get`, `fs.write`, and `json.encode`:

```yaml
name: healthcheck
on: [run]

vars:
  url: https://httpbin.org/status/200
  status_file: /tmp/nova-healthcheck.json

steps:
  - name: probe
    run: |
      {% set res = http.get(url) %}
      {% if res.status >= 200 and res.status < 300 %}
        {{ info('healthy: ' ~ url ~ ' -> ' ~ res.status) }}
        {{ fs.write(status_file, json.encode({'url': url, 'status': res.status, 'healthy': true})) }}
      {% else %}
        {{ error('unhealthy: ' ~ url ~ ' -> ' ~ res.status) }}
      {% endif %}
```

### Read a file back and act on it

Load the file written above, decode it, and branch on its contents — `fs.read` + `json.decode`:

```yaml
name: notify
on: [run]

vars:
  status_file: /tmp/nova-healthcheck.json

steps:
  - name: read_status
    run: |
      {% set report = json.decode(fs.read(status_file)) %}
      {% if report.healthy %}
        {{ info('notify: ' ~ report.url ~ ' is UP (' ~ report.status ~ ')') }}
      {% else %}
        {{ warn('notify: ' ~ report.url ~ ' is DOWN (' ~ report.status ~ ')') }}
      {% endif %}
```

---

## Examples

The [`examples/`](examples/) directory doubles as a tutorial — each is runnable end to end.

| Example | What it shows | Run |
|---------|---------------|-----|
| **order-guard** | Argument schema validation; a guard routine calls `place_order` with good and bad inputs. | `nova run 'examples/order-guard/*.yml'` |
| **build-pipeline** | A prioritized pipeline: writes/reads a JSON artifact, loops over build stages, asserts via shell exit codes, delegates to a `report` subroutine. | `nova run 'examples/build-pipeline/*.yml'` |
| **http-healthcheck** | Probes a URL with `http.get`, writes a status file, then a lower-priority `notify` routine reads it back — cross-file ordering by priority. | `nova run 'examples/http-healthcheck/*.yml'` |
| **batch-report** | Fans out over records with a `{% for %}` loop, calls a `format` subroutine per item, aggregates with shell, round-trips through YAML. | `nova run 'examples/batch-report/*.yml'` |
| **review-triage** | Runs rust-bert NLP over customer reviews: a `classify` subroutine scores sentiment and extracts entities and keywords for each. Needs the `ai` feature. | `nova run 'examples/review-triage/*.yml'` |

---

## Architecture

Nova is a Cargo workspace. The root `nova` crate re-exports feature-gated capability crates.

| Crate     | Feature   | Role                                                                 |
|-----------|-----------|----------------------------------------------------------------------|
| `core`    | (always)  | The runtime engine: manifests, scopes, routines, steps, diagnostics. |
| `reflect` | `reflect` | The reflection / type system used across the workspace.              |
| `schema`  | `schema`  | JSON-Schema-style validators for `args`.                             |
| `macros`  | `macros`  | Scope and diagnostic macros (`call!`, `set!`, `info!`, …).           |
| `fs`      | `fs`      | Filesystem module (`fs.read` / `fs.write`).                          |
| `codec`   | `codec`   | Encoding modules (`json`, `yaml`).                                   |
| `http`    | `http`    | HTTP module (`http.get`).                                            |
| `ai`      | `ai`      | AI module: rust-bert NLP routines (`ai.sentiment`, `ai.embeddings`, …). |

For a deeper walkthrough of the design and a candid review of strengths and rough edges, see [REPORT.md](REPORT.md).

---

## Project status

Nova is **pre-release (v0.0.0)** and evolving — APIs, the manifest schema, and the CLI may change. The `docs.rs` and homepage links in [`Cargo.toml`](Cargo.toml) are placeholders for now.

**Backlog**

- AI text completion — `my name is {{ ai.complete('my name is') }}` (the `ai` module ships NLP routines today; generative completion is still pending).
