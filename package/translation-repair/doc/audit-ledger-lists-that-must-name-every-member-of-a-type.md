# Audit ledger: lists that must name every member of a type

Lists and switches meant to cover every member of a type that could silently miss one.

Part of [Audit ledger](audit-ledger.md),
which lists every file of this record.

## B169: a pre-grade marking a duplicate had its whole file refused

Red in `8f0d7dd6f`,
fixed in `2e02fbd4e`.

Found on 2026-10-05 (UTC)
by the second review of the trial's added cases (a production observation beside its finding 17).
`KNOWN_VERDICTS` (`grade-agreement.ts`) listed the verdicts a recorded pre-grade may carry,
documented as exactly the verdicts a sheet reader produces.
The list was written in `9ebe41cee` (2026-08-06);
`duplicate` joined `GradeVerdict` in `52fea9e82` (2026-08-12) and the list never gained it,
its `satisfies` clause checking that each listed verdict exists and not that every verdict is listed.
So `parsePreGrades` refused a file holding `"verdict": "duplicate"`
with `pre-grade 1 carries a verdict outside the vocabulary`.
A blind pre-grader does meet such items:
round three's annotated in its notes the same seven repeats the grader marked.
No code writes the file;
an agent writes it by hand,
and `parsePreGrades`,
called by `score-agreement`,
is its one reader.

The fix:
the vocabulary is a record keyed by `GradeVerdict`,
so the compiler refuses a verdict the sheet reader gains and the vocabulary lacks,
and `duplicate` is in it.
`scoreGradeAgreement` is unchanged,
and its TSDoc says what it already did:
an item the human marked duplicate or declined is out of the denominator whatever the pre-grade said,
and a pre-grade's `duplicate` or `unscored` on an item the human scored is a disagreement named in `disagreed`,
so a pre-grader cannot shrink the denominator of the rate that decides whether it may filter a later round.
The red case reads a pre-grade marking a duplicate back whole;
a second case pins the counting over a three-item sheet.

Open to the owner's veto:
counting such a pre-grade as a disagreement,
where leaving the item out of `compared` and reporting it in a field of its own was the other way.

Open when this entry was written,
closed in B227,
whose census found none of them behind its union:
the other values of this package that must name every member of a type declared apart from them.
A search for membership lists read through a string-array cast found eight;
seven define their union,
which therefore cannot gain a member the list lacks,
and `METER_STATES` (`corpus-run/meter-sample-read.ts`) does not,
though it names every member `MeterState` has today.
Lists typed `readonly` over a union declared elsewhere,
and sets built from such lists,
have not been read against their unions.

Prevention:
`mistake-prevention.md`,
"Lists that must name every member of a type".

## B227: no other list was behind its union, and each that must be complete is now held by the compiler

Changed in `f7653073b`,
a refactor with no red case.

A census after B169,
which had left open the package's other values that must name every member of a type declared apart from them.
The agent read the lists,
lookup tables and branch chains over a union in the production source:
the lead's candidates and what its own searches found.
None was behind its union,
so no behaviour changes.
`METER_STATES` (`corpus-run/meter-sample-read.ts`),
the one list B169 named as read through a cast without defining its union,
was complete and is now a record keyed by `MeterState`.

The change is that a member added to a union fails to compile until each of these names it,
in one of three forms:

- a union derived from its `as const` list,
  where the list is also iterated:
  the wire vocabularies that feed a guard and a schema `enum`,
  provider order,
  size bands,
  stream channels,
  damage kinds,
  repair dispositions and round stages;
- a record keyed by the union,
  read through `requireKeyOf` (`artifact-exact-guard.ts`) or an own-key guard,
  where a stored or typed-in word is checked against the vocabulary:
  the artifact readers,
  meter states,
  cached dispositions and settlement terminals;
- a chain's closing refusal of a member no branch handles,
  where the last member had taken the tail by falling through,
  in fourteen functions,
  each with a `@throws` line.

A deliberate subset keeps its list,
typed by its union,
with TSDoc naming the members left out and why.
Hand-spelled copies of one vocabulary are gone for the list they copied,
and `SETTLEMENT_TERMINALS` became `SETTLEMENT_WAYS`,
a record by terminal.
By the agent's account,
the readers of older artifact generations are keyed by their frozen vocabularies and never by the live unions.

An independent reviewer read every production hunk for a change of order,
value or member and found none,
comparing member by member where a list feeds prompt text,
schema bytes,
refusal text or report rows.
Its four findings were closed before the commit:
`@throws` lines missing from the fourteen functions;
a filter in `corpus-run/editor-standing-read.ts` whose comment promised more than a `typeof` test holds,
now a record of the absences;
two comments giving a reason for a tuple annotation that nobody had measured,
now stating the two compiler errors seen without it (`TS9039`,
`TS9013`);
and no case for a member whose discriminant is not text.

The agent's measurements,
not rerun by the lead:
removing one entry of a converted record,
or renaming one handled branch,
gives a compile error where the form promises one (`TS2741`,
`TS2322`).
The whole suite's run on `f7653073b` is recorded in B222.

The closing refusal was written as a call to a helper,
a shape the coverage census reads as code no test ran;
B231 records that and its repair.

Open when this entry was written,
answered in B249 (`df807a8b5`):
whether the decision `no-voice-heard`,
which maps to a terminal cached as settled,
can reach the persistence check
(the agent's question,
handed to a follow-up launched on 2026-10-06 (UTC)).
No input reaches it today,
by that follow-up's sweep,
and a settlement on which no voice was heard is no longer kept.
Read by the agent and left unconverted,
each complete at this commit by its reading:
`CARD_PROVIDERS` (`model-card-derive.ts`),
a reordered twin of the provider names whose order command output pins,
and the size-band and damage-lane lists that shadow their unions locally.

Recurrence:
`mistake-prevention.md`,
"Lists that must name every member of a type".
