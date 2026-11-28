# Second-pass reads of the alpha releases: what each confirmed

What the second pass over each alpha release of this package confirmed with the
released binaries,
one section per release,
and what this machine showed of the one condition a Medium finding needed.

Status:
**unapproved and non-final**.
Written by the coordinating session from the second passes' reports.
Each pass was another agent of the same vendor,
so a second pass and not an independent review,
and each is named by the release it read.
Every figure below is one that pass measured.
What each confirmation says about the tool's own behaviour stands in this
package's `README.md`;
this file holds only what the released binaries themselves showed,
on fixtures with real kills.

## The alpha.14 pass: what held as claimed

The pass compared `0.2.7-alpha.14` with `0.2.7-alpha.13`.
Right as claimed:

- the exchange leaves no absence:
  none in 25 rounds on ZFS and on tmpfs,
  where `0.2.7-alpha.13` showed one in 3 and in 1 of 25,
  and 4 of 4 with its first rename slowed;
- an unfinished copy and another version's finished copy were never published;
- whether a refused start is repeated did not change;
- 53 command lines are identical between the two binaries.

## The alpha.14 pass: what this machine shows of `hidepid`

This machine ran `0.2.7-alpha.14` until `0.2.7-alpha.15` was installed at
2026-11-27T13:29.
`findmnt /proc` showed no `hidepid` here,
so the reading a live installer the reader cannot see gets was never met on
this host,
and the race a Medium finding needs is the one that needs a killed install's
leftovers and two installs at once.

## The alpha.15 read: what the released binaries showed

The pass compared `0.2.7-alpha.15` with `0.2.7-alpha.14`.
Shown with both released binaries on fixtures with real kills:

- a claimed finished copy that could not be published stays for the next
  install,
  which publishes it with no download;
- the copy a kill moved aside is not put back beside a live claim or a live
  staging name;
- a recovery never publishes an older version beside a newer one in either
  toolchain root;
- each of the launcher's five start messages names the file that is missing or
  busy.

With no killed install behind a run,
nothing differs between the two releases but the version and the marker's text.

## The alpha.16 read: what the released binaries showed on fixtures with real kills

The pass compared `0.2.7-alpha.16` with `0.2.7-alpha.15`.
Shown with both released binaries on fixtures with real kills:

- the order of a recovery's last two steps,
  where a start in the window exits 127 on `0.2.7-alpha.15` and runs the newer
  version on `0.2.7-alpha.16`;
- the raced publish,
  where `0.2.7-alpha.15` ends 1 with both hidden copies kept and
  `0.2.7-alpha.16` ends 0 with the other install's copy as the version;
- every message of the round on a hidden name nobody can judge;
- all twelve text blocks of the README's section,
  byte for byte.

On input with nothing left behind,
38 commands compared between the two binaries differ in the version string
alone.

## The alpha.17 read: what the released binaries showed

The pass compared `0.2.7-alpha.17` with `0.2.7-alpha.16`.
It confirmed with the released binaries that:

- 38 commands on an input with nothing left behind differ in the version string
  alone,
  as the alpha.16 read found of its pair;
- a move of `current` that fails once and a put-back behave in
  `0.2.7-alpha.17` as in `0.2.7-alpha.16`;
- the README's blocks for a kill match what `0.2.7-alpha.17` prints.
