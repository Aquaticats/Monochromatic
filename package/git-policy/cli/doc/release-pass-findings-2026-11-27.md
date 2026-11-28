# What the wrapper's `0.6.1-alpha.48` release passes found

Round of 2026-11-27,
unapproved and non-final.

Two passes read the release `0.6.1-alpha.48`:
a behaviour pass,
which exercised alpha.47 and alpha.48 side by side,
and a splits pass,
which read what the 344 refactor commits of the range did.
This record keeps the findings and the measurements that no other file holds,
in the passes' own words where they had them.
Every figure is the pass's own measurement,
taken on 2026-11-27;
nothing here was re-measured for this record.

The behaviour the release rests on is the code,
the gate,
and the service-manager install proof named under
"No change of behaviour is not demonstrated" below.

## The behaviour pass found no High finding

Neither pass raised a High finding.
The behaviour pass's two Mediums and six Lows were all fixed by rounds merged on 2026-11-27,
each with a control that fails without the fix;
what each said is history,
and the controls pin the fixed words.

The splits pass's Mediums and Lows are below.

## The behaviour pass: what held

These held,
exercised with both binaries:

- the slot refusals,
  for the spellings they name;
- the merge pass's union of deletions;
- the self-check's memory,
  62.7 to 66.1 MB in alpha.47 and 9.3 to 11.0 MB in alpha.48;
- named pipes,
  answered and not waited on;
- the typed ssh relay at a terminal,
  which ended about 100 ms sooner.

## The behaviour pass: read and not run

Three surfaces were read only,
because nothing outside the process can start them:
the prune race,
the conversion environment's precedence,
and the service-only changes.

## The splits pass: no change of behaviour is not demonstrated

Its verdict is that "no change of behaviour" is not demonstrated.
Of the items compared at both ends,
97.9 percent have identical text,
no test-only switch is in either release binary,
no static became two and no trait impl was lost;
but 318 of the 344 refactor commits carry changed lines that are not moves,
and the pass read three of them in full.

Neither pass ran `git commit` in a release binary,
because without a session bus every commit form stops at one refusal.
So for the service paths the evidence is the reading,
the gate,
and the service-manager install proof,
which issue 6587's second round ran to its end (`fdd9ce0e83`):
alpha.48 installed over alpha.47 in a container of the gate image,
and the proof printed

```text
THE PROOF PASSED: the instance the older release loaded started as this release, and the pair of commits each landed once
```

## The splits pass: reach

At least 31 definitions became nameable from outside the crate,
seven of them private helpers a split made bare `pub`,
one of which removes directories.
The list of published names is
`src/native/source_guard/items_nameable_from_outside_release.txt`,
and its control is
`src/native/source_guard/nothing_becomes_nameable_from_outside_tests.rs`,
which fails on a published name that has no line there.

## The splits pass: refactor commits read and not read

The extract-function refactors are unread but for ten commits.
A second read of the seven the pass named first,
all on the client's and the service's commit paths,
found each equivalent by reading
(`e90482f100`,
`dde5c4be3b`,
`bad45d0420`,
`01c2f32e2c`,
`6a24c6342b`,
`f4e8d2fb9b`,
`8af8f56255`).
One guard covers less than its name says:
`src/native/source_guard/hook_self_invocation_tests.rs` reads 263 of the 699 code lines
the commit preparation spans,
which the list-guards round took first.
About three hundred refactor commits stay unread.

## The splits pass: guards that name moved files

The pass found 71 pairs of a source guard and a file it names,
where code left the file in a split and the guard does not name where it went.
Two were checked,
and neither was a hole.
The list-guards round holds the pairs now.

## The splits pass: low findings

Eight points,
as the pass put them:

- the sweep's warnings carry four new log tags;
- a commit labelled `docs` changed the `shutdown-unmount` help row
  (`066e33a47d`,
  noted on the commit);
- the panic notice's source locations moved;
- 34 items lost a platform gate on split-off files,
  with no effect on x86_64 Linux;
- tag constants are copied into more files;
- the README's allow-list counts are stale;
- two commits in the range cannot build alone;
- the linter the binary carries moved four alphas.

The platform gates are fixed:
a split-off file keeps the platform gate of the file it left (`4d6f09f1f2`).

## The splits pass: a split that was not a pure move

Not every split is a pure move.
The landing split's last commit (`3b9ada357b`) boxed the commit inside the service's answers
and made the commit command pass its own copy of its client configuration,
both for Clippy after the move,
which the release pass that reads the range reads as a change.
