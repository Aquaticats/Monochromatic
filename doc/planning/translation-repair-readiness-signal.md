# Translation repair: the production readiness signal, and what it rests on

Proposal for the owner,
written 2026-08-26 ahead of the `AskUserQuestion` call the readiness task prescribes.
The signal is the owner's cue to disable branch protection temporarily so the corpus text committed along
the way can be sanitized (`doc/handover/translation-repair.md`,
"Corpus exposure is not a blocker").
This document is the evidence behind the question and the decisions the question bundles apart.

## Contents

This record is kept as one file per subject or span,
each of at most 1,000 lines and named for what it holds.
New text goes into the file whose subject it belongs to;
a file that would pass the limit gives up a subject to a new file,
named for that subject and listed here.

- [Readiness signal: the proposal and the owner's answers of 2026-08-26](translation-repair-readiness-signal-proposal-and-owner-answers-2026-08-26.md):
  What the readiness signal first rested on,
  what was known and not done,
  and the sanitization inventory;
  the four decisions the question kept apart and the owner's answers,
  including the rejection that sent the work to the actual output;
  what changed after the answers the same day,
  and what had to come before any re-signal.
  Sections:
  - [What "ready" rests on](translation-repair-readiness-signal-proposal-and-owner-answers-2026-08-26.md#what-ready-rests-on)
  - [What is known and not done](translation-repair-readiness-signal-proposal-and-owner-answers-2026-08-26.md#what-is-known-and-not-done)
  - [Sanitization inventory](translation-repair-readiness-signal-proposal-and-owner-answers-2026-08-26.md#sanitization-inventory)
  - [The decisions the question keeps apart](translation-repair-readiness-signal-proposal-and-owner-answers-2026-08-26.md#the-decisions-the-question-keeps-apart)
  - [The owner's answers (2026-08-26)](translation-repair-readiness-signal-proposal-and-owner-answers-2026-08-26.md#the-owners-answers-2026-08-26)
  - [Since the answers (2026-08-26, updated the same day)](translation-repair-readiness-signal-proposal-and-owner-answers-2026-08-26.md#since-the-answers-2026-08-26-updated-the-same-day)
- [Readiness signal: the pages and launches read on 2026-09-04](translation-repair-readiness-signal-pages-read-on-2026-09-04.md):
  The six passes read that day and the three defects no gate had found;
  the luxuanwen3 runs on three builds,
  the SS3B_0016 page and the Uekawakuyuurei attempts;
  each launch's reading of the belief and what would move it.
  Sections:
  - [What the 2026-09-04 pages showed](translation-repair-readiness-signal-pages-read-on-2026-09-04.md#what-the-2026-09-04-pages-showed)
  - [What the luxuanwen3 re-run of 2026-09-04 showed](translation-repair-readiness-signal-pages-read-on-2026-09-04.md#what-the-luxuanwen3-re-run-of-2026-09-04-showed)
  - [What the luxuanwen3 re-run on the either-rendering build showed (2026-09-04, 12:43 UTC)](translation-repair-readiness-signal-pages-read-on-2026-09-04.md#what-the-luxuanwen3-re-run-on-the-either-rendering-build-showed-2026-09-04-1243-utc)
  - [What the two runs on the publisher fix showed (2026-09-04, 13:51 UTC)](translation-repair-readiness-signal-pages-read-on-2026-09-04.md#what-the-two-runs-on-the-publisher-fix-showed-2026-09-04-1351-utc)
  - [What the Uekawakuyuurei attempt on the pronoun build showed (2026-09-04, 15:03 UTC)](translation-repair-readiness-signal-pages-read-on-2026-09-04.md#what-the-uekawakuyuurei-attempt-on-the-pronoun-build-showed-2026-09-04-1503-utc)
  - [What Uekawakuyuurei on the picture-fix build showed (2026-09-04, 15:25 UTC)](translation-repair-readiness-signal-pages-read-on-2026-09-04.md#what-uekawakuyuurei-on-the-picture-fix-build-showed-2026-09-04-1525-utc)
- [Readiness signal: the yulianNyanner launches and the TLL1122 page, 2026-09-06 to 2026-09-07](translation-repair-readiness-signal-yulian-nyanner-and-tll1122-launches-2026-09-06-to-2026-09-07.md):
  The change to the plain invocation on 2026-09-06 and the first run on it;
  the pages on the class-six,
  class-seven and class-eight builds,
  each finding the next class;
  the TLL1122 page,
  the first read page with a footnote,
  which found no class.
  Sections:
  - [What 2026-09-06 changed before the next page](translation-repair-readiness-signal-yulian-nyanner-and-tll1122-launches-2026-09-06-to-2026-09-07.md#what-2026-09-06-changed-before-the-next-page)
  - [What the first plain-invocation run showed (2026-09-06, 20:21 UTC)](translation-repair-readiness-signal-yulian-nyanner-and-tll1122-launches-2026-09-06-to-2026-09-07.md#what-the-first-plain-invocation-run-showed-2026-09-06-2021-utc)
  - [What the first page on the plain invocation showed (2026-09-06, 21:03 UTC)](translation-repair-readiness-signal-yulian-nyanner-and-tll1122-launches-2026-09-06-to-2026-09-07.md#what-the-first-page-on-the-plain-invocation-showed-2026-09-06-2103-utc)
  - [What the page on the class-six build showed (2026-09-06, 22:10 UTC)](translation-repair-readiness-signal-yulian-nyanner-and-tll1122-launches-2026-09-06-to-2026-09-07.md#what-the-page-on-the-class-six-build-showed-2026-09-06-2210-utc)
  - [What the page on the class-seven build showed (2026-09-06, 23:11 UTC)](translation-repair-readiness-signal-yulian-nyanner-and-tll1122-launches-2026-09-06-to-2026-09-07.md#what-the-page-on-the-class-seven-build-showed-2026-09-06-2311-utc)
  - [What the page on the class-eight build showed (2026-09-07, 00:12 UTC)](translation-repair-readiness-signal-yulian-nyanner-and-tll1122-launches-2026-09-06-to-2026-09-07.md#what-the-page-on-the-class-eight-build-showed-2026-09-07-0012-utc)
  - [What the TLL1122 page showed (2026-09-07, 01:11 UTC)](translation-repair-readiness-signal-yulian-nyanner-and-tll1122-launches-2026-09-06-to-2026-09-07.md#what-the-tll1122-page-showed-2026-09-07-0111-utc)
- [Readiness signal: the four Huasheng launches of 2026-09-07](translation-repair-readiness-signal-huasheng-launches-2026-09-07.md):
  The container class found before the first page,
  and the relaunch that stopped at the block floor;
  the directory-name refusal,
  with the owner's decision on the block floor and on the name check;
  the fourth launch,
  which settled with a page and found the provider-limit class.
  Sections:
  - [What the Huasheng attempt showed (2026-09-07, 01:44 UTC)](translation-repair-readiness-signal-huasheng-launches-2026-09-07.md#what-the-huasheng-attempt-showed-2026-09-07-0144-utc)
  - [What the Huasheng relaunch showed (2026-09-07, 03:09 UTC)](translation-repair-readiness-signal-huasheng-launches-2026-09-07.md#what-the-huasheng-relaunch-showed-2026-09-07-0309-utc)
  - [What the third Huasheng launch showed (2026-09-07, 08:43 UTC)](translation-repair-readiness-signal-huasheng-launches-2026-09-07.md#what-the-third-huasheng-launch-showed-2026-09-07-0843-utc)
  - [What the fourth Huasheng launch showed (2026-09-07, 15:43 UTC)](translation-repair-readiness-signal-huasheng-launches-2026-09-07.md#what-the-fourth-huasheng-launch-showed-2026-09-07-1543-utc)
- [Readiness signal: the hakureico launches and the front matter decision, from 2026-09-07 evening to the small hours of 2026-09-09](translation-repair-readiness-signal-hakureico-and-yuki-launches-2026-09-07-to-2026-09-09.md):
  The first to twelfth hakureico launches,
  from the wait after a hold to the whole-bench page;
  the owner's decision to publish the archive's front matter as is,
  and the E2B calibration and merge;
  the yuki418330012 launches and the letter's landing,
  whose closing paragraphs run on to launches started at 00:22 and 01:35 UTC on 2026-09-09.
  Sections:
  - [What the three hakureico launches of 2026-09-07 showed (21:00 to 22:00 UTC)](translation-repair-readiness-signal-hakureico-and-yuki-launches-2026-09-07-to-2026-09-09.md#what-the-three-hakureico-launches-of-2026-09-07-showed-2100-to-2200-utc)
  - [What the sixth hakureico launch showed (2026-09-08, 00:38 UTC)](translation-repair-readiness-signal-hakureico-and-yuki-launches-2026-09-07-to-2026-09-09.md#what-the-sixth-hakureico-launch-showed-2026-09-08-0038-utc)
  - [What the seventh and eighth hakureico launches on Bedrock alone showed (2026-09-08, 01:49 and 02:19 UTC)](translation-repair-readiness-signal-hakureico-and-yuki-launches-2026-09-07-to-2026-09-09.md#what-the-seventh-and-eighth-hakureico-launches-on-bedrock-alone-showed-2026-09-08-0149-and-0219-utc)
  - [What the ninth hakureico launch on four wet providers showed (2026-09-08, 11:27 UTC)](translation-repair-readiness-signal-hakureico-and-yuki-launches-2026-09-07-to-2026-09-09.md#what-the-ninth-hakureico-launch-on-four-wet-providers-showed-2026-09-08-1127-utc)
  - [What the owner decided on the front matter (2026-09-08, 13:15 UTC)](translation-repair-readiness-signal-hakureico-and-yuki-launches-2026-09-07-to-2026-09-09.md#what-the-owner-decided-on-the-front-matter-2026-09-08-1315-utc)
  - [What the E2B calibration and the merge showed (2026-09-08, 16:07 to 16:18 UTC)](translation-repair-readiness-signal-hakureico-and-yuki-launches-2026-09-07-to-2026-09-09.md#what-the-e2b-calibration-and-the-merge-showed-2026-09-08-1607-to-1618-utc)
  - [What the tenth hakureico launch showed (2026-09-08, 16:18 to 16:55 UTC)](translation-repair-readiness-signal-hakureico-and-yuki-launches-2026-09-07-to-2026-09-09.md#what-the-tenth-hakureico-launch-showed-2026-09-08-1618-to-1655-utc)
  - [What the eleventh hakureico launch showed (2026-09-08, 16:56 to 18:21 UTC)](translation-repair-readiness-signal-hakureico-and-yuki-launches-2026-09-07-to-2026-09-09.md#what-the-eleventh-hakureico-launch-showed-2026-09-08-1656-to-1821-utc)
  - [What the twelfth hakureico launch showed (2026-09-08, 18:21 to 20:10 UTC)](translation-repair-readiness-signal-hakureico-and-yuki-launches-2026-09-07-to-2026-09-09.md#what-the-twelfth-hakureico-launch-showed-2026-09-08-1821-to-2010-utc)
  - [What the yuki418330012 launch and the letter's landing showed (2026-09-08, 20:18 to 22:10 UTC)](translation-repair-readiness-signal-hakureico-and-yuki-launches-2026-09-07-to-2026-09-09.md#what-the-yuki418330012-launch-and-the-letters-landing-showed-2026-09-08-2018-to-2210-utc)
- [Readiness signal: the launches read on 2026-09-09, from 01:35 to 23:32 UTC](translation-repair-readiness-signal-launches-2026-09-09.md):
  The seventh yuki418330012 launch and hakureico launches nineteen to twenty-three,
  with the four faces of the footnote class;
  the fourth gqt launch,
  the noname launches,
  the Mio and hulicaijia launches,
  and the eighth Mio launch that ended the day.
  Sections:
  - [What the seventh yuki418330012 launch showed (2026-09-09, 01:35 to 02:51 UTC)](translation-repair-readiness-signal-launches-2026-09-09.md#what-the-seventh-yuki418330012-launch-showed-2026-09-09-0135-to-0251-utc)
  - [What the nineteenth hakureico launch showed (2026-09-09, 01:35 to 03:03 UTC)](translation-repair-readiness-signal-launches-2026-09-09.md#what-the-nineteenth-hakureico-launch-showed-2026-09-09-0135-to-0303-utc)
  - [What the twentieth hakureico launch showed (2026-09-09, 03:21 to 04:28 UTC)](translation-repair-readiness-signal-launches-2026-09-09.md#what-the-twentieth-hakureico-launch-showed-2026-09-09-0321-to-0428-utc)
  - [What the twenty-first hakureico launch showed (2026-09-09, 04:39 to 06:13 UTC)](translation-repair-readiness-signal-launches-2026-09-09.md#what-the-twenty-first-hakureico-launch-showed-2026-09-09-0439-to-0613-utc)
  - [What the twenty-second hakureico launch showed (2026-09-09, 06:20 to 07:46 UTC)](translation-repair-readiness-signal-launches-2026-09-09.md#what-the-twenty-second-hakureico-launch-showed-2026-09-09-0620-to-0746-utc)
  - [What the twenty-third hakureico launch showed (2026-09-09, 08:01 to 09:31 UTC)](translation-repair-readiness-signal-launches-2026-09-09.md#what-the-twenty-third-hakureico-launch-showed-2026-09-09-0801-to-0931-utc)
  - [What the fourth gqt launch showed (2026-09-09, 08:01 to 09:46 UTC)](translation-repair-readiness-signal-launches-2026-09-09.md#what-the-fourth-gqt-launch-showed-2026-09-09-0801-to-0946-utc)
  - [What the first noname launch showed (2026-09-09, 09:50 to 11:45 UTC)](translation-repair-readiness-signal-launches-2026-09-09.md#what-the-first-noname-launch-showed-2026-09-09-0950-to-1145-utc)
  - [What the second noname and sixth Mio launches showed (2026-09-09, 11:59 to 12:33 UTC)](translation-repair-readiness-signal-launches-2026-09-09.md#what-the-second-noname-and-sixth-mio-launches-showed-2026-09-09-1159-to-1233-utc)
  - [What the third noname launch showed (2026-09-09, 16:45 to 18:08 UTC)](translation-repair-readiness-signal-launches-2026-09-09.md#what-the-third-noname-launch-showed-2026-09-09-1645-to-1808-utc)
  - [What the Mio and hulicaijia launches showed (2026-09-09, 18:20 to 20:30 UTC)](translation-repair-readiness-signal-launches-2026-09-09.md#what-the-mio-and-hulicaijia-launches-showed-2026-09-09-1820-to-2030-utc)
  - [What the eighth Mio launch showed (2026-09-09, 21:58 to 23:32 UTC)](translation-repair-readiness-signal-launches-2026-09-09.md#what-the-eighth-mio-launch-showed-2026-09-09-2158-to-2332-utc)
- [Readiness signal: the Mio verse remedy and the evidence gathered from 2026-09-09 night to 2026-09-10](translation-repair-readiness-signal-mio-verse-remedy-and-follow-up-evidence-2026-09-10.md):
  The ninth and tenth Mio launches and the classes they found;
  the verse remedy measured,
  reviewed and withdrawn as a first claim,
  and the Mio12 page that preserved the poem;
  the naming and archive-correction evidence written by later sessions,
  in reverse order of writing.
  Sections:
  - [Archive correction has current-stage evidence; audit and temporal context remain (2026-09-10)](translation-repair-readiness-signal-mio-verse-remedy-and-follow-up-evidence-2026-09-10.md#archive-correction-has-current-stage-evidence-audit-and-temporal-context-remain-2026-09-10)
  - [Naming has current-stage evidence; archive and context work remains (2026-09-10)](translation-repair-readiness-signal-mio-verse-remedy-and-follow-up-evidence-2026-09-10.md#naming-has-current-stage-evidence-archive-and-context-work-remains-2026-09-10)
  - [The stored-review mismatch is fixed; wording follow-up remains (2026-09-10)](translation-repair-readiness-signal-mio-verse-remedy-and-follow-up-evidence-2026-09-10.md#the-stored-review-mismatch-is-fixed-wording-follow-up-remains-2026-09-10)
  - [Mio12 preserves published verse but does not establish readiness (2026-09-10)](translation-repair-readiness-signal-mio-verse-remedy-and-follow-up-evidence-2026-09-10.md#mio12-preserves-published-verse-but-does-not-establish-readiness-2026-09-10)
  - [The verse remedy has passage-level evidence (2026-09-10)](translation-repair-readiness-signal-mio-verse-remedy-and-follow-up-evidence-2026-09-10.md#the-verse-remedy-has-passage-level-evidence-2026-09-10)
  - [The upstream verse remedy is being measured (2026-09-10)](translation-repair-readiness-signal-mio-verse-remedy-and-follow-up-evidence-2026-09-10.md#the-upstream-verse-remedy-is-being-measured-2026-09-10)
  - [The verse remedy is under review (2026-09-10, 02:18 UTC)](translation-repair-readiness-signal-mio-verse-remedy-and-follow-up-evidence-2026-09-10.md#the-verse-remedy-is-under-review-2026-09-10-0218-utc)
  - [What Mio10 showed (2026-09-10, 00:15 to 01:36 UTC)](translation-repair-readiness-signal-mio-verse-remedy-and-follow-up-evidence-2026-09-10.md#what-mio10-showed-2026-09-10-0015-to-0136-utc)
  - [What the ninth Mio launch showed (2026-09-09, 23:49 UTC to 2026-09-10, 00:07 UTC)](translation-repair-readiness-signal-mio-verse-remedy-and-follow-up-evidence-2026-09-10.md#what-the-ninth-mio-launch-showed-2026-09-09-2349-utc-to-2026-09-10-0007-utc)

## The owner's words on the goal, 2026-09-04

Asked when a supervised sweep over all 92 entries should launch,
the owner answered:

> We never ever need a all 92 entries launch.
> If you believe it's ready for all 92 entries launch after reading the artifacts,
> that means we had reached the goal of making it production ready.

So the deliverable is the pipeline's readiness,
judged by reading its artifacts and pages,
and no 92-entry sweep is planned or budgeted.
The 2026-09-03 OpenRouter budget arithmetic that priced such a sweep in
[`translation-repair-openrouter-2026-09-03.md`](translation-repair-openrouter-2026-09-03.md)
is an upper bound on what the corpus would cost if it were ever run,
not a plan.
The readiness signal is put when a reading of the artifacts would let the sweep launch,
whether or not it does.

## Where readiness stands on 2026-09-24

The sections dated 2026-09-09 and 2026-09-10 are those days' readings and stay as evidence.
The current state is in the handover
([`translation-repair-handover-2026-09-06.md`](../handover/translation-repair-handover-2026-09-06.md),
"What to do next")
and the pass log;
this section is the readiness reading of it.

- The signal is still not given.
  The rule of this document holds:
  a class found on a read page is fixed before the next launch,
  and a page is read before the next entry is launched.
  Classes twenty-six to one hundred nineteen were found that way between 2026-09-16 and 2026-09-24,
  one hundred nineteen in all,
  and the rate has not fallen to zero:
  of the last fifteen reads (yingying8 at 00:31 UTC to CuspariaKLSY13 at 09:15 UTC 2026-09-24),
  four found no class,
  ten found one each
  (one hundred eight to one hundred sixteen,
  with the chain arm of one hundred eleven found on its own read)
  and CuspariaKLSY13 found two (one hundred seventeen,
  list spacing,
  and one hundred eighteen,
  GLM-5.3-Flash's latency on Synthetic).
  The eight reads before zheermao12 each found one,
  five of them on `mikaela_khara`,
  where the carried-insertion fold and the pairing met new shapes in a row,
  yingying10's on the translate slate rather than the page
  and zheermao11's on the clock (a translate lane of 616 s against 77,
  the page itself clean).
  zheermao12 found none,
  the class one hundred sixteen rule taking the translate lane to 62 s with the Hyper seats still asked.
  CuspariaKLSY13's page was faithful in wording,
  the death paragraph complete for the first time;
  its two classes are a spacing the archive keeps and a seat's latency,
  not a mistranslation.
  CuspariaKLSY14 and hulicaijia17 found no class.
  shi_Yumiaoya19 found one (one hundred nineteen):
  小药娘 shipped in Han,
  a third rendering in three runs,
  settled by the owner's answer of 2026-09-24 as "trans girl" or "trans woman" and built as a glossary entry with a floor.
  shi_Yumiaoya22 found none:
  both lines read "trans girl" on the page,
  with no Han and no pinyin outside the archive's comment.
  XingZ6010 found one (one hundred twenty):
  a page-assembly pass wrote a song title into a heading's `align` attribute because the tag spaces its equals sign,
  a broken page the bench did not cause;
  fixed and replayed over the run.
  mikaela17 found one (one hundred twenty-one):
  a judge preferred "Xiawafang" to the archive's XIAWAFANG,
  so the body and its footnote spelled the station two ways;
  the archive's all-capitals form is now restored at page assembly,
  and a corpus census shows the rule firing only on acronyms and names.
- What "ready" would rest on now:
  a rotation over every entry in the read set
  (`XingZ60`,
  `hulicaijia`,
  `mikaela_khara`,
  `shi_Yumiaoya`,
  `zheermao101`,
  `CuspariaKLSY`,
  `yingying`)
  in which every page ships SETTLED and every read finds no class,
  on one frozen build,
  under the seven-hour ceiling,
  on whichever providers are wet.
  No such rotation has completed;
  the longest run of no-class reads on one build is three
  (hulicaijia16,
  mikaela11 on `0f685f551`;
  shi_Yumiaoya17,
  yingying7,
  zheermao7 on `6361d74d0` before class one hundred three).
- The rotation the owner chose on 2026-10-19 runs the read set and `TianqiChen666` one entry at a time
  on pipeline `sha256-tree-v1:a92725968adf…`.
  Its first read,
  zheermao14,
  shipped SETTLED with no class
  ([the pass log of 2026-10-19](translation-repair-openrouter-2026-09-03-2026-10-19.md)),
  with three Gemma seats dark on HTTP 401 and every meter wet.
  The second,
  `CuspariaKLSY17`,
  shipped SETTLED with no class,
  with two Gemma seats dark on HTTP 401 and every meter wet;
  `CuspariaKLSY16` ran on another build and is not evidence.
  The third,
  `XingZ6015`,
  shipped SETTLED with five classes
  ([the pass log of 2026-10-20](translation-repair-openrouter-2026-09-03-2026-10-20.md)),
  so the rotation on that build ended there with two no-class reads,
  and `yingying` was not launched.
  The rotation restarted from its first entry on the build that fixes the five classes
  and the owner's reader-protection ruling of 2026-10-20
  (`0c64519b57`,
  digest `sha256-tree-v1:c003a383…`).
  `zheermao15` shipped SETTLED with one class,
  a note between translators lost when the archive block review replaced its block
  ([the pass log of 2026-10-20](translation-repair-openrouter-2026-09-03-2026-10-20.md)).
  `CuspariaKLSY18`,
  on the build that carries the fix for that note
  (`a6a0ed7dc0`,
  digest `sha256-tree-v1:1149dceac4ea…`),
  shipped SETTLED with no class.
  Three fixes to the cut of those notes landed after it
  (`a5dc23c5bb`,
  digest `sha256-tree-v1:16f46cb79c96…`),
  so neither read is on the build the rotation now runs,
  and the rotation restarted from `zheermao101` as `zheermao16` at 14:58 UTC on 2026-10-20.
- Fixes that change no shipped page do not restart the rotation.
  Three more fixes landed while `zheermao16` ran
  (ledger B704 to B706 in `package/translation-repair/doc/`):
  the naturalness review's sheet,
  whose findings are evidence only and never change the page;
  the footnote reorder,
  which changes no pinned page under either order measured;
  and the CRLF breaks of the note carry,
  which no pinned page reaches since the corpus read folds CRLF.
  The rotation goes on on the frozen build `a5dc23c5bb`,
  and the branch worktree stays there;
  these fixes reach the rotation with the next fix that changes a shipped page,
  which restarts it as before.
  Decided without asking the owner,
  under "Make decisions yourself and don't block on me",
  since a restart for each such fix would keep the rotation from finishing:
  a read takes about 50 min of pass.
- `zheermao16` shipped SETTLED with one class
  ([the pass log of 2026-10-20](translation-repair-openrouter-2026-09-03-2026-10-20.md)):
  the consolidation re-romanized and glossed the handle 寒涟漪,
  which the archive renders "Han Lianyi",
  so the rotation on `a5dc23c5bb` ended with no read that counts.
  `CuspariaKLSY19` ran on that build to find more classes,
  settled with no class,
  and does not count.
  No further hunt runs there,
  since Hyper's 111 left is kept for the runs that count;
  the rotation restarts at `zheermao101` on the build that carries the fix.
- It restarted on `5ae2173d23`,
  the build with the fixes of B707,
  B708 and B709:
  `zheermao17` shipped SETTLED with no class
  ([the pass log of 2026-10-20](translation-repair-openrouter-2026-09-03-2026-10-20.md)),
  the first read that counts on that build.
  It spent 65 of Hyper's 110,
  so `CuspariaKLSY` was not launched:
  the NeuralWatt provider the owner added on 2026-10-20 serves two of the seats Hyper paid for,
  it changes the build,
  and a read on `5ae2173d23` would stop counting once it lands.
  The rotation restarts at `zheermao101` on the build that carries NeuralWatt.
- It restarted on `5b5dca49a9`:
  `zheermao19` shipped SETTLED with no class traced to code
  ([the pass log of 2026-10-20](translation-repair-openrouter-2026-09-03-2026-10-20.md)),
  but on a short bench:
  Synthetic and Hyper went dry during it,
  the expired Bedrock key withheld the three Gemma seats,
  and the consolidation judged on three or four voices with a quorum of 2.
  Its page lost several intensifiers and small words zheermao17's had kept.
  Four changes then went in together
  (the NeuralWatt day and account-wide count,
  the DeepSeek 1017 ids,
  a refused credential holding out the provider instead of the seat,
  and the readonly types),
  so the rotation restarts at `zheermao101` on the combined build.
- It restarted on `31ce5369e1`,
  the combined build:
  `zheermao20` and `CuspariaKLSY20` shipped SETTLED with no class traced to code
  ([the pass log of 2026-10-21](translation-repair-openrouter-2026-09-03-2026-10-21.md)).
  `zheermao20`'s letter fell back toward the archive's wording,
  for a cause the log shows and that is design:
  the archive block review picks one whole revision,
  and only two narrow ones reached it.
  `XingZ60` is held:
  the fixes in w253 and w254 change the build when they land,
  and the rotation then restarts at `zheermao101`.
- It restarted on `3637592bd4`
  (digest `sha256-tree-v1:06c97a69446d…`):
  `zheermao21` and `CuspariaKLSY21` shipped SETTLED
  (reads running),
  and `XingZ6016` stopped INCOMPLETE after 6.7 min of pass
  with `visual evidence incomplete for 1 referenced assets`.
  Its one picture needs two corroborating readings;
  `minimax-m3` read it,
  `GLM-5.3-Flash` returned an empty reply,
  Qwen's only provider was out of budget,
  and both Gemma readers are on Bedrock,
  whose key is never renewed.
  So the rotation on that build cannot finish with every page SETTLED,
  and no further entry runs on it:
  OpenRouter's balance is kept for runs that count.
  The rotation restarts at `zheermao101` on the build that lands the archive review fixes
  (B712 to B715 and their follow-ups)
  with a picture bench that two wet readers can corroborate.
- It restarted on `2a336603bf`
  (digest `sha256-tree-v1:5d4265dfab83…`),
  the build that lifted the Gemma readers and landed a KaTeX shim;
  the owner ruled that polished text wins ties and that reasons come before votes everywhere
  ([the restart overview](translation-repair-openrouter-2026-09-03-2026-10-21-gemma-lift-restart.md)).
  `zheermao22` SETTLED in 79.0 min of pass:
  the email and the opening quote recovered,
  and Arts's letter regressed.
  The read names three code classes
  (K1:
  NeuralWatt text calls carry no reasoning effort;
  K2:
  stream guards that cannot fire,
  and comment lines counted as progress;
  K3:
  accepted minor claims never reach the polish writers)
  and one design class
  (K4:
  the composite refuses line-count changes).
  The fixes are in progress,
  and the rotation restarts at `zheermao23` on the build that lands them.
  `CuspariaKLSY22` SETTLED in 39.3 min with a mixed page
  (slices 0 to 2 better than `CuspariaKLSY21`'s,
  slice 3 worse),
  and no new ruling was shown to act.
  The run asked Bedrock once and got HTTP 401 although the owner ruled to leave it off,
  its `METERS` line read `bedrock=wet` while `JUDGE SEATS` read `bedrock=dry`,
  33 `inception/mercury-2.5` streams were abandoned with no content,
  and Hyper returned its keys in alphabetical order on 102 of 103 vote replies.
  NeuralWatt's plan meter rose 37.3 Wh against 21.11 Wh of the run's own calls;
  the owner said on 2026-10-22 that the NeuralWatt key of the dsh harness is on the same account,
  which is where the rest can come from.
  A round of fixes for these is decided and not yet landed
  (Bedrock off by configuration,
  `METERS` and `JUDGE SEATS` made to agree,
  no-content abandons counted against the seat,
  Hyper key-order handling).
  `XingZ6017` SETTLED in 268.7 min over 118 slices
  ([the run](translation-repair-openrouter-2026-09-03-2026-10-21-gemma-lift-restart-xingz6017-run.md)).
  The page is better than the unfinished archive in 64 of the 88 slices that have a counterpart,
  and keeps one factual error about when she died,
  one game and one contributor under two names,
  and 29 em-dash runs the source does not carry.
  Synthetic,
  Hyper and NeuralWatt ran dry in turn after Bedrock was held out at its first ask,
  OpenRouter alone carried the run for about 114 min and spent 9.218 USD,
  and 24 rounds waited 250 s or more on a dead stream that held the quorum.
  NeuralWatt's measured energy was 2.84 times its estimate;
  that no stop line fired at the dry reading is the designed behaviour,
  and ledger B808 recalibrated the under-reading estimate on 2026-10-22 (UTC).
  Hung rounds (B809) were fixed in part
  by a first-content window of 90000 ms on the Mercury card (open to the owner's veto),
  and a hold on seats re-asked after abandoned calls (B810) was examined and rejected;
  the coverage rounds over an untranslated tail (B811) were examined and kept unchanged.
  The run's build lacked commits `e336d8efc7` (B804 to B807) and `f829fc31b6` (B803),
  so its K2 split (52 mercury abandonments) and its K3 lost claims (6 verified)
  are a baseline from before those fixes,
  not recurrences after them.
- It restarted on `8864316eb2`
  (digest `sha256-tree-v1:4e516f924c6f…`),
  the build that carries those fixes and the owner's answers of 2026-10-22
  ([the restart overview](translation-repair-openrouter-2026-09-03-2026-10-22-answers-restart.md)).
  `zheermao23` SETTLED in 49.4 min of pass
  ([the run](translation-repair-openrouter-2026-09-03-2026-10-22-answers-restart-zheermao23-run.md)),
  against 79.0 min for `zheermao22`.
  Its read covers the log,
  the seats and the slice verdicts;
  the shipped page was not read line by line.
  NeuralWatt made no call,
  since its day was spent before the launch,
  and Synthetic ran dry at 13:11 UTC,
  so Hyper and OpenRouter carried the rest.
  Mercury's first content came within 12,820 ms on every call that returned content,
  so its 90,000 ms window never fired.
  `CuspariaKLSY23` SETTLED in 17.1 min of pass
  ([the run](translation-repair-openrouter-2026-09-03-2026-10-22-answers-restart-cuspariaklsy23-run.md)),
  against 39.3 min for `CuspariaKLSY22`;
  all 64 Mercury streams completed,
  and for the fourth `CuspariaKLSY` run in a row the repair lane left tense issues unresolved
  (4 accepted,
  2 of them major).
  Both runs printed plan energy with no NeuralWatt call (326.9 and 255.7 Wh);
  the owner said on 2026-10-22 that the dsh harness's NeuralWatt key is on the same account,
  so that figure counts the harness's use too.
  `XingZ6018` ended after 62.4 min with no page
  ([the run](translation-repair-openrouter-2026-09-03-2026-10-22-answers-restart-xingz6018-run.md)):
  Synthetic,
  NeuralWatt and Bedrock read dry from the first reading,
  Hyper ran out at 14:28 UTC and OpenRouter at 14:45 UTC,
  and the router's every-provider-dry refusal was absorbed as a lost voice 990 times over 77.7 s
  before the writing bench's floor ended the entry
  (ledger B818;
  B819 for the NeuralWatt time its text got wrong,
  and B820 for a slice cached with a silent panel;
  all fixed on 2026-10-22 UTC,
  after the run).
  The third Mercury recheck was clean:
  594 streams completed with a maximum first content of 15,283 ms,
  so 779 streams across three runs have neither been cut nor come near the 90,000 ms window,
  which still shows neither that the window works nor that it is needed.
  `XingZ6019` ended after 19.2 min on a build made before those fixes,
  with editors 1 of 3 and refiners 1 of 3 reachable against a floor of 2 while NeuralWatt was dry
  ([the record](translation-repair-openrouter-2026-09-03-2026-10-22-answers-restart-xingz6019.md)).
  The seat reader did not count NeuralWatt's known return (19:15:08 UTC,
  48 s after the stop) as a named return,
  so the floor stopped the entry instead of waiting (ledger B826,
  open,
  the fix in progress).
  Note of 2026-10-25 UTC:
  B826 is fixed by the merge `9eea0009d1`.
  `XingZ6020` was relaunched at 19:32:29 UTC on the same build.
  Its read is filed,
  as checked by a second reader
  ([the run](translation-repair-openrouter-2026-09-03-2026-10-22-answers-restart-xingz6020-run.md)).
  It settled on its second attempt after 10 h 32 min 51 s,
  the first attempt having ended on the 7 h hard deadline inside a wait for a hold,
  and it does not move the answer toward yes.
  The page it shipped is missing its last 30 of 118 slices (31.5 percent of the source,
  every footnote definition among them) with nothing on the page to say so,
  and the run was idle for 249.0 of the 336.8 min after Synthetic's weekly credit ran out.
  The build had read that weekly block as unreadable since Synthetic renamed it
  ([the defects](translation-repair-openrouter-2026-09-03-2026-10-22-answers-restart-xingz6020-defects.md),
  S1 and D1 to D7,
  each with the commit that fixed it).
  The `XingZ6021` entry stays the latest reading.
- `XingZ6021` ran on 2026-10-24 and 2026-10-25 on the build `bcff23b8ce`,
  which carries the fixes for the defects found in `XingZ6020`,
  and SETTLED on its first attempt in 2 h 5 min 45 s
  ([the run](translation-repair-openrouter-2026-09-03-2026-10-22-answers-restart-xingz6021-run.md)).
  All 118 slices were filled,
  tail included,
  with no hold,
  no HTTP error status and 0.4 min of idle wall clock.
  It does not move the answer toward yes.
  The page shipped clean,
  with no `DEFECTS` line,
  while its Shinonome section (slices 67 to 70) is the archive's text:
  an invented *Metal Gear Solid 2* quotation and two paragraphs that the source lacks,
  after the run's own critics had flagged it
  ([the defects](translation-repair-openrouter-2026-09-03-2026-10-22-answers-restart-xingz6021-defects.md),
  N1).
  The cause is a chain of absolute floors
  (tally weight 3,
  3 non-translation votes,
  a slate of 2 ballots)
  that a bench of two cannot meet,
  reached when Synthetic's credit ran dry 72.4 min in;
  the archive stayed by omission and not by a recorded decision,
  and no publish check measures whether the wording follows the source.
  The owner ruled on those floors on 2026-10-25
  and the change is in progress,
  with no commit yet;
  readiness is not claimed on this run,
  and the check that the change worked is a rerun of the `XingZ60` page whose Shinonome section no longer ships the
  archive's text.
  Two of the seven fixes of the build were not exercised:
  the source text under a note,
  and the NeuralWatt hold
  ([the stages](translation-repair-openrouter-2026-09-03-2026-10-22-answers-restart-xingz6021-stages.md)).
- The entries never run on a recent build are the rest of the 92-entry corpus.
  The owner's rule stands:
  "We never ever need a all 92 entries launch";
  readiness is claimed on the read set and the classes it stopped finding.
- The providers are now a constraint on the roster,
  not on money.
  At the end of `XingZ6015` (00:05 UTC on 2026-10-20) Synthetic's weekly allowance and Hyper's balance were both 0,
  OpenRouter held 16.13 USD and Bedrock 5.02 USD,
  and three Gemma seats refused credentials.
  Hyper is prepaid and will not be recharged;
  Synthetic gives back about 2% every 202 minutes
  (`package/translation-repair/doc/provider-availability.md`).
  So the next rotation runs mostly on OpenRouter seats,
  3 of 7 seats within reach of the contest and consolidation lanes as in `XingZ6015`'s later lanes,
  and `XingZ6015` alone spent 5.80 USD of OpenRouter.
  At the end of `zheermao17` (19:54 UTC on 2026-10-20) Hyper held 45,
  OpenRouter 15.23 USD and Bedrock 5.02 USD,
  and Synthetic had gone dry at 19:38 UTC.
  NeuralWatt's 550 Wh a day of free energy on `deepseek-v4.1-flash` and `glm-5.3-flash`
  (owner,
  2026-10-20)
  reaches the 119 and 50 asks those two seats took in `zheermao17`.
