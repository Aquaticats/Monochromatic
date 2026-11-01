# Preserving authored Nextest scheduler arguments

## Observed pre-body rejection

Cargo-nextest0.9.148 rejected the first current-source Windows GNU adapter with:

```text
error: the argument '--no-fail-fast' cannot be used multiple times
```

The frozen `test:windows-gnu` owner in
[`package/music-player/desktop-app/mise.toml`](../../package/music-player/desktop-app/mise.toml)
already supplied that argument.
The research adapter added it again.
Public run `39144499822` therefore compiled/discovered no test body in its GNU context.
The other host contexts have separate actual runtime outcomes.
This rejection is a research harness failure, not an application assertion failure.

## Qualified continuation

The [immutable GNU admission](../audit/depot-windows-gnu-owner-correction-admission-2026-10-18.evidence.json)
retains actual decoded bootstrap equality, source hashes and the corrected argument boundary.
The GNU expanded owning task now stays byte-identical to its original frozen expansion.
Owners lacking the scheduler flag receive one copy.
Application assertions and default Cargo/Nextest concurrency are unchanged.
The continuation runs only the GNU context, with a 30-minute job cap.
An actual functional outcome still requires its repository-owner execution.

The installed Nextest0.9.148 parser reproduced the duplicate diagnostic with exit status2.
Its single-argument control passed argument parsing and reached a Cargo metadata attempt,
then stopped without compiling or executing a test.
That local parser control does not qualify the application toolchain or tests.

## Resolve executable paths directly

The first local parser probe assumed a Cargo executable under `mise where rust` plus `/bin/cargo`.
That file was absent in the installed Rust backend layout.
The installed `mise which --help` describes executable resolution,
and `mise which cargo --tool=rust@nightly-2026-10-17` resolved the actual Cargo wrapper.
The final parser control invokes the separately qualified installed Nextest binary directly.
No Rust compiler version or build result follows from this parser-only probe.
