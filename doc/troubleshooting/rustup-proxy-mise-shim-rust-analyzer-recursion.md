# rustup 1.29.3's rust-analyzer fallback recurses through a mise shim until "infinite recursion detected", which mise 2026.11.3 reports as a rust backend install failure

Measured on 2026-11-10 with rustup 1.29.3
([commit `edbc831ce`](https://github.com/rust-lang/rustup/blob/edbc831cec44dc2858a4a4dce768c74361d027dc),
the newest release tag)
and mise 2026.11.3
([commit `e6ae670b`](https://github.com/jdx/mise/blob/e6ae670b979f27d04e6c2ca77a7e88d300ee2add)).
The first measurement,
on 2026-10-14,
saw the same loop from the IDE's rust-analyzer server start
and recorded the same final error.

## Symptom

A `mise exec` of `rust-analyzer` through a Rust toolchain that lacks it,
with a mise shim of that name on `PATH`,
prints one pair of lines per cycle and then a mise install failure:

```console
$ mise exec rust@nightly-2026-11-09 -- rust-analyzer --version
info: `rust-analyzer` is unavailable for the active toolchain
info: falling back to "/home/user/.local/share/mise/shims/rust-analyzer"
... (this pair repeats; 21 pairs were measured)
mise ✗ rust@nightly-2026-11-09  18ms · failed: rustup show profile failed for rust@nightly-2026-11-09: error: infinite recursion detected
```

The reported form that started this investigation was the same line for the version
`toolchain.lock.json` then recorded:

```text
mise ✗ rust@nightly-2026-11-09  54ms · failed: rustup show profile failed for rust@nightly-2026-11-09: error: infinite recursion detected
```

The version is not the trigger:
`mise exec rust@nightly-2026-11-09 -- rustc -V` and
`mise exec rust@nightly-2026-11-10 -- rustc -V` both succeed.
The trigger is the binary:
`rust-analyzer`,
which mise shims,
which rustup proxies,
and which the selected toolchain has no `bin/rust-analyzer` for.
`rust-analyzer` is the only binary with this shape,
because rustup's PATH fallback exists only for it;
a missing `cargo-miri` fails cleanly instead
(measured,
see the failing catalog in "Verification").

### The IDE surface (measured 2026-10-14)

In the IDE the same loop surfaced as a failed server start,
with the two `info:` lines and the `error:` line as the server's last standard-error lines.
The IDE's window states those lines in the start failure's reason
(`package/ide/app/README.md`,
"Why a feature is missing").

That day the window reported
`FailedToStart { reason: "the server process ended before it finished starting" }`,
every hover answered `no-server`,
and no inlay hints appeared.
Before that date the window showed no cause at all;
the lines were only in the log (`language server process ended ... stderr_tail=[...]`).
The sandbox passes `RUSTUP_TOOLCHAIN`,
`RUSTUP_HOME` and `CARGO_HOME` into the server
(`package/ide/app/src/language/confine/recipe.rs`,
`ALLOWED_ENVIRONMENT`),
so the server sees the same toolchain selection as the user's shell.

An earlier reading of this file blamed the sandbox (a `mise.toml` in the project that the
sandbox does not trust).
That was wrong:
the message appeared only because the disposable project held a copy of this
repository's `mise.toml`,
and removing the copy did not change the failure.
The chain in "Root cause" is what the failure is.

## Root cause

The loop is a cycle between rustup 1.29.3's rust-analyzer fallback and mise's shim of the same name.
The earlier version of this file stopped at step 4 with
"What the shim does between the two lines was not traced";
this section replaces that gap with a source trace and an `execve` trace.

1.  The active toolchain has no `bin/rust-analyzer`,
    so rustup's proxy search runs.
    `Toolchain::command` special-cases `rust-analyzer`
    ([`src/toolchain.rs#L330-L357`](https://github.com/rust-lang/rustup/blob/edbc831cec44dc2858a4a4dce768c74361d027dc/src/toolchain.rs#L330-L357)):

    ```rust
    } else if let "rust-analyzer" | "rust-analyzer.exe" = binary
        && let Some(cmd) = self.maybe_do_rust_analyzer_fallback(binary)?
    {
        info!("`rust-analyzer` is unavailable for the active toolchain");
        info!(
            r#"falling back to "{}""#,
            Path::new(cmd.get_program()).display()
        );
        return Ok(cmd);
    }
    ```

2.  The fallback walks `$PATH` and skips only a candidate that is the same file as the running
    rustup binary
    ([`src/toolchain.rs#L401-L433`](https://github.com/rust-lang/rustup/blob/edbc831cec44dc2858a4a4dce768c74361d027dc/src/toolchain.rs#L401-L433)):

    ```rust
    let me = env::current_exe()?;

    // Try to find the first `rust-analyzer` under the `$PATH` that is both
    // an existing file and not the same file as `me`, i.e. not a rustup proxy.
    for mut p in env::split_paths(&path) {
        p.push(binary);
        let is_external_ra = p.is_file()
            && !is_same_file(&me, &p).unwrap_or(true);
    ```

    The mise shim `/home/user/.local/share/mise/shims/rust-analyzer` is a symbolic link to
    `mise`,
    so `is_same_file` reports a different file and the fallback picks it.
    This is the gap:
    the guard excludes a rustup proxy by identity,
    not a program that starts
    the same proxy.

3.  rustup runs the chosen program in place of itself
    ([`src/cli/proxy_mode.rs#L48`](https://github.com/rust-lang/rustup/blob/edbc831cec44dc2858a4a4dce768c74361d027dc/src/cli/proxy_mode.rs#L48)),
    so the shim inherits rustup's environment,
    including the `--version` arguments and
    `RUSTUP_TOOLCHAIN`.

4.  The mise shim resolves `rust-analyzer` to the `rust` tool of the same version,
    because `mise exec` put its tool request in the environment.
    `mise exec` writes the request to `__MISE_TASK_TOOL_ARGS`
    ([`src/cli/exec.rs#L378-L390`](https://github.com/jdx/mise/blob/e6ae670b979f27d04e6c2ca77a7e88d300ee2add/src/cli/exec.rs#L378-L390)),
    the shim reads it back into a tool set
    ([`src/shims.rs#L118-L147`](https://github.com/jdx/mise/blob/e6ae670b979f27d04e6c2ca77a7e88d300ee2add/src/shims.rs#L118-L147)),
    and `which_shim` then finds that tool's binary
    ([`src/cli/shim.rs#L420-L428`](https://github.com/jdx/mise/blob/e6ae670b979f27d04e6c2ca77a7e88d300ee2add/src/cli/shim.rs#L420-L428)):

    ```rust
    for lookup_name in [shim_name, bin_name].into_iter().unique() {
        if let Some((p, tv)) = ts.which(config, lookup_name).await
            && let Some(bin) = p.which(config, &tv, lookup_name).await?
        {
            return Ok((ShimCommand::Tool(bin), ts));
        }
    }
    ```

    The rust tool's install directory in mise holds the rustup proxy directory,
    so the tool binary mise runs is `$CARGO_HOME/bin/rust-analyzer`,
    which `ls -l` shows as a symbolic link to `rustup`
    (`readlink -f` gives `/home/user/.cargo/bin/rustup`).

5.  The shim runs that path
    ([`src/cli/shim.rs#L73-L82`](https://github.com/jdx/mise/blob/e6ae670b979f27d04e6c2ca77a7e88d300ee2add/src/cli/shim.rs#L73-L82)),
    which is rustup again,
    this time in proxy mode for the arguments it was given
    (`--version` in this measurement).
    Every rustup start increments `RUST_RECURSION_COUNT` for the program it runs
    ([`src/toolchain.rs#L167-L180`](https://github.com/rust-lang/rustup/blob/edbc831cec44dc2858a4a4dce768c74361d027dc/src/toolchain.rs#L167-L180)):

    ```rust
    env_var::inc("RUST_RECURSION_COUNT", cmd, self.cfg.process);
    ```

    The cycle is therefore `rustup` then `mise shim` then `rustup`,
    with the counter rising by
    one per cycle.

6.  rustup refuses to run once that counter exceeds
    `RUST_RECURSION_COUNT_MAX = 20`
    ([`src/env_var.rs#L8`](https://github.com/rust-lang/rustup/blob/edbc831cec44dc2858a4a4dce768c74361d027dc/src/env_var.rs#L8),
    [`src/bin/rustup-init.rs#L127-L137`](https://github.com/rust-lang/rustup/blob/edbc831cec44dc2858a4a4dce768c74361d027dc/src/bin/rustup-init.rs#L127-L137)):

    ```rust
    fn do_recursion_guard(process: &Process) -> Result<()> {
        let recursion_count = process
            .var("RUST_RECURSION_COUNT")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);
        if recursion_count > RUST_RECURSION_COUNT_MAX {
            return Err(anyhow!("infinite recursion detected"));
        }

        Ok(())
    }
    ```

    The guard runs before rustup dispatches any subcommand.
    mise's rust plugin runs `rustup show profile` while it checks whether the shim's tool version
    is installed
    ([`src/plugins/core/rust.rs#L590-L612`](https://github.com/jdx/mise/blob/e6ae670b979f27d04e6c2ca77a7e88d300ee2add/src/plugins/core/rust.rs#L590-L612)),
    so at the first cycle whose counter is over the limit that query fails with rustup's guard
    message,
    mise reports it as a failed install of the requested version,
    and the chain ends
    there instead of running the binary.

mise has a shim recursion guard of its own,
but it covers only the system-fallback handoff chain
([`src/cli/shim/fallback.rs#L54-L70`](https://github.com/jdx/mise/blob/e6ae670b979f27d04e6c2ca77a7e88d300ee2add/src/cli/shim/fallback.rs#L54-L70)).
In this cycle mise does not take a system fallback:
it resolves a configured tool version and
runs that tool's binary,
so the guard is not consulted.

### What is measured versus inferred

The `execve` trace below shows the programs actually started,
in order.
It is the evidence for the cycle,
and it also rules out this repository's shadows and launchers:
neither `package/cargo-shadow/cli` nor `package/mpm-shadow/cli` appears in the chain,
and the rustup that mise starts is the real rustup binary.

```console
$ strace -f -e trace=execve -o loop.strace mise exec rust@nightly-2026-11-10 -- rust-analyzer --version
$ rg -o 'execve\("[^"]+"' loop.strace | sed 's/execve("//' | head -8
/home/user/.local/bin/mise"
/home/user/.cargo/bin/rustup"
/home/user/.cargo/bin/rustup"
/home/user/.rustup/toolchains/nightly-2026-11-10-x86_64-unknown-linux-gnu/bin/rustc"
/home/user/.cargo/bin/rust-analyzer"
/home/user/.local/share/mise/shims/rust-analyzer"
/home/user/.cargo/bin/rust-analyzer"
/home/user/.local/share/mise/shims/rust-analyzer"
```

Counted over the whole trace:

- 21 `falling back` lines,
  21 executions of the mise shim,
  and 21 executions of the proxy as the
  shim's tool binary:
  one cycle each.
- 47 `rustup` executions in total:
  21 `component list --installed` queries,
  25 `show profile`
  queries and 1 `toolchain install`.
  The chain's cycle runs its two queries,
  and the last cycle
  stops on `show profile`,
  which is the query that fails.
- `$CARGO_HOME/bin/rustup` is byte-identical to the rustup in
  `~/.local/share/monochromatic-dev/toolchain-global/rustup/1.29.3/rustup`
  (`sha256 b1c8cb14e30889e7e2999e6b69e62d9e9b9ffc0f4df8180564d306d9680e6c55`),
  and `rustup --version` reports `rustup 1.29.3 (edbc831ce 2026-11-02)`.
  It is not a shadow,
  launcher or shim of this repository:
  no program under `.cache/toolchain-launchers` or `.cache/*-shadow-bin` appears in the trace.

This repository's own PATH entries are not the picked candidate:
`.cache/toolchain-bin` and
`~/.local/share/monochromatic-dev/toolchain-global/exposure` hold links to the rustup proxies
for `cargo`,
`rustc`,
`rustdoc`,
`rustfmt`,
`rust-gdb`,
`rust-gdbgui`,
`rust-lldb` and `rustup`,
but neither holds `rust-analyzer`,
so rustup's PATH walk continues to the mise shim and names it in the `falling back` line.

### Upstream state

rustup main already carries the guard this cycle needs.
[PR 5136](https://github.com/rust-lang/rustup/pull/5136),
"fix(toolchain):
don't fall back to external RA redirecting to rustup",
merged 2026-10-27 as
[commit `3d011da95`](https://github.com/rust-lang/rustup/blob/3d011da95de82efe13ec2731834ac77f2775903e/src/toolchain.rs#L386-L415):
a candidate must now also exit zero for `--version` with `RUST_RECURSION_COUNT` set to
`RUST_RECURSION_COUNT_MAX + 1`,
so a program that reroutes to rustup is skipped.

```rust
// - Can be executed with `--version` while `RUST_RECURSION_COUNT` is set to
//   `RUST_RECURSION_COUNT_MAX + 1`, i.e. doesn't redirect to rustup.
// ...
            ra.arg("--version")
                .env(
                    "RUST_RECURSION_COUNT",
                    (env_var::RUST_RECURSION_COUNT_MAX + 1).to_string(),
                )
```

No release contains that commit:
the newest tag is `1.29.3`,
whose `maybe_do_rust_analyzer_fallback` has only the `is_same_file`
check quoted above (measured on the tag's source and on the installed binary,
which loops).

## Verification

Versions under test:

- rustup `1.29.3` (`edbc831ce`,
  release tag,
  the installed binary;
  `rustup --version`).
- mise `2026.11.3` (`e6ae670b`;
  `mise --version`).
- Toolchains `nightly-2026-11-09-x86_64-unknown-linux-gnu` and
  `nightly-2026-11-10-x86_64-unknown-linux-gnu` in `~/.rustup`,
  neither with the `rust-analyzer` component.
  `rustup component list --toolchain nightly-2026-11-10` lists
  `rust-analyzer-x86_64-unknown-linux-gnu` without the installed marker.

Harness,
from a checkout of this repository
(each command is bounded by `timeout` and runs in its own `systemd-run --user --scope`):

```console
# Loop, counted.
$ mise exec rust@nightly-2026-11-10 -- rust-analyzer --version 2>loop.err
$ rg -c 'falling back' loop.err
21
$ tail -1 loop.err
mise ERROR Failed to install core:rust@nightly-2026-11-10: rustup show profile failed for rust@nightly-2026-11-10: error: infinite recursion detected

# The same for the version the lock recorded when the report was made.
$ mise exec rust@nightly-2026-11-09 -- rust-analyzer --version 2>loop-1109.err
$ rg -c 'falling back' loop-1109.err
21
```

Catalog that works cleanly:

```console
$ mise exec rust@nightly-2026-11-10 -- rustc -V
rustc 1.102.0-nightly (ac8310ba6 2026-11-09)
$ mise exec rust@nightly-2026-11-09 -- rustc -V
rustc 1.102.0-nightly (5b7b37aef 2026-11-08)
$ mise exec rust@nightly-2026-11-10 -- rustup show profile
default
```

Catalog that fails,
two variants:

- `rust-analyzer` through `mise exec rust@<channel>` when the toolchain lacks it:
  the 21-cycle loop above,
  ending in the mise install failure.
- `cargo-miri` through the same command:
  no loop,
  because rustup's PATH fallback is rust-analyzer-only.

  ```console
  $ mise exec rust@nightly-2026-11-10 -- cargo-miri --version
  error: 'cargo-miri' is not installed for the toolchain 'nightly-2026-11-10-x86_64-unknown-linux-gnu'.
  help: run `rustup component add --toolchain nightly-2026-11-10-x86_64-unknown-linux-gnu miri` to install it
  ```

- The bare proxy with no tool request in the environment ends differently:
  `rust-analyzer --version` outside `mise exec` stops at
  `mise ERROR No version is set for shim: rust-analyzer`,
  because the shim then has no tool version to resolve.

The 2026-10-14 measurement of which toolchains carried the component:
`nightly-2026-09-12`,
`nightly-2026-09-16` and `nightly-2026-09-22` printed a path for
`env --unset=RUSTUP_TOOLCHAIN rustup which --toolchain <toolchain> rust-analyzer`;
`stable`,
`nightly`,
`nightly-2026-09-11`,
`nightly-2026-10-11` to `nightly-2026-10-13`
and `1.98.1` printed `error: unknown binary 'rust-analyzer'`.
That is why the same command failed for every version the report's author tried:
the failing input is the binary,
not the version.

Probe of the fix on main,
using the installed 1.29.3 shim:

```console
$ mise exec rust@nightly-2026-11-10 -- \
    sh -c 'RUST_RECURSION_COUNT=21 /home/user/.local/share/mise/shims/rust-analyzer --version'
mise ERROR Failed to install core:rust@nightly-2026-11-10: rustup show profile failed for rust@nightly-2026-11-10: error: infinite recursion detected
$ echo $?
1
```

A nonzero exit under `RUST_RECURSION_COUNT_MAX + 1` is exactly what the main-branch guard treats
as "not an external rust-analyzer",
so the shim would be skipped and the loop would end.

## Verified workarounds

- Set `RUSTUP_TOOLCHAIN` to a toolchain that has the component before starting the IDE
  (`nightly-2026-09-22` on 2026-10-14).
  Tradeoff:
  the IDE passes that variable to every server,
  so cargo-based tooling inside rust-analyzer uses that toolchain too.
- Add the component to the toolchain in use:

  ```console
  $ rustup component add rust-analyzer --toolchain nightly-2026-11-10
  ```

  Tradeoff:
  it writes the user's toolchains,
  adds a component this repository's
  `toolchain.lock.json` does not request (`rustComponents` is `clippy`,
  `rust-src`,
  `llvm-tools`,
  `rustc-dev`),
  and a later `toolchain:install` or toolchain update can leave it
  behind or remove it.
  Not run here:
  it changes `~/.rustup`,
  which this investigation must leave alone.
- Do not run a shimmed binary the selected toolchain lacks.
  `rustc`,
  `cargo`,
  `rustup` and the toolchain binaries these resolve are unaffected
  (measured above).
  Tradeoff:
  the IDE then has no language server for a toolchain without the component,
  which is the failure this file's first symptom described.
- Upgrade rustup once a release contains PR 5136.
  Tradeoff:
  none measured;
  no release contains it today,
  so this workaround is not available yet,
  and it
  changes no repository input until the lock or the toolchain plan names a newer rustup.

## What does not work

- Removing the project's `mise.toml` from a disposable copy:
  the loop is in the shim and the proxy,
  not in the project's config.
- Asking the shim directly:
  it is the loop's second half,
  as the trace shows.
- Setting `RUSTUP_TOOLCHAIN` around `mise run`:
  mise sets that variable itself,
  so the outer value does not reach the tool
  (measured on 2026-10-14 for the IDE inspections).
- Relying on `MISE_RUST_VERSION` from outside a task:
  on 2026-10-14 `MISE_RUST_VERSION=nightly-2026-09-22` made `mise run` select a toolchain that
  had the component,
  which passed `inspect:language` and `inspect:language-lifecycle`;
  it is a selection of a different toolchain,
  not a fix for a toolchain that lacks the binary.
  `inspect:language-confinement` still failed its `scripted-pwd-alias` case
  (`WrongWorkingDirectory("/tmp")`,
  an empty `/tmp/.git` directory Helix treats as a workspace
  marker),
  and an older build from the main checkout failed the same checks.
- Pointing the PATH at this repository's shadows:
  no shadow,
  launcher or shim of this repository is in the chain (measured by the `execve` trace),
  so changing `cargo-shadow`,
  `mpm-shadow` or `.cache/toolchain-bin` cannot break the loop.

## Upstream filing decision

`.out-of-scope/` was read on 2026-11-10 and holds no exemption for rustup or mise:
its files cover Bun's installer,
Cargo workspaces,
Claude Code,
the Codex harness,
JSR,
LightningCSS,
low-impact formatter rules,
`module/es`,
Pi's GPT-5.5 context metadata and
terminal-title parity tests.

Duplicate search on 2026-11-10:

```console
$ gh search issues --repo rust-lang/rustup "infinite recursion detected" --limit 10
rust-lang/rustup  5133  closed  rust-analyzer fallback recurses through Homebrew’s wrapper
$ gh search issues --repo jdx/mise "infinite recursion" --limit 10
(no rustup or rust-analyzer loop; the hits are about Windows shims and env templates)
$ gh search issues --repo jdx/mise "rustup show profile" --limit 10
(rustup queries in mise: 13698 "Every shim call runs `rustup` twice when `rust` is in the
config" and 14388 "mise --env regression?"; neither is this loop)
$ gh search issues --repo jdx/mise "rust-analyzer recursion" --limit 6
(no hits)
```

[rustup issue 5133](https://github.com/rust-lang/rustup/issues/5133) is the same defect on
macOS through Homebrew's wrapper script.
It is closed as completed,
on the day PR 5136 merged (2026-10-27),
and a maintainer described this exact gap in it:
the fallback "clearly didn't consider the case where a `rust-analyzer` binary can reroute to
rustup itself",
with the `RUST_RECURSION_COUNT` probe as the proposed fix.
The Linux and mise variant measured here is that same gap,
already fixed on main.

Work in progress check on 2026-11-10:

```console
$ gh search prs --repo rust-lang/rustup "toolchain.rs" --state open --limit 10
(no results)
$ gh api 'repos/rust-lang/rustup/commits?path=src/toolchain.rs&per_page=5' --jq '.[] | ...'
3d011da95  fix(toolchain): don't fall back to external RA redirecting to rustup
34311c9d7  refactor(toolchain): reduce rightward drift
```

Nothing is in flight;
the fix is merged and waiting for a release.

Trust and blocklist check on 2026-11-10,
against the repository at `main`:
rustup's root listing
holds `triagebot.toml` but no trust or blocklist file,
and `.github/workflows` holds only
`ci.yaml`;
the binding rule for communication is the dev guide's AI policy quoted in constraint 4.
A filer must check the live repository again on the day of posting.

The six constraints:

1.  Is it really upstream's fault?
    Partly.
    The loop needs both halves:
    rustup's fallback guard compares file identity only,
    and mise's shim resolves a tool version
    to a binary that is the rustup proxy of the same name.
    rustup's maintainers accepted the guard gap in issue 5133 and fixed it.
2.  Can upstream fix it?
    Yes,
    and it did (PR 5136).
3.  Are they supporting this use case?
    Yes.
    The fallback exists for toolchains that do not ship rust-analyzer
    (`src/toolchain.rs`,
    comment citing issues 3299 and 3846),
    and rustup's component list offers the `rust-analyzer` component this host leaves out.
4.  Would the repo welcome our contribution?
    No,
    not one written here.
    rustup's dev guide "AI policy" says
    "AI should not be used to generate comments when communicating with maintainers.
    Comments are
    expected to be written by humans",
    and requires an issue author to describe the problem in their own words.
    We cannot truthfully provide human-authored text,
    so this constraint fails for any filing this
    session could make.
5.  Will they likely fix it?
    Already fixed on main;
    no release contains it yet.
6.  Have we prototyped a minimal fix compatible with their architecture?
    Upstream's own fix is the prototype,
    verified against this host's shim in "Verification".
    No prototype was written for mise,
    and none was attempted here:
    the investigation had to leave mise and rustup unedited.

Outcome:
not filed,
and nothing is kept as a fileable draft,
because constraint 4 fails and issue
5133 already carries the fix.
A human who wants to comment there can add what this measurement adds
(a Linux reproduction through a mise shim,
the 21-cycle count,
and mise's
`rustup show profile failed` line),
but the comment must be written in their own words under that
policy,
and posting it puts the poster's account in front of strangers.

### Readiness check

Not run:
a clean-runner reproduction repository,
a public reproduction workflow,
and a draft
written in a human's words.
The measurements above are reproducible from this host with the commands shown;
the reason no filing draft is kept is constraint 4,
not missing evidence.
