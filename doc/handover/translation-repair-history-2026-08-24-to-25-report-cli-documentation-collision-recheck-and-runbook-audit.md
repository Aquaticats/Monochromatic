# Translation repair history 2026-08-24 to 2026-08-25: documenting the three report commands, re-checking the landing for collisions and auditing the runbook against the source

The documentation of the three report commands,
found necessary by measuring what the readiness question required,
how the documented output was obtained,
two things worth knowing before operating them,
a rough edge found while capturing the output,
the recheck that the landing was still safe,
and the audit of the runbook against the source.

Part of [Translation repair history: 2026-08-24 to 2026-08-25](translation-repair-history-2026-08-24-to-25.md),
which lists every file of this record.

Historical evidence only:
no next-action list here is a current instruction,
and the stop condition in the index applies.

## The three report CLIs are documented, and the landing was re-checked for collisions

Opened by asking what `#219` actually requires before production readiness can be
signalled,
and answering it by measurement rather than by assumption.

### What the measurement found

Of 47 mise tasks the package will carry after the landing,
the README named 6.
It did not name `corpus-pass`,
the primary entry point,
and it did not link
`doc/runbook/translation-repair-corpus-pass.md`,
which exists and carries the whole
operating procedure.
A reader of the design document had no route to running anything.

None of the three CLIs the landing adds,
`ledger-report`,
`run-timing-report` and
`spend-report`,
appeared in the README or the runbook.
Three user-facing tools were about
to land undocumented,
which is exactly what PKG exists to catch.

### How the documented output was obtained

Not from memory.
A throwaway worktree was cut at `852e84f3a`,
the parked tarball extracted,
the 40 recorded deletions applied,
and the package built clean.
Every block now quoted in
the runbook is output captured from that build.

For the populated cases the inputs were fixtures,
because the two run directories on disk
both predate the writers.
The timing fixture was hand-computed first:
rounds of 60000 and
30000 milliseconds with 40000 in grace,
and call intervals of `[0,10]`,
`[2,8]` and
`[15,20]` seconds.
The tool returned `1.50min`,
`40.00s` at `44.4%`,
`mean 1.05` and
`peak 2`,
matching the hand computation exactly.

THAT DOUBLES AS THE POSITIVE CONTROL.
It proves the reader can report non-zero,
so
`NO ROUND LINE` on the live calibration log is a true absence rather than a broken parser.
Without it the zero would have been an unvalidated null.

### Two things worth knowing before operating them

The three tools disagree on the exit code for "nothing recorded",
and the difference is
deliberate rather than an oversight.
`ledger-report` exits 1,
while `run-timing-report` and
`spend-report` exit 0.
An empty ledger usually means `TRANSLATION_REPAIR_RUNS_DIR` was never
set,
which is operator error worth failing on.
A log with no `SPEND` or round lines is
simply an older log and says nothing about the operator.
The runbook explains this rather
than smoothing it over.

`ledger-report --model <id>` prints candidate text verbatim,
which on a real run is corpus
wording from an unlicensed archive,
together with judges' reasons quoting it.
The runbook
now says plainly that its output must not be pasted anywhere.
The summary view carries only
model identifiers and counts and is safe to share.

### A rough edge found while capturing the output, filed as `#220`

A ledger file whose top level is an array rather than a single round object,
which is the
shape a reader would guess,
aborts the entire report with an uncaught `LedgerShapeError`,
a page of minified JavaScript,
and exit 1.
The message itself is good and names both the
file and the field.
Nothing catches it,
and `reportLedger` reads every file through one
`Promise.all`,
so a single truncated write destroys a report over every good file beside it.
A truncated write is the expected failure,
because the ledger is written during a run that
can be killed at any moment.

Not fixed now,
deliberately:
editing `ledger-report.ts` would invalidate the rehearsed
landing for a rough edge that costs nothing while the ledger is machine-written.

### The landing is still safe, and this was verified rather than assumed

The parked tarball was cut before the last doc commits,
which raised the question of whether
extracting it would clobber them.
It does not.
The tarball holds 124 entries under exactly
three prefixes,
all inside `package/module/translation-repair` (moved on 2026-10-18 to the packages under `package/translation-repair`),
and contains no `doc/` path
at all.
The only file changed on the branch since the rehearsal base `9569f9d79` is
`doc/handover/translation-repair.md`.
The intersection is empty,
so step 2 of the landing
procedure cannot overwrite a doc commit.

The landing procedure itself is unchanged.

### The runbook was audited against the source, and nothing in it is stale

Every environment variable it names exists in source:
both API keys,
`TRANSLATION_REPAIR_RUNS_DIR`,
and `TRANSLATION_REPAIR_HARD_CAP_MINUTES`.
Both flags it uses exist:
`--plan` at `src/corpus-run/corpus-pass.ts:513`,
and `--only` at `src/corpus-run/entry-filter.ts:24`,
read at `src/corpus-run/corpus-pass.ts:318`.
Every mise task it names exists,
apart from the three arriving with the parked work.

Every string it tells the operator to watch for is emitted by non-test source:

-   `PLAN ok tip=` at `src/corpus-run/corpus-pass.ts:515`,
    carrying exactly the `pipeline=`,
    `client=constructed`,
    `pending=` and `first=`
    fields the runbook claims it does.
-   `ONLY` at `src/corpus-run/corpus-pass.ts:328`.
-   `CAP OVERRIDDEN` at `src/corpus-run/corpus-pass.ts:491`.
-   `CAP TOO TIGHT` at `src/corpus-run/cap-override.ts:156`.
-   `REATTEMPT` and `STALLED` at `src/corpus-run/entry-attempt-queue.ts:109` and `:116`.
-   `METERS` at `src/provider-budget.ts:14`.

`RUNDIR` is set in Steps,
before What to check uses it,
so the additions are reachable
by a reader working through the document in order.

TWO SEARCHES LIED BEFORE THIS SETTLED,
both toward a false absence.
`rg --fixed-strings "--plan"` returned zero hits because `rg` read the pattern as its own
flag rather than as text.
An `ONLY` search capped with `head` at five lines hid the one
real emission behind four unrelated prompt strings.

Either would have read as "the runbook names something that does not exist",
and acting on
either would have meant editing a correct instruction into a wrong one.
Both are the QRY
failure mode exactly as it is written down,
and the uncapped,
flag-safe re-runs found
everything.
Worth remembering that the dangerous direction here is the empty result,
not
the noisy one.
