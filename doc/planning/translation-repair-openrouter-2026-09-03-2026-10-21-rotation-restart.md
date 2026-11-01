# OpenRouter and the corpus passes: the rotation restarted on build 3637592bd4 (2026-10-21)

What was read on 2026-10-21 of the first three passes of the rotation after it restarted at `zheermao101`.
Two entries settled and the third stopped on a picture that only one model had read.
Entries:
zheermao21,
CuspariaKLSY21,
XingZ6016.

Part of [Translation repair: OpenRouter as the paid fallback provider](translation-repair-openrouter-2026-09-03.md),
which lists every file of this record.
The passes before the restart (`zheermao20` and `CuspariaKLSY20`) are in
[the pass log of 2026-10-21](translation-repair-openrouter-2026-09-03-2026-10-21.md).
The standing account of the readiness signal is [the readiness signal](translation-repair-readiness-signal.md).

## Where each pass is recorded

Each pass has its own file or files,
so that no file passes the repository's limit of 1,000 lines:

- `zheermao21`:
  [the run](translation-repair-openrouter-2026-09-03-2026-10-21-rotation-restart-zheermao21-run.md)
  (log,
  NeuralWatt meter,
  seats,
  the archive block review of the letter and the email,
  why the repair counts rose)
  and [the page](translation-repair-openrouter-2026-09-03-2026-10-21-rotation-restart-zheermao21-page.md)
  (the shipped page line by line,
  departures D01 to D24,
  the comparison with `zheermao20`).
- `CuspariaKLSY21`:
  [the run](translation-repair-openrouter-2026-09-03-2026-10-21-rotation-restart-cuspariaklsy21-run.md)
  (log,
  NeuralWatt meter,
  seats and quorums,
  no archive blocks)
  and [the page](translation-repair-openrouter-2026-09-03-2026-10-21-rotation-restart-cuspariaklsy21-page.md)
  (the shipped page line by line,
  departures,
  the comparison with `CuspariaKLSY20`).
- The causes of ten findings,
  their classes and what fixed them:
  [the causes](translation-repair-openrouter-2026-09-03-2026-10-21-rotation-restart-causes.md)
  (traced on 2026-10-21;
  the page and run files keep their original text and carry a note where a cause is now traced).
- `XingZ6016`:
  [the picture stop](translation-repair-openrouter-2026-09-03-2026-10-21-rotation-restart-xingz6016.md)
  (who served the picture,
  why GLM replied empty,
  the corroboration rule,
  the candidate readers,
  the remedies the read ranked).

## The build

- Tip `3637592bd43652fe2c6ff3cf2ec31e5a27c39c7d`,
  pipeline `sha256-tree-v1:06c97a69446dc452fa2c602da834c36febb7f0d7b4b1010b5dfa9fab81097eec`.
  The `START` lines of `zheermao21` and `CuspariaKLSY21` carry both as given;
  the picture-stop read names the tip `3637592bd4`.
- It is the merge that gate g138 landed on 2026-10-21 (UTC),
  as [the handover](../handover/translation-repair-handover-2026-09-06-neuralwatt-provider-2026-10-20-zheermao20-to-zheermao21-reads-and-the-w253-to-w257-fix-rounds.md#next) records.
- The corpus input is the pin `a41fc607ea` (`corpusSha` `a41fc607ea5a70d8a7625cc67d5ed8c444f53379`).
- The archive block review fixes B712 to B715 are not in this build:
  the review still picks one whole revision,
  still waits the 120 s reader grace,
  and still anchors a retention quote by exact text.
- The launch times the reads were given:
  `zheermao21` 05:20:09 UTC (log from 05:21:48.250 to 06:02:32.294 UTC),
  `CuspariaKLSY21` 06:03:23 UTC (log from 06:04:38.972 to 06:16:25.926 UTC).
  The read of `XingZ6016` carries no clock time for its pass.

## The three outcomes

- `zheermao21` (entry `zheermao101`):
  SETTLED in 40.7 min of pass,
  `TALLY ... status=SETTLED slices=15 ... selection=contested ms=2443411`.
  The read named no class.
  The letter recovered (the archive block review chose a comprehensive revision from `deepseek-v4.1-flash` on NeuralWatt);
  the opening quote and the email regressed;
  the page has two `U+2014` characters.
  24 departures were listed.
- `CuspariaKLSY21` (entry `CuspariaKLSY`):
  SETTLED in 11.8 min of pass,
  `TALLY ... status=SETTLED slices=4 ... selection=contested ms=706897`.
  The page is worse than `CuspariaKLSY20`'s,
  most in slice 3 ("where doctors fought to save her" and the lost 已经) and slice 1 ("was given to her" for 起的).
  The read traced three departures to code,
  all by design,
  and listed twelve wording choices with no code cause found.
- `XingZ6016`:
  INCOMPLETE after 6.7 min of pass with `visual evidence incomplete for 1 referenced assets`.
  The picture got one reading.
  `minimax-m3` read it;
  `hf:zai-org/GLM-5.3-Flash` was cut at the 18316-token completion cap while reasoning and returned an empty reply;
  `hf:Qwen/Qwen3.8-27B` had no provider (Synthetic and Hyper dry,
  and the OpenRouter side held off by the owner's `openrouter-dropped` hold);
  both Gemma readers are Bedrock-only and Bedrock was held out on an expired credential.
  No page was written.

## What the two settled runs show about the build

Both settled reads checked the two changes of this build in the log.

- The NeuralWatt baseline was taken once,
  on the first attempt,
  in the plain wording
  (`NeuralWatt quota at the start of the run: 3.1687 kWh charged this period`),
  in both runs.
  Neither log holds the failed-baseline warn or the late-start wording,
  so the 60 s retry spacing and `laterStartAt` were not exercised by either run,
  and neither run says whether they work.
- No NeuralWatt stop line fired in either run,
  and both closing lines read that no plan energy was spent.
  Both runs sit inside the NeuralWatt day that began 2026-10-20T19:15:08.000Z.
- The `SEAT` lines now carry `unreachable=N`.
  A seat the router refuses before any request is counted there and no longer as an ask that threw:
  `threw=0` on every `SEAT` line of both runs,
  and `SEATS DARK` did not print in either (it counts only seats asked).
- No stage threw and neither log holds an `[error]` line.
- Measured NeuralWatt energy ran over the estimate in both:
  `zheermao21` 44.7156 Wh against 36.4328 Wh over 247 calls,
  `CuspariaKLSY21` 21.0145 Wh against 10.5074 Wh over 85 calls.
- Bedrock's key was expired in all three passes (HTTP 401),
  and Hyper's balance was 0 in all three.
- Synthetic's weekly meter:
  `zheermao21` read 2% at 05:21:51 UTC and went dry at 05:53:09.782 UTC;
  `CuspariaKLSY21` read 0% throughout;
  `XingZ6016` read dry at the start of its pass.

## Where the reads disagree with each other, the readiness signal or the handover

[The readiness signal](translation-repair-readiness-signal.md) and [the handover](../handover/translation-repair-handover-2026-09-06-neuralwatt-provider-2026-10-20-zheermao20-to-zheermao21-reads-and-the-w253-to-w257-fix-rounds.md)
state the outcome of the three passes in short:
`zheermao21` and `CuspariaKLSY21` shipped SETTLED,
`XingZ6016` stopped INCOMPLETE after 6.7 min with the text `visual evidence incomplete for 1 referenced assets`,
and the rotation on this build is held there.
The reads agree with that outcome.
The handover's list of findings under tracing
(the 2 to 2 polish gate on 起的,
the repair correction accepted 4 to 0 and lost to the translate lane,
已经,
一直,
观望 and 二次元,
the opening quote losing 有一天 and 你 to a minimum-weight rule,
the thinner email,
the two `U+2014` characters,
a probe that closed short,
the 0.372 Wh gap between the meter and the account,
and the glossary's "doctors fought to save her")
is each found in the reads,
in the files listed under "Where each pass is recorded".

Differences found:

- Qwen's reach in `XingZ6016`.
  The readiness signal says Qwen's only provider was out of budget.
  The picture-stop read says Qwen has an OpenRouter side that the owner's `openrouter-dropped` hold removes
  (`model-cards.ts:118`),
  with Synthetic and Hyper dry.
  The result is the same (no provider);
  the wording differs.
- The cause of GLM's empty reply.
  The handover says the default `max` effort spent the whole output cap on reasoning.
  The read shows the cap cut a reasoning stream before any answer
  (40652 reasoning characters,
  0 content characters,
  `finishReason: "length"`)
  and marks the `max` effort as its guess for why the reasoning ran that long.
- Bedrock.
  The readiness signal says Bedrock's key is never renewed.
  The read shows an expired token (HTTP 401) with the meter at 5.02 USD,
  and says nothing about renewal.
- One cited line.
  The picture-stop read cited `reach-past-holds.ts:40` for `reachOfRequest`;
  at `3637592bd4` the function is defined at line 45,
  and this record cites 45.
  The other line numbers the read cited were re-checked against that commit.
- The handover names "a probe that closed short".
  The reads found three short closes:
  the introduced-defect probe of slice 3 in `zheermao21`
  (the slice shipped unchanged and was not cached;
  traced on 2026-10-21 (UTC):
  the slice did change and only the caching was withheld,
  and the line is fixed,
  ledger B733),
  the absolute naturalness review of slice 11 in `zheermao21` (3 of 7 usable,
  recorded as evidence),
  and the consolidation `select` of slice 1 in `CuspariaKLSY21` (3 of 7 heard,
  recovered in the same stage by a recovery round).
- The repair lane's re-seat line.
  `chunk re-seated under a hold: ... unserved-checkers=...` printed 23 times in `zheermao21`
  and zero times in `CuspariaKLSY21`,
  though Bedrock was held out in both and `hf:Qwen/Qwen3.8-27B` was the unreachable third checker in all five checker rounds of the second.
  Cause not yet traced.
- The reads disagree with one another on no number they share:
  the baseline (3.1687 kWh),
  the day anchor (19:15:08.000 UTC),
  the pin and the digest are the same in both settled reads.

## What is not yet known

Another reading is tracing the findings of the two settled reads to causes and classing them.
Until it reports,
each finding in the page files carries the read's own label,
and the words "cause not yet traced" stand where the read found no code cause.
Its results are recorded in [the causes](translation-repair-openrouter-2026-09-03-2026-10-21-rotation-restart-causes.md).
Where that file settles an item of the lists that follow,
a note says so beside the item and the original text stands.

Traced on 2026-10-21 (UTC):
the 0.372 Wh gap is an in-flight hold (`reservedWh`),
`withheld=` and `unserved=` are two different rules,
and the log line that said a changed slice "ships unchanged" is fixed (ledger B733).
Still open after it:
the 21.244 Wh gap to `zheermao20`,
the missing re-seat line of `CuspariaKLSY21`,
and the items the causes file lists under "What is still not traced".
Open items the reads left,
by file:

- `zheermao21` run:
  the 0.372 Wh gap between the meter and the account,
  the 21.244 Wh gap to `zheermao20`'s last reading,
  why Kimi is `withheld=` while Qwen is `unserved=`,
  the 2 asks by which the Synthetic `SEAT` lines exceed the `SPEND` lines,
  and why the local photo reads took 4.4 min longer than in `zheermao20`.
- `zheermao21` page:
  the declared-names sheet was not read,
  so D23 (the "epitaph" for 悼文) stays open.
- `CuspariaKLSY21` run:
  whether the missing re-seat line is a fault.
- `XingZ6016`:
  the OpenRouter endpoints behind two models were not checked for image support one by one,
  the allowlist state for `google/gemma-4-31b-it` is unverified,
  and Synthetic's weekly reset time was not read.

## What follows

From [the readiness signal](translation-repair-readiness-signal.md) and [the handover](../handover/translation-repair-handover-2026-09-06-neuralwatt-provider-2026-10-20-zheermao20-to-zheermao21-reads-and-the-w253-to-w257-fix-rounds.md#next):

- The rotation on this build cannot finish with every page SETTLED,
  so no further entry runs on it,
  and OpenRouter's balance is kept for runs that count.
- The rotation restarts at `zheermao101` on the build that lands the archive review fixes (B712 to B715 and their follow-ups)
  with a picture bench that two wet readers can corroborate.
- The owner ruled on 2026-10-21 that `google/gemma-4-26b-a4b-it` on OpenRouter is measured as a second picture reader,
  and that NeuralWatt requests may carry documented parameters
  (no other provider may).
  The picture-stop file records both.
