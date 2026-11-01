# agent-browser 0.39.1: a relative `screenshot` path resolves against the daemon's working directory, not the caller's

## Symptom

On 2026-10-20 on this Linux host,
after the session daemon had been started from another directory,
this failed although `chr/index-1440/` existed relative to the calling shell:

```text
agent-browser screenshot chr/index-1440/00.png
✗ Failed to save screenshot to chr/index-1440/00.png: No such file or directory (os error 2)
```

The same call with an absolute path worked.

Reproduced here (agent-browser 0.39.1) with two directories,
`a/` where the daemon started and `b/` where the caller runs:

```sh
base="${HOME}/temp/agent/ab-cwd3"
mkdir --parents "${base}/a" "${base}/b/out"            # `a/out` does not exist yet
cd -- "${base}/a" && agent-browser --session cwd-probe open about:blank
readlink /proc/$(cat "${XDG_RUNTIME_DIR}/agent-browser/cwd-probe.pid")/cwd    # the daemon's cwd is a/
cd -- "${base}/b" && agent-browser --session cwd-probe screenshot out/rel.png
```

Observed outcomes:

- Caller in `b/` (which has `out/`),
  daemon in `a/` (which has none),
  relative path:
  `✗ Failed to save screenshot to out/rel.png: No such file or directory (os error 2)`.
- Absolute path into `b/out/`:
  `✓ Screenshot saved to <absolute path>`,
  and the file is there.
- Caller in `a/` after `mkdir a/out`,
  relative path:
  `✓ Screenshot saved to out/rel.png`,
  and the file is in `a/out/`.
- `a/out/` and `b/out/` both present,
  caller in `b/`,
  relative `out/same-name.png`:
  `✓ Screenshot saved to out/same-name.png`,
  exit code 0,
  and the file landed in `a/out/`,
  not `b/out/`.

The last case is the dangerous one:
it reports success with a relative path that looks right,
and the file is somewhere else.

## Root cause

Source checked:
`vercel-labs/agent-browser` tag `v0.39.1`,
commit `1165952d74ceed98fdc2dd97190ad41f44dd0e24`
(`cli/Cargo.toml` says `version = "0.39.1"`).

### The CLI sends the path string unchanged

`cli/src/commands.rs:852-880` picks the path argument and puts it into the command JSON as given:

```rust
let (selector, path) = match (positional.first(), positional.get(1)) {
    // ...
};
let mut cmd = json!({
    "id": id, "action": "screenshot",
    "path": path, "selector": selector,
    "fullPage": full_page, "annotate": flags.annotate
});
```

No `current_dir`,
`canonicalize` or `absolute` call touches it in `cli/src/commands.rs` or `cli/src/main.rs`
(`rg -n "current_dir|canonicalize|absolute"` over `main.rs`,
`commands.rs`,
`connection.rs` finds only session-scope
and executable-path uses,
at `cli/src/main.rs:600-629` and `cli/src/connection.rs:868`).

### The daemon inherits the working directory of whichever CLI process spawned it

`cli/src/connection.rs:877-890` spawns the daemon with no `current_dir`,
only `setsid`:

```rust
let mut cmd = Command::new(&exe_path);
cmd.env("AGENT_BROWSER_DAEMON", "1");
apply_daemon_env(&mut cmd, session, opts);

unsafe {
    cmd.pre_exec(|| {
        libc::setsid();
        Ok(())
    });
}
```

So the daemon's working directory is the directory of the first command of that session,
and it stays there for up to the one hour idle timeout.
Restarting the daemon (a version mismatch,
an idle exit,
a `close`) from another directory changes it silently.

### The daemon writes the raw string

`cli/src/native/actions.rs:6048` copies `path` into `ScreenshotOptions`,
and `cli/src/native/screenshot.rs:549` and `cli/src/native/screenshot.rs:568-569` use it verbatim:

```rust
Some(path) => path.to_string(),
// ...
std::fs::write(&save_path, &bytes)
    .map_err(|e| format!("Failed to save screenshot to {}: {}", save_path, e))?;
```

`std::fs::write` resolves a relative path against the process working directory,
which is the daemon's.
The success message prints the same unresolved string,
which is why the wrong-directory case looks fine.

Other daemon-side path code does this on purpose:
`cli/src/native/actions.rs:7740-7746` (`handle_download`) joins a relative path onto
`std::env::current_dir()` of the daemon.

### A second classification trap for the single-argument form

`cli/src/commands.rs:857-871`:
with one positional argument,
a leading `.` that is not `./` or `../` makes the argument a selector,
so `screenshot .shots/a.png` is not treated as a path.
An absolute path starts with `/` and avoids that too.
This is recorded on the upstream thread (see the filing decision),
not reproduced here.

## Verification

Version under test:
agent-browser 0.39.1,
tag `v0.39.1`,
commit `1165952`,
headless Chrome 149.0.7827.54.
Probes used throwaway sessions and directories under `${HOME}/temp/agent`.

- Works:
  absolute path (verified from a caller directory different from the daemon's).
- Works:
  relative path when the caller's directory is the daemon's directory.
- Fails loudly:
  relative path whose parent directory exists only in the caller's directory.
- Fails silently:
  relative path whose parent directory exists in both;
  the file goes to the daemon's directory.
- Not exercised here:
  `pdf`,
  `upload`,
  `state save`,
  `trace stop`,
  `record start`;
  the thread for issue 137 lists them as affected.

## Verified workarounds

### Pass absolute paths

```sh
agent-browser --session probe screenshot "$(pwd)/chr/index-1440/00.png"
```

Verified in all three cases above.
Tradeoff:
the caller must create the directory first (the CLI does not create parent directories for an explicit path),
and every call site must resolve the path itself.

### Pin the daemon's directory

Start every session from one fixed directory
(for example by running the first `open` with `cd -- <dir> &&` in a wrapper),
then relative paths mean that directory.
Not exercised beyond the observation that the daemon keeps its spawn directory.
Tradeoff:
any daemon restart from another directory moves the base again,
so this is weaker than absolute paths.

## What does not work

- Assuming the success line proves the destination:
  it prints the path as typed.
- Creating the directory only in the caller's directory:
  the daemon does not look there.
- Reading `/proc/<daemon pid>/cwd` once and trusting it:
  a restart changes it.

## Upstream filing decision

`.out-of-scope/` was searched with `rg -n -i "agent-browser|browser" .out-of-scope` (exit code 1,
no match);
no exemption applies.
Duplicate search (`gh search issues` and `gh search prs`,
open and closed,
on `vercel-labs/agent-browser`) found:

- [Issue 137][issue-137] (open since 2026-01-16):
  "Relative file paths resolve to daemon's working directory,
  not CLI's".
  It names `screenshot`,
  `pdf`,
  `trace`,
  `state`,
  `upload` and carries the same root cause.
  Two comments exist,
  both from May and September 2026 on the leading-dot classification trap.
- [PR 667][pr-667] (open):
  "fix:
  resolve relative file paths in CLI before sending to daemon",
  touching only `cli/src/commands.rs`.
  `git apply --check` of its diff against `v0.39.1` fails (`error: patch failed: cli/src/commands.rs:475`),
  so it needs a rebase.
- [Issue 1534][issue-1534] (open) and its closed,
  unmerged [PR 1588][pr-1588] mention the same daemon-cwd resolution
  and a clearer error for a missing parent directory.

Nothing here is absent from issue 137 except the 0.39.1 retest,
the daemon-spawn source lines and the silent wrong-directory case.
That is a small additive comment,
kept as a draft and not posted:

~~~md
Retested on 0.39.1 (tag v0.39.1, Linux x64): still reproduces. The daemon is spawned without `current_dir`
(`cli/src/connection.rs:877-890`), `screenshot` sends `path` unresolved (`cli/src/commands.rs:879`), and
`save_screenshot` calls `std::fs::write` on the raw string (`cli/src/native/screenshot.rs:549,568`).
When the same relative directory exists under both the caller's and the daemon's working directory, the command
exits 0 and prints `Screenshot saved to out/x.png` while the file is written under the daemon's directory.
PR #667 no longer applies to v0.39.1 (`git apply --check`: patch failed at `cli/src/commands.rs:475`).
~~~

1.  **Upstream fault?**
    Yes in effect:
    the success message hides where the file went;
    whether the decoupling was
    intended is not stated,
    and no documentation of relative-path semantics was found in `README.md`.
2.  **Can upstream fix it?**
    Yes;
    PR 667 does it in the CLI.
3.  **Supported use case?**
    Relative output paths are the documented example form (`README.md:97`,
    `agent-browser screenshot page.png`).
4.  **Contribution welcome?**
    No `CONTRIBUTING.md`,
    templates or filing policy;
    the clone's `.github/` has only `workflows/`.
    PR 667 is marked generated with Claude Code and was not rejected.
    No AI-assistance ban was found.
5.  **Will they likely fix it?**
    Uncertain:
    issue 137 is nine months old and PR 667 is unmerged.
    That is absence of signal,
    not a documented refusal,
    so it does not fail.
6.  **Minimal fix prototyped?**
    Not by us.
    The fix already exists as PR 667 (stale),
    and the only artifact is an additive
    comment,
    not a new issue,
    so a duplicate prototype in a disposable clone would add nothing the thread lacks.
    No new-issue draft is kept.

[issue-137]: https://github.com/vercel-labs/agent-browser/issues/137
[issue-1534]: https://github.com/vercel-labs/agent-browser/issues/1534
[pr-667]: https://github.com/vercel-labs/agent-browser/pull/667
[pr-1588]: https://github.com/vercel-labs/agent-browser/pull/1588
