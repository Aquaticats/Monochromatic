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
so their directory is `.../mise/tmp`.

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
`src/language/mise/state.rs`)
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

## What stays

- Two processes on one project still share one private `/tmp` per server:
  the launch of the second empties the files of the first one's running server.
  The first server's directory stays valid after the fix
  (it was a deleted directory before),
  but its temporary files are gone.
  A scratch directory per process (`tmp-<pid>`,
  with removal of the directories of dead processes) would remove the sharing;
  it was not done here.
- A bubblewrap failure for another cause shows the new notice,
  and its cause is whatever bubblewrap says.
