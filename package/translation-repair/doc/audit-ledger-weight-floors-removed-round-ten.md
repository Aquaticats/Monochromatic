# Audit ledger: the weight floors removed, round 10

Round 10 of the work recorded in
[the weight floors removed and two ballots deciding](audit-ledger-weight-floors-removed-and-two-ballots-decide.md),
[rounds 2 and 3](audit-ledger-weight-floors-removed-rounds-two-and-three.md),
[round 4](audit-ledger-weight-floors-removed-round-four.md),
[round 5](audit-ledger-weight-floors-removed-round-five.md),
[round 6](audit-ledger-weight-floors-removed-round-six.md)
and [rounds 7 to 9](audit-ledger-weight-floors-removed-round-seven.md),
which holds B933 to B947:
the nits that the eighth and ninth independent re-checks left,
answered on a branch cut from the merge of main at `16c2c7ddb1` (`1362f15c40`).
The corrections in place are in the entries they correct,
in the file of rounds 7 to 9:
B938 (the known limit),
B945 (benches of seven) and B947 (the sweep's result lines).
This round continues at B948.

Part of [Audit ledger](audit-ledger.md),
which lists every file of this record.

## Round 10 at a glance

The eighth re-check (nit A and the nit on a TSDoc line)
and the ninth re-check (minor finding 1 and nits 2,
4 and 5)
left five items that no code defect forced.
They are answered on a branch cut from the merge of main at `16c2c7ddb1` (`1362f15c40`).
No re-check found a defect in the code of the earlier rounds.

The four-seat bench that a split settles by the seat the rotation leaves out is B948.
The position scan that read a verb,
a floating quantifier and "that" as the object of a comparison,
and the thirteen words that left its list,
are B949.
The refiner left out of the window,
the TSDoc reflow,
the cache versions and the gates are B950.
The corrections in place are in B938 (the known limit),
B945 (benches of seven) and B947 (the sweep's result lines).

## B948: a split four-seat bench is settled by the seat the window leaves out (round 10)

Status:
recorded on 2026-10-25 (UTC);
no code change.
On the owner's veto-open list.

Minor finding 1 of the ninth re-check.
Since `1dee3ca797` the selection window is quorum plus one,
so a bench of four asks three seats where it asked four.
Two shapes follow,
both measured by the re-check on the real `selectChunkPatch` with ballots scripted by seat,
under the sweep's salts 0 to 9:
- A 2-2 split:
  two seats name one candidate and two name another.
  The full bench is a tie,
  which is indecision and sends the fallback repair on,
  but the window of three decides,
  and the winner follows the seat it omits:
  candidate 2 ships at salts 0,
  3,
  4,
  7 and 8,
  and candidate 1 at salts 1,
  2,
  5,
  6 and 9.
- A lone namer:
  one seat names and three decline.
  Where the window leaves the namer out,
  every judge asked declines and the existing English ships (salts 2 and 6);
  otherwise the fallback repair goes on.

The same already held for benches of five and over,
where the window has applied since `b6927f7ba9`;
it now holds for a bench of four.
It breaks no ruling:
two ballots decide,
one judge alone never decides,
a run always ships,
and whatever goes on still has to pass the resolution checkers and beat the existing English on the measurements.
It is a property of every window narrower than its bench,
and it is on the owner's veto-open list.
The one-line alternative,
asking the whole bench on benches of fewer than five seats,
would make four-seat benches deterministic;
it is the owner's choice and this record does not recommend it.
The docs do not say whether any real roster's selection bench is four seats,
so how often the effect occurs in a run is not known.

## B949: the position scan read a verb, a floating quantifier and "that" as the object of a comparison (round 10)

Status:
fixed on 2026-10-25 (UTC),
red `a871f02aaf`,
fix `e1fcb14d66`;
thirteen words removed from the list,
red `ba23352795`,
fix `1af8000a84`.

Nit A of the eighth re-check.
The comparison list of the position scan (`COMPARED_OBJECTS`) hid a pointer in three ways that B938 did not name.
A word of the list can be a verb
("the lines below zero the counter",
with zero,
exact,
baseline,
threshold,
half and total),
the quantifiers all,
both,
each and some float away from their noun
("the cases below all pass"),
and "that" can open a clause
("the statement above that a run-off keeps its key holds").
Two pointers in the texts passed for the last reason,
in `consistency/src/consistency-cache-version.ts` and `slice-reading/src/pairing-cache-version.ts`,
and the same shape hid two more in ledger entries.

What the scan reads now.
The words of the list that are verbs or open a clause
(baseline,
zero,
threshold,
total and that)
compare unless a determiner,
a quantifier or a pronoun follows them,
joined by spaces alone.
The quantifiers all and each compare only when a determiner,
of,
else,
other or others follows,
or when no word joins them to the word after them
("above all,
it naps").
The reading sits in `source-scan/src/position-references-compared.test-fixture.ts`,
which the line limit of the scan file required,
and the vocabulary doc of `COMPARED_OBJECTS` states it.

What it still does not read.
- A word of the list followed by a bare verb:
  "the lines below that write it" (`typography/src/semantic-wrap-binary.ts:247`)
  and "the paragraphs above that explain"
  (`audit-ledger-test-suite-unrun-code-reverification-of-2026-10-05.md:15`)
  are pointers in the texts and the scan does not find them,
  since it cannot tell a verb from a noun.
  They were not reworded.
- A quantifier followed by "of" compares:
  "below each of them" is left,
  and so is "the cases below all of which fail".
- Half left the list with the thirteen words:
  "votes below half the bowl" reads as a pointer,
  and the one comparison of that shape in the texts is exempted with its reason.
  Putting half back is the owner's call.
- Any other word that is both a noun and a verb,
  added to the list later,
  lets a pointer through unless it joins a group.

What it found.
Over the 4113 texts the guard reads (the package's sources,
docs,
READMEs and task files,
and the living repository docs),
the tightened scan finds ten phrases more than before.
Seven are pointers and were reworded:
the two cache-version docs,
the ledger entries `audit-ledger-process-launch-checks-and-cache-versions.md` and
`audit-ledger-which-seats-a-quorum-counts.md`,
`adjudication/src/self-certification-weight.unit.test.ts`,
`typography/README.md` and `doc/decision/translation-repair-front-matter-guard.md`.
Three are comparisons and are exempted with their reasons:
"above both reference rules" (`archive/src/reference-precedence.unit.test.ts`),
"above some slice count" (`run/src/corpus-run/cap-override.ts`)
and "below half an even roster" (`stage/src/stage-quorum.unit.test.ts`).
The raw findings outside the guard's own files are 77 before and 80 after,
the same 77 keys and the three exempted ones;
unexempted findings are none before and after,
and no exemption is stale.

The thirteen words.
Re-check 8 named 13 of the 40 words as changing nothing on 3925 texts.
Re-measured on the 4113 texts of this tree before the fix,
deleting one word at a time and counting unexempted findings,
11 words changed nothing:
an,
his,
her,
exact,
regression,
four,
six,
seven,
eight,
nine and ten.
Anything and three each hit in the living decision docs
(`translation-repair-runaway-call-termination-answer-volume-bound-measured-and-corrected.md:171`,
`translation-repair-multi-provider.md:14` and `:495`),
so they stay.
After the fix,
counting raw findings with no exemption applied over the 4114 texts (the new fixture file is one),
13 words changed nothing:
an,
his,
her,
some,
half,
exact,
both,
four,
six,
seven,
eight,
nine and ten.
Regression is not one of them:
its one hit (`adjudication/src/select-candidate.unit.test.ts:132`,
"resolution above regression count") is a true comparison that the window of a neighbouring exemption covers,
so counting unexempted findings alone would have removed it.
The 13 words left the list,
which now holds 27,
and the raw findings outside the guard's own files are the same 80 keys before and after the removal.
A position before one of the 13 words is a pointer,
so a comparison such as "a rate above four" now needs an exemption with its reason.
The owner may prefer to keep the words the texts do not need but a reader might write.

Commit-message slips of this round:
`e1fcb14d66` is titled for the two cache-version pointers and rewords seven pointers;
its body lists all seven.
No false statement was found in the other messages.

## B950: the refiner left out of the window is pinned, with the cache versions and gates (round 10)

Status:
done on 2026-10-25 (UTC),
test `b0c05bdcd5`,
loop correction `80923e74b7`,
reflow `41faa2539b`;
no production code changed.

Nit 4 of the ninth re-check.
Round 9 moved the two refine cases that read the self-vote to a bench of three,
so the other half had no case:
on a bench of four,
a window that leaves the refiner out has no self-vote finding
and the winner draws 3 of 3 ballots.
The case "LEAVES OUT THE SELF-VOTE FINDING AND THE DISCOUNT WHEN THE WINDOW OF QUORUM PLUS ONE LEFT THE REFINER OUT ..."
of `repair/src/refine-stage.unit.test.ts` seats the refiner at each of the four places of the bench in turn.
The four seatings send one selection sheet
(twelve sheets sent,
one distinct),
so the window leaves out one seat of the four whatever the rotation,
and the refiner takes each seat once:
it is left out in exactly one seating.
The case expects three seats asked in every seating,
the refiner left out once,
and the findings of each seating to follow whether it was asked:
`select-self-vote (<the refiner>)` with `refine-selected (weight 2.5 of 3 ballots)`,
or neither the finding nor the discount
(`refine-selected (weight 3 of 3 ballots)`).
A first draft varied the closing sentence of the paragraph to move the rotation and was dropped:
over 200 sheets the hash started the bench only at the third or the fourth seat
(the low bits of FNV-1a over the sheets this fixture can make),
so the refiner,
the first seat,
was never left out at salt 0.

Evidence:
- Under salts 0 to 9,
  run on the single file with the sweep's hook from the package directory,
  every case of the file passes,
  the new one included.
- Planted fault,
  in a scratch copy of the family at `80923e74b7` (`translate-stage/src/candidate-select.ts`):
  the self-vote findings named from the stake in candidate 1 instead of from the ballots cast,
  so the finding appears for a refiner the window left out.
  The case fails at all ten salts,
  with `select-self-vote` only in the actual findings and `refine-selected (weight 3 of 3 ballots)`
  only in the expected ones,
  and the other cases of the file pass.
  The control copy without the fault passes at all ten.

The TSDoc nit of the eighth re-check.
The paragraph of `SHIPS_A_LEVEL_TIE_ELSE_PRODUCES_AGAIN` in `adjudication/src/select-decline-consequence.ts`
that broke a line after "once" is rewrapped at 120 columns (`41faa2539b`).
No word and no string constant changed.

Cache versions.
`git log --extended-regexp -G '_CACHE_VERSION = [0-9]+' 1362f15c40..HEAD` lists nothing,
and the control over `938e84afb9..b37685721e` lists `6c81859c79` and `53e23a728c`,
so the query reads the declarations.
No prompt,
sheet or stored decision changed,
so no number needed to move.

Gates,
keyless and one at a time,
on the tree of `80923e74b7`.
Build,
`lint:types` and `lint:oxlint` (0 warnings and 0 errors) passed in repair,
adjudication,
consistency,
slice-reading,
typography and source-scan.
`test:unit` passed 149,
118,
198,
59,
43 and 68 suites in those packages with none failed.
`source-scans` passed 53 on the tree with this text.
