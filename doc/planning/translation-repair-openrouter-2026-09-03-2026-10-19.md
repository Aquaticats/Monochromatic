# OpenRouter and the corpus passes: 2026-10-19

What was launched,
read and decided on 2026-10-19.
Entries:
zheermao14,
CuspariaKLSY16,
CuspariaKLSY17,
XingZ6015.

Part of [Translation repair: OpenRouter as the paid fallback provider](translation-repair-openrouter-2026-09-03.md),
which lists every file of this record.

## zheermao14 read, 2026-10-19, 17:00 UTC: SETTLED in 77.9 min of pass; no class

zheermao14 was the first entry of the readiness rotation the owner chose that day,
one entry at a time over the read set.
It launched at 15:42 UTC at `9c130e74a6` through the runbook's unit form into a fresh runs directory,
with `--only zheermao101`,
on pipeline `sha256-tree-v1:a92725968adf…` (`START ... files=3`),
built by the worktree's `rolldown v1.2.13`.
It settled in 4,673,785 ms of pass
(`TALLY zheermao101 status=SETTLED slices=15 repairStatus=repaired ... selection=contested`),
and the unit ended at about 17:00 UTC with `Result=success` and exit status 0.
`verify-published` read the page back:
1 of 1 page carries every wording its artifact promised,
at the length it implies
(`wordings=15 silent=0 chars=5878=expected missing=0 consistency-groups=0 read-at-generation=19`).
`DESTINATIONS zheermao101 source=2 page=3 dropped=0`,
no `DEFECTS` line,
no `[error]` line,
and 225 `warn` lines.
The refusal vocabulary of the seven steps finds nothing:
no floor refusal,
no `REATTEMPT`,
no `STALLED`,
no `mdx-downgraded`,
no `StreamOverrunError`,
no error finish.

### Seats and meters

Three Gemma seats were dark:
`gemma-4-26b-a4b-it` and `google.gemma-4-e2b` threw on all four asks each,
and `google.gemma-4-31b` on all three,
each opening with `SEAT CREDENTIAL REFUSED ... HTTP 401`
(`SEATS DARK: 3 of 12 seats asked produced nothing usable this run`).
The owner will not top up Bedrock,
so the rotation runs without them.
The other nine seats answered every ask;
Qwen3.8-27B had 11 replies abandoned at the straggler deadline and Kimi-K3 2.
The meters read every provider wet in all 52 readings.
Synthetic's weekly budget fell from 86.76% to 75.88%,
Hyper from 250 to 212,
OpenRouter from 22.12 to 22.00 USD,
and Bedrock stayed at 5.02 USD.
The spend report prices the metered seats at 54.69 credits
and OpenRouter at 0.1136 USD;
the `cost=` fields sum to the same 0.1136.
Synthetic's weekly meter is the account's,
so the 10.9 points are an upper bound on this run's share.

### Where the wall clock went

zheermao12 and zheermao13 settled this entry in 21.6 and 21.0 minutes on 2026-09-24 and 2026-09-25,
and the two-lane cost run of 2026-08-17 in 65.3;
what moved the time between those builds and this one was not measured.
By lane the slices summed 63.6 min of repair,
21.0 of translate and 59.7 of consolidation,
which overlap in the pass.
Rounds waited 1.92 h after quorum,
60.8% of round time;
calls in flight averaged 4.14 and peaked at 20.

### The page

Read beside the source and the archive,
all three open:

- Front matter equal to the archive's,
  2 headings as the archive,
  no Han,
  no TA,
  no double blank line,
  no dash.
- The three straight apostrophes after a letter are the `PhotoScroll` path literals,
  as in the archive;
  every apostrophe in prose is curly,
  and every ellipsis is three dots
  (6 on the page,
  7 in the archive,
  no U+2026).
- Canadian spelling:
  "favourite" twice,
  where the archive writes "favorite".
- The opening quote renders 奶茶 as "milk tea",
  where the archive has "beverages";
  `desc` keeps the archive's wording by the front-matter rule.
- The reply sentence reads "One Among Us members saw it and sent her a reply that same night:",
  plural and 连夜 carried,
  and the later paragraph "One Among Us members contacted her on Telegram".
  The pass warned `introduced repetition in slices 9, 11 (archive 0, shipped 2): "One Among Us members"`;
  the source repeats 那些秋叶的成员 in both,
  so the repetition is the original's.
- Labels one line each:
  "English translation of the letter from Arts:"
  and "English translation of this email conversation:".
- The email's sentence the archive left as a translator's note
  (`{/* 可以避开黑暗的锋芒，到那些有光的地方躲一躲。 这句不知道如何翻译 */}`)
  is translated:
  "you can slip past the darkness’s edge and take shelter where there is light."
  The note is gone with the gap it named.
- Two `>   > ` blank lines in the email end in a space.
  One is the archive's own;
  the other separates the closing line "Wishing you the blessings of spring.",
  which the archive lacks,
  and copies the archive's form byte for byte.
- The galgame title reads *Mystery Lover 2:
  Forgotten Truth*,
  where the archive writes *Mystery Lover 2*.
  The run's title lookup,
  cached since 2026-09-19,
  carries VNDB `v36674` with one English release under that title,
  two indienova pages giving it,
  and the Nintendo Hong Kong eShop listing English;
  the Steam release has no English.
  The lead first called the subtitle invented from the Steam page alone,
  and retracted it on reading the lookup
  (M279 in the package's audit ledger).
- Narration of her life is in the past tense throughout;
  the letter and the email,
  written to her while she lived,
  keep theirs.

Slices 0,
5 and 11 were archive-disputed:
the repair lane's adjudicators accepted 3,
2 and 3 accuracy claims against the archive's rendering,
so the repair text stood in (classes one hundred seven and one hundred seventy-six).
Slice 11's refine was rolled back by its probe
(`refine-rolled-back-by-probe (1 added-damage and 0 removal claims admitted against the rewrite)`).
No class.

## CuspariaKLSY16 stopped at its START line, 2026-10-19, 17:07 UTC: a different build

CuspariaKLSY16 launched at 17:06 UTC at `2eeef91fac`.
Between `9c130e74a6` and that commit no family source changed:
the merges brought the KaTeX removal and `main`'s auto-mode,
Depot and `mise.toml` changes.
Its `START` line read `pipeline=sha256-tree-v1:475fbdbb…`,
not the rotation's `a92725968adf…`,
and the unit was stopped after five `SPEND` lines.
The build had run `rolldown v1.2.14`,
the main checkout's,
because the launch passed the session shell's whole `PATH`
(M278 in the package's audit ledger);
the worktree's lockfile installs `1.2.13`.
The two bundles differ only where the bundler wraps CommonJS modules.
Nothing from this run is read.

## CuspariaKLSY17 launched, 2026-10-19, 17:09 UTC

CuspariaKLSY17 launched at 17:09 UTC at `2eeef91fac`,
with `--only CuspariaKLSY`,
into a fresh runs directory,
with the main-checkout entries removed from `PATH`
and the worktree's `node_modules/.bin` first.
Its build ran `rolldown v1.2.13`,
and `START` read `pipeline=sha256-tree-v1:a92725968adf…`,
the rotation's build.

## CuspariaKLSY17 read, 2026-10-19, 18:12 UTC: SETTLED in 59.0 min of pass; no class

It settled in 3,537,566 ms of pass
(`TALLY CuspariaKLSY status=SETTLED slices=4 repairStatus=repaired ... alignmentFindings=1 selection=contested`),
and the unit ended at about 18:08 UTC with `Result=success` and exit status 0.
`verify-published` read the page back:
1 of 1 page carries every wording its artifact promised,
at the length it implies
(`wordings=4 silent=0 chars=1831=expected missing=0 consistency-groups=0 read-at-generation=19`).
`DESTINATIONS CuspariaKLSY source=0 page=0 dropped=0`,
no `DEFECTS` line,
no `[error]` line,
and 161 `warn` lines.
The refusal vocabulary of the seven steps finds nothing:
no floor refusal,
no `REATTEMPT`,
no `STALLED`,
no `mdx-downgraded`,
no `StreamOverrunError`,
no error finish.
Block pairing paired 18 of 18 blocks on both sides from 5 voices.

### Seats and meters

Two Gemma seats were dark,
`google.gemma-4-e2b` and `gemma-4-26b-a4b-it`,
each throwing on all four asks after `SEAT CREDENTIAL REFUSED ... HTTP 401`
(`SEATS DARK: 2 of 11 seats asked produced nothing usable this run`);
`google.gemma-4-31b` was not asked.
The other nine seats answered;
Qwen3.8-27B had 12 replies abandoned at the straggler deadline,
one of them an introduced-defect probe cut after 51,935 reasoning characters and no answer,
and DeepSeek-V4.1-Flash 1.
The meters read every provider wet in all 32 readings.
Synthetic's weekly budget fell from 75.80% to 71.57%,
Hyper from 211 to 193,
OpenRouter from 22.00 to 21.92 USD,
and Bedrock stayed at 5.02 USD.
The spend report prices the metered seats at 20.97 credits
and OpenRouter at 0.0613 USD,
6 calls reckoned rather than reported.

### Where the wall clock went

The 91 rounds took 2.52 h in all,
1.23 h of it waiting after quorum
(48.7% of round time);
calls in flight averaged 4.11 and peaked at 20,
4.04 h of calls across 58.96 min of run.
The repair lane ran from 17:09 to 17:37 UTC,
translate from 17:37 to 17:51 UTC,
and consolidation from 17:51 UTC to the end.

### The page

Read beside the source and the archive,
all three open:

- Front matter equal to the archive's,
  1 heading as the archive,
  no Han,
  no TA,
  no double blank line,
  no trailing space,
  no straight apostrophe after a letter.
- Narration is in the past tense throughout,
  where the archive keeps the present
  ("She loves Sunflowers",
  "Her first language is Russian").
- "She thought she didn't have many friends",
  "She often told me she was going without meals" (吃不上饭)
  and "She wasn't much of a believer in God" (不是太信上帝)
  follow the source where the archive drifts
  ("cannot have a good meal",
  "doesn't trust God 100%").
- One em dash:
  "while feeding fish—the name means “fish in clear water.”"
  The gloss is the archive's own second sentence,
  which the source does not have;
  no house rule governs a dash on a page.
- The list of reasons follows the source item by item,
  item 2 now with 一直在观望 ("She had been watching from the sidelines")
  and ACG expanded once;
  the markers are `1. ` throughout,
  as in the source,
  where the archive mixes `1.  ` and `2. `.
- The last paragraph follows the source:
  when word came again she "had been confirmed brain-dead" (确认脑死亡),
  where the archive says only "confirmed to be dead".
  The archive's "She swallowed lots of her meds that night" is gone;
  the source does not say it,
  and slice 3 was archive-disputed on 14 accepted claims.

Slices 0,
1 and 3 were archive-disputed:
the repair lane's adjudicators accepted 3,
1 and 14 claims against the archive's rendering,
so the repair text stood in (classes one hundred seven and one hundred seventy-six).
Every introduced-defect probe found nothing.
No class.

## XingZ6015 launched, 2026-10-19, 18:14 UTC

XingZ6015 launched at 18:14 UTC at `2eeef91fac`,
with `--only XingZ60`,
into a fresh runs directory,
by `launch-rotation.ts`,
which removed 51 main-checkout entries from `PATH`.
Its build ran `rolldown v1.2.13`,
and `START` read `pipeline=sha256-tree-v1:a92725968adf…`,
the rotation's build.
