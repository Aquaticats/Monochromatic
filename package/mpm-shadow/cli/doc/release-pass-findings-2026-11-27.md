# What the `mpm-shadow` release passes found over `0.2.7-alpha.14`

Round of 2026-11-27,
unapproved and non-final.

This record keeps what the release passes of this package measured and reported
where no other file holds it:
the alpha.12 pass over alpha.11,
the alpha.13 pass,
and the signature reading the alpha.15 read took against alpha.14.
Every figure is the pass's own measurement of 2026-11-27,
and nothing here was re-measured for this record.

## No pass raised a High finding

The alpha.12 pass and the alpha.13 pass each ended with no High finding.
The alpha.13 pass's three faults were fixed by the round merged on 2026-11-27,
and `doc/troubleshooting/mpm-shadow-a-start-refused-because-the-program-is-being-written.md`
and `doc/troubleshooting/mpm-shadow-a-start-that-does-not-happen-says-what-is-known.md`
hold what the fixed words say.

## Alpha.12: no change of behaviour over alpha.11

The claim of no change of behaviour holds.
Of 929 production items,
891 are identical token for token,
31 differ only in layout,
five are real edits that the splits needed
(the dispatch split into five functions,
and four structs whose fields became visible to sibling files),
and none was removed.
All 780 tests survive.
216 of 223 command lines gave the same output,
status and written files from both binaries,
and the other seven differ only in the version string.

Its one Medium is that the binary is built by a newer compiler and linker,
and by a lock that moved two crates,
which no diff of the crate shows.

## Alpha.13: what held

43 commands were compared between the two binaries,
41 identical and two differing only in the version string.

## Alpha.15 read: signatures changed since alpha.14

`being_written_cause_and_remedy_for` lived only in commits between the two releases
and never shipped,
by the alpha.15 read's `git log -S`.
Against `0.2.7-alpha.14` that read lists the changed signatures as
`publish_staged`,
`write_complete_marker`,
`being_written_cause_and_remedy`,
`describe_nesting_refusal`
and a new field `StartFailure.lookup`.
