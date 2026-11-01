# OpenRouter and the corpus passes: the stages of CuspariaKLSY23 on build 8864316eb2 (2026-10-22)

What the stages of `CuspariaKLSY23` (entry `CuspariaKLSY`) did on 2026-10-22:
the stage outcomes,
the verdict of every contested slice,
the recurrence of the known classes,
the side checks
and the open items.
The run facts,
the NeuralWatt meter,
the seats and the Mercury recheck are in
[the run of CuspariaKLSY23](translation-repair-openrouter-2026-09-03-2026-10-22-answers-restart-cuspariaklsy23-run.md),
and the log line behind every figure is in
[the evidence of CuspariaKLSY23](translation-repair-openrouter-2026-09-03-2026-10-22-answers-restart-cuspariaklsy23-evidence.md).

Part of [Translation repair: OpenRouter as the paid fallback provider](translation-repair-openrouter-2026-09-03.md),
which lists every file of this record.

## CuspariaKLSY23 stages, 2026-10-22

### Stage outcomes

Preparation.

- No reference,
  picture or archive block review ran:
  `REFERENCE`,
  `ATTESTED`,
  `archive review`,
  `archive composite` and `PROMPT-REUSE` each have 0 matches,
  and the only `picture` match is the `JUDGE SEATS phase=pictures` line (227).
  The entry has no photo and no cited reference.
  The `ARCHIVE NOTE` line (201) reads the contributor note as advisory
  (`jiushigeshazi via email`),
  and the shipped page carries it as a closing comment.
- Block pairing:
  one round,
  5 of 6 heard in 6,275 ms (line 225),
  `google.gemma-4-e2b` unreachable (line 214),
  and the outcome `paired 18 of 18 original and 18 of 18 translation blocks across 18 relations,
  from 5 usable voices of 5 heard` (226).
- `OVERLAP CuspariaKLSY value=4 source=fallback` (line 200).

Lanes.

- Both lanes began at 13:22:29.985 UTC over 4 slices and 1 aligned unit with 1 alignment finding (lines 229 and 230).
- Repair lane:
  status `repaired`,
  slices changed 0,
  1,
  2 and 3 (4);
  the critic stage heard 4 of 6 on each slice,
  with 11 claims on slice 2 (line 263),
  16 on slice 3 (280),
  11 on slice 0 (384)
  and 15 on slice 1 (445);
  15 panel clusters in all (3,
  5,
  4 and 3 per slice);
  4 of 4 introduced-defect probe rounds `3/3 heard`,
  one of which corroborated one added-damage claim (slice 1,
  line 1259);
  the refine stage declined on slice 3 because its winner drew weight 1 against a minimum of 2.00
  (lines 1392 and 1393);
  the repair lane shipped at 13:29:39.157 UTC (line 1395).
  26 of the 30 accepted issues are resolved (see "Why repairIssues rose from 29 to 36").
  `semantic wrap` skipped all 4 line-structured repair outcomes (line 1394).
  The `content lost at slice N` lines (1396 to 1399) name 2,
  4,
  3 and 2 distinctive archive words,
  and the finding `content-survival` reads 59 of 70 words kept,
  11 lost
  ("swallowed",
  in slice 3,
  is the invented method that the accepted claims asked to remove).
- Translate lane:
  status `complete`,
  slices changed 0,
  1,
  2 and 3 (4),
  none refused by a guard (`translated 4 slices (0 resumed): 4 changed, 0 refused by a guard`,
  line 1671);
  the winners are `hf:zai-org/GLM-5.3-Flash` for slice 0 (weight 3,
  line 1577),
  `minimax-m3` for slices 1 and 2 (weight 2.5 each,
  lines 1616 and 1668)
  and a composite of `hf:zai-org/GLM-5.3-Flash`,
  `hf:moonshotai/Kimi-K3` and `minimax-m3` for slice 3 (weight 3.5,
  line 1490).
  Seven `candidate fails the publication rule` warns were each followed by a revised candidate
  (warns 1437,
  1472,
  1820,
  1848,
  1871,
  1872 and 1873;
  revisions 1450,
  1478,
  1844,
  1865,
  1893,
  1898 and 1903).
- The lane texts differ in 4 of 4 slices (line 1674),
  and the log says `repair repaired, translate changed 4/4 slices; neither was chosen over the other` (1672).

Why repairIssues rose from 29 to 36:

- Issues by slice (`art3.ts`):
  slice 0 holds 8,
  slice 1 holds 9,
  slice 2 holds 5 and slice 3 holds 14.
  The status split is 30 accepted (22 minor and 8 major),
  4 rejected (all on slice 1)
  and 2 `needs-human` (slice 1 major,
  slice 2 minor).
  Slice 3,
  the passage with the invented suicide method,
  accounts for 14 accepted issues,
  all resolved.
- `repairResolved` 26 of 30 accepted
  (`CuspariaKLSY22`:
  23 of 26).
  The four unresolved accepted issues are all about tense (`art4.ts`):
  three on slice 0 (a major style/register claim with 3 of 3 checkers `not-fixed`,
  a minor fluency/inconsistency claim with 3 of 3 `not-fixed`
  and a major fluency/inconsistency claim with 2 of 3 `not-fixed`),
  and one minor fluency/inconsistency claim on slice 1 (2 of 3 `not-fixed`).
  This is the behaviour the read of `CuspariaKLSY22` named:
  the repair lane leaves the present tense of a memorial page in place.
- The rise is concentrated in slice 3 (14 issues,
  all accepted and all resolved);
  the critic stage heard 4 of 6 voices on every slice,
  and the log does not say why the count rose,
  so no cause is given.

### Slice verdicts and disputes

Per slice,
the lane contest verdict (ballot lines),
then the consolidation terminal and the gates
(from the artifact's `laneSelection` and `consolidation` fields and the gate lines):

- Slice 0:
  `translate` won,
  5 of 5 ballots (verdict line 1726,
  ballots 1721 to 1725),
  every ballot calling the archive flawed;
  the ballot reasons cite the past tense of a memorial page,
  and the repair lane's text still carries "likes",
  "thinks",
  "is" and "knows" (the artifact's `comparison` record);
  terminal `consolidated`,
  consolidate gate 4/4 usable,
  `consolidated` (line 1933);
  polish gate 5 of 6 usable,
  polished 3,
  base 1,
  neither 1,
  `settled on polished by rule polished-lead` (line 2148),
  and the polished text ships ("a very deep predestined affinity").
- Slice 1:
  `translate` won,
  3 to 2 over `repair` (verdict line 1735,
  ballots 1730 to 1734);
  the probe's one corroborated damage claim was shown to the judges (line 1675);
  four ballots listed the translate text as dropping something and two listed the repair text as unsupported
  (the fields `dropped` and `unsupported`,
  described at `contest-ballot-wire.ts:88` and declared at `:153` to `167`);
  terminal `slate-endorsed-standing`,
  the shipped record `unchanged`,
  no gate asked,
  and no polish round (`refine-candidates (3/3 heard, 0 proposing)`).
- Slice 2:
  `translate` won,
  4/6 usable (verdict line 1742,
  ballots 1738 to 1741);
  Kimi was cut at the grace (line 1743) and `google.gemma-4-e2b` was unreachable;
  terminal `consolidated`,
  consolidate gate 4/5 usable,
  `consolidated` 3 and `neither` 1 (line 2171);
  the polish step skipped (`refine-skipped (0 eligible paragraphs)`;
  the slice is a numbered list,
  as in `CuspariaKLSY22`,
  which is read as the reason and not shown).
- Slice 3:
  settled on `neither`,
  archive declined,
  5 of 5 usable (verdict line 1709,
  ballots 1704 to 1708:
  one `translate`,
  four `neither`),
  every ballot calling the archive flawed;
  terminal `consolidated`;
  the log warns `consolidation standing text lacks contest endorsement and remains retryable` (1759),
  announces that a tied slate is run off (1788),
  and offers the translate lane's text on the slate (1789);
  the consolidate gate heard 4/5 usable,
  `neither` 3 and `consolidated` 1,
  and settled on `neither` (line 1834),
  and the warn at line 1835 says it "settled on neither over a standing a majority of contest ballots called flawed,
  so the proposal the slate chose ships".
  The shipped slice 3 carries "fought to save her",
  "confirmed brain dead",
  "August 1,
  2023" and "again" and omits the invented method
  (four accepted claims checked against the shipped page;
  the other ten were not).

Totals:
the translate lane won 3 slices,
1 contest settled on `neither` (slice 3),
and the repair lane won none of the four
(`selection=contested`).
All 19 ballots of the four contests (5,
5,
4 and 5) called the archive rendering flawed (`art3.ts`,
the `laneSelection` ballots).
The consolidation terminals read 3 `consolidated` (slices 0,
2 and 3) and 1 `slate-endorsed-standing` (slice 1).
One polish gate ran (slice 0).
Absolute naturalness reviews:
6 ran,
4 acceptable and 2 unacceptable (lines 2191 and 2201,
slices 0 and 2),
and the findings of the two are recorded as evidence while the gated text ships
(`absolute naturalness rejection recorded as evidence; the gated text ships`,
a finding in the polish record of slices 0 and 2).
The page assembly then restored one name gloss on slice 1
(`name-gloss-restored`,
line 2204):
the repair panel had rejected the claim that the gloss is an addition (issue 5f4d91,
line 855),
the slice 1 text that consolidation shipped unchanged lacked the gloss,
and the archive's gloss line is on the shipped page.

Disputes.

- `translate-archive-disputed` printed for slices 1 and 3 (lines 1402 and 1403),
  with 5 and 4 accepted disputing claims
  ("accuracy/addition at any severity,
  any other accuracy claim the panel settled at major or worse"),
  and again at the start of consolidation (lines 1755 and 1758).
  On both the log words it as "the repair lane's text stands in for [the archive rendering]"
  (classes one hundred seven and one hundred seventy-six),
  and at consolidation as "the stand-in is the consolidation's incumbent".
- No `introduced repetition` line printed (0;
  2 in `zheermao23`).
- Tied finalists run off:
  not exercised.
  No `judges tied` line belongs to the consolidation rule:
  the two `judges tied` lines (1172 and 1173) are the repair lane's editor select on slice 2
  (`judges tied at weight 2; keeping the fallback`,
  then `judges tied; shipping the strongest repair anyway`),
  and the announcement of the run-off at line 1788 had no tie to act on
  (the slate round,
  line 1812,
  chose a candidate at weight 2.5 of 4 ballots).
- Polished text wins ties:
  not exercised.
  The one polish gate did not tie (3 to 1).
  This record says "no tie occurred",
  not "worked".
- Ledger B731 (the polish gate sheet shows the accepted claims):
  not shown.
  None of the 5 ballots of the polish gate cites a repair panel claim
  (a pattern search for `repair panel`,
  `flagged by` and `accepted claim` over the log finds 0 lines).

### Recurrence of the known classes

The code was not re-traced except as named.
The classes are those of the `CuspariaKLSY22` read and the `zheermao23` draft,
and each status is what the log and the artifact show.

- Qwen abandonments and grace:
  not exercisable.
  The log holds no `hf:Qwen/Qwen3.8-27B` stream and no Qwen `SPEND` line;
  the name appears only in the `withheld=` field of the six `JUDGE SEATS` lines
  (12 matches in all).
  The grace that Qwen cost is 0 s,
  against 780 s in `zheermao23`.
- A similar shape on Kimi:
  one cut,
  in the lane contest of slice 2 (stream line 1737,
  warn 1743):
  `cut-mid-reply after 40 answer chars and 25012 reasoning chars (310531 chars on the wire),
  first byte at 13688ms`,
  after 153,209 ms of stream.
  The 40 answer characters begin "Let me analyze the four lines.",
  as the `opening` field of the stream line prints them.
  The cut was at the 120 s ordinary grace (the round had its quorum of 4 at 33,214 ms),
  and the round lost the 120,004 ms.
  Kimi's stream ran as long as the round in three of the four contest rounds
  (slice 1:
  92,371 ms of a 92,388 ms round at lines 1727 and 1729;
  slice 2:
  the cut one,
  153,209 ms of 153,218 ms at lines 1737 and 1736;
  slice 3:
  18,788 ms of 18,795 ms at lines 1701 and 1703),
  with a median of 18,813 ms over its 40 streams (`kimi.ts`).
  `CuspariaKLSY22` had two Kimi overruns of the content bound and no cut;
  `zheermao23` had none.
  No window guard acted on it:
  its first content came at 13,688 ms and its largest gap was 422 ms,
  so neither the 180 s first-content window nor the 120 s idle window could end it,
  and only the grace did.
- K1 (text calls ending at the cap with no content):
  recurred on Hyper with `minimax-m3`,
  three times
  (stream lines 1512,
  1618 and 1851;
  `SPEND` lines 1513,
  1619 and 1852;
  `finishReason: max_tokens` in 3 payloads),
  each with 0 content characters and 43,960 to 47,268 reasoning characters,
  at 10,822,
  21,645 and 10,822 completion tokens.
  The card's cap for the model is 10,822 (`model-cards.ts:218`),
  and `completionCapFor` returns the card's cap or a lower requested one (`completion-cap.ts`),
  so 21,645 is above the cap the client sends;
  the same value ended a `minimax-m3` call on Hyper in `CuspariaKLSY22`
  (prompt 6930),
  along with 10,822 twice.
  Why 21,645 exceeds the cap is not known
  (it is 2 x 10,822 + 1;
  whether it sums two attempts was not traced).
  Each of the three was re-asked on OpenRouter within 3 to 11 s and answered usably
  (`reaskElsewhereNudged` lines 1514 and 1517,
  1620 and 1630,
  1853 and 1869).
  Each sat inside a round that spent 73.9 s (line 1518),
  76.1 s (line 1631) and 37.7 s (line 1870) in grace.
  Stored replies that ended at a limit:
  3 of 336 (`CuspariaKLSY22`:
  7 of 316,
  4 on Hyper and 3 on NeuralWatt;
  `zheermao23`:
  4 of 892).
  No energy cost on Hyper.
- K2 (a stream that sends bytes and no content,
  cut only by the grace):
  absent for Mercury (0 of 64 streams).
- K3 (a repair-lane fix for a minor accepted claim lost when the lane contest settles `neither`):
  the contest settled `neither` once (slice 3),
  and the four accepted claims checked against the shipped page were carried
  (see "Slice verdicts and disputes");
  the other ten claims of the slice were not checked.
- K4 (the composite refuses a revision that changes the line count):
  not exercised;
  no archive composite was attempted.
- N1 (a hung stream holds a quorum that equals the live bench):
  three rounds closed with 0 ms in grace on a short count (see "Seats,
  units and meters");
  none is shown to be a hung stream.
- Irregular panel ballots:
  8 `panel ballot ... irregular` warns,
  all from `gemma-4-26b-a4b-it`,
  all `group-index-out-of-range` with the indices 2 and 3 (lines 343 and 344),
  2,
  3 and 4 (468 to 470)
  and 2,
  3 and 4 (653 to 655),
  in 3 of the 15 panel rounds (rounds at lines 342,
  467 and 652;
  clusters with the hash prefixes 1ed349,
  984cbb and c23e82).
  `CuspariaKLSY22` printed 6,
  `CuspariaKLSY21` 7,
  and `zheermao23` 30 (29 from the same model),
  so the count in this entry is steady at 6 to 8;
  other entries print more
  (`zheermao22` 28,
  `zheermao23` 30,
  `XingZ6017` 96).
  The same model also needed 4 re-asks on OpenRouter after a `schema-mismatch` on Hyper
  (lines 307,
  428,
  733 and 752).
  Whether the 8 lines are counted in the 31 `panel-packet` findings was not checked.
- U1 (the Hyper key order):
  not testable.
  The key is `assessment`,
  and no payload has a top-level `reason` key.
  63 of the 336 payloads carry `reason` inside their `verdicts` entries
  (229 entries:
  Hyper 175 in 49 payloads,
  OpenRouter 54 in 14);
  all 229 put `reason` before `vote`,
  but in alphabetical key order (`claim`,
  `reason`,
  `severity`,
  `vote`),
  so they cannot tell the wanted order from alphabetical order either.
  Of the replies that carry an `assessment` and a vote key,
  Hyper 113 are assessment-first and 0 vote-first,
  OpenRouter 38 and 0 (`payloads2.ts`).
  164 of 165 multi-key Hyper objects are in alphabetical key order,
  and 53 of 54 on OpenRouter,
  and `assessment` sorts before every vote key,
  so alphabetical order and the wanted order coincide.
  The one out-of-order object on each provider is a malformed reply with prose in a key.
  7 replies did not parse as JSON (Hyper 4,
  OpenRouter 3),
  the three `max_tokens` replies among the Hyper ones.
- The tense behaviour of the repair lane:
  see "Why repairIssues rose".
  The read of `CuspariaKLSY22` names unresolved tense issues of the repair lane in `CuspariaKLSY20`,
  `CuspariaKLSY21` and `CuspariaKLSY22`,
  and here it recurs a fourth time,
  with the repair lane winning no slice of the contest.
- `NeuralWatt PLAN ENERGY SPENT` end line (2211):
  255.7 Wh with no NeuralWatt call
  (see "NeuralWatt:
  baseline,
  stop and energy").
  It is the fourth nonzero figure in a row,
  after 37.3,
  54.3 and 326.9 Wh.

Defects and regressions against `CuspariaKLSY22`,
in the order this read ranks them:

- The plan energy figure,
  255.7 Wh with no NeuralWatt request in the log,
  at 14.9 Wh per min,
  2.3 times the rate of `zheermao23`.
  Not a code defect:
  the owner's dsh harness shares the NeuralWatt account
  (owner,
  2026-10-22).
- The repair lane's unresolved tense,
  4 accepted issues (2 major) on slices 0 and 1,
  and no slice won by the repair lane.
- Three `minimax-m3` cap-outs on Hyper with no content,
  one at 21,645 tokens (above the card's cap),
  holding three rounds for 74 s,
  76 s and 38 s of grace.
- Kimi cut at the grace once,
  120 s,
  in the lane contest of slice 2
  (153 s round),
  and 92 s on the contest of slice 1.
- `repairIssues` 36 against 29 and `repairAccepted` 30 against 26.
- 8 irregular panel ballots from `gemma-4-26b-a4b-it`.
- The last 70 s of the run were one `deepseek-v4.1-flash` re-ask on the OpenRouter endpoint `Morph`.
- The OpenRouter balance,
  3.93 USD,
  and the Hyper balance,
  157 credits at 13:37 UTC.
- The brief's `DONE` time (13:43 UTC) disagrees with the log (13:39:30 UTC).

Against these:
the Mercury stall did not recur (0 of 64 streams),
the run was 22.1 min shorter,
the asks close exactly against the `SPEND` lines,
no quorum failed,
Bedrock was off cleanly with no credential refused,
there is no `[error]` line,
no `HTTP 4xx` or `5xx` call,
and no recovery round waited out a grace.

### Traces and side checks

Seven files of the worktree were read as files
(the six in this list and `completion-cap.ts`,
cited in the `minimax-m3` entry),
and two git ref files,
and nothing was run in it.
The worktree is `/var/home/user/worktrees/translation-repair`,
its `HEAD` file reads `ref: refs/heads/translation-repair-rebased`,
and the branch ref file reads the `START` tip;
the working tree was not compared with it.

- `package/translation-repair/shared-contract/src/model-cards.ts:431`:
  the Mercury card names `firstContentMs: 90_000` (ledger B809 comment at 418 to 430,
  with p50 6.7 s,
  p99 20.3 s and a longest of 56,772 ms over 4,636 streams at 424 to 425).
- `package/translation-repair/shared-contract/src/model-cards.ts:218`:
  the `minimax-m3` card names `completionCap: 10_822`.
- `package/translation-repair/provider/src/openrouter-client.ts:462`:
  the request carries the card's own first-content window where the card names one.
- `package/translation-repair/provider-stream/src/stream-idle-guard.ts:89` and `:106`:
  `STREAM_FIRST_CONTENT_MS = 180_000` and `STREAM_IDLE_MS = 120_000`,
  which are the artifact's recorded `callConfig` values.
- `package/translation-repair/stage/src/seat-tally.ts:370` to `375`:
  a call the caller aborted is counted as abandoned,
  not as asked.
- `package/translation-repair/provider/src/neuralwatt-quota.ts:9` to `26`:
  the plan figure and what it cannot separate;
  the end line is built at `:465`.
- `package/translation-repair/lane-contest/src/contest-ballot-wire.ts:88` and `:153` to `167`:
  `unsupported` and `dropped` are per-candidate findings lists
  (described at 88,
  declared at 153 to 167).
- `.git/refs/heads/translation-repair-rebased`:
  `8864316eb29b9848d95ceeaf4c039671c0c430e8`.
- `.git/worktrees/translation-repair/HEAD`:
  the branch the worktree is on.

### Open items and what the read did not do

Open items:

- The 255.7 Wh of plan energy is settled as the owner's dsh harness on the same NeuralWatt account
  (owner,
  2026-10-22);
  the plan figure's rate (2.6,
  6.6 and 14.9 Wh per min across the three intervals) follows that harness's use,
  which the log does not show.
- Whether the 90000 ms Mercury window works:
  no run has had a Mercury stall since it landed
  (185 streams in two runs).
- Why `minimax-m3` on Hyper runs to its cap with no content three times in 58 calls,
  and why the completion figure 21,645 exceeds the card's cap of 10,822 (it recurs from `CuspariaKLSY22`).
- Why Kimi on Hyper sometimes reasons for minutes in a contest round,
  and whether the card should bound it as the Mercury card bounds its first content.
- Why the OpenRouter `Morph` endpoint took 70 s for a `deepseek-v4.1-flash` call that Hyper answered in 8 s.
- Why the repair lane leaves the present tense of a memorial page open on each of the last four `CuspariaKLSY` runs.
- Why `gemma-4-26b-a4b-it` returns group indices out of range in panel ballots.
- Why the translate lane's Mercury calls ran slower than the repair lane's.

What the read did not do:

- It did not read the shipped page line by line,
  so it has no departure list and no class count by departure.
- It did not read any prompt
  (the run directory holds replies,
  not prompts),
  so what the contest,
  polish and gate sheets showed is known only from the ballot texts that quote them.
- It did not trace code for the classes,
  except for the side checks in "Traces and side checks".
- It did not run a spread measurement.
- It did not check the host for the cause of any slowness.
- It did not probe any provider.
- It did not read the systemd unit,
  so `Result=success` is the brief's.
