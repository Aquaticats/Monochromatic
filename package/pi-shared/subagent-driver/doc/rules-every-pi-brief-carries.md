# Rules every Pi brief carries

A brief for a Pi run names this file and tells the agent to read it first.
It holds the rules that bind every run on this machine,
so a brief itself only has to say what its own task needs.
Each rule is the project's own requirement.

## Rules that bind the run

- Rule `ATH`:
  scratch goes in `${HOME}/temp/agent`,
  never `/tmp`.
- The run has one scratch folder,
  `${HOME}/temp/agent/<the run's name>-scratch`,
  and everything it writes outside its worktree goes there.
  That folder is deleted when the run ends,
  except its `keep/` subfolder:
  put the report,
  the logs and the measurements a reader of the report needs in `keep/`,
  and nothing that a build or a clone recreates
  (build folders,
  checkouts,
  fixture repositories,
  binaries,
  dependency folders).
  Name every folder inside it for what it holds,
  so someone who was not there needs no context
  (`twenty-commits-timed-against-real-git`,
  never `runs` or `r3-before`).
  Make one fixture repository and reuse it,
  or make a copy-on-write copy (`cp --reflink=always`),
  instead of a full copy for each round.
  On 2026-11-13 the scratch folder held about 64 GiB of its own from two days of runs
  (`btrfs filesystem du`;
  `du` read 232 GiB because it counts data shared between copies once for each copy),
  and a full home drive had ended twenty runs the day before.
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
- Send a signal only to a process your run started,
  named by a process number your own command got back when it started it.
  A number taken from `$PPID`,
  from a parent's record or from a file is not that:
  a script whose parent has ended belongs to the account's service manager,
  and `$PPID` then names the manager.
  A probe that stops or kills "its parent" starts that parent itself and signals the number it was given,
  or runs in a process namespace of its own
  (`unshare --user --map-root-user --pid --fork --mount-proc`,
  where it sees its own processes and no others,
  or a container).
  On 2026-11-15 a review's stand-in Git ran `kill -STOP "$PPID"`,
  the number was the account's service manager,
  and for three minutes and five seconds the manager took no request:
  no agent on the machine could start a command in a scope or a unit,
  until the reviewer found the manager stopped and sent it `SIGCONT`.
- A fixture's own `HOME` and `XDG_RUNTIME_DIR` do not keep a program off this machine's session bus:
  `DBUS_SESSION_BUS_ADDRESS` is inherited from the run's environment and still names it.
  Unset it,
  or point it at a private `dbus-daemon` of the run's own,
  for every command that runs a wrapper,
  a service or any program that asks a service manager for something.
  On 2026-11-13 a measurement script that set the two directories and left the address
  had this machine's user manager start two commit service units for throwaway repositories;
  the run noticed,
  stopped and removed them.
- A program built in a worktree cannot open connections to outside hosts on this machine.
  Its application firewall (OpenSnitch) refuses a program it has no rule for:
  each `connect` ends with `ECONNABORTED`,
  and the firewall keeps a rule that rejects that program's path from then on.
  `curl`,
  `node`,
  `python3`,
  `cargo` and the installed tools are allowed.
  So a test or a probe of a built program takes what it needs from disk or from a server the test starts itself,
  and a report says so where a path through the network was left unrun for this reason.
  The firewall and its rules are not a run's to change.
  On 2026-11-13 a freshly built `mpm-shadow` could reach neither crates.io,
  the npm registry nor go.dev,
  while `curl` reached the same addresses from the same environment;
  `/var/log/opensnitchd.log` holds the two rules saved for its debug and release paths.
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
  `mise env` in any form,
  `sops --decrypt`,
  even piped through a filter or `head`),
  never print a file that holds credentials
  (`~/.bashrc`,
  `~/.pi/agent/models.json`,
  `~/.pi/agent/auth.json`,
  `.env.local.json` decrypted),
  and never read another process's environment (`/proc/<pid>/environ`,
  `ps e`,
  `ps -E`):
  it holds tokens,
  and everything a command prints is sent to the model provider.
  To learn one variable,
  name it (`printenv CARGO_TARGET_DIR`).
  That ban stands as before,
  and a run does not rely on it alone:
  see the next rule.
- A run that cannot read a place that holds a credential does not start:
  a store that is there and cannot be read,
  a file of Pi's that cannot be read,
  a start-up file whose credential line the driver cannot read exactly,
  or an age identity the machine declares and this code cannot read.
  "Not there" and "could not be read" are told apart by the operating system's own answer alone,
  so a folder that denies the way is a stop, never "nothing there",
  and the message names the file and, for a start-up file, the line.
- A run that resumes a session masks it into a copy first and resumes that copy;
  the session file it was given is left exactly as it was,
  which is the stated limit of resuming:
  the old file keeps whatever it holds,
  and nothing of it reaches a request.
  A free-form Pi argument that names or resumes a session is refused,
  because only the driver's own `--session` is masked before the start.
- What a run's Pi masks for you
  (the driver reads these values once, in its own process,
  and never writes them anywhere):
  every value the checkout's `.env.local.json` declares,
  the provider keys of Pi's own `~/.pi/agent/auth.json` and `~/.pi/agent/models.json`,
  a token the shell start-up files export
  (`~/.bashrc`, `~/.bash_profile`, `~/.profile`),
  the age key that decrypts the store,
  and a credential-named variable of the driver's own environment.
  They are replaced in each tool result,
  in every message Pi keeps in its session,
  in the whole transcript before every model request,
  in the run's feed,
  its summary and Pi's standard error,
  and in the session file once the run is over,
  by a marker that names where the value came from,
  as `[masked secret value of MISE_GITHUB_TOKEN]`:
  seeing that marker means the value you asked for was there and was taken out,
  so read the name and go on without the value.
  What is masked is the value alone and its JSON-escaped,
  percent-encoded and base64 forms;
  a value inside a message or a file someone encoded some other way
  is not one of those forms,
  which is why the ban above is unchanged.
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

  `cargo metadata` names a folder the build does not use;
  to learn where a build put its output,
  ask cargo with `--message-format=json`.
- Never walk `~/.cargo/build`,
  the build directory every agent shares:
  no `find`,
  `du`,
  `ls --recursive`,
  `rg` or `fd` over it,
  for any purpose.
  A walk takes most of an hour and slows every other agent while it runs
  (one `find` for a dependency's source ran twenty-eight minutes on 2026-11-12 before it was ended).
  A dependency's source is under `~/.cargo/registry/src/`,
  and `cargo metadata --format-version=1 --locked --offline` gives each one's `manifest_path`.
- On 2026-11-12 an integration run ended a turn with its build running in a `process` job,
  expecting to be brought back,
  and was closed as `idle` 129 seconds later.
  The driver now keeps the run while it can see work the agent left running,
  and sends the agent back to read the result when that work ends.
  The default idle window is `0`: when Pi has settled and nothing else holds the run,
  the driver does not wait for a new steering message.
  If the next step needs a result from work the driver cannot detect,
  run that work as a foreground command with a bound of its own.
- Detached work is started through the `process` tool,
  as a named unit (`systemd-run --user --unit=<name>`) when it should outlive that client too,
  so the driver keeps the run for it:
  a `systemd-run --scope` put in the background whose caller exits
  belongs to the service manager and keeps no run open,
  and a unit started any other way, from a `bash` call for example, has no record to keep the run.
- A workaround for a defect of the Git wrapper that is fixed in the installed release (0.6.1-alpha.15) and in `main`
  since commit `afc0545123`,
  and that a wrapper built from a branch without that commit still has:
  such a wrapper,
  when it looks for the holder of a file,
  reads the descriptor record of every process of the account,
  and one such read can cost hundreds of megabytes of kernel memory and minutes while an IDE's file watcher runs
  (on 2026-11-12 a wrapper in a scope capped at 512 MB was killed by the kernel for it,
  which raised a low-memory alarm on the desktop).
  So a scope or container that runs a wrapper built from your branch gets a memory cap of at least 4G
  until your branch holds that commit
  (`git merge-base --is-ancestor afc0545123 HEAD` answers);
  the installed wrapper needs no such cap.
  To be removed when no branch in work is older than that commit.
  And nobody reads `/proc/<pid>/fdinfo/<n>` of a process they did not start.
- Bound every read of the journal:
  `journalctl` gets `--since` and `--lines`,
  and one unit instead of a pattern where the unit is known.
  The journal of the commit-service units is very large:
  on 2026-11-12 one unbounded read of `monochromatic-git-policy-cli@*` took over eight minutes,
  and a run that kept asking it made fourteen turns in seventy minutes.
  Save what a read gave to a scratch file and work from the file.
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

## What another program prints

- A constant,
  fixture or test string that stands for another program's output
  (an error message,
  a status line,
  an API response,
  a log line,
  a command's output)
  is copied from a real capture or built the way the producer's source builds it,
  and the line beside it says which:
  the capture's file path,
  or the producer's installed `path:line`.
  A stand-in written by hand says it is a stand-in where it is defined,
  and no claim about real behaviour rests on it.
  On 2026-11-18 a writer's tests used refusal messages no installed Pi adapter produces,
  its README claimed on their strength that a real OpenAI refusal would hand off,
  and its review found the adapter's own form (`OpenAI API error (429): ...`,
  built by `formatProviderError` in `@earendil-works/pi-ai`) was not recognised.
- A review checks each such constant against its producer
  (the installed source or a real capture)
  and reports each one that has neither,
  and each claim about real behaviour that rests only on a stand-in.

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
Mark each claim about behaviour,
in the report and in any README or document the run writes,
with what it rests on:
a real invocation,
the producer's source,
or a stand-in.
A run cannot ask questions while it works,
so open questions go in the report.

## Nothing shows on the desktop

Start a program that opens a window only in its own headless mode,
or inside `~/Monochromatic/.cache/toolchain-bin/monochromatic-nested-wayland-session --headless`.
Probe a CLI that can launch a terminal or a browser only with a disposable `HOME` and a `PATH` without `terminal-exec`.
Desktop notifications are the exception and are wanted:
never disable,
silence or work around the Pi plugin that sends one when an agent settles
(`package/pi-plugin/agent-settled-notification`).
