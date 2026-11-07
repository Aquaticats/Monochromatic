# repository-link-check

The shared check that tells a tool whether the repository link to the 860 drive is usable
(work item W1 of `doc/planning/repository-on-the-860-drive.md`).

## Bins

- `repository-link-check [--identity=<file>] [--for=git] [--deadline-seconds=<n>]`:
  prints the plain message and exits 0 when ready,
  75 (`EX_TEMPFAIL`) when not,
  or 128 with `--for=git` (Git's own fatal status).
  It creates nothing.
- `repository-link-run [--wait-seconds=<n>] [--] <command...>`:
  checks,
  then replaces itself with the command (`process.execve`).
  With `--wait-seconds` it asks again every second until the deadline.
- `repository-link-launch`,
  started through a link named like the program it stands for (`spawn-claude`,
  `spawn-pi`):
  checks,
  then replaces itself with that program (the table is `LAUNCHED_PROGRAMS` in `src/constants.ts`,
  paths below the repository link),
  run by the same Node with the arguments it was given.
  A link in `~/.local/bin` to this bin tells the person why the program is not there
  where a link into the repository would give the shell's bare "No such file or directory".
  A link name it does not know exits 2.
- `repository-link-record --phase=before|after`:
  the cut-over step that records the identity.
  The check only reads the file.

## Answers

Ready in phase `after` (a link to the recorded root on the recorded filesystem,
with the recorded marker
`.git/monochromatic-root-commit`) and in phase `before` (a real folder on the NVMe);
the drive is not mounted;
the drive is mounted but the folder is missing;
a real folder replaced the link;
the link is missing;
the link resolves to the wrong filesystem or the wrong repository (the message names both values);
the identity file is missing;
the drive did not answer within the deadline.

## Library

`checkRepositoryLink`,
the identity file reader and writer,
`recordIdentity`,
the real probe,
and `openRepositoryRoot()`,
which runs the check once and returns the resolved root as a directory descriptor,
so later calls never look the link up again.

## Install outside the repository

`mise run //package/repository-link/check:install:outside-repository` (after the build) copies the bins to
`~/.local/share/monochromatic-dev/repository-link/`.
The bins import only node's own modules,
so the copy runs with the repository away.

## Unverified

The stalled drive on a hung mount and a wrong filesystem mounted at the real mount point need root.
The deadline is tested against a probe that never answers;
whether the process exits promptly with a thread stuck in uninterruptible I/O is not testable here.
