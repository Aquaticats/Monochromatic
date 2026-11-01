# Depot Cargo wraps local compilation and requires installed sccache

## Symptom

The visible `cargo` command appeared in the installed Depot root help,
but its recursively collected help invocation failed:

```text
# Observed from depot cargo --help with sccache absent from PATH.
sccache not found in PATH: exec: "sccache": executable file not found in $PATH

Please install sccache: cargo install sccache
```

This was the only nonzero entry in the captured command-tree enumeration.
It is a failed real invocation,
not a retrieved Cargo-wrapper help page.

## Root cause

Installed CLI `2.102.23`,
source `1f96d6d488f938de97ea640b115e9567ebfd6fbf`,
declares `DisableFlagParsing: true` in `pkg/cmd/cargo/cargo.go:23`.
The command manually removes its `--org` option,
then forwards the remaining arguments to a locally executed `cargo` process.
It resolves `sccache` through `PATH` before starting Cargo.

The current entrypoint configures compiler caching;
it does not submit Rust compilation to a separate remote-execution pool.
The [2025 command announcement][cargo-announcement] describes automatic sccache configuration.
The installed implementation requires sccache to be installed first.
The captured source and actual invocation establish the current prerequisite.

## Authentication and configuration

`pkg/helpers/token.go` resolves this command's authentication in this order:

1.  `DEPOT_TOKEN`.
2.  The saved user API token.
3.  `DEPOT_JIT_TOKEN`.
4.  `DEPOT_CACHE_TOKEN`.

The resolver used here does not call the OIDC-provider resolver used by project authentication.
The final job-cache-token fallback is implemented and exercised with a synthetic marker.

An explicit or saved organization selects WebDAV username/password authentication.
Without an organization,
the wrapper sets a WebDAV token.
The current [Cache authentication documentation][cache-authentication] supports user,
organization,
and job cache tokens,
and excludes build-project tokens.
A caller's build-project `DEPOT_TOKEN` therefore takes precedence over an available job cache token
despite the documented cache-token incompatibility.
That precedence is proven;
the backend's rejection of a real project token was not exercised.

The wrapper removes every inherited `SCCACHE_*` variable,
including cache tuning,
then sets its endpoint and selected credentials.
It also replaces `RUSTC_WRAPPER`.
Existing compiler-cache configuration needs review before this wrapper is substituted for Cargo.

In a noninteractive process with no credentials,
the installed wrapper still starts the child with an empty WebDAV password.
Successful child launch is not proof that the remote cache accepted authentication.

## Verification

The actual installed Depot executable ran against isolated executable and configuration fixtures.
The fake Cargo process exposed only the explicitly supplied synthetic environment.
The fake sccache executable established the prerequisite lookup without compiling Rust.

The boundary controls verified:

- Missing sccache causes the quoted diagnostic even for `--help`.
- Explicit-token,
  saved-token,
  JIT-token,
  and automatic-job-token precedence matches the pinned implementation.
- Organization and token-only WebDAV forms select the expected synthetic credentials.
- Inherited `SCCACHE_*` tuning disappears and the wrapper path is replaced.
- Both forms of the organization option are consumed while remaining arguments reach the child unchanged.
- A child exit of one remains an outer exit of one.
- Missing noninteractive credentials do not prevent child launch.

No real credential,
Rust compilation,
cloud allocation,
or remote-cache acceptance is part of this fixture.
The raw receipt and deciding source are preserved as `depot-cargo-wrapper-boundary.json`
inside the compressed sources linked by the
[BuildKit research receipt](../audit/depot-buildkit-platform-campaign-2026-10-18.evidence.json).
The native sccache workload and real BuildKit Cargo mounts have separate receipts.

## Verified correction

The capability inventory now classifies `depot cargo` as a current local Cargo/sccache convenience entrypoint.
It records its installed-tool prerequisite and credential precedence.
The failed help probe remains visible in the command inventory.
No replacement Rust execution service or newly installed cache tool is implied.

## What does not work

Treating `depot cargo --help` as a passive Cobra-help request misses the actual child-command behavior.
Treating the announcement's acceleration claim as remote Rust compute confuses compiler-result reuse with execution.
Reusing a build-project token for the cache contradicts its documented accepted token types.

## Upstream filing decision

The current source implements the observed wrapper behavior,
and the command announcement describes the underlying sccache mechanism.
No new regression or source/behavior discrepancy was established.
No external issue or message was sent.

[cargo-announcement]: https://depot.dev/changelog/2025-06-30-depot-cargo-command
[cache-authentication]: https://depot.dev/docs/cache/authentication
