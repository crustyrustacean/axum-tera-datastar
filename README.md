# axum-tera-datastar

A minimal, opinionated starting point for a Rust web app using
[axum](https://github.com/tokio-rs/axum), [Tera](https://keats.github.io/tera/)
for templating, and [Datastar](https://data-star.dev) for hypermedia-driven UI
updates over SSE.

It ships a small working to-do app — add an item, delete an item — but the
point is the *shape* of the project, not the app.

## What you get

- **Library + binary split.** `src/lib.rs` exports the app so integration tests
  can boot the real thing instead of a mock. See `tests/api/helpers.rs`.
- **Layered router** with request IDs, tracing, and static file serving, built in
  `src/app.rs`.
- **Layered configuration** — `configuration/base.toml` plus an environment file,
  overridable by env vars.
- **Structured JSON logging** via `tracing` + Bunyan formatting, with per-request
  `request_id` and `matched_path` on every span.
- **Typed errors per route** (`thiserror` + `IntoResponse`) instead of `Box<dyn Error>`.
- **Graceful shutdown** on Ctrl-C and SIGTERM.
- **Integration tests** that assert on the actual SSE wire format.

## Requirements

Rust stable (edition 2024). The toolchain is pinned in `rust-toolchain.toml`;
if you have `rustup`, it will be selected automatically.

## Running it

```sh
cargo run
```

Then open <http://127.0.0.1:8000>.

The app reads config at startup relative to the **current working directory**,
so run it from the repo root.

## Configuration

| Layer | Purpose |
| --- | --- |
| `configuration/base.toml` | Defaults for every environment. |
| `configuration/{local,production}.toml` | Per-environment overrides. `local.toml` is intentionally empty. |
| Environment variables | Override anything, using `APP_` prefix and `__` separator. |

```sh
APP_ENVIRONMENT=production cargo run
APP_APPLICATION__PORT=5001 cargo run
```

## Tests

```sh
cargo test
```

To see logs from the test harness:

```sh
TEST_LOG=1 cargo test
```

Tests boot the real application on an ephemeral port. They assert against the
SSE wire format (`event: datastar-patch-elements`, `data: mode append`, …) and
against escaping behaviour, not against incidental template whitespace.

## Project layout

```
configuration/     layered TOML config
src/
  bin/main.rs      thin entrypoint
  app.rs           Application, router, middleware wiring
  configuration.rs Settings + environment detection
  state.rs         AppState, Tera, in-memory items
  telemetry.rs     tracing subscriber, request spans, request IDs
  utils.rs         shutdown signal, error chain formatting
  routes/
    index.rs       GET  /             server-rendered page
    items.rs       POST /items        append patch
                   DELETE /items/{id} remove patch
    health_check.rs GET  /health_check
templates/
  index.html       page shell
  item.html        the single <li> partial
static/            datastar.js, CSS (served at /static)
tests/api/         integration tests
```

## Conventions worth keeping

**One template per fragment, rendered on both paths.** `templates/item.html` is
the only place the `<li>` markup exists. The page render includes it from
`index.html`; the create handler renders it through the same Tera instance and
sends the result as an SSE patch. Because both go through Tera, user input is
escaped identically on both paths.

This is not incidental. An earlier version of this template hand-built the
`<li>` in the handler with `format!` and interpolated the item text directly —
the server-rendered page escaped it, the SSE patch did not, and the result was a
stored XSS. **Don't hand-build HTML in handlers; render a partial.**

```rust
// Good
let patch = PatchElements::new(render_item(&state, id, &new_item.item)?)

// Avoid — reintroduces the escaping split
let patch = PatchElements::new(format!("<li>{text}</li>"))
```

If you add a `| safe` filter to a template, you are opting out of this. Make
sure that's deliberate.

## A note on `static/datastar.js`

The Datastar client script is vendored rather than bundled, and the Rust crate
and the JS file version independently. When you bump the `datastar` crate,
check which JS version it expects and update the file to match — nothing will
catch a mismatch for you. The version is on the first line of the file.

## What this is not

This is a starting point, not a product.

- **State is in-memory.** `AppState` holds `Arc<Mutex<Vec<Item>>>` and a counter.
  Everything is lost on restart, and nothing is shared between processes. Swap
  in a real store when you need one.
- **There is no auth, no CSRF protection, and no rate limiting.**
- **There is no database, no migrations, and no session handling.**
- **Error messages are deliberately generic** to avoid leaking internals. Add
  structured error reporting before running this anywhere real.
- **No deployment story.** No Dockerfile, no process manager config, no CI yet.

## License

MIT — see [LICENSE](LICENSE).
