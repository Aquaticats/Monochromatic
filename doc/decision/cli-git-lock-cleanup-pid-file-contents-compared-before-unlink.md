# cli-git lock cleanup: the PID file's contents are compared before it is unlinked

## Status

Answered in the question tool on 2026-11-22,
written into this record on 2026-11-23T04:17:37-05:00.
Until then the answer stood only in the handover and in cause L of
`doc/troubleshooting/git-policy-cli-container-gate-on-a-zfs-host.md`,
which the second integration pass of issue 6342's branch reported as a gap.
The branch is merged (`1ba2fe767d`).
One part of the answer is not built as chosen;
"What is built and what is owed" says which.

## What was asked

When the wrapper clears an abandoned Git lock it also deletes the lock's PID file,
and only if that file is still the one it judged:
the same device,
inode and change time to the millisecond.
The first integration pass of the fix for issue 6342 found a real program that breaks this:
GitButler's `grit` rewrites its PID file in place,
so if it takes the freed lock within the same clock tick,
the wrapper deletes the live program's PID file
(never a lock,
only that file).
On this machine's ZFS the tick is coarse:
957 of 1000 quick rewrites had identical time stamps.

## What was chosen

The project chose "B:
also compare bytes (Recommended)":
"Before deleting the PID file,
read it again and delete only if its contents are also unchanged.
Pros:
closes the route on tmpfs and on ZFS.
Cons:
one more small read during abandoned-lock cleanup;
a program that rewrote the very same bytes in place would still lose the file,
which the round is told to state."

Not chosen:
"A:
correct the words only",
which left the code as it was and fixed the documentation.

## What is built and what is owed

This section is the coordinator's account,
not the project's words.

The merged code compares the number the PID file names with the number it judged
(commit `ffc177fbc0`).
That tells a writer of another number apart on every file system.
It does not tell apart a rewrite that leaves the same number in other bytes
(padding,
a leading zero,
a carriage return),
which is a change of contents.
The second integration pass ranked that a low finding and worded it as a matter of documentation.
Read against the option's own words,
"delete only if its contents are also unchanged",
it is a difference in behaviour:
the follow-up round on the pass's lows
(`pi-lock-control-second-pass-lows`,
started 2026-11-23)
was first told to document it,
and was then told to build the byte comparison instead,
with controls for each of those rewrites.

The limit the project was shown stays:
a program that writes the very same bytes back still loses the file,
and so does one that writes between the last read and the unlink.
`SPEC.md` and cause L state both.
