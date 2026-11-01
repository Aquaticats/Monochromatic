# OpenRouter and the corpus passes: the run of CuspariaKLSY22 on build 2a336603bf (2026-10-21)

What was read on 2026-10-21 of the run of the second pass after the Gemma readers were lifted:
the log,
the new rulings and whether they fired,
the classes named,
the open items,
the fixes decided,
and what changed against `CuspariaKLSY21`.
Entry:
CuspariaKLSY22 (entry `CuspariaKLSY`).
The NeuralWatt meter and the plan energy,
the seats and the stalled `inception/mercury-2.5` seat are in
[the providers of CuspariaKLSY22](translation-repair-openrouter-2026-09-03-2026-10-21-gemma-lift-restart-cuspariaklsy22-providers.md),
and the shipped page,
line by line,
with the departures from the Chinese and the comparison with the page of `CuspariaKLSY21`,
is in
[the page of CuspariaKLSY22](translation-repair-openrouter-2026-09-03-2026-10-21-gemma-lift-restart-cuspariaklsy22-page.md).

Part of [Translation repair: OpenRouter as the paid fallback provider](translation-repair-openrouter-2026-09-03.md),
which lists every file of this record;
the build and what the passes of the restart share are in
[the Gemma-lift restart overview](translation-repair-openrouter-2026-09-03-2026-10-21-gemma-lift-restart.md).
The same build's first pass is in
[the run of zheermao22](translation-repair-openrouter-2026-09-03-2026-10-21-gemma-lift-restart-zheermao22-run.md).

## CuspariaKLSY22 read, 2026-10-21, 23:41 UTC: SETTLED in 39.3 min; page mixed; no new ruling shown to act

### Verdict in short

The pass settled in 39.3 min (3.3 times the 11.8 min of `CuspariaKLSY21`).
The page is better than that of `CuspariaKLSY21` in slices 0 to 2 and worse in slice 3.
None of the new rulings is shown to act:
no polish tie fell,
and the reason comes before the vote on only 1 of 103 Hyper replies that carry a vote.
The run time and the nonzero plan energy come from a stalled provider and from the account,
not from a code fault the read could trace.

Three things changed at once between `CuspariaKLSY21` and `CuspariaKLSY22`,
so no difference in this read is credited to the build
unless a line of code or a log line ties it to the build:

- The build moved from `3637592bd4` to `2a336603bf`
  (the rulings "Polished text wins ties" and "Reason first everywhere",
  the accepted-claims note for the polish gate,
  a Gemma picture-reader lift,
  a KaTeX shim,
  and the cache versions).
- Hyper read wet (`hyperBalance=204`;
  it was dry in `CuspariaKLSY21`),
  so `glm-5.3` and `hf:moonshotai/Kimi-K3` were served,
  `minimax-m3` and `gemma-4-26b-a4b-it` moved from OpenRouter to Hyper,
  and `hf:Qwen/Qwen3.8-27B` was withheld instead of unserved.
- `inception/mercury-2.5` on OpenRouter began to stall after about 21:59 UTC:
  33 of its 69 streams in this run ended with zero content characters after 120 s of grace.

What the run shows:

- The page is better than that of `CuspariaKLSY21` on the departures the read marked worst
  (the giver in 起的,
  "proper meal",
  "many friends",
  the lost 一直 and 观望,
  the ambiguous list ending),
  with new additions
  (",
  though",
  a gloss on "futa",
  a "(ACG)" parenthetical)
  and a worse slice 3
  (又 lost,
  "came to" for 醒来,
  "rushed").
- None of the new rulings was shown to act:
  no polish gate tied,
  so "Polished text wins ties" was not exercised;
  the polish gate note cannot be seen in any stored record (the records hold replies only),
  so that fix is unverified;
  and 102 of the 103 stored Hyper replies that carry a vote put the vote before the reason,
  against 44 of 44 replies from NeuralWatt and OpenRouter that put the reason first.
- Two classes by mechanism are named in the closing list:
  the alphabetical property order of Hyper replies,
  and the full-grace wait on a seat whose stream is dead,
  which the seat tally is designed never to count against the seat.
- The run asked Bedrock once and got HTTP 401,
  although the owner ruled on 2026-10-20 to leave Bedrock off
  ([the handover](../handover/translation-repair-handover-2026-09-06-neuralwatt-provider-2026-10-20.md) records the ruling).
  The `METERS` lines read `bedrock=wet` on every line while the `JUDGE SEATS` lines read `bedrock=dry`
  from the second line on.
- One open item is for the owner and not for code:
  `NeuralWatt PLAN ENERGY SPENT: 37.3 Wh`,
  the first nonzero figure of this entry's series,
  against 21.11 Wh of settled calls on the free energy.
  Where the rest came from is unverified.

### Run facts

`CuspariaKLSY22` (entry `CuspariaKLSY`) ran on tip `2a336603bf38a6a53aeb4f3e70f0a3447acf5487`
and pipeline `sha256-tree-v1:5d4265dfab83b5a6e61b5b8c23d597519c7ca24293065473c883ec82af54c78c`,
both exactly as the brief expects.
The `START` line (log line 196) carries both values,
and the artifact's `tip`,
`pipelineDigest` and `corpusSha` (`a41fc607ea5a70d8a7625cc67d5ed8c444f53379`) are the same strings
(checked by a script that reads the artifact).
The run directory has the layout the brief expects:
the artifact `CuspariaKLSY.json`,
the reply-only `prompt-payloads/` (316 files),
`fixed/people/CuspariaKLSY/page.en.md`,
and also `ledger/` (30 files),
`slice-cache/` and `attempts.json` (`{"CuspariaKLSY": 1}`).

The brief gives the launch as 22:25:13 UTC.
The build printed `Finished in 8.38s` (line 193;
`CuspariaKLSY21` printed `47.01s`).
The log's first timestamped line is 22:25:26.307 UTC (line 198,
`republish: 0 settled pages judged`),
the page was written at 23:04:42.032 UTC (line 2299,
`publish:`),
the last timestamped line is 23:04:42.719 UTC (line 2303,
a quota stream),
and the final line is `Finished in 2368.39s` (line 2315).
The log has 2315 lines by `wc --lines` (`CuspariaKLSY21`:
1875).
The corpus pin is `a41fc607ea`;
the pinned directory holds no `.git`,
so the read checked the bytes:
the source `page.md` is 555 characters and 1329 bytes,
equal to `preparation.sourceChars` and `preparation.sourceBytes`,
and the archive `page.en.md` equals `preparation.archiveText` byte for byte (1720 characters).

The `TALLY` line (line 2300) reads:

```text
TALLY CuspariaKLSY status=SETTLED slices=4 repairStatus=repaired repairIssues=29 repairAccepted=26 repairResolved=23 repairFindings=126 repairChanged=4 translateStatus=complete translateChanged=4 documentsDiffer=4 pageChanged=4 pageSilent=0 alignmentFindings=1 selection=contested ms=2355675
```

The `DESTINATIONS` line (2301) reads `DESTINATIONS CuspariaKLSY source=0 page=0 dropped=0`,
the same three zeros as `CuspariaKLSY21`.
The entry has no link,
image or reference
(a scratch search of the source,
the archive and the shipped page for `http`,
`](` and `![` finds none),
so `dropped=0` is true but could not have been otherwise.
The `DONE` line (2302) reads `DONE processed=1 of pending=1; artifacts=1/1 declined=0 ... elapsed=2355731ms`,
which is 39.3 min
(`CuspariaKLSY21`:
11.8 min at 706959 ms;
`CuspariaKLSY20`:
11.5 min;
`CuspariaKLSY19`:
16.0 min).
The publish line reads `publish: wrote 4 slices into a page of 1842 characters`;
a count of the shipped file gives 1842 characters and 1864 bytes,
and it ends in a newline
(`CuspariaKLSY21`:
1932 characters;
`CuspariaKLSY20`:
1929).

Every count names its method:
scripts that count log lines matching a pattern,
list the round lines,
sum the `SPEND` lines and sum the NeuralWatt energy,
run over the pass log.
Code was read with `git show` and `git grep` at `2a336603bf`.

Figure by figure against `CuspariaKLSY21` (its figures from its read and its log):

- `repairIssues` 29 against 20,
  `repairAccepted` 26 against 16,
  `repairResolved` 23 against 14,
  `repairFindings` 126 against 169,
  `documentsDiffer` 4 against 3
  (slice 3 differs only by one line break this time;
  in `CuspariaKLSY21` both lanes wrote identical text),
  `ms` 2355675 against 706897.
  Everything else is equal
  (`translateChanged=4`,
  `pageChanged=4`,
  `pageSilent=0`,
  `alignmentFindings=1`,
  `selection=contested`).
- 103 lines start with `[warn]` (163).
  45 carry `seat unreachable` (143),
  2 carry `HTTP 401` (2),
  0 carry `HTTP 429` (0),
  13 carry `schema-mismatch` (7;
  11 of them are `reaskElsewhereNudged` info lines and 2 are warns),
  37 carry `voice lost` (1),
  33 carry `cut-mid-reply` (0),
  0 carry `quorum-not-met` (0).
- `[error]`,
  `DEFECTS`,
  `REATTEMPT`,
  `STALLED`,
  `SEATS DARK`,
  `chunk re-seated under a hold`,
  `name-gloss-restored`,
  `page assembly`,
  any `archive block` line and any `NOT cached` line:
  none of them (0 matches each).
  `StreamOverrunError` also has 0 matches as a name,
  but the warn text `ended a call that exceeded its content bound` appears twice (see the seat events in
  [the providers of CuspariaKLSY22](translation-repair-openrouter-2026-09-03-2026-10-21-gemma-lift-restart-cuspariaklsy22-providers.md)),
  so the class did fire.
- Six decision lines read `keeping the fallback` (4):
  see the section on tied finalists.
- The 103 warns sum from the groups the counting script printed:
  45 `seat unreachable`,
  33 `abandoned ... after quorum` (all `inception/mercury-2.5`),
  9 `candidate fails the publication rule, so its author is asked about it`
  (the first three,
  lines 1470,
  1471 and 1810,
  are slice 0 candidates that came back as 10 blocks against the original's 9),
  6 `panel ballot ... irregular`,
  2 for the Bedrock credential refusal,
  2 for the Kimi content bound,
  2 `schema-mismatch ... voice lost`,
  2 `translate-archive-disputed` (slices 1 and 3 at 22:48:59.620 UTC),
  1 for `glm-5.3: no re-ask elsewhere`,
  and 1 `recovery round for 1 unreadable answer`.

Where the time went,
from the `SLICE-START`,
`SLICE-COST` and stage lines
(`CuspariaKLSY21` in brackets):

- Preparation 22:25:27 UTC to 22:27:35 UTC:
  the block pairing round took 125594 ms,
  of which 120003 ms were grace waiting for `inception/mercury-2.5` (line 227).
- Repair lane 22:27:35 UTC to 22:48:59.606 UTC
  (line 1402,
  `repair shipped: 4/4 slices changed, 29 issues`):
  21.4 min [3.2 min].
  The four `SLICE-COST` lines read 809574,
  841787,
  1031511 and 1108229 ms [86214 to 191880 ms].
- Translate lane 22:48:59 UTC to 22:51:54.405 UTC:
  2.9 min [2.2 min].
- Lane contest 22:51:55 UTC to 22:54:55.514 UTC:
  3.0 min [0.5 min].
- Consolidation 22:54:56 UTC to 23:04:41.933 UTC:
  9.8 min [5.0 min]
  (slice costs 218449,
  253375,
  501982 and 585131 ms [144363 to 299882 ms]).
- The cause of the increase is one seat's grace.
  A script that lists every round line `N/M heard, T ms total, Q ms to quorum, G ms in grace` finds
  33 rounds with 100000 ms or more in grace (`CuspariaKLSY21`:
  0),
  and their grace sums to 3960 s;
  21 of the 33 are in the repair lane,
  and the repair lane's slice 3 alone waited 720 s over 6 such rounds.
  The rounds of the four slices run side by side,
  so the sum is not wall time,
  but the repair lane's 21.4 min against 3.2 min is this effect
  (the read's reading of the round lines;
  the other lane lengths follow the same pattern).
- The writer grace of 180000 ms never fired:
  every abandonment line says `abandoned 120000ms after quorum`,
  and none says 180000.

### The new rulings, and whether they fired

The brief names four rulings and a lift.
Each is checked in the log,
the artifact and the stored payloads.

#### Polished text wins ties

Not exercised.
Three polish gates ran,
and none tied:

- slice 1:
  `5/6 usable, polished 4, base 0, settled on polished by rule polished-lead, ships polished` (line 2100);
- slice 3:
  `5/5 usable, polished 4, base 1, settled on polished by rule polished-lead, ships polished` (line 2217);
- slice 0:
  `4/6 usable, polished 0, base 3, settled on base by rule base-lead, ships base` (line 2232).
  The proposal changed "she and sunflowers shared a deep,
  fated bond" to "she had a deep,
  fateful bond with them";
  three ballots chose base and one `neither`,
  which is the result the gate should give.
- slice 2 has no polish gate:
  the artifact's polish record for it reads `refinersHeard: []` and `roundCount: 0`
  (no refiner round ran;
  the log's refiner rounds are at lines 1924,
  2031 and 2073,
  slices 0,
  1 and 3).
  Why none ran is not in the log;
  the slice is a numbered list,
  which the read takes to be the reason,
  and that is a guess.

The one visible change of the ruling is the new wording `settled on X by rule Y` on the gate line,
which the code prints
(`package/translation-repair/consolidate/src/consolidation-polish-gate-stage.ts`:
the rule name `tie-polished-wins` at line 119 and line 217,
the tie test `(polished === base) && (polished > 0)` at line 209,
and the tie's own log branch at line 342).
No `tied ... so polished wins the tie` line exists in the log.
There is no positive control in this run,
so the read cannot say the tie path works.

#### Reasons before votes everywhere

The declared order is reason first in the code at the tip:
`candidate-select-wire.ts:357` to `365` (`required: ['reason','best']`,
properties `reason` then `best`),
its instruction at line 506 (`{"reason": "...", "best": 1}`),
`contest-ballot-wire.ts:151` to `157` (`reason`,
`unsupported`,
`dropped`,
`choice`),
and `absolute-naturalness-review-wire.ts:142` to `146` (`reason`,
`findings`,
`acceptable`).

The log cannot witness the order:
every ballot line prints `chose candidate N at weight W: reason`,
the choice first whatever the wire order was.
The stored payloads can.
A script reads the 316 files of `prompt-payloads/`
(each is `{version, reply: {text, finishReason, usage, servedBy}}`,
so they hold replies and not prompts),
parses each reply,
and lists the key order by `servedBy`:

- `neuralwatt` and `openrouter`,
  44 vote-bearing replies:
  `reason>best` (13 and 12),
  `reason>unsupported>dropped>choice` with or without `archive` (3 + 3 and 4 + 3),
  `reason>findings>acceptable` (4 and 2).
  All 44 carry the reason first,
  as declared.
- `hyper`,
  103 vote-bearing replies:
  `best>reason` 53,
  `acceptable>findings>reason` 21,
  `choice>dropped>reason>unsupported` 15,
  `archive>choice>dropped>reason>unsupported` 9,
  and five odd shapes
  (`reason>best` 1,
  a leaked tool-markup key 1,
  two with an extra key,
  a bare `best` 1).
  One of the 103 carries the reason first.
  Every multi-key Hyper shape is in alphabetical key order
  (`best` before `reason`;
  `acceptable`,
  `findings`,
  `reason`;
  `archive`,
  `choice`,
  `dropped`,
  `reason`,
  `unsupported`;
  `groups` before `verdicts`;
  `explanation` before `translation` on the rewrite wire),
  and the declared order is not alphabetical in any of them.
- Hyper served 190 of the run's 373 calls:
  `minimax-m3` 58,
  `gemma-4-26b-a4b-it` 55,
  `kimi-k3` 43,
  `glm-5.3` 31,
  `deepseek-v4.1-flash-1017` 2,
  `glm-5.3-flash` 1.
  So the ruling's schema and instruction were in the sheet,
  on NeuralWatt and OpenRouter the stored replies witness it,
  and on Hyper the reply carries the vote first on 102 of 103 replies.
- The order is on the wire and not made by this package:
  `provider/src/anthropic-completion.ts:326` appends each `input_json_delta` fragment as it arrives
  (line 76 calls them fragments "in arrival order"),
  and the read found no code in the provider or provider-stream packages that sorts a reply's keys.
  The one key-sorting function it met,
  `canonicalPromptValue` in `stage/src/prompt-uniqueness-client.ts:78`
  (`doc/audit-ledger-code-points-and-word-boundaries.md:652` says it sorts record keys as canonical JSON does),
  serializes prompts for the uniqueness check;
  the read did not check whether any of its output reaches the schema Hyper is sent,
  so that is the one place it did not rule out.
  Why Hyper emits sorted keys
  (its tool schema handling,
  or the models under forced tool use)
  could not be told from stored replies;
  a probe with a non-alphabetical schema would decide it,
  and none was run.
- Whether reason-first changes votes was never measured
  (the ruling itself says no bench measures it).
  Here,
  on Hyper,
  the question is moot.

The same finding is in the `zheermao22` read as 200 of 227 multi-key objects
(see [the handover](../handover/translation-repair-handover-2026-09-06-neuralwatt-provider-2026-10-20-the-landings-the-zheermao22-restart-and-the-state-after-k4.md)),
so this entry confirms it on a second entry.

#### Tied finalists run off

Two different rules print `keeping the fallback` or `judges tied`,
and they should not be read as one.

- The translate lane's own decline-challenge
  (`translate-retry.ts:274`:
  `translate stage: declined-indecision; challenging same panel under distinct responsibility`) fired twice:
  - translate slice 0 at 22:50:26.127 UTC after `judges tied at weight 2`
    (lines 1525 to 1527,
    candidates 4 and 6 each with two ballots);
    the challenge round ended `candidate 1 from glm-5.3 won weight 3 (ballots: 4)` (line 1584);
  - translate slice 2 at 22:50:36.933 UTC after `judges tied at weight 1`
    (four ballots,
    four candidates;
    lines 1547 to 1549);
    the challenge ended `candidate 1 from glm-5.3 won weight 3 (ballots: 4)` (line 1602).

  Both behaved as that rule says.
  This is the translate lane's older rule and is not the consolidation ruling of 2026-10-11.
- The consolidation ruling
  ("Run off the tied finalists",
  the bullet "A tie the standing drew no ballot in is run off among the tied" at
  `package/translation-repair/doc/design-commitments-votes-repairs-and-run-offs.md:171`,
  implemented by `standingDrewNoBallot` (line 163) and `judgeSingleRoundOverStanding` (the call at line 218) in
  `package/translation-repair/consolidate/src/consolidate-settle-judge.ts`)
  was in play once:
  consolidation slice 2,
  `select round: 3/5 heard`,
  `minimax-m3` chose candidate 2,
  `gemma-4-26b-a4b-it` chose candidate 1,
  `glm-5.3` chose candidate 3,
  `judges tied at weight 1; keeping the fallback` (line 2140),
  `translate stage: judges tied; keeping the incumbent` (line 2143),
  terminal `slate-declined-standing`.
  No second round ran.
  By the code that is right if the standing drew a ballot,
  and wrong (a run-off was due) if it drew none.
  Which candidate is the standing could not be read:
  the artifact keeps no consolidation slate,
  and the log's numbering is not fixed
  (consolidation slice 3 names its incumbent as candidate 2,
  line 2047).
  What the log does give:
  the slate had four candidates (a judge's reason at line 2139 says "All four candidates"),
  the three ballots cast fell on candidates 1,
  2 and 3,
  so the standing drew no ballot only if it was candidate 4;
  and the shipped standing is the singular "The character she created",
  while a judge's reason in the slice (line 2133) says candidate 4 "uses plural".
  So by that judge's reading the standing was not candidate 4,
  it drew a ballot,
  and no run-off was due.
  That is an inference from a model's reading of its slate,
  labelled as such;
  if it is wrong,
  this is a run-off that did not run.
- The other `keeping the fallback` lines are not ties:
  repair slice 1 (line 687,
  `judges tied at weight 2`,
  then `leaving it unedited in the composite`,
  line 688),
  repair slice 3 (line 1000,
  `winner drew only weight 1 (ballots: 3, abstentions: 2) against a minimum of 2.00`)
  and repair slice 0 (line 1053,
  `winner drew only weight 1.5 (ballots: 3, abstentions: 0)`).

#### The accepted-claims note on the polish gate (ledger B731)

Unverifiable from the stored records.
`prompt-payloads/` holds replies only (see the section on reasons before votes),
and a count of `acceptedRepairClaims` and `REPAIR PANEL CLAIMS` in the artifact finds nothing,
so no stored record shows whether the fenced `REPAIR PANEL CLAIMS ACCEPTED` block
(built by `acceptedRepairClaimsNote` in `package/translation-repair/archive/src/accepted-repair-claims.ts:74`,
placed on the sheet at
`package/translation-repair/consolidate/src/consolidation-polish-gate-wire.ts:414` to `419`)
was on any gate sheet.
No gate in this run was placed to show an effect either:
slice 0's base text already read "went hungry"
(the accepted claim about 吃不上饭,
issue 4 of the repair lane,
is met in both candidates),
and slice 1's proposal touched 觉得 and 信上帝,
not a disputed claim.
The gate ballots do not mention a repair claim.

#### Other changes in the build

- The Gemma picture-reader lift and the KaTeX shim are not exercised:
  the entry has one photo,
  a drawn avatar with a Monster Energy can and no text (the reader opened `profile.jpg`),
  and no math.
  The log has no picture line.
  The `JUDGE SEATS` lines carry a new field `seeing=` (5 on the first line,
  4 after Bedrock was held out);
  the read takes it to count picture-capable seats,
  which is a guess (its source was not read).
- The cache versions moved with the fixes (ledger B734 and commit `4b9fc9e645`),
  and the log says `translated 4 slices (0 resumed)` (line 1650):
  nothing was resumed from a cache.
- Ledger B733 (the cache refusal line):
  no `NOT cached` line printed.
  The one short probe
  (`introduced-defect-probe round: 2/3 heard`,
  repair slice 1,
  line 1085)
  met its quorum of 2,
  so the refusal did not apply;
  the line is not exercised.

### What happened to the findings of the CuspariaKLSY21 read

From the read of `CuspariaKLSY21` and
[the causes](translation-repair-openrouter-2026-09-03-2026-10-21-rotation-restart-causes.md),
each finding that concerns this entry.
The wording choices of the page are in
[the page of CuspariaKLSY22](translation-repair-openrouter-2026-09-03-2026-10-21-gemma-lift-restart-cuspariaklsy22-page.md).

- Finding 9 (the meter gap):
  the gap on the `METERS` lines (0.152 Wh at 22:56:03 UTC,
  0.745 Wh at 22:49:53 UTC) is a whole multiple of the in-flight holds
  (0.0758 Wh and 0.3725 Wh),
  the explanation given there.
- Finding 10 (`withheld=` against `unserved=`):
  both fields printed with different members,
  as the document says.
- Behaviours:
  the repair lane left the present tense unresolved again
  (issue 2,
  major,
  `resolved=false`,
  one checker `fixed` and one `not-fixed`;
  the repair lane's slice 0 kept "is persistent",
  "likes" and "thinks"),
  the lane contest settled on the text of the translate lane by the tense scan,
  and the repair lane's other good edits left with it.
  This is the same pattern as issue 3 of `CuspariaKLSY21` and the tense issues of `CuspariaKLSY20`;
  the consolidation draw recovered "went hungry" this time.
  Repair issues still not resolved:
  2 (tense,
  slice 0),
  3 (rejected),
  5 (tense of one line,
  no region),
  7 (tense,
  slice 1,
  2 to 2 needs-human),
  8 (tense,
  slice 1,
  one checker not-fixed),
  10 (其实,
  2 to 2 needs-human):
  6 of 29,
  which equals `29 - 23`.
- Items from the list of what the `CuspariaKLSY21` read could not read:
  the NeuralWatt retry spacing and `laterStartAt` paths are again not exercised (the first read succeeded);
  the `chunk re-seated under a hold` line printed 0 times again;
  the Bedrock credential is still expired.

### Classes named, open items, and what changed

#### Classes named (by mechanism, with traces)

1. Hyper-served replies carry the vote before the reason.
   - Trace:
     the declared order is reason first at `candidate-select-wire.ts:357` to `365`,
     `contest-ballot-wire.ts:151` to `157` and `absolute-naturalness-review-wire.ts:142` to `146`;
     the transport joins `input_json_delta` fragments in arrival order (`anthropic-completion.ts:326`)
     and the read found no key sorting in the provider or provider-stream sources
     (the one sorting function it met,
     `canonicalPromptValue`,
     serializes prompts for a uniqueness check,
     and whether its output reaches the schema was not read);
     the stored replies are alphabetical in every multi-key shape from Hyper
     (1 of 103 reason first)
     and reason first in 44 of 44 from NeuralWatt and OpenRouter.
   - It recurs whenever Hyper serves a ballot,
     and Hyper is now wet and serves 190 of 373 calls.
   - Effect on any page:
     unmeasured (the ruling itself says no bench measures reason first).
   - Cause on Hyper's side:
     untraced.
     The read searched the provider and provider-stream sources for key sorting and read the transport's
     concatenation;
     the decisive probe (one call to Hyper with a non-alphabetical schema) was not run.
2. A seat whose stream is dead is waited for in full in every round,
   and the tally cannot withhold it.
   - Trace:
     `stage-round.ts:471` to `474` (abandons after `graceMs`,
     warns `abandoned ... after quorum`,
     the voice not answered);
     `seat-tally.ts:120` to `122` and `346` to `352` (an abandoned call is not counted as asked and
     "cannot be withheld for it",
     so no seat ever leaves the bench for being abandoned).
   - Here:
     33 abandonments of `inception/mercury-2.5`,
     33 rounds with 100 s or more of grace (3960 s summed over the rounds of the four slices),
     a repair lane of 21.4 min against 3.2,
     and 0.02907 USD of estimated spend on empty replies.
   - The trigger is environmental
     (a provider window from about 21:59 UTC,
     seen in `zheermao22` too);
     the mechanism recurs for any stalled seat,
     and the design says so in the tally's own comment.
   - Not a trace but a proposal the read left for the owner to weigh:
     a seat that has produced zero content characters in two abandoned rounds could be given a shorter wait for the
     rest of the entry,
     or the round could stop waiting once its remaining voice has shown zero content for a set time.
     That changes a rule the tally documents on purpose,
     so the read called it an owner decision.

#### One-offs and by-design items (not classes)

- The "futa" gloss and the "(ACG)" parenthetical:
  the house policy.
- "doctors fought to save her":
  the glossary (carried over from `CuspariaKLSY21`).
- "When she came to":
  a repair-lane filler carried by the stand-in
  (see the trace in [the page of CuspariaKLSY22](translation-repair-openrouter-2026-09-03-2026-10-21-gemma-lift-restart-cuspariaklsy22-page.md)).
- ",
  though" and the list markers:
  carried from the archive;
  no stage names them.
- The `<arg_key>` leak of `glm-5.3` (one ballot lost,
  no second provider to ask) and the two Kimi-K3 overruns (32008 characters against a bound of 32000).

#### Open items

- `NeuralWatt PLAN ENERGY SPENT: 37.3 Wh` against 21.1113 Wh of settled calls,
  487.978 Wh of free energy left,
  and a baseline of 3.712 kWh that did not move across `zheermao22`.
  Untraced;
  the quota is read twice per run,
  and the by-model route reports one covered-model total.
  The owner can look at the NeuralWatt dashboard for 22:25 UTC to 23:05 UTC by model.
  Until then the source of the other 16.2 Wh is unverified.
- Whether the polish gate sheet showed the accepted claims (B731):
  the stored records cannot say.
  A stored copy of the gate's prompt,
  or one line in the log naming the block when it is present,
  would settle it.
- Whether "Polished text wins ties" and the tie log line work:
  no polish gate tied in this run,
  so there is no positive control.
- Whether the consolidation slice 2 tie should have been run off:
  it was due only if the standing drew no ballot;
  the log's reasons suggest the standing was not candidate 4 and so did draw one,
  but the artifact keeps no slate to confirm which candidate was the standing
  (see the section on tied finalists).
- Hyper's sorted keys:
  one probe would decide whether it is Hyper's schema handling or the models.
- `chunk re-seated under a hold` printed 0 times for the second entry in a row with a held credential
  (untraced;
  the read of `CuspariaKLSY21` did not test whether it should print either).
- `bedrock=wet` on the `METERS` lines against `bedrock=dry` on the `JUDGE SEATS` lines:
  by design as the read understands `run-seats.ts`,
  but a reader of the meters alone is misled while the credential is refused.
- The Bedrock token has been expired for three runs in a row;
  `google.gemma-4-e2b` serves nothing while it is.
  The run still asked Bedrock once and was refused before the hold,
  although the owner ruled to leave Bedrock off.

#### What is decided and not yet landed

One round of work is decided from this read and from the read of `zheermao22`.
None of it has landed:

- Bedrock off by configuration,
  so that no run asks it.
- The `METERS` and `JUDGE SEATS` readings of a held provider made to agree.
- A seat whose streams keep ending with no content counted against that seat
  (the tally never counts it now,
  so every round waits it out;
  the read left a shorter wait as an owner decision,
  and this fix is the decision).
- Hyper's key-order handling.

#### What changed between CuspariaKLSY21 and CuspariaKLSY22

- Build:
  `3637592bd4` to `2a336603bf` (ledger B730 to B734,
  the cache versions,
  the Gemma lift,
  the KaTeX shim).
  Of these,
  only the reason-first schema is visible in this run,
  and it is absent on Hyper;
  the polish gate fixes were not exercised or not visible;
  the Gemma lift and the shim have nothing to act on in this entry.
- Providers:
  Hyper dry to wet (`hyperBalance=204` at the start);
  `glm-5.3` and `hf:moonshotai/Kimi-K3` seated and served;
  `hf:Qwen/Qwen3.8-27B` unserved to withheld;
  OpenRouter spend 0.1728 USD to 0.0730 USD;
  NeuralWatt calls 85 to 74.
- Provider health:
  `inception/mercury-2.5` stalled (0 cut streams to 33),
  which took the run from 11.8 min to 39.3 min.
- Seat counts:
  asked 312 to 333,
  unreachable 143 to 45,
  abandoned 0 to 33,
  threw 0 to 2,
  unusable 1 to 2.
- Energy:
  the plan figure printed nonzero (37.3 Wh) for the first time in this entry's series.
- Page:
  see the comparison in
  [the page of CuspariaKLSY22](translation-repair-openrouter-2026-09-03-2026-10-21-gemma-lift-restart-cuspariaklsy22-page.md).

### What the run read did not do

- The reader could not read the pinned commit `a41fc607ea` from disk (no `.git` in the corpus directory);
  the source and archive were confirmed by bytes against the artifact.
- Code was read only with `git show` and `git grep` on `2a336603bf`.
  No commit diff was read,
  so what a ruling changed rests on the files at the tip and on
  `package/translation-repair/doc/design-commitments-rulings-of-2026-10-21.md`,
  not on a diff of code.
- Two `SPEND` lines are not accounted for by asks,
  abandonments and re-asks (373 against 375).
- `prompt-payloads/` holds replies and not prompts,
  so what any sheet showed could not be seen,
  including whether the accepted-claims block was on a polish gate sheet.
- The artifact keeps no consolidation slate,
  so which candidate was the standing in the consolidation slice 2 tie is inferred from log reasons
  and not read from a record.
- The lane contest ballots and the translate-lane sheets for slice 0 were not read,
  so why the repair lane's slice 0 lost the contest is inferred from its text (still in the present tense).
- Hyper,
  NeuralWatt and OpenRouter were not probed,
  and no pass,
  test or build was run;
  the Hyper key order,
  the mercury stall and the plan energy are therefore described and not explained.
- The logs of `zheermao22` and `XingZ6017` were read only to count `inception/mercury-2.5` streams and the settled
  NeuralWatt energy of `zheermao22`.
- The Markdown was not rendered,
  and no external source was looked up for the page (it has none).
- The `chunk re-seated under a hold` line,
  the NeuralWatt retry spacing and `laterStartAt`,
  the archive block review,
  the Gemma picture lift and the KaTeX shim were not exercised by this entry.
