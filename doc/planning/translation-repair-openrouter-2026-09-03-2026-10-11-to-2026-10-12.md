# OpenRouter and the corpus passes: 2026-10-11 to 2026-10-12

What was launched,
read and decided on 2026-10-11 to 2026-10-12.
Entries:
TianqiChen66622,
TianqiChen66623.

Part of [Translation repair: OpenRouter as the paid fallback provider](translation-repair-openrouter-2026-09-03.md),
which lists every file of this record.

## TianqiChen66622 read, 2026-10-11: SETTLED, two Bedrock seats dark all run

TianqiChen66622 launched on 2026-10-11 at 18:31 UTC at `f58e30822`
through the runbook's task (`mise run //package/module/translation-repair:corpus-pass -- --only TianqiChen666` (moved on
2026-10-18 to `//package/translation-repair/corpus-pass:corpus-pass`),
a systemd user unit,
the owner's go-ahead of that day)
into a throwaway runs directory,
and settled in 7,377,286 ms
(`TALLY TianqiChen666 status=SETTLED slices=24`):
159 repair issues,
104 accepted,
101 resolved,
every slice changed.
`verify-published` read the page back:
24 of 24 wordings present at the length the artifact implies.
`DESTINATIONS` dropped none,
no `DEFECTS` line printed,
and the log holds no `[error]` line.
An empty `DEFECTS` grep did not mean a clean page here:
slice 4 shipped the archive's present-tense sentence,
which the page read found and no check reports.

### Two seats dark all run

The pass's own closing lines:
`SEAT gemma-4-26b-a4b-it asked=270 usable=0 unusable=0 threw=270`,
`SEAT google.gemma-4-e2b asked=266 usable=0 unusable=0 threw=266`
and `SEATS DARK: 2 of 11 seats`.
Kimi-K3 threw 213 of 366.
Every Bedrock Gemma stream ended in under a second with 150 raw characters and no content,
the Bedrock meter read `wet` with no spend all run,
and no seat was withheld (`withheld=none` on every `JUDGE SEATS` line).
The run log names no thrown message,
and no failure at all:
it holds 2 `warn` lines against TianqiChen66621's 101.
That is the launch's fault,
not the pipeline's.
The unit was started with `StandardOutput=truncate:` and `StandardError=append:` on the same file,
two descriptors on one file,
so stdout,
writing at its own offset,
overwrote every line stderr appended:
every `warn` (`console.warn` writes to stderr) and every line mise prints for its tasks.
A control unit writing three lines to stdout and two to stderr kept only the three stdout lines that way,
and all five in order with stderr left to inherit stdout's file.
The lost lines cannot be recovered,
so why Bedrock answered every Gemma call with 150 characters and no content is not known;
the source the two seats send has not changed since TianqiChen66621 (read-only diff of the request body,
route,
headers and served ids).
The next launch writes one file through stdout alone.

Rounds without quorum were 212 of 696
(TianqiChen66621 had 0 of 523),
voices never heard 753
(TianqiChen66621 had 14);
the artifact's `stage-voice-lost` findings name the two Gemma seats and Kimi-K3 for all but two.
A round over six seats that needs five cannot reach quorum with two seats that never answer,
so it waits for every ask to settle and buys a top-up round.

### Where the time went

7,377,286 ms for the entry against TianqiChen66621's 3,169,499 ms.
This run kept four slices in flight (the production default since 2026-09-06),
TianqiChen66621 eight (a dialled arm);
calls in flight averaged 5.40 against 11.93,
over about the same total call time (11.07 h against 10.50 h).
The dark seats' full-settle waits and top-up rounds come on top.
The share each cause holds is inferred,
not measured:
concurrent elapsed times are not a matched comparison.

### Spend

Hyper's balance fell from 250 to 140 credits.
`spend-report` priced 136.07 from a table read on 2026-09-11,
which disagrees with the balance by 26,
so 136 is not quoted as the run's cost.
The table was read again keyless from Hyper's public list the same day (`d02c0cfc3`):
three rows had moved,
`deepseek-v4.1-flash` up among them,
so at today's rates the estimate rises by about 4 credits and the gap to the balance stays;
prompt-cache pricing fits it,
which the estimator cannot see (inferred).
OpenRouter billed 0.2427 USD;
Bedrock billed nothing;
Synthetic's weekly meter went from 99.57% to 92.21%.

### Slice 4: the archive's present tense ships

Page line 59 is the archive's sentence word for word,
in the present tense the owner's rule forbids for a life that has ended.
The lane contest went 5 of 7 usable:
repair 2,
translate 2,
neither 1,
with 4 of 5 ballots calling the archive flawed;
the fifth called it publishable for a reason that spoke only of the candidates.
The owner's rule of 2026-09-24 runs the consolidation slate off only when every contest ballot calls the archive
flawed,
so the slate had its single round:
three candidates tied at weight 1,
the archive drew no ballot
(one judge named its tense a critical failure),
and the tie kept it.
The owner answered on 2026-10-11:
a majority of flawed ballots now runs the slate off
("Majority flawed yes"),
a tie the standing drew no ballot in is run off among the tied
("Run off the tied finalists"),
and a deterministic check may help on tense
("as for tense I suppose some deterministic code can mitigate the problem to some extent");
measured,
it is shown to judges as candidates only
(the package's `design-commitments.md`).

### Seen again

- Line 52,
  "sweeping toward her":
  扑面而来 aimed at her again,
  where the original has the glow coming at everyone she knew;
  TianqiChen66621 read the same.
  The owner asked for a dictionary entry (2026-10-11),
  added in `6bc235960`;
  the corpus carries the idiom on four pages,
  three of them a feeling washing over the person herself,
  so the entry refuses no form and says the target is whoever faces it.
- The refrain 所以她是个……女孩吧 renders four ways,
  one of them slice 4's archive sentence,
  and the repeated sentence 她想以可爱的样子陪伴在更多人的身边 two ways (page lines 101 and 161).
  The owner chose a page-level consistency pass (2026-10-11).

## TianqiChen66623 read, 2026-10-12: SETTLED, the tense holds and the consistency pass changes nothing

TianqiChen66623 launched on 2026-10-12 at 08:58 UTC at `9c0406f113`
through the runbook's unit form
(`mise run //package/module/translation-repair:corpus-pass -- --only TianqiChen666` (moved on 2026-10-18 to
`//package/translation-repair/corpus-pass:corpus-pass`),
`StandardError=inherit` read back from systemd right after the launch)
into a throwaway runs directory,
and settled in 7,339,457 ms
(`TALLY TianqiChen666 status=SETTLED slices=24`):
144 repair issues,
101 accepted,
100 resolved,
every slice changed.
`verify-published` read the page back:
1 of 1 page carries every wording its artifact promised,
at the length it implies.
`DESTINATIONS` dropped none,
no `DEFECTS` line printed,
the log holds no `[error]` line,
and it holds 529 `warn` lines,
where TianqiChen66622 kept 2.

### The two Gemma seats: an expired credential

With stderr kept,
the cause the last read could not name is in the log:
every call to `google.gemma-4-e2b` and `gemma-4-26b-a4b-it` was refused with HTTP 401,
the provider saying "The security token included in the request has expired."
The run withheld each seat after it threw on every ask
(`SEAT google.gemma-4-e2b asked=6 ... threw=6`,
`SEAT gemma-4-26b-a4b-it asked=5 ... threw=5`),
where TianqiChen66622 asked them 270 and 266 times;
rounds without quorum fell to 4 of 503 from 212 of 696,
and voices never heard to 146 from 753 (`run-timing-report`).
That TianqiChen66622's Gemma failures had the same cause is likely,
not shown:
its stderr is lost.
The message names an expired token,
not spent credit;
the owner does not top up Bedrock (2026-09-07),
and with both seats withheld after their first calls no launch waits on them.

### Time and spend

7,339,457 ms against TianqiChen66622's 7,377,286 ms,
with 13.72 h of calls against 11.07 h:
the run asked more,
not waited more,
and the share of each new stage in it is not measured.
Calls in flight averaged 6.73;
rounds spent 47.7% of their time waiting after quorum.
`spend-report` priced 135.66 credits at the rates read on 2026-10-11;
the providers' own balances were not read for this section.

### Tense

The passage TianqiChen66622 shipped in the archive's present tense now reads in the past
(page lines 49 to 56,
"That glow was like thin mist ... Then it left her ashes behind");
three repair judges flagged the archive's present tense against the house rule for a life that has ended.

### The consistency pass changed nothing

The pass found both groups the last read named and asked the bench about each
(`settled groups=2 changed slices=0`):

- The refrain 所以她是个……女孩吧 (source lines 17,
  25,
  33 and 37):
  two ballots chose the third place ("So she was a girl who could be trusted,
  wasn't she?"),
  two the fourth ("So she was a gentle,
  tender-hearted girl,
  wasn't she?"),
  so no strict majority.
  The two frames put the slot on either side of "girl",
  so a splice of either by code could not carry every place's own slot as English.
  The page still reads the refrain three ways.
- The repeated sentence 她想以可爱的样子陪伴在更多人的身边 (source lines 35 and 55):
  three of four ballots chose the first place,
  yet the pass found no agreed quote,
  since it compared quotes as exact strings and two judges quoted the place without its final period,
  two with it.
  Had it located,
  the first place's "She also wanted" renders the `也` that stands before the repeat on source line 35 and not on
  line 55,
  so a splice would have put a word the source lacks into page line 178.

The lead's decision on these,
under the owner's instruction of 2026-10-11 to decide without asking,
is in the package's audit ledger with the work that carries it.

### Em-dashes

The page carries five.
The source carries none,
and the human archive carries them on five lines;
the owner's ruling of 2026-09-21 leaves em-dashes as they are.
