# IDE language sandbox: `bwrap: Can't find source path .../tmp` when two processes prepare one project

Measured on 2026-10-25 with the IDE branch `worktree-agent-a763037d1a881724e`
(installed binary `monochromatic-ide` of 16:02 as the old arm),
the nested compositor 0.6.1 (`--headless`),
the generated 1,500-line `long.ts` of the row-map work,
and the project's TypeScript 7.1.0-beta server.

## Symptom

An information row of the IDE reads
`mise said: bwrap: Can't bind mount ... on /tmp: ... No such file or directory`
(or `bwrap: Can't find source path .../language/proj-<hash>/mise/tmp: No such file or directory`,
or `Can't open source ...`),
and the row map of the window is taller by the height of that row (90 to 154 px).
The same text reaches the reader as `typescript-native failed to start: bwrap: ...` when the failing launch is the server's.
It appeared in 2,
1 and 4 of 160 sessions in three arms of the row-map measurement,
which started three IDE processes at a time on one project with one cache folder.

## Root cause

Which component prints it:
bubblewrap (`/usr/bin/bwrap`),
the sandbox of the language servers and of mise,
before the confined program starts.
Mise itself never ran in these sessions.

What it is bound to:
every launch binds the private `/tmp` of the server,
`$XDG_CACHE_HOME/monochromatic-ide/language/<project>-<hash>/<server>/tmp`,
over `/tmp` of the sandbox (`--bind <state>/tmp /tmp`,
`src/language/confine/recipe.rs`).
The mise runs of one resolution use the server name `mise`,
so their directory is `.../mise/tmp`
(since 2026-10-29 the equivalent launch is the toolchain lookup, `launch_request` in `src/language/toolchain/command.rs`,
with the server name `toolchain`).

Why the source is missing:
`prepare` (`src/language/launch/state.rs`) emptied a scratch directory by `remove_dir_all` and then `create_dir_all`.
The path is the same for every IDE process of the same user and project,
because the state folder is named by project and server only.
A second process that prepares the path after the first has prepared it and before the first's bubblewrap opens the source
finds,
or leaves,
a moment in which the directory does not exist,
and bubblewrap stops.
The window is milliseconds wide
(`prepare`,
then the spawn of bubblewrap,
then bubblewrap's own mount setup).

The three components checked and ruled out:
a task runner (none runs),
the IDE's reread of directory safety (`check_state_directory` only reads),
and a temporary directory of another program removing the path
(no other writer of that folder exists in the code,
and the sessions with a private cache folder each never showed it).

## Reproduction with counts

Input-free sessions (no wheel,
no click),
3 s after the server's ready line,
a log search for `bwrap:`:

- Old binary,
  three at a time on one shared cache folder and one project,
  both CPU settings:
  1 of 120 (a first group),
  1 of 300 (mixed with the fixed binary,
  only the old binary's own failure counted),
  2 of 289 (a second group).
  In all,
  4 of 709.
  The failing launch was mise's in 3 and the server's in 1.
- Old binary,
  three at a time,
  but each session with a cache folder of its own:
  0 of 120.
- Old binary,
  one at a time on one shared folder:
  0 of 120 (60 with the one-core limit,
  60 without).
- Fixed binary,
  three at a time on one shared folder:
  0 of 300.
  The two failures seen when old and fixed binaries ran together on one folder were the old binary's
  `remove_dir_all` removing the path under a fixed process's launch (both arms must share no folder with an old process).

These counts do not separate the two binaries on their own
(2 of 289 against 0 of 300:
Fisher exact test,
one-sided,
p = 0.24):
the race needs two processes inside a few milliseconds of each other.
The deterministic evidence is the test below.

## Fix

`empty_in_place` (`src/language/launch/state.rs`) removes what is inside the scratch directory and keeps the directory.
A link or a file where the directory belongs is replaced by a directory and never followed.
An entry removed by another process at the same time is skipped.

- `preparation_empties_a_scratch_directory_without_replacing_it` (`src/language/launch_tests.rs`)
  holds the directory open across a preparation and needs it still linked and still the directory at the path.
  Against the removing version it fails
  (`the scratch directory was removed: a sandbox that holds it keeps a deleted directory`);
  with the fix it passes.
- `preparation_does_not_follow_a_link_where_the_scratch_directory_belongs`:
  a link out of the state root is refused and what it led to stays,
  a link inside is replaced and what it led to stays.
- Guard case `scratch-kept-in-place` of `bin/inspect-language-guards.ts`.

## Can a real user hit it

Yes,
with two IDE windows (processes) on one project at the same time,
which share the cache folder and the state folder of the project;
the rate measured here is about 0.6 percent of launches for three processes started together,
and it is lower when the processes start apart.
Another program that empties the folder `~/.cache/monochromatic-ide/language` while the IDE runs would cause the same message;
no program in this repository does.

## What the reader sees

The mise path gives a notice with the raw sandbox text:
before the change it said `Fix what mise names (mise doctor ...)` and `mise said: bwrap: ...`,
which sent the reader to a program that never ran.
Now (`Failed::stopped_by_sandbox`,
`src/language/mise/state.rs`;
moved on 2026-10-29 to `src/language/toolchain/state.rs` when the mise runs were replaced by the toolchain lookup)
it says the sandbox that runs mise stopped before mise started,
that the project's mise configuration is not the cause,
that the message does not say why,
that displaying a file again asks again,
and what to check (`bwrap --version`,
the private folder's existence,
space and permission,
another window on the project);
then it quotes bubblewrap.
The server path (`failed to start: <stderr lines>`,
remedy:
display the file again) already named the input and the remedy.

## Second fix: a scratch directory for each process

Emptying in place stopped the `bwrap:` message but left the sharing:
a second process on the project prepared the same path,
emptied it,
and took the temporary files of the first one's running server.
The state folder is named by project and server only,
so every process of the same user and project met in `<server state>/tmp`.

### Where the scratch lives now

Each process binds `<server state>/tmp-<process id>-<hex nanoseconds at first use>` as the server's `/tmp`
(`scratch_name` in `src/language/launch/scratch.rs`,
used by `recipe` in `src/language/confine/recipe.rs` for both the bind and the directory `prepare` empties).
The name is made once per process,
so a server that starts again in the same process gets a clean directory at the same path,
emptied in place as before.
The nonce keeps two live processes apart when two process-id namespaces share one cache folder.
Every server of a project has its own state folder,
so a process holds one scratch directory per server it has launched
(the toolchain lookup, `launch_request` in `src/language/toolchain/command.rs`, uses the server name `toolchain`).

### How a dead owner is detected

A live process holds an exclusive `flock` (`File::try_lock`) on `tmp-<...>.lock`, beside its directory,
from its first launch of that server until it ends.
The lock file is made before the directory,
and its descriptor stays open in a process-wide list,
so the lock lasts as long as the process.
Every preparation (`reap_ended`) lists the sibling entries whose names match `tmp-<digits>-<hex>` exactly
and are directories.
For each, it opens the lock file and tries to lock it:

- The lock is free (or the lock file is missing): the owner ended,
  by exit,
  crash,
  or `kill -9`,
  because the system releases a lock with its process.
  The directory and the lock file are removed while the remover holds the lock,
  so two removers cannot both succeed.
- The lock is held: the owner lives,
  and the directory stays.

A process id alone would not do:
it is reused,
and it says nothing across process-id namespaces.
A failed removal is logged (`tracing::warn!`) and does not stop the launch.
Names that do not match are never touched:
the shared `tmp` of earlier versions (an older process may still run in it),
`cache`,
`target`,
files,
and links.

### Tests

- `a_scratch_directory_is_named_for_the_process`
- `a_second_process_does_not_empty_the_scratch_directory_of_a_running_first_one`:
  starts a real second process (the test binary again, one `#[ignore]` helper test) on the same state folder,
  and needs the first one's file to survive its preparation,
  and the second one's file to survive a restart of the first.
- `the_scratch_directory_of_an_ended_process_is_removed_by_the_next_preparation`:
  the helper stays alive while this process prepares (its directory must stay),
  ends,
  and the next preparation must remove its directory and lock file.
- `a_scratch_directory_whose_lock_is_held_is_kept_and_one_without_a_lock_is_removed`:
  the test holds a lock itself (a lock taken through another open of one file conflicts even inside one process).
- `entries_that_are_not_scratch_directories_of_a_process_are_left`
- `a_process_empties_its_own_scratch_directory_in_place_and_keeps_it`

The tests were written first and fail against the shared name (6 of 6 new tests; the commit
`test(desktop-app-ide): a second process must not empty ...` holds that state).
Guard cases in `bin/inspect-language-guards.ts` (each observed failing with the code removed and passing again):
`scratch-per-process` (the process-specific name),
`scratch-lock-held` (the process keeps its lock),
`scratch-ended-removed` (the removal of an ended process's directory).

### Measurement

Tool: a scratch script `measure.ts` (not in the repository) that starts `ide-language-inspect`,
which runs the production `LanguageWorker` with the production bubblewrap launch,
three at a time on one project and one cache folder,
with the scripted server `ide-scripted-lsp` (it creates `/tmp/marker-<id>` at its start).
Old arm:
the debug binaries built in the main checkout on 2026-10-29 at 00:45
(they contain `empty_in_place`; no `tmp-` directories).
New arm:
this branch.
Arms alternated per round (old, new, old again),
each with a cache folder of its own,
and the second old arm is the same-binary control for the spread.
The host load average read 89.8 over one minute on 28 cores at 01:19 on 2026-10-29, before the run started;
the run took whatever load other sessions put on the host.

Sharing trial (30 per arm).
Process A starts and writes its marker in its `/tmp`;
processes B and C start together after that and finish;
the marker of A is read from the host path while A's server still runs.

- Old binary, first cache: A's marker gone in 30 of 30 trials.
- Old binary, second cache (same-binary control): 30 of 30.
- New binary: 30 of 30 markers kept (0 lost).

The old arm is the positive control:
the harness reports the loss on the old code every time.
The marker is read from the host side of the bind,
the same files the server sees as `/tmp`;
the server itself did not check for its file.

Burst trial (200 rounds per arm, three processes started together, 600 launches per arm):
`bwrap:` in the output of 0 of 600 launches for the first old arm,
0 of 600 for the second,
0 of 600 for the new binary.
These zeros do not separate the binaries:
the old arm already has the in-place emptying that ended the `bwrap:` stop,
so this run has no positive control for that message
(the earlier rate of the removing version was 4 of 709).
The new binary shows no new cause for it.

Kill trial (10, new binary):
a process killed with `SIGKILL` left its directory in 10 of 10
(nothing can clean up after it),
the next launch removed it in 10 of 10,
and a second live process's directory stayed in 10 of 10.
At the end of the run, each old arm held 1 directory (`tmp`)
and the new arm held 2 (the directories of the last two processes, both ended,
which only a later launch would remove).

### What stays

- A directory of an ended process stays until the next launch of the same server on the same project.
  It holds only what the server left in its `/tmp`.
- The `tmp` directory of earlier versions is never removed by the new version,
  so one remains in each server state folder that an older version used.
- The sandbox binds the whole server state folder writable,
  so a server can delete a neighbour's lock file or directory.
  That is the same trust boundary as before (a server could always empty its own state folder),
  and the effect is bounded to temporary files.
- Two launches of one server inside one process (two toolchain lookups of one project at the same time)
  still share the process's directory.
  Whether the application ever runs two such lookups at once was not checked.
- The measurement used server launches only;
  the toolchain lookup uses the same `prepare` and the same recipe,
  but it was not run in the harness.
- A bubblewrap failure for another cause shows the new notice,
  and its cause is whatever bubblewrap says.
