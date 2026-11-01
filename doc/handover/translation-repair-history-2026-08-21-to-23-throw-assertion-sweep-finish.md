# Translation repair history 2026-08-21 to 2026-08-23: finishing the throw-assertion sweep

The last conversions of throw assertions to class checks,
the one real defect the sweep caught,
the assertions skipped for a property of the converter,
the note that the runner's verdict lines count suites rather than tests,
and what remained open afterwards.
The false-citation chain its opening mentions is recorded in `translation-repair-history-2026-08-21-to-23-advisor-attribution-and-live-pass-warnings.md`.

Part of [Translation repair history: 2026-08-21 to 2026-08-23](translation-repair-history-2026-08-21-to-23.md),
which lists every file of this record.

Historical evidence only:
no next-action list here is a current instruction,
and the stop condition in the index applies.

## 2026-08-23: the throw-assertion sweep finished, and what finishing it turned up

`#127`,
`#180` and `#163` are all closed.
This section is the durable record of the finish,
because until it was written the gate numbers and the one real defect the sweep caught
lived only in commit messages and task descriptions,
which is precisely the state that produced the false-citation chain recorded earlier in this file.

### What landed

`9f61dc1aa` converted the last 24 throw assertions across nine test files:
`sheet-binding` 6,
`splice-slices` 5,
`select-candidate` 4,
`publish-fixed` 2,
`probe-telemetry` 2,
`bench-draw` 2,
`score-probe` 1,
`run-config` 1,
`damage-region-v2` 1.

`ab507eb76` then did the three that had been written down as named follow-ups rather than folded in
silently,
plus one absence check sitting beside them.

Gate on both,
run the same way each time:
`lint` 0 warnings 0 errors,
`lint:types` exit 0,
suite 531 PASS / 0 FAIL / exit 0,
identical to the pre-conversion baseline.

### The sweep caught one real defect, which is the whole argument for having run it

`spliceShuffledIndices` raises `SliceIndexingError`,
not `SliceSpliceError`.
`spliceSlices` delegates to `slice-indexing.ts`,
a module the test never names,
and the message-only assertion had been passing all along while the wrong class was thrown.
Nothing inside the test's own file could have revealed this:
the message matched,
and the message was all the old assertion read.

That failure doubles as the positive control the rest of the batch needed.
A wrong class does not pass quietly here,
because every error class in this package extends `Error` directly with no shared intermediate base,
so no sibling class can satisfy `toBeInstanceOf` by accident.

### Two assertions that were skipped for a property of the converter, not of themselves

`run-config:282` and `:295` assert `API_KEY_VAR`,
an identifier rather than a quoted string,
so the converter refused them by design and reported them.
They were never exceptions to the sweep's rule.
Both raise `RunConfigError`,
traced to the single `throw` in `run-config.ts`.

`publish-fixed:599` had been recorded as permanently out of scope,
because it rejects Node's `readFile` and Node raises no class of ours.
That reasoning stopped one step early.
The test claims the publisher wrote no page at all,
and a bare `.toThrow()` passes for ANY read failure,
so a page written unreadably would have satisfied a test whose name promises absence.
There is no class to assert,
so the discriminator is the code:
it now asserts `toHaveProperty('code', 'ENOENT')`.

### The runner's verdict lines count suites, not tests

Running one file and counting `] PASS ` returned 1 for `publish-fixed`,
which reads like a file that ran almost nothing.
It is not.
The runner emits one verdict per `describe` and concatenates every `it` name into that single line.
Read the names inside the line,
or read the exit code,
before concluding a run was empty.
This is `TLY` one level up:
the count disagreeing with the exit code is the count's bug.

### What is open

`#181` is the only thing the verification opened,
and it is a DECISION item rather than a fix item:
`settled-neither` ships the archive with the gate unasked,
including at a slice where the size note fired.
Three shapes are recorded on the task and none should be built before the shape is chosen.

The measurement that does not need the decision,
and which the task's own text sanctions:
how often does `settled-neither` coincide with a tripped incumbent tail across settled artifacts?
One entry gave 1 of 5 contested slices,
and one entry is not a rate.
The two readers that between them read both halves are
`~/temp/agent/163-ballots.mjs` and `~/temp/agent/163-tail-census.mjs`.

The alignment cluster,
`#68`,
`#90`,
`#91`,
`#94`,
`#98`,
`#100` and `#106`,
stays where it was.
`#98`,
`#100` and `#94` gate on `#106`,
and `#106` gates on question 28 in `doc/planning/translation-repair-open-decisions.md`.
