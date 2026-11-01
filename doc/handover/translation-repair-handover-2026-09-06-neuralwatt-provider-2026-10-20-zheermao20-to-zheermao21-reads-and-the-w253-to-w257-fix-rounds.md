# Translation repair handover 2026-09-06: the NeuralWatt rotation reads and the first fix rounds, 2026-10-21

The runs `zheermao20` and `CuspariaKLSY20` and their reads,
the seating investigation,
the quota session and rotation sweep fixes (w253),
the seating fix (w254),
the archive block review round (w255 to w257),
and the reads of `zheermao21` and `CuspariaKLSY21`.

Part of [Translation repair handover snapshot: 2026-09-06](translation-repair-handover-2026-09-06.md),
which lists every file of this record.
It continues [NeuralWatt as a provider](translation-repair-handover-2026-09-06-neuralwatt-provider-2026-10-20.md),
which holds the owner's terms.

## Next

The `zheermao19` read is recorded in the pass log of 2026-10-20:
SETTLED with no class traced to code,
on a short bench,
its page losing several intensifiers and small words zheermao17's had kept.
Done since:
w249 landed as `624c5ff583` (ledger P18);
the energy recalibration
(measured means for the 4,096 to 16,384 token bucket and a margin of 1,
ledger P19),
w247's account-wide day with its alias rows,
and the retry of an unknown day anchor
(ledger P17)
were merged in w252,
gated by g136
(2680 pass,
0 fail)
and landed as `31ce5369e1` on 2026-10-21 (UTC).
The branch worktree and w252 digest that build alike
(`sha256-tree-v1:e9eda9fe032032f4896f29b0c82099ae5b04084f209ce37144bb83d78eadcfcd`).
No cache version moved:
every commit since `5b5dca49a9` outside the provider package is a type-only change,
the credential hold
or the DeepSeek 1017 ids,
and `dadeb42ba6` found no cache key that reads a served id.
Its body left reusing decisions cached under the pre-1017 weights to the owner;
this session decided to keep them,
since a version bump discards every model's cached decisions of that stage,
not only DeepSeek's
(veto-open).

`zheermao20` (entry zheermao101) runs from 02:15 UTC on 2026-10-21 on that build.
Its log shows the anchor learned from the quota
(`NeuralWatt day starts at 19:15:08.000 UTC every day`),
the by-model usage read,
NeuralWatt wet
and DeepSeek and GLM calls settling on it.
It settled in 1,977,563 ms of pass
(`TALLY zheermao101 status=SETTLED slices=15 ... repairAccepted=13 ... pageChanged=8 ... selection=contested`),
and its end report read
`NeuralWatt plan energy: no plan energy was spent between the start and the end of the run`.
Its 215 settled NeuralWatt calls measured 31.33 Wh in all.
In the 4,096 to 16,384 token bucket the measured means
are DeepSeek 0.0819 Wh over 156 calls against 0.0758 Wh estimated
and GLM 0.3686 Wh over 49 calls against 0.3725 Wh,
so the bucket's recalibration holds;
the other buckets hold 2 to 4 calls each,
too few to recalibrate.
OpenRouter's reported cost was USD 0.498.

The page read is recorded in
[the pass log of 2026-10-21](../planning/translation-repair-openrouter-2026-09-03-2026-10-21.md):
no class traced to code;
the letter fell back toward the archive,
because the archive block review picks one whole revision
and only two narrow revisions reached it.
An investigation of two seating questions ran as a read-only Sonnet agent.
The seating questions:
why `glm-5.3` was seated at preparation though its every provider was out of budget,
and why `hf:Qwen/Qwen3.8-27B` stayed seatable,
and was re-seated as a checker under a hold,
after Synthetic ran dry,
when `hf:moonshotai/Kimi-K3` was withheld
(it threw 137 of 158 asks).
The investigation found the seating as documented.
`doc/decision/translation-repair-short-bench-share.md` keeps a seat no wet provider serves on the bench,
to be lost at call time.
The checker bench pads with unserved seats up to three.
Kimi-K3 was withheld by the owner's OpenRouter cost rule
(`OPENROUTER_WITHHELD`),
which does not name Qwen.
One defect stands,
checked against `package/translation-repair/stage/src/seat-tally.ts`:
`recordThrow` counts the router's own refusal (`NoProviderForModelError`) as an ask that threw,
where the decision says a refusal is not an ask.
So `glm-5.3` reads `asked=4 threw=4` though it was never sent a request,
Qwen's 137 throws are almost all such refusals,
and the tally withholds such a seat for the run even if a provider becomes wet again.
Neither `JUDGE SEATS` nor the re-seat line names a seated seat no wet provider serves.
The fix,
red first,
is queued in keyless worktree w254:
the refusal counts as `unreachable`,
`JUDGE SEATS` names the unserved seats,
and the re-seat line marks its padding.
`CuspariaKLSY20` (entry CuspariaKLSY) ran on the same build from 02:52 UTC on 2026-10-21
and settled in 690,310 ms of pass
(`TALLY CuspariaKLSY status=SETTLED slices=4 ... repairAccepted=21 ... pageChanged=4 ... selection=contested`),
again with no plan energy spent;
its read in the same pass log found no class
and a better page than CuspariaKLSY19's.
Its 80 settled NeuralWatt calls measured 21.19 Wh.
In the 4,096 to 16,384 token bucket its means were well over the estimates:
DeepSeek 0.1996 Wh over 62 calls against 0.0758 Wh,
GLM 0.5500 Wh over 16 calls against 0.3725 Wh.
So the bucket's mean moves a great deal from entry to entry,
and two runs are too few to recalibrate from.
What an estimate costs when it runs low is the energy of calls in flight at the daily boundary,
since every settled call counts its measured energy;
the day stood at about 52 Wh of 550 after these two runs.
Open:
whether the estimate should follow the measured mean of the run in progress,
decided once more runs have settled.

XingZ60 is held until w253 and w254 land:
a fix that lands changes the build,
the rotation then restarts at `zheermao101`,
and a read of XingZ60 on this build would not count,
while it would spend a large share of the day's free energy.
The B701 rotation sweep landed as `7c9d5fa35f` after gate g137
(2690 pass,
0 fail).

Two defects the w247 and w251 agents reported were fixed in keyless worktree w253,
red first:

- `createNeuralWattQuotaSession` (`package/translation-repair/provider/src/neuralwatt-quota-session.ts`)
  cached a failed first quota read as the run's baseline for good,
  so when the day-anchor retry later read the quota,
  the end report still said the starting reading could not be taken.
  It now tries the baseline again no sooner than `NEURALWATT_QUOTA_RETRY_SECONDS` (60 s) later,
  takes the first successful `readNow` after a failed baseline as the baseline,
  and the end report says the comparison starts late
  (`laterStartAt`;
  ledger P20).
- A failed rotation sweep printed a stack trace where an operator-facing refusal is meant.
  It now prints a refusal naming the failing cases and exits 6,
  and a bad option is refused the same way by `SweepOptionRefusalError`
  (ledger B710).
  The process-global writes guard
  (`package/translation-repair/source-scan/src/global-writes-sequenced.unit.test.ts`)
  gains a `PROCESS_ENTRIES` list naming only `source-scan/rotation-sweep/rotation-sweep-cli.test-fixture.ts`:
  that fixture is a process's own entry,
  so its `process.exitCode` write outside any case is allowed there and nowhere else;
  its other findings still count,
  and the package case fails if the named file is gone.
  This session accepted the exception after reading the diff
  (veto-open).

The seating defect was fixed in keyless worktree w254,
red first
(ledger B711):
the router's refusal counts as `unreachable`,
never as an ask or a throw,
so it moves no withholding threshold;
the `SEAT` line prints `unreachable=`;
`JUDGE SEATS` prints `unserved=`;
and the repair lane's re-seat line ends with `unserved-checkers=`.
This session accepted the agent's choices
(veto-open):
a refusal after the caller aborted counts `unreachable` rather than `abandoned`;
a seat the owner's cull refuses counts `unreachable` too;
and only the repair lane's re-seat line marks padding.
Both agents took B710 as the next free id;
w253 numbered first,
so w254's entry became B711 in the merge commit `22cd3e200c`,
and the subjects of `1ce1e851b0`
and `29ce2d9daa`
and `f239810f90` still say B710.
Gate g138 on that merge was green
(145 steps as in g137;
2693 pass,
0 fail),
and it landed as `3637592bd4` on 2026-10-21 (UTC).
Corrective commit comments name B711 on the three subjects
and the case count on `da0ec11dd6`.
The branch worktree was rebuilt at `3637592bd4`;
it and w254 build alike
(`sha256-tree-v1:06c97a69446dc452fa2c602da834c36febb7f0d7b4b1010b5dfa9fab81097eec`).
Since w254 added scripted cases that name seats,
the rotation sweep runs on w254 before the rotation restarts at `zheermao101`.

A read-only investigation of the zheermao20 letter
(recorded in the pass log of 2026-10-21)
found the archive block review as documented,
with three gaps,
and a fourth question.
This session decided to fix all four,
red first,
in keyless worktrees based on `22cd3e200c`,
with ledger ids assigned in advance so the merge does not collide again:

- B712:
  a composite candidate built from revisions that keep the block's line count and change disjoint lines,
  checked by the same shape and footnote gates,
  as the editor lane already builds
  (`package/translation-repair/repair/src/editor-candidates.ts`)
  (veto-open).
- B713:
  the review round gets the writer window
  (180000ms),
  since its revise replies are writer voices under the owner's writer rule;
  the select round keeps 120000ms
  (veto-open).
- B714:
  `repairArchiveBlocks` keeps the stage's findings in the artifact,
  so a block that lost a writer shows it.
- B715:
  trace why both `source-supported` replies on the letter were dropped as unanchored,
  and fix it if the anchor test refuses a valid quote.

B712 to B714 run in w255,
B715 in w256.
B715 is done in w256
(`4e0cbc1b5f` to `c9e9f735d9`;
not landed):
one of the two dropped replies was refused because the anchor compared a quote with a picture transcript's hard line breaks,
and now anchors through the whitespace-free `quoteIsIn` the reference attestation uses;
the other stitches three runs from two pictures and stays refused by design;
every dropped reply is now logged with its reason.
Accepted veto-open:
whitespace set aside entirely rather than line breaks only,
the sheet's wording unchanged,
and quotes spanning pictures still refused.
After landing,
post corrective commit comments on `4e0cbc1b5f` and `e0dbf84a38`:
their messages say the stitched quote also reaches photo3,
which the replay disproves.
B712 to B714 are done in w255
(`a38bc52e29` to `bfff357798`;
not landed),
each red first,
with the package units,
lint and source scans green and no cache version moving.
Accepted veto-open beyond the review round's 180 s window:
no composite when any changing revision differs in line count,
an identical change to one line counts as an overlap,
three revisions with only two disjoint build none,
a revision equal to the archive contributes nothing,
and the stage's findings travel verbatim,
reviewer reasons included.
After landing,
post corrective commit comments on `49194af623`
(16 cases,
15 of them red,
not fifteen)
and `96d9d0be32`
(five files pin the launch line,
not four).
w255 and w256 merged in w257 at `0818ca282a`
(two index-list conflicts,
both sides kept).
In w257,
red first,
before gate g139:
the composite reaches the judges with its contributors' assessments
(it reached them with none,
because evidence is matched by a voice's proposed text);
`archive-block-repair.ts` refuses a missed `archiveBlockSourceContexts` lookup
instead of anchoring against `''`;
and the launch line drops B713's dated veto note,
which belongs in the decision doc,
not in every run log.
Then gate g139 on w257,
land,
the corrective comments,
rebuild the branch worktree,
a new frozen digest,
the rotation sweep,
and the rotation restarts at `zheermao101`.
On the g138 build,
`zheermao21` and `CuspariaKLSY21` shipped SETTLED
(reads running),
and `XingZ6016` stopped INCOMPLETE:
its one picture got a single reading
(`minimax-m3`;
`GLM-5.3-Flash` replied empty,
Qwen's provider out of budget,
both Gemma readers on Bedrock),
so nothing corroborated it.
The rotation on that build is held there,
since it can no longer finish with every page SETTLED.
The two settled reads
(`zheermao21-read.md` and `CuspariaKLSY21-read.md` in the reads folder)
are being traced to causes before any of them is called a class:
a 2 to 2 polish gate keeping "was given to her" for 起的,
a repair correction accepted 4 to 0 lost when the translate lane won its slice,
已经,
一直,
观望 and 二次元 dropped,
the opening quote losing 有一天 and 你 to a minimum-weight rule,
the email's lines thinner than `zheermao20`'s,
two U+2014 dashes,
a probe that closed short,
and a 0.372 Wh gap between the meter and the account.
The 抢救 rendering "doctors fought to save her" is the glossary's on purpose.
Classes found there are fixed before gate g139 too,
since a restarted rotation would meet them again.
