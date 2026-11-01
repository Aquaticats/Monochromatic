# Translation repair handover 2026-09-06: owner questions answered on 2026-09-09 and 2026-09-19

The thin-bench,
glossary and shortfall budget questions as they were put to the owner,
each with its options and ranking,
and where its answer was built.

Part of [Translation repair handover snapshot: 2026-09-06](translation-repair-handover-2026-09-06.md),
which lists every file of this record.

## The thin-bench questions

ANSWERED AT 17:25 UTC ON 2026-09-09:
the first by a share of the reachable weight (`doc/decision/translation-repair-short-bench-share.md`),
the second by keeping the incumbent
(addendum of `doc/decision/translation-repair-ineligible-standing.md`);
the build follows in "What to do next".
The questions as they were asked:

Two decisions for the owner,
raised by the second `noname` and the sixth `Mio` on Bedrock alone (2026-09-09,
12:22 to 12:33 UTC;
planning log,
the same section).

The first:
what a deciding bench short of quorum but not empty should do.
Today it runs on what is reachable (the thirteenth class),
and `MIN_SELECTION_WEIGHT` of 2 in `candidate-select-model.ts` seats a winner only when every reachable
Bedrock judge names it,
so insertions stay unfilled and slices ship unchanged.

- A,
  widen the fifteenth class:
  a deciding bench short of quorum stops the entry INCOMPLETE for a pass that has one.
  Pro:
  no thin page ships,
  deterministic,
  and the shape the owner chose for the empty bench.
  Con:
  on one provider nothing ships.
- B,
  ask a winner for a share of the reachable bench's weight rather than an absolute 2,
  keeping 2 as the floor against one judge deciding.
  Pro:
  Bedrock alone seats winners.
  Con:
  three judges decide,
  and the share is a number to choose.
- C,
  as now:
  the phase runs on what is reachable and the page ships with the findings saying so.
  Pro:
  no change.
  Con:
  a page one writer and three judges made goes to `fixed/` beside one a whole bench made.

Widening Bedrock's judge benches with `gemma-4-31b` is not an option:
it was measured for the judge seat on 2026-09-07 (6 of 12 complete-text picks against a seated median of 9.5,
declining the rest) and stayed out;
no other Bedrock model is approved.

Ranking A > B > C:
A over B because a floor that stands beats a threshold that moves,
and it is the answer the owner gave for the empty bench;
B over C because a page the findings alone distinguish is not a production page.

The second:
what the consolidation should do when the standing text is ineligible and the incumbent is valid.
The 2026-09-04 decision stops the entry;
it was taken on `luxuanwen3`,
where the incumbent itself was ineligible.
On `Mio` slice 3 the archive's paragraph and list were valid and were not offered.

- A,
  keep the incumbent for that slice with the finding recorded,
  and stop only when the incumbent is ineligible too.
  Pro:
  an entry is not lost over one slice whose safe text exists;
  the luxuanwen3 case still stops.
  Con:
  a slice the lanes wanted to change ships unchanged,
  said only in the findings.
- B,
  as now.
  Pro:
  the decision as written.
  Con:
  one thin slice costs the page.

Ranking A > B because the decision's reason ("an ineligible standing was never going to ship") does not reach
an incumbent that can.

## The glossary question

ANSWERED AT 17:25 UTC ON 2026-09-09:
both the corpus glossary file and the archive's rendering as a candidate the judges must weigh
(`doc/decision/translation-repair-community-glossary.md`);
the build follows in "What to do next".
The question as it was asked:

The same page rendered two community terms wrongly where the archive had them right:
`自切`,
the community's word for self-surgery,
shipped as `self-harmed by cutting` where the archive had `attempted self-surgery`;
`超天酱`,
the KAngel character of *Needy Streamer Overload*,
shipped as `Choco-chan` where the archive named the game.
Both are regressions of correct archive renderings by a bench that does not know the community's words,
and nothing in the pipeline tells it.
The seventh `yuki418330012` page (`d6db46519`) shipped both again,
`After she began cutting herself` and `Chōten-chan`.
This is a design question for the owner,
put in the session's report with the options and a ranking:
a corpus glossary of community terms fed to the identity context
(a file beside the corpus pin,
a term,
its rendering and one line of why,
read into every sheet that sees the term;
buys the archive's knowledge for every entry and costs curating it and keeping it out of terms the archive got
wrong),
the archive's rendering as a candidate the judges must weigh where a term of the glossary's shape appears
(no curation,
but the judges already saw the archive and chose against it twice),
or leaving it
(nothing to build,
and the community's own words ship wrong on a memorial).
Nothing is landed;
the owner decides.

## The shortfall budget question

ANSWERED AT 02:55 UTC ON 2026-09-19:
"Do #1",
the untranslated tail's own budget read off the pairing's last agreed pair
(`5bda732de`,
the pass log's heading dated 2026-09-19 03:20 UTC,
the absence-verdict decision's addendum on the tail).
Measured before building:
a neighbour-sized window refuses real omissions,
so the region is the tail alone,
admitted when its expectation exceeds the last agreed pair's whole rendering.
The question as it was asked:

Raised by the XingZ608 read on 2026-09-19
(the pass log's "Class fifty-nine" heading dated 2026-09-19 02:45 UTC).
The whole-page shortfall budget
(`doc/decision/translation-repair-absence-verdict.md`,
`coverage-corroboration.ts`)
reads how much English the page lacks against the corpus median of 2.65 code points per source code point,
and spends that over the absent-voted passages in document order.
XingZ60's archive stops at "Under Construction" with the source's second half untranslated,
and its translated half runs long
(about 3.3 code points per source code point),
so the budget reads 10,794 where the untranslated half alone expects about 13,600.
On XingZ608 that shortfall admitted the prose and refused the footnote definitions,
which class fifty-nine now admits by the page's own graph;
a longer untranslated tail would run out of budget on prose,
which no structural rule admits,
and the page would ship with a recorded gap.
The design says the budget is measured,
not tuned,
and the corpus median is used as is;
the options,
ranked,
are in the session's report of 2026-09-19:
a budget per aligned region
(the untranslated tail's own expectation,
read off the pairing's last agreed pair)
ahead of a whole-page budget that treats a page the archive left unfinished as one whose translated part is complete,
ahead of leaving the budget as it stands and reading the recorded gaps.
Built as `5bda732de`.
