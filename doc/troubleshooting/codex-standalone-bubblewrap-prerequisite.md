# Codex 0.163.0 standalone Linux host completes hooks but cannot launch shell commands without Bubblewrap

## Symptom

The frozen installed-host fixture in `package/codex-plugin/bash-limit/tests/native_host.rs`
receives 40 synchronous hook completions,
but its synthetic `codex/cgroup.procs` stays empty and its final assertion fails.
The real local provider receives tool outputs with exit code `101`:

```text
bubblewrap is unavailable: no system bwrap was found on PATH and no bundled
codex-resources/bwrap binary was found next to the Codex executable
```

The actual panic location is `linux-sandbox/src/launcher.rs:60`.
Codex also emits a startup diagnostic saying external Bubblewrap is absent and it will use the bundled helper.
In this prepared image,
the standalone executable has no adjacent bundled helper.

## Root cause

The official standalone Codex executable was checksum/signature admitted,
but its consumer image lacked the Linux sandbox executable prerequisite.
This is an environment admission failure,
not evidence that the plugin ignored its command rewrite.
The [current sandbox prerequisites][sandbox] explicitly require installing distribution `bubblewrap` on Linux.

The signed release points to upstream commit `83086188ae0614bd882818ff1aa23f3194962012`.
Its `codex-rs/linux-sandbox/src/launcher.rs:135` tries a compatible system executable,
then the bundled helper,
then records `Unavailable`:

```rust
//! Quoted upstream launcher excerpt.
// openai/codex: codex-rs/linux-sandbox/src/launcher.rs:140
if let Some(path) = find_system_bwrap_in_path(file_system_policy, sandbox_policy_cwd)
    && let Some(launcher) = system_bwrap_launcher_for_path(&path)
{
    return BubblewrapLauncher::System(launcher);
}
match bundled_bwrap::launcher() {
    Some(launcher) => BubblewrapLauncher::Bundled(launcher),
    None => BubblewrapLauncher::Unavailable,
}
```

`codex-rs/linux-sandbox/src/launcher.rs:60` panics on that unavailable state before the requested shell executes:

```rust
//! Quoted upstream launcher excerpt.
// openai/codex: codex-rs/linux-sandbox/src/launcher.rs:59
BubblewrapLauncher::Unavailable => {
    panic!(
        "bubblewrap is unavailable: no system bwrap was found on PATH and no bundled \
         codex-resources/bwrap binary was found next to the Codex executable"
    )
}
```

The earlier suspicion that completed hooks established rewritten shell execution was incorrect.
The transparent localhost capture in `mwwmknc37b` retains actual `function_call_output` failures.
Its local server still accepted the next request because the unchanged fixture checks the entire conversation body:

```rust
//! Quoted repository fixture excerpt.
// package/codex-plugin/bash-limit/tests/host_support/server.rs:67
if index > 0 {
    let request = String::from_utf8(body).map_err(io::Error::other)?;
    assert!(request.contains("host-payload-executed"), "{request}");
}
```

The preceding function-call arguments already contain that string.
The same captured request therefore passes the server check despite its failed tool output.
The independent final membership assertion correctly rejects the overall consumer.

## Verification

The [Rust campaign evidence](../audit/depot-rust-coverage-2026-10-18.evidence.json)
and matching raw receipts retain the official executable's SHA-256,
release signature admission,
frozen application source and every provider request.
The executable is Codex `0.163.0`,
SHA-256 `399ff69c431c066e40de318fba45c2b07f45a76b19939d247ee4828fc70c4215`.
The frozen application source is `e02be6c8b8fe9efb41a7f8f1a0b311c17796588a`.

- `c4p5g7lx48` and `xdv4v475fq` retain the failing unchanged installed-host case.
- `mwwmknc37b` adds a transparent JSONL tee and a byte-preserving localhost HTTP relay.
  It captures 41 provider requests and the actual failed shell tool outputs.
  No paid provider or API credential is involved.
- Hook configuration admission and synchronous completion work cleanly.
  The absent sandbox executable prevents the downstream shell boundary.
- The prepared binary's `--version` and `app-server --help` pass.
  Neither command exercises sandbox launch.

The isolated research branch's runnable owner is:

```text
# Isolated research worktree, after the immutable harness revision is pushed
RESEARCH_RUST_PROFILE=codex-host mise run //package/monorepo-task/core-exec:research:rust:host
```

That task uses only disposable fixture homes,
the existing localhost Responses server,
the official prepared Codex binary and the unchanged native-host test.

## Verified workarounds

The admitted correction installs distribution `bubblewrap` in the disposable CI sandbox,
records its package version and executable hash,
and probes real namespace creation before invoking the existing fixture.
Run `1h6qf8j86f` at immutable harness `28465cf9ba7f6c9df8329f6b254e7b36c2c815d6` passed
the unchanged installed-host Nextest case and its independent membership assertion.
The retained final provider request contains 40 successful `function_call_output` items,
each with code `0` and `host-payload-executed` output.
There are 41 local provider requests in total.
This verifies the installed host and command rewrite against the deterministic fixture.
No sandbox bypass or account setting change is part of this correction.

## What does not work

- Retrying the same prepared image without its missing sandbox prerequisite reproduces the failure.
- Trusting the configuration hash makes hooks eligible but cannot supply the missing sandbox executable.
- Counting hook completions,
  raw conversation substring matches or binary help output as successful shell execution overstates coverage.
- Reading only the host's abbreviated stderr misses the full tool error.
  The transparent provider capture exposes the returned panic without modifying application or test source.

## Upstream filing decision

The `.out-of-scope/codex-harness.md` policy describes maintenance priority,
and explicitly permits authorized investigation.
It does not exempt this task from ordinary verification.
External messages are outside this campaign's authorization.

1.  Upstream fault:
    not established.
    The observed environment omits a documented prerequisite.
2.  Upstream fixability:
    no upstream implementation fix is needed to test the admitted distribution package.
3.  Supported use case:
    the current Linux sandbox documentation supports this setup with Bubblewrap installed.
4.  Contribution policy:
    upstream `docs/contributing.md` welcomes issue reports and rejects external code PRs.
5.  Fix intent:
    no contrary maintainer position is needed to resolve this environment failure.
6.  Prototype:
    the correction is a consumer environment setup change,
    with an actual sandbox probe and unchanged native-host rerun required before acceptance.

A duplicate search for the exact panic found [openai/codex#50347](https://github.com/openai/codex/issues/50347).
The complete issue and comments describe the same missing Bubblewrap prerequisite in a WSL/Desktop setup,
alongside a separate missing-launcher incident.
The distribution-package workaround is already reported there.
This Linux CI fixture adds no required upstream fix or novel recovery mechanism.
Nothing is ready to file as an upstream defect.
The campaign retains its local reproduction and prerequisite correction instead.

[sandbox]: https://developers.openai.com/codex/concepts/sandboxing#prerequisites
