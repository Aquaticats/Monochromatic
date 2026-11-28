# What the `cargo-shadow` release passes found over `0.1.5-alpha.10`

Round of 2026-11-27,
unapproved and non-final.

This record keeps what the alpha.11 and alpha.12 release passes of this package
measured and reported where no other file holds it.
Every figure is the pass's own measurement of 2026-11-27,
and nothing here was re-measured for this record.

The behaviour each release carries is in this package's `README.md`;
the fixes below are the release code,
each with a control that fails without it.

## Neither pass raised a High finding

The alpha.11 pass and the alpha.12 pass each ended with no High finding.

## Alpha.11: what the splits showed

The reformat is rustfmt's own output on its parent,
byte for byte.
The splits match every old item once and lose no `cfg`.
All 892 test names survive.

Three findings were routed to rounds and fixed:
the guard `no_lean_source_starts_cargo_or_itself` reads only the top of `src/`,
and so stopped reading 18 lean files the splits moved into subfolders;
48 public paths the splits added to the library were not meant to be published;
and issue 6552 threads a byte-count function through 18 items,
with every production call passing the same one.

## Alpha.12: what held

41 commands were compared between the two binaries,
identical but for the version line.

## Alpha.12: the decision does not read the words of the report

"Never what is decided" holds by reading:
both releases refuse exactly when the direct child was not reaped or the report is not empty,
and nothing in release code reads the report's words.

The stop's decision is written in `README.md` under the bounded run's containment,
and `doc/troubleshooting/a-cgroup-does-not-count-a-member-that-has-exited.md`
holds what the kernel's last word about a scope means.

## Alpha.12: the retire of a run's unit cost an ordinary run

The retire of a run's unit made the release slower.
A trivial shadow call took 1.52 times alpha.11's time at the median
(range 0.89 to 2.17 over 18 interleaved rounds,
against 1.00 and 0.68 to 1.25 for a same-binary control),
about 15 to 27 ms more per scoped run:
in ten of ten logged retires the manager had already collected the unit,
so both questions to it bought nothing.
With a manager that does not answer,
each run's end waits two five-second bounds.

The cost was removed by the round merged as `d0b821414c`:
the ordinary run asks the manager nothing and waits for nothing,
and the retirement's own bound applies only where the scope is the leftover of issue 6578
(`README.md`,
under the bounded run's containment).
