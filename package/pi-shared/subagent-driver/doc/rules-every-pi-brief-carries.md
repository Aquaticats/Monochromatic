# Rules every Pi brief carries

A brief for a Pi run names this file and tells the agent to read it first.
It holds the rules that bind every run on this machine,
so a brief itself only has to say what its own task needs.
Each rule is the project's own requirement.

## Rules that bind the run

- Rule `ATH`:
  scratch goes in `${HOME}/temp/agent`,
  never `/tmp`.
- Never generate synthetic host load in any session or subagent:
  no `stress-ng`,
  busy loops,
  `--stress-count`,
  or repeated full test runs;
  cap the test's own container (`--cpus` sized to the test) instead.
- Run every command that is not trivial in its own cgroup,
  as `AGENTS.md` says (rules `RXI` and `RXS`).
- A probe that mounts anything,
  FUSE above all,
  runs in a private mount namespace (`unshare --user --map-root-user --mount --propagation private`),
  and one that stops a filesystem server does so from a process that can be killed,
  with a deadline around the mount's life.
  A FUSE mount in the shared table whose server stopped once left another agent's `find` under `${HOME}/temp/agent`,
  and the probe itself,
  in uninterruptible sleep,
  and `timeout` could not end either.
- Never start a service manager on this machine (`systemd --user`,
  `/usr/lib/systemd/systemd` in any mode),
  even under `timeout` or in a scope,
  and never request a well-known name on this machine's session bus.
  A test or probe that needs a manager runs inside a container
  (`podman run` with `--cpus`,
  `--memory`,
  `--pids-limit`)
  or a throwaway VM (`mvm`),
  whose bus is not this machine's.
  A private `dbus-daemon` on a socket in the run folder,
  with `DBUS_SESSION_BUS_ADDRESS` pointed at it,
  is fine;
  it never joins this machine's bus.
  On 2026-11-11 a probe's nested `systemd --user` took the name `org.freedesktop.systemd1` on the session bus
  for almost three hours,
  and every `systemctl --user` and `systemd-run --user` call in that time went to it.
  A guard now kills any process that takes that name,
  and the run that started it ends with it.
- The run has its own Git worktree on its own branch.
  Other agents work in other worktrees,
  and the main checkout holds other sessions' uncommitted changes.
  Never restore,
  stash,
  reset or revert anyone else's work,
  and edit only the files the task needs.
  If dependencies are missing,
  run `mise exec -- pnpm install --offline --frozen-lockfile` in the worktree.
- Call Git as plain `git`,
  never `/usr/bin/git`:
  an absolute path skips this repository's guards against discarding work.
  Never restore,
  reset or clean anything in the main checkout,
  even an edit the run made itself:
  leave it and say so in the report.
- Never print the environment (`env`,
  `printenv`,
  `set`,
  `export -p`,
  even piped through a filter),
  and never read another process's environment (`/proc/<pid>/environ`,
  `ps e`,
  `ps -E`):
  it holds tokens,
  and everything a command prints is sent to the model provider.
  To learn one variable,
  name it (`printenv CARGO_TARGET_DIR`).
- Never write to this repository's Git config (`git remote set-url`,
  `git config` outside a throwaway repository):
  the worktree shares it with every other agent and the main checkout.
- To get a Rust test binary's path,
  ask nextest from the package folder with the task's own flags:

  ```sh
  # run in the package folder, for example package/git-policy/cli
  mise exec -- cargo nextest list --locked --offline --lib --message-format json \
    | jq --raw-output '."rust-suites"[]."binary-path"'
  ```

  Walking `/var/mnt/encrypted/cargo-build` takes most of an hour each time and slows every other agent,
  and `cargo metadata` names a folder the build does not use.
  Never run `du` or `find` over that folder;
  to learn where a build put its output,
  ask cargo with `--message-format=json`.
- Do not edit `doc/decision/`,
  or anything `file-enforcer` generates.
- Nothing may wait forever,
  and that covers the run's own commands:
  run every probe and test harness under `timeout`,
  and give each step inside it a deadline.
  A tool call that never returns cannot read a note sent to the run.
- Do not publish a release or dispatch a workflow.
  If one should be run,
  say which in the report.
- Windows and macOS are out of scope for now,
  and mutation campaigns are paused:
  run none.

## Where the run works

The working folder is the worktree the run was started in.
Run every command and make every edit inside it.
The main checkout (the repository path without `.claude/worktrees`) is read-only for the run:
an edit left there blocks merges for everyone,
and a build or test run there does not test the run's code.

## Committing

Stay on the branch that is checked out in the worktree.
Commit the run's own work there with explicit paths,
as `AGENTS.md` says,
naming every new file,
and commit often:
a failing test may be committed before its fix.
Do not merge into `main`;
the coordinating agent merges the branch after the run ends.
Quote each new test's failing output in the report.

## The final report

Give each change,
each commit hash,
what was run and its results,
each disagreement with the brief's reading,
and anything that could not be done.
A run cannot ask questions while it works,
so open questions go in the report.

## Nothing shows on the desktop

Start a program that opens a window only in its own headless mode,
or inside `/var/home/user/Monochromatic/.cache/toolchain-bin/monochromatic-nested-wayland-session --headless`.
Probe a CLI that can launch a terminal or a browser only with a disposable `HOME` and a `PATH` without `terminal-exec`.
Desktop notifications are the exception and are wanted:
never disable,
silence or work around the Pi plugin that sends one when an agent settles
(`package/pi-plugin/agent-settled-notification`).
