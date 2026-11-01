# rust-analyzer through the rustup proxy fails with "infinite recursion detected" when the active toolchain lacks the component and a mise shim exists

The failure was measured on 2026-10-14 on this host with the rustup proxy `~/.cargo/bin/rust-analyzer`
and the mise shim `~/.local/share/mise/shims/rust-analyzer` (mise 2026.10.0).
The IDE's `rust-analyzer` server ends before it finishes starting.

## Symptom

Running the proxy from any directory prints:

```text
info: `rust-analyzer` is unavailable for the active toolchain
info: falling back to "/home/user/.local/share/mise/shims/rust-analyzer"
info: `rust-analyzer` is unavailable for the active toolchain
info: falling back to "/home/user/.local/share/mise/shims/rust-analyzer"
error: infinite recursion detected
```

In the IDE the Language module reports
`FailedToStart { reason: "the server process ended before it finished starting" }`.
Hover, definitions, and inlay hints for every Rust file are then missing.
Before 2026-10-14 the window showed no cause at all;
the lines above were only in the log (`language server process ended ... stderr_tail=[...]`).
The window now states them in the reason of the failed start and in the `language support` entry
(`package/ide/app/README.md`, "Why a feature is missing").

## Root cause

This is the chain the output shows, not a source trace of rustup or mise:

1. `PATH` resolves `rust-analyzer` to the rustup proxy.
2. The toolchain rustup selects has no `rust-analyzer` component.
   `RUSTUP_TOOLCHAIN` selects it; the user's systemd session sets `RUSTUP_TOOLCHAIN=1.98.1`,
   and the default `nightly-x86_64-unknown-linux-gnu` lacks it too.
3. rustup prints the fallback line naming the mise shim.
4. The loop then closes through the shim: the output repeats the two `info` lines before the error.
   What the shim does between the two lines was not traced.
5. rustup reports `infinite recursion detected` and exits.

The IDE's confinement passes `RUSTUP_TOOLCHAIN`, `RUSTUP_HOME`, and `CARGO_HOME` into the sandbox
(`package/ide/app/src/language/confine/recipe.rs`, `ALLOWED_ENVIRONMENT`),
so the server sees the same selection as the user's shell.
An earlier reading blamed the sandbox (a `mise.toml` in the project that the sandbox does not trust);
that message appeared only because the disposable project held a copy of the repository's `mise.toml`,
and removing the copy did not change the failure.

## Verification

```sh
# Each toolchain, with the override unset.
for toolchain in $(rustup toolchain list | cut --delimiter=' ' --fields=1); do
  env --unset=RUSTUP_TOOLCHAIN rustup which --toolchain "$toolchain" rust-analyzer
done
```

Measured output: the toolchains `nightly-2026-09-12`, `nightly-2026-09-16`, and `nightly-2026-09-22` print a path;
`stable`, `nightly`, `nightly-2026-09-11`, `nightly-2026-10-11`, `nightly-2026-10-12`, `nightly-2026-10-13`,
and `1.98.1` print `error: unknown binary 'rust-analyzer'`.

The same Rust file in a disposable copy of `package/jsonc-edit/core-rust`, through `ide-language-inspect`:

- `RUSTUP_TOOLCHAIN=1.98.1`: `FailedToStart`, 290 hover attempts all answered `no-server`, no hints.
- `RUSTUP_TOOLCHAIN=nightly-2026-09-22-x86_64-unknown-linux-gnu`: `Ready` with every feature;
  the first hover is empty while the project is indexed and the next ones return content.

## Verified workarounds

- Set `RUSTUP_TOOLCHAIN` to a toolchain that has the component before starting the IDE.
  Tradeoff: the IDE passes the variable to every server, so cargo-based tooling inside rust-analyzer
  uses that toolchain too.
- Install the component for the toolchain in use (`rustup component add rust-analyzer --toolchain <name>`).
  Not run here: it writes to the user's toolchains.

## Effect on the IDE's real-server inspections

Inside `mise run`, mise sets `RUSTUP_TOOLCHAIN` to the floating nightly named in the root `mise.toml`
(`nightly-2026-10-13` when measured), which has no `rust-analyzer`.
The Rust cases of `inspect:language`, `inspect:language-lifecycle`, and `inspect:language-confinement`
therefore fail on this host with the same `FailedToStart`,
and setting `RUSTUP_TOOLCHAIN` around `mise run` has no effect because mise overrides it.
Measured workaround: `MISE_RUST_VERSION=nightly-2026-09-22` makes mise select a toolchain that has the component;
with it `inspect:language` and `inspect:language-lifecycle` passed.
`inspect:language-confinement` still fails its `scripted-pwd-alias` case on this host,
with `WrongWorkingDirectory("/tmp")`,
while an empty `/tmp/.git` directory (created 2026-10-13) exists,
which Helix treats as a workspace marker.
Removing that directory was not tried, so it is not determined to be the cause;
an older build from the main checkout fails the same three checks, so the failure does not come from this work.

## What does not work

- Removing the project's `mise.toml` from the disposable copy: the failure is the same.
- Asking the shim directly: it fails the same way, which is how the loop was found.

## Upstream filing decision

Not filed.
The behavior needs a source trace of rustup's proxy fallback and mise's shim dispatch, which was not done,
and the host configuration (a mise shim for a tool mise does not manage, beside the rustup proxy) is the user's own.
The six constraints were not walked; there is no draft.
