# Translation repair: OpenRouter as the paid fallback provider

Planning record,
2026-09-03.
Nothing here is ratified;
the owner's answers to the questions at the end decide the seating.
Measurements were taken on 2026-09-03 between 15:24 and 15:45 UTC unless dated otherwise.

## Contents

This record is kept as one file per subject or span,
each of at most 1,000 lines and named for what it holds.
New text goes into the file whose subject it belongs to;
a file that would pass the limit gives up a subject to a new file,
named for that subject and listed here.

- [OpenRouter and the corpus passes: the proposal, the transport probes and the build plan](translation-repair-openrouter-2026-09-03-proposal-transport-probes-and-build-plan.md):
  Why OpenRouter was proposed as the paid fallback provider on 2026-09-03,
  what it answered and would cost,
  the decisions and constraints taken then,
  the transport probes,
  and the build plan.
  Sections:
  - [Why now](translation-repair-openrouter-2026-09-03-proposal-transport-probes-and-build-plan.md#why-now)
  - [What OpenRouter answered](translation-repair-openrouter-2026-09-03-proposal-transport-probes-and-build-plan.md#what-openrouter-answered)
  - [Roster models on OpenRouter](translation-repair-openrouter-2026-09-03-proposal-transport-probes-and-build-plan.md#roster-models-on-openrouter)
  - [What an entry would cost bought entirely on OpenRouter](translation-repair-openrouter-2026-09-03-proposal-transport-probes-and-build-plan.md#what-an-entry-would-cost-bought-entirely-on-openrouter)
  - [Decisions taken here, open to veto](translation-repair-openrouter-2026-09-03-proposal-transport-probes-and-build-plan.md#decisions-taken-here-open-to-veto)
  - [Constraints for the build](translation-repair-openrouter-2026-09-03-proposal-transport-probes-and-build-plan.md#constraints-for-the-build)
  - [Questions put to the owner](translation-repair-openrouter-2026-09-03-proposal-transport-probes-and-build-plan.md#questions-put-to-the-owner)
  - [The `[DONE]` sentinel, fixed before the probe could measure](translation-repair-openrouter-2026-09-03-proposal-transport-probes-and-build-plan.md#the-done-sentinel-fixed-before-the-probe-could-measure)
  - [Probe v2: three transports per model](translation-repair-openrouter-2026-09-03-proposal-transport-probes-and-build-plan.md#probe-v2-three-transports-per-model)
  - [Build plan, transport-independent layers first](translation-repair-openrouter-2026-09-03-proposal-transport-probes-and-build-plan.md#build-plan-transport-independent-layers-first)
- [OpenRouter and the corpus passes: 2026-09-03](translation-repair-openrouter-2026-09-03-2026-09-03.md):
  What was launched,
  read and decided on 2026-09-03.
  Entries:
  keyword233.
  Sections:
  - [What landed, 2026-09-03](translation-repair-openrouter-2026-09-03-2026-09-03.md#what-landed-2026-09-03)
  - [The first live pass with OpenRouter in the order, keyword233, 16:38 to 16:54 UTC](translation-repair-openrouter-2026-09-03-2026-09-03.md#the-first-live-pass-with-openrouter-in-the-order-keyword233-1638-to-1654-utc)
  - [The second live pass, keyword233, 17:06 to 17:28 UTC, with Parasail ignored and endpoints named](translation-repair-openrouter-2026-09-03-2026-09-03.md#the-second-live-pass-keyword233-1706-to-1728-utc-with-parasail-ignored-and-endpoints-named)
  - [The third live pass, keyword233, 18:15 to 18:36 UTC, OpenRouter alone](translation-repair-openrouter-2026-09-03-2026-09-03.md#the-third-live-pass-keyword233-1815-to-1836-utc-openrouter-alone)
  - [The fourth live pass, keyword233, 19:33 to 19:55 UTC, OpenRouter alone at a 120 s grace](translation-repair-openrouter-2026-09-03-2026-09-03.md#the-fourth-live-pass-keyword233-1933-to-1955-utc-openrouter-alone-at-a-120-s-grace)
  - [The fifth live pass, keyword233, 19:58 to 20:21 UTC: the withhold holds through every stage](translation-repair-openrouter-2026-09-03-2026-09-03.md#the-fifth-live-pass-keyword233-1958-to-2021-utc-the-withhold-holds-through-every-stage)
  - [The sixth live pass, keyword233, 20:21 to 20:43 UTC: the built-in window, no dial](translation-repair-openrouter-2026-09-03-2026-09-03.md#the-sixth-live-pass-keyword233-2021-to-2043-utc-the-built-in-window-no-dial)
- [OpenRouter and the corpus passes: 2026-09-04, 04:54 to 05:45 UTC](translation-repair-openrouter-2026-09-03-2026-09-04-0454-to-0545-utc.md):
  What was launched,
  read and decided on 2026-09-04 04:54 to 05:45 UTC.
  Entries:
  luxuanwen3.
  Sections:
  - [The picture passes, 2026-09-04, and the double-quote undercount](translation-repair-openrouter-2026-09-03-2026-09-04-0454-to-0545-utc.md#the-picture-passes-2026-09-04-and-the-double-quote-undercount)
  - [ModelRun's timeouts, 2026-09-04, and what the log called them](translation-repair-openrouter-2026-09-03-2026-09-04-0454-to-0545-utc.md#modelruns-timeouts-2026-09-04-and-what-the-log-called-them)
  - [The 429 hold under concurrency, 2026-09-04, 04:54 to 05:18 UTC](translation-repair-openrouter-2026-09-03-2026-09-04-0454-to-0545-utc.md#the-429-hold-under-concurrency-2026-09-04-0454-to-0518-utc)
  - [The two shape passes, 2026-09-04, 05:45 UTC onward, and what they cost](translation-repair-openrouter-2026-09-03-2026-09-04-0454-to-0545-utc.md#the-two-shape-passes-2026-09-04-0545-utc-onward-and-what-they-cost)
  - [luxuanwen3's front matter, and the rule the archive breaks](translation-repair-openrouter-2026-09-03-2026-09-04-0454-to-0545-utc.md#luxuanwen3s-front-matter-and-the-rule-the-archive-breaks)
- [OpenRouter and the corpus passes: 2026-09-04, 09:22 to 15:25 UTC](translation-repair-openrouter-2026-09-03-2026-09-04-0922-to-1525-utc.md):
  What was launched,
  read and decided on 2026-09-04 09:22 to 15:25 UTC.
  Entries:
  luxuanwen3.
  Sections:
  - [The luxuanwen3 re-run, 2026-09-04, 09:22 UTC onward: slice 0 passes, slice 1 stops the entry](translation-repair-openrouter-2026-09-03-2026-09-04-0922-to-1525-utc.md#the-luxuanwen3-re-run-2026-09-04-0922-utc-onward-slice-0-passes-slice-1-stops-the-entry)
  - [The luxuanwen3 re-run on the either-rendering build, 2026-09-04: every slice passes, the publisher refuses](translation-repair-openrouter-2026-09-03-2026-09-04-0922-to-1525-utc.md#the-luxuanwen3-re-run-on-the-either-rendering-build-2026-09-04-every-slice-passes-the-publisher-refuses)
  - [The two runs on the publisher fix, 2026-09-04, 13:51 UTC onward: both ship, and what the reading found](translation-repair-openrouter-2026-09-03-2026-09-04-0922-to-1525-utc.md#the-two-runs-on-the-publisher-fix-2026-09-04-1351-utc-onward-both-ship-and-what-the-reading-found)
  - [The Uekawakuyuurei run on the pronoun build, 2026-09-04, 15:03 UTC: stopped by its own pictures](translation-repair-openrouter-2026-09-03-2026-09-04-0922-to-1525-utc.md#the-uekawakuyuurei-run-on-the-pronoun-build-2026-09-04-1503-utc-stopped-by-its-own-pictures)
  - [Uekawakuyuurei on the picture-fix build, 2026-09-04, 15:25 UTC: ships, and the balance runs out under it](translation-repair-openrouter-2026-09-03-2026-09-04-0922-to-1525-utc.md#uekawakuyuurei-on-the-picture-fix-build-2026-09-04-1525-utc-ships-and-the-balance-runs-out-under-it)
  - [The picture gather's wait, re-read on three-reader pictures, 2026-09-04](translation-repair-openrouter-2026-09-03-2026-09-04-0922-to-1525-utc.md#the-picture-gathers-wait-re-read-on-three-reader-pictures-2026-09-04)
- [OpenRouter and the corpus passes: 2026-09-06](translation-repair-openrouter-2026-09-03-2026-09-06.md):
  What was launched,
  read and decided on 2026-09-06.
  Sections:
  - [The providers come back, 2026-09-06: two defaults move and a rule is written](translation-repair-openrouter-2026-09-03-2026-09-06.md#the-providers-come-back-2026-09-06-two-defaults-move-and-a-rule-is-written)
  - [The first plain-invocation run, 2026-09-06, 20:21 UTC: the fifth class, and no page](translation-repair-openrouter-2026-09-03-2026-09-06.md#the-first-plain-invocation-run-2026-09-06-2021-utc-the-fifth-class-and-no-page)
  - [The third launch, 2026-09-06, 21:03 UTC: the first page on the plain invocation, and the sixth class](translation-repair-openrouter-2026-09-03-2026-09-06.md#the-third-launch-2026-09-06-2103-utc-the-first-page-on-the-plain-invocation-and-the-sixth-class)
  - [The fifth launch, 2026-09-06, 22:10 UTC: the headings hold, and the seventh class](translation-repair-openrouter-2026-09-03-2026-09-06.md#the-fifth-launch-2026-09-06-2210-utc-the-headings-hold-and-the-seventh-class)
  - [The sixth launch, 2026-09-06, 23:11 UTC: the apostrophes hold, and the eighth class](translation-repair-openrouter-2026-09-03-2026-09-06.md#the-sixth-launch-2026-09-06-2311-utc-the-apostrophes-hold-and-the-eighth-class)
- [OpenRouter and the corpus passes: 2026-09-07, 00:12 to 08:43 UTC](translation-repair-openrouter-2026-09-03-2026-09-07-0012-to-0843-utc.md):
  What was launched,
  read and decided on 2026-09-07 00:12 to 08:43 UTC.
  Entries:
  TLL1122.
  Sections:
  - [The seventh launch, 2026-09-07, 00:12 UTC: the class-eight page reads clean, and one ellipsis](translation-repair-openrouter-2026-09-03-2026-09-07-0012-to-0843-utc.md#the-seventh-launch-2026-09-07-0012-utc-the-class-eight-page-reads-clean-and-one-ellipsis)
  - [The shape census, 2026-09-07, 00:20 UTC: what no read page has carried](translation-repair-openrouter-2026-09-03-2026-09-07-0012-to-0843-utc.md#the-shape-census-2026-09-07-0020-utc-what-no-read-page-has-carried)
  - [The TLL1122 launch, 2026-09-07, 01:11 UTC: the first footnote](translation-repair-openrouter-2026-09-03-2026-09-07-0012-to-0843-utc.md#the-tll1122-launch-2026-09-07-0111-utc-the-first-footnote)
  - [The TLL1122 page, 2026-09-07, 01:41 UTC: the footnote holds, and no class](translation-repair-openrouter-2026-09-03-2026-09-07-0012-to-0843-utc.md#the-tll1122-page-2026-09-07-0141-utc-the-footnote-holds-and-no-class)
  - [The run order after TLL1122, decided 2026-09-07, 01:50 UTC](translation-repair-openrouter-2026-09-03-2026-09-07-0012-to-0843-utc.md#the-run-order-after-tll1122-decided-2026-09-07-0150-utc)
  - [The Huasheng launch, 2026-09-07, 01:44 UTC: the first container](translation-repair-openrouter-2026-09-03-2026-09-07-0012-to-0843-utc.md#the-huasheng-launch-2026-09-07-0144-utc-the-first-container)
  - [The Huasheng pass killed, 2026-09-07, 02:56 UTC: the ninth class](translation-repair-openrouter-2026-09-03-2026-09-07-0012-to-0843-utc.md#the-huasheng-pass-killed-2026-09-07-0256-utc-the-ninth-class)
  - [The ninth class fixed, 2026-09-07, 03:05 UTC](translation-repair-openrouter-2026-09-03-2026-09-07-0012-to-0843-utc.md#the-ninth-class-fixed-2026-09-07-0305-utc)
  - [The Huasheng relaunch, 2026-09-07, 03:09 UTC](translation-repair-openrouter-2026-09-03-2026-09-07-0012-to-0843-utc.md#the-huasheng-relaunch-2026-09-07-0309-utc)
  - [The Huasheng relaunch stops at slice 21, 2026-09-07, 05:21 UTC: the tenth class, a design question](translation-repair-openrouter-2026-09-03-2026-09-07-0012-to-0843-utc.md#the-huasheng-relaunch-stops-at-slice-21-2026-09-07-0521-utc-the-tenth-class-a-design-question)
  - [The owner answers A, 2026-09-07, 08:38 UTC: either rendering, bounded to splits](translation-repair-openrouter-2026-09-03-2026-09-07-0012-to-0843-utc.md#the-owner-answers-a-2026-09-07-0838-utc-either-rendering-bounded-to-splits)
  - [The third Huasheng launch, 2026-09-07, 08:43 UTC, on the class-ten build](translation-repair-openrouter-2026-09-03-2026-09-07-0012-to-0843-utc.md#the-third-huasheng-launch-2026-09-07-0843-utc-on-the-class-ten-build)
- [OpenRouter and the corpus passes: 2026-09-07, 11:14 to 21:08 UTC](translation-repair-openrouter-2026-09-03-2026-09-07-1114-to-2108-utc.md):
  What was launched,
  read and decided on 2026-09-07 11:14 to 21:08 UTC.
  Sections:
  - [The third Huasheng launch ends at the publisher, 2026-09-07, 11:14 UTC: the eleventh class](translation-repair-openrouter-2026-09-03-2026-09-07-1114-to-2108-utc.md#the-third-huasheng-launch-ends-at-the-publisher-2026-09-07-1114-utc-the-eleventh-class)
  - [The owner answers on the name, 2026-09-07, 11:30 UTC: the pinyin check and the alias exemption](translation-repair-openrouter-2026-09-03-2026-09-07-1114-to-2108-utc.md#the-owner-answers-on-the-name-2026-09-07-1130-utc-the-pinyin-check-and-the-alias-exemption)
  - [The fourth Huasheng launch, 2026-09-07, 15:43 UTC, on the class-eleven build](translation-repair-openrouter-2026-09-03-2026-09-07-1114-to-2108-utc.md#the-fourth-huasheng-launch-2026-09-07-1543-utc-on-the-class-eleven-build)
  - [The fourth Huasheng launch finds the twelfth class, 2026-09-07, 19:40 UTC: Hyper's daily limit](translation-repair-openrouter-2026-09-03-2026-09-07-1114-to-2108-utc.md#the-fourth-huasheng-launch-finds-the-twelfth-class-2026-09-07-1940-utc-hypers-daily-limit)
  - [The owner adds Amazon Bedrock, 2026-09-07, 19:50 UTC](translation-repair-openrouter-2026-09-03-2026-09-07-1114-to-2108-utc.md#the-owner-adds-amazon-bedrock-2026-09-07-1950-utc)
  - [The fourth Huasheng page, 2026-09-07, 20:49 UTC](translation-repair-openrouter-2026-09-03-2026-09-07-1114-to-2108-utc.md#the-fourth-huasheng-page-2026-09-07-2049-utc)
  - [The first hakureico launch, 2026-09-07, 21:00 UTC, on the four-provider build](translation-repair-openrouter-2026-09-03-2026-09-07-1114-to-2108-utc.md#the-first-hakureico-launch-2026-09-07-2100-utc-on-the-four-provider-build)
  - [The first hakureico launch was on the wrong roster; the second, 2026-09-07, 21:08 UTC](translation-repair-openrouter-2026-09-03-2026-09-07-1114-to-2108-utc.md#the-first-hakureico-launch-was-on-the-wrong-roster-the-second-2026-09-07-2108-utc)
- [OpenRouter and the corpus passes: 2026-09-07, 21:19 to 23:38 UTC](translation-repair-openrouter-2026-09-03-2026-09-07-2119-to-2338-utc.md):
  What was launched,
  read and decided on 2026-09-07 21:19 to 23:38 UTC.
  Sections:
  - [The owner moves Bedrock ahead of Hyper; the third hakureico launch, 2026-09-07, 21:47 UTC](translation-repair-openrouter-2026-09-03-2026-09-07-2119-to-2338-utc.md#the-owner-moves-bedrock-ahead-of-hyper-the-third-hakureico-launch-2026-09-07-2147-utc)
  - [The first hakureico page, 2026-09-07, 22:00 UTC, and the thirteenth class](translation-repair-openrouter-2026-09-03-2026-09-07-2119-to-2338-utc.md#the-first-hakureico-page-2026-09-07-2200-utc-and-the-thirteenth-class)
  - [The fourth hakureico launch, 2026-09-07, 22:21 UTC, on the class-thirteen build](translation-repair-openrouter-2026-09-03-2026-09-07-2119-to-2338-utc.md#the-fourth-hakureico-launch-2026-09-07-2221-utc-on-the-class-thirteen-build)
  - [The fourth hakureico pass stops INCOMPLETE; the thirteenth class's second face, 2026-09-07, 22:56 UTC](translation-repair-openrouter-2026-09-03-2026-09-07-2119-to-2338-utc.md#the-fourth-hakureico-pass-stops-incomplete-the-thirteenth-classs-second-face-2026-09-07-2256-utc)
  - [The fifth hakureico launch, 2026-09-07, 23:09 UTC, on the per-chunk-wait build](translation-repair-openrouter-2026-09-03-2026-09-07-2119-to-2338-utc.md#the-fifth-hakureico-launch-2026-09-07-2309-utc-on-the-per-chunk-wait-build)
  - [The fifth pass stops in 67 seconds; the fourteenth class, 2026-09-07, 23:10 UTC](translation-repair-openrouter-2026-09-03-2026-09-07-2119-to-2338-utc.md#the-fifth-pass-stops-in-67-seconds-the-fourteenth-class-2026-09-07-2310-utc)
  - [The sixth hakureico launch, 2026-09-07, 23:38 UTC, on the seated build](translation-repair-openrouter-2026-09-03-2026-09-07-2119-to-2338-utc.md#the-sixth-hakureico-launch-2026-09-07-2338-utc-on-the-seated-build)
  - [Measuring the two Bedrock-only sizes, 2026-09-07, 21:19 UTC](translation-repair-openrouter-2026-09-03-2026-09-07-2119-to-2338-utc.md#measuring-the-two-bedrock-only-sizes-2026-09-07-2119-utc)
- [OpenRouter and the corpus passes: 2026-09-08, 00:38 to 11:27 UTC](translation-repair-openrouter-2026-09-03-2026-09-08-0038-to-1127-utc.md):
  What was launched,
  read and decided on 2026-09-08 00:38 to 11:27 UTC.
  Sections:
  - [The sixth pass settles; the second footnote page, 2026-09-08, 00:38 UTC](translation-repair-openrouter-2026-09-03-2026-09-08-0038-to-1127-utc.md#the-sixth-pass-settles-the-second-footnote-page-2026-09-08-0038-utc)
  - [The seventh hakureico launch on Bedrock alone stops at the pictures, 2026-09-08, 01:49 UTC](translation-repair-openrouter-2026-09-03-2026-09-08-0038-to-1127-utc.md#the-seventh-hakureico-launch-on-bedrock-alone-stops-at-the-pictures-2026-09-08-0149-utc)
  - [The eighth hakureico launch on Bedrock alone, on the reader build, 2026-09-08, 02:19 UTC](translation-repair-openrouter-2026-09-03-2026-09-08-0038-to-1127-utc.md#the-eighth-hakureico-launch-on-bedrock-alone-on-the-reader-build-2026-09-08-0219-utc)
  - [The ninth hakureico launch, four providers wet, 2026-09-08, 11:27 UTC](translation-repair-openrouter-2026-09-03-2026-09-08-0038-to-1127-utc.md#the-ninth-hakureico-launch-four-providers-wet-2026-09-08-1127-utc)
- [OpenRouter and the corpus passes: 2026-09-08, 16:07 to 22:42 UTC](translation-repair-openrouter-2026-09-03-2026-09-08-1607-to-2242-utc.md):
  What was launched,
  read and decided on 2026-09-08 16:07 to 22:42 UTC.
  Sections:
  - [The E2B calibration prints its standing; the translator seat, 2026-09-08, 16:07 UTC](translation-repair-openrouter-2026-09-03-2026-09-08-1607-to-2242-utc.md#the-e2b-calibration-prints-its-standing-the-translator-seat-2026-09-08-1607-utc)
  - [The front-matter branch merges; the tenth hakureico launch, 2026-09-08, 16:18 UTC](translation-repair-openrouter-2026-09-03-2026-09-08-1607-to-2242-utc.md#the-front-matter-branch-merges-the-tenth-hakureico-launch-2026-09-08-1618-utc)
  - [The tenth pass is killed at 37 minutes; the seventeenth class, 2026-09-08, 16:55 UTC](translation-repair-openrouter-2026-09-03-2026-09-08-1607-to-2242-utc.md#the-tenth-pass-is-killed-at-37-minutes-the-seventeenth-class-2026-09-08-1655-utc)
  - [The eleventh hakureico launch on the false-start build, 2026-09-08, 16:56 UTC](translation-repair-openrouter-2026-09-03-2026-09-08-1607-to-2242-utc.md#the-eleventh-hakureico-launch-on-the-false-start-build-2026-09-08-1656-utc)
  - [The eleventh pass is killed at 85 minutes; the eighteenth class; the twelfth launch, 2026-09-08, 18:21 UTC](translation-repair-openrouter-2026-09-03-2026-09-08-1607-to-2242-utc.md#the-eleventh-pass-is-killed-at-85-minutes-the-eighteenth-class-the-twelfth-launch-2026-09-08-1821-utc)
  - [The twelfth pass settles in 108 minutes under the three checks and rewrites the letter, 2026-09-08, 20:10 UTC](translation-repair-openrouter-2026-09-03-2026-09-08-1607-to-2242-utc.md#the-twelfth-pass-settles-in-108-minutes-under-the-three-checks-and-rewrites-the-letter-2026-09-08-2010-utc)
  - [The letter sealed and the yuki page read and the nineteenth class fixed, 2026-09-08, 22:10 UTC](translation-repair-openrouter-2026-09-03-2026-09-08-1607-to-2242-utc.md#the-letter-sealed-and-the-yuki-page-read-and-the-nineteenth-class-fixed-2026-09-08-2210-utc)
  - [The nineteenth class's second face where the crossing definitions starve the pairing, 2026-09-08, 22:36 UTC](translation-repair-openrouter-2026-09-03-2026-09-08-1607-to-2242-utc.md#the-nineteenth-classs-second-face-where-the-crossing-definitions-starve-the-pairing-2026-09-08-2236-utc)
  - [The relabel must close over the archive's labels, 2026-09-08, 22:42 UTC](translation-repair-openrouter-2026-09-03-2026-09-08-1607-to-2242-utc.md#the-relabel-must-close-over-the-archives-labels-2026-09-08-2242-utc)
- [OpenRouter and the corpus passes: 2026-09-09, 00:25 to 12:10 UTC](translation-repair-openrouter-2026-09-03-2026-09-09-0025-to-1210-utc.md):
  What was launched,
  read and decided on 2026-09-09 00:25 to 12:10 UTC.
  Sections:
  - [The twentieth class stops the fifth yuki pass at the carried-insertion guard and the Arita page is read, 2026-09-09, 00:25 UTC](translation-repair-openrouter-2026-09-03-2026-09-09-0025-to-1210-utc.md#the-twentieth-class-stops-the-fifth-yuki-pass-at-the-carried-insertion-guard-and-the-arita-page-is-read-2026-09-09-0025-utc)
  - [The twentieth class's second face where a partial vote's quote is not evidence, 2026-09-09, 01:35 UTC](translation-repair-openrouter-2026-09-03-2026-09-09-0025-to-1210-utc.md#the-twentieth-classs-second-face-where-a-partial-votes-quote-is-not-evidence-2026-09-09-0135-utc)
  - [The seventh yuki page settles under the closed relabel and the full-vote evidence, 2026-09-09, 02:55 UTC](translation-repair-openrouter-2026-09-03-2026-09-09-0025-to-1210-utc.md#the-seventh-yuki-page-settles-under-the-closed-relabel-and-the-full-vote-evidence-2026-09-09-0255-utc)
  - [The nineteenth hakureico page is read and the twenty-first class takes the sibling note with the orphan, 2026-09-09, 03:25 UTC](translation-repair-openrouter-2026-09-03-2026-09-09-0025-to-1210-utc.md#the-nineteenth-hakureico-page-is-read-and-the-twenty-first-class-takes-the-sibling-note-with-the-orphan-2026-09-09-0325-utc)
  - [Two shapes-first entries launched beside it, 2026-09-09, 03:24 UTC](translation-repair-openrouter-2026-09-03-2026-09-09-0025-to-1210-utc.md#two-shapes-first-entries-launched-beside-it-2026-09-09-0324-utc)
  - [The twenty-first class's second face where the ledger says what was decided and the document carries the trim, 2026-09-09, 04:45 UTC](translation-repair-openrouter-2026-09-03-2026-09-09-0025-to-1210-utc.md#the-twenty-first-classs-second-face-where-the-ledger-says-what-was-decided-and-the-document-carries-the-trim-2026-09-09-0445-utc)
  - [The twenty-first class's third face where two notes one line apart are one block to the trim, 2026-09-09, 06:25 UTC](translation-repair-openrouter-2026-09-03-2026-09-09-0025-to-1210-utc.md#the-twenty-first-classs-third-face-where-two-notes-one-line-apart-are-one-block-to-the-trim-2026-09-09-0625-utc)
  - [The twenty-first class's fourth face where the consolidation puts the orphan back and the page is a document nobody guarded, 2026-09-09, 08:05 UTC](translation-repair-openrouter-2026-09-03-2026-09-09-0025-to-1210-utc.md#the-twenty-first-classs-fourth-face-where-the-consolidation-puts-the-orphan-back-and-the-page-is-a-document-nobody-guarded-2026-09-09-0805-utc)
  - [The twenty-third hakureico page settles under the page assembly and closes the twenty-first class, 2026-09-09, 09:40 UTC](translation-repair-openrouter-2026-09-03-2026-09-09-0025-to-1210-utc.md#the-twenty-third-hakureico-page-settles-under-the-page-assembly-and-closes-the-twenty-first-class-2026-09-09-0940-utc)
  - [The fourth gqt page is read and the twenty-second class declines an English original, 2026-09-09, 09:55 UTC](translation-repair-openrouter-2026-09-03-2026-09-09-0025-to-1210-utc.md#the-fourth-gqt-page-is-read-and-the-twenty-second-class-declines-an-english-original-2026-09-09-0955-utc)
  - [The first noname page is read and the twenty-third class decides a contested target by votes, 2026-09-09, 12:10 UTC](translation-repair-openrouter-2026-09-03-2026-09-09-0025-to-1210-utc.md#the-first-noname-page-is-read-and-the-twenty-third-class-decides-a-contested-target-by-votes-2026-09-09-1210-utc)
- [OpenRouter and the corpus passes: 2026-09-09, 12:40 to 23:49 UTC](translation-repair-openrouter-2026-09-03-2026-09-09-1240-to-2349-utc.md):
  What was launched,
  read and decided on 2026-09-09 12:40 to 23:49 UTC.
  Entries:
  Mio8.
  Sections:
  - [The second noname page confirms the pairing on a bench OpenRouter left, and the sixth Mio stops at slice 3, 2026-09-09, 12:40 UTC](translation-repair-openrouter-2026-09-03-2026-09-09-1240-to-2349-utc.md#the-second-noname-page-confirms-the-pairing-on-a-bench-openrouter-left-and-the-sixth-mio-stops-at-slice-3-2026-09-09-1240-utc)
  - [The owner asks where 200 USD went, and six levers land, 2026-09-09, 16:50 UTC](translation-repair-openrouter-2026-09-03-2026-09-09-1240-to-2349-utc.md#the-owner-asks-where-200-usd-went-and-six-levers-land-2026-09-09-1650-utc)
  - [The owner answers the three recorded questions, 2026-09-09, 17:25 UTC](translation-repair-openrouter-2026-09-03-2026-09-09-1240-to-2349-utc.md#the-owner-answers-the-three-recorded-questions-2026-09-09-1725-utc)
  - [The third noname page is read and the Kimi-K3 leak closes, 2026-09-09, 18:40 UTC](translation-repair-openrouter-2026-09-03-2026-09-09-1240-to-2349-utc.md#the-third-noname-page-is-read-and-the-kimi-k3-leak-closes-2026-09-09-1840-utc)
  - [The ninth Mio launches on the structural-withdrawal proof, 2026-09-09, 23:49 UTC](translation-repair-openrouter-2026-09-03-2026-09-09-1240-to-2349-utc.md#the-ninth-mio-launches-on-the-structural-withdrawal-proof-2026-09-09-2349-utc)
  - [Mio8 stops on a link lost by blanket assembly withdrawal, 2026-09-09, 23:46 UTC](translation-repair-openrouter-2026-09-03-2026-09-09-1240-to-2349-utc.md#mio8-stops-on-a-link-lost-by-blanket-assembly-withdrawal-2026-09-09-2346-utc)
  - [The open threads are checked against code and recorded measurements, 2026-09-09, 22:00 UTC](translation-repair-openrouter-2026-09-03-2026-09-09-1240-to-2349-utc.md#the-open-threads-are-checked-against-code-and-recorded-measurements-2026-09-09-2200-utc)
  - [The eighth Mio launches with picture support before archive review, 2026-09-09, 21:58 UTC](translation-repair-openrouter-2026-09-03-2026-09-09-1240-to-2349-utc.md#the-eighth-mio-launches-with-picture-support-before-archive-review-2026-09-09-2158-utc)
  - [Picture support reaches archive review before removal, 2026-09-09, 21:46 UTC](translation-repair-openrouter-2026-09-03-2026-09-09-1240-to-2349-utc.md#picture-support-reaches-archive-review-before-removal-2026-09-09-2146-utc)
  - [The calibration is read, the anchor judge loses a seat, and two classes open, 2026-09-09, 21:00 UTC](translation-repair-openrouter-2026-09-03-2026-09-09-1240-to-2349-utc.md#the-calibration-is-read-the-anchor-judge-loses-a-seat-and-two-classes-open-2026-09-09-2100-utc)
- [OpenRouter and the corpus passes: 2026-09-10](translation-repair-openrouter-2026-09-03-2026-09-10.md):
  What was launched,
  read and decided on 2026-09-10.
  Entries:
  Mio12,
  Mio11,
  Mio10,
  Mio9.
  Sections:
  - [Clustered panels and complete repair evidence implemented, 2026-09-10](translation-repair-openrouter-2026-09-03-2026-09-10.md#clustered-panels-and-complete-repair-evidence-implemented-2026-09-10)
  - [Heading-boundary context reaches the repair path, 2026-09-10](translation-repair-openrouter-2026-09-03-2026-09-10.md#heading-boundary-context-reaches-the-repair-path-2026-09-10)
  - [Archive decision boundaries and English correction verified, 2026-09-10](translation-repair-openrouter-2026-09-03-2026-09-10.md#archive-decision-boundaries-and-english-correction-verified-2026-09-10)
  - [Naming scope reaches a bounded production stage, 2026-09-10](translation-repair-openrouter-2026-09-03-2026-09-10.md#naming-scope-reaches-a-bounded-production-stage-2026-09-10)
  - [Class thirty retains naturalness quorum provenance, 2026-09-10](translation-repair-openrouter-2026-09-03-2026-09-10.md#class-thirty-retains-naturalness-quorum-provenance-2026-09-10)
  - [Mio12 publishes the preserved poem but exposes a stored-review mismatch, 2026-09-10](translation-repair-openrouter-2026-09-03-2026-09-10.md#mio12-publishes-the-preserved-poem-but-exposes-a-stored-review-mismatch-2026-09-10)
  - [Mio12 launches on the measured verse remedy, 2026-09-10, 03:45 UTC](translation-repair-openrouter-2026-09-03-2026-09-10.md#mio12-launches-on-the-measured-verse-remedy-2026-09-10-0345-utc)
  - [The verse probes select the upstream source-display remedy, 2026-09-10](translation-repair-openrouter-2026-09-03-2026-09-10.md#the-verse-probes-select-the-upstream-source-display-remedy-2026-09-10)
  - [The delegated verse remedy reaches the live probe, 2026-09-10](translation-repair-openrouter-2026-09-03-2026-09-10.md#the-delegated-verse-remedy-reaches-the-live-probe-2026-09-10)
  - [Mio11 is stopped for remedy reassessment, 2026-09-10, 02:18 UTC](translation-repair-openrouter-2026-09-03-2026-09-10.md#mio11-is-stopped-for-remedy-reassessment-2026-09-10-0218-utc)
  - [Mio11 launches with the source-only break floor, 2026-09-10, 02:16 UTC](translation-repair-openrouter-2026-09-03-2026-09-10.md#mio11-launches-with-the-source-only-break-floor-2026-09-10-0216-utc)
  - [Mio10 preserves the screenshots but flattens the closing poem, 2026-09-10](translation-repair-openrouter-2026-09-03-2026-09-10.md#mio10-preserves-the-screenshots-but-flattens-the-closing-poem-2026-09-10)
  - [The tenth Mio launches with quoted revision replies accepted, 2026-09-10, 00:15 UTC](translation-repair-openrouter-2026-09-03-2026-09-10.md#the-tenth-mio-launches-with-quoted-revision-replies-accepted-2026-09-10-0015-utc)
  - [Mio9 loses revision proposals to a stricter handwritten guard, 2026-09-10, 00:12 UTC](translation-repair-openrouter-2026-09-03-2026-09-10.md#mio9-loses-revision-proposals-to-a-stricter-handwritten-guard-2026-09-10-0012-utc)
- [OpenRouter and the corpus passes: 2026-09-16, 01:58 to 16:05 UTC](translation-repair-openrouter-2026-09-03-2026-09-16-0158-to-1605-utc.md):
  What was launched,
  read and decided on 2026-09-16 01:58 to 16:05 UTC.
  Entries:
  Mio14,
  Mio15,
  Mio16,
  Mio17,
  Mio18,
  Mio19.
  Sections:
  - [The thirteenth Mio launches on the torn-down tree, 2026-09-16, 01:58 UTC](translation-repair-openrouter-2026-09-03-2026-09-16-0158-to-1605-utc.md#the-thirteenth-mio-launches-on-the-torn-down-tree-2026-09-16-0158-utc)
  - [The fourteenth Mio launches on class thirty-one, 2026-09-16, 03:04 UTC](translation-repair-openrouter-2026-09-03-2026-09-16-0158-to-1605-utc.md#the-fourteenth-mio-launches-on-class-thirty-one-2026-09-16-0304-utc)
  - [Mio14 killed on the one-transcript landing, Mio15 launched, 2026-09-16, 04:06 UTC](translation-repair-openrouter-2026-09-03-2026-09-16-0158-to-1605-utc.md#mio14-killed-on-the-one-transcript-landing-mio15-launched-2026-09-16-0406-utc)
  - [Mio15 read, 2026-09-16, 06:46 UTC: no page, class thirty-two, and the one-transcript verdict](translation-repair-openrouter-2026-09-03-2026-09-16-0158-to-1605-utc.md#mio15-read-2026-09-16-0646-utc-no-page-class-thirty-two-and-the-one-transcript-verdict)
  - [Mio16 launches on class thirty-two, 2026-09-16, 07:02 UTC](translation-repair-openrouter-2026-09-03-2026-09-16-0158-to-1605-utc.md#mio16-launches-on-class-thirty-two-2026-09-16-0702-utc)
  - [Mio16 read, 2026-09-16, 09:42 UTC: settled, the chat's speakers swapped, class thirty-three](translation-repair-openrouter-2026-09-03-2026-09-16-0158-to-1605-utc.md#mio16-read-2026-09-16-0942-utc-settled-the-chats-speakers-swapped-class-thirty-three)
  - [Mio17 launches on class thirty-three, 2026-09-16, 10:05 UTC](translation-repair-openrouter-2026-09-03-2026-09-16-0158-to-1605-utc.md#mio17-launches-on-class-thirty-three-2026-09-16-1005-utc)
  - [Mio17 read, 2026-09-16, 13:29 UTC: class thirty-three held, INCOMPLETE at slice 16, class thirty-four](translation-repair-openrouter-2026-09-03-2026-09-16-0158-to-1605-utc.md#mio17-read-2026-09-16-1329-utc-class-thirty-three-held-incomplete-at-slice-16-class-thirty-four)
  - [Mio18 launches on class thirty-four, 2026-09-16, 13:52 UTC](translation-repair-openrouter-2026-09-03-2026-09-16-0158-to-1605-utc.md#mio18-launches-on-class-thirty-four-2026-09-16-1352-utc)
  - [Mio19 read, 2026-09-16, 16:05 UTC: settled, class thirty-four held, the page reads clean](translation-repair-openrouter-2026-09-03-2026-09-16-0158-to-1605-utc.md#mio19-read-2026-09-16-1605-utc-settled-class-thirty-four-held-the-page-reads-clean)
- [OpenRouter and the corpus passes: 2026-09-16, 15:58 to 22:58 UTC](translation-repair-openrouter-2026-09-03-2026-09-16-1558-to-2258-utc.md):
  What was launched,
  read and decided on 2026-09-16 15:58 to 22:58 UTC.
  Entries:
  hulicaijia5,
  Mio20,
  Mio21,
  Mio22,
  Mio23,
  Mio24.
  Sections:
  - [hulicaijia5 launches on the tip, 2026-09-16, 15:58 UTC](translation-repair-openrouter-2026-09-03-2026-09-16-1558-to-2258-utc.md#hulicaijia5-launches-on-the-tip-2026-09-16-1558-utc)
  - [Class thirty-five, 2026-09-16, 16:43 UTC: the sheets delete accurate detail the translator took from a cited page](translation-repair-openrouter-2026-09-03-2026-09-16-1558-to-2258-utc.md#class-thirty-five-2026-09-16-1643-utc-the-sheets-delete-accurate-detail-the-translator-took-from-a-cited-page)
  - [Mio20 launches on class thirty-five, 2026-09-16, 16:43 UTC](translation-repair-openrouter-2026-09-03-2026-09-16-1558-to-2258-utc.md#mio20-launches-on-class-thirty-five-2026-09-16-1643-utc)
  - [Mio20 read, 2026-09-16, 18:44 UTC: the clause the panel now keeps is lost one stage later](translation-repair-openrouter-2026-09-03-2026-09-16-1558-to-2258-utc.md#mio20-read-2026-09-16-1844-utc-the-clause-the-panel-now-keeps-is-lost-one-stage-later)
  - [Class thirty-six, 2026-09-16, 19:03 UTC: the candidate sheets read the cited references too](translation-repair-openrouter-2026-09-03-2026-09-16-1558-to-2258-utc.md#class-thirty-six-2026-09-16-1903-utc-the-candidate-sheets-read-the-cited-references-too)
  - [Mio21 launches on class thirty-six, 2026-09-16, 19:04 UTC](translation-repair-openrouter-2026-09-03-2026-09-16-1558-to-2258-utc.md#mio21-launches-on-class-thirty-six-2026-09-16-1904-utc)
  - [Mio21 read, 2026-09-16, 20:40 UTC: the clause ships by the archive standing, not by judgment](translation-repair-openrouter-2026-09-03-2026-09-16-1558-to-2258-utc.md#mio21-read-2026-09-16-2040-utc-the-clause-ships-by-the-archive-standing-not-by-judgment)
  - [Class thirty-eight, 2026-09-16, 20:46 UTC: the block review restores the archive's quote style](translation-repair-openrouter-2026-09-03-2026-09-16-1558-to-2258-utc.md#class-thirty-eight-2026-09-16-2046-utc-the-block-review-restores-the-archives-quote-style)
  - [Class thirty-seven, 2026-09-16, 20:58 UTC: reference-backed archive details are attested word for word and screened before the panel](translation-repair-openrouter-2026-09-03-2026-09-16-1558-to-2258-utc.md#class-thirty-seven-2026-09-16-2058-utc-reference-backed-archive-details-are-attested-word-for-word-and-screened-before-the-panel)
  - [Mio22 launches on class thirty-seven, 2026-09-16, 21:00 UTC, killed at 21:01 UTC](translation-repair-openrouter-2026-09-03-2026-09-16-1558-to-2258-utc.md#mio22-launches-on-class-thirty-seven-2026-09-16-2100-utc-killed-at-2101-utc)
  - [Mio23 launches on class thirty-seven, 2026-09-16, 21:05 UTC](translation-repair-openrouter-2026-09-03-2026-09-16-1558-to-2258-utc.md#mio23-launches-on-class-thirty-seven-2026-09-16-2105-utc)
  - [Mio24 launches on class thirty-nine, 2026-09-16, 22:58 UTC](translation-repair-openrouter-2026-09-03-2026-09-16-1558-to-2258-utc.md#mio24-launches-on-class-thirty-nine-2026-09-16-2258-utc)
  - [Class thirty-nine, 2026-09-16, 22:55 UTC: the translate writer never sees the attested details](translation-repair-openrouter-2026-09-03-2026-09-16-1558-to-2258-utc.md#class-thirty-nine-2026-09-16-2255-utc-the-translate-writer-never-sees-the-attested-details)
  - [Mio23 read, 2026-09-16, 22:40 UTC: the attestation holds and the translate lane wins without the clause](translation-repair-openrouter-2026-09-03-2026-09-16-1558-to-2258-utc.md#mio23-read-2026-09-16-2240-utc-the-attestation-holds-and-the-translate-lane-wins-without-the-clause)
- [OpenRouter and the corpus passes: 2026-09-17, 12:42 to 18:56 UTC](translation-repair-openrouter-2026-09-03-2026-09-17-1242-to-1856-utc.md):
  What was launched,
  read and decided on 2026-09-17 12:42 to 18:56 UTC.
  Entries:
  XingZ602,
  XingZ601,
  yingying1,
  shi_Yumiaoya5,
  shi_Yumiaoya4,
  shi_Yumiaoya3,
  shi_Yumiaoya2,
  shi_Yumiaoya1,
  hulicaijia7.
  Sections:
  - [XingZ602 launches on class fifty-two, 2026-09-17, 18:56 UTC](translation-repair-openrouter-2026-09-03-2026-09-17-1242-to-1856-utc.md#xingz602-launches-on-class-fifty-two-2026-09-17-1856-utc)
  - [Class fifty-two, 2026-09-17, 18:50 UTC: a picture three readers read is corroborated on the first two alone](translation-repair-openrouter-2026-09-03-2026-09-17-1242-to-1856-utc.md#class-fifty-two-2026-09-17-1850-utc-a-picture-three-readers-read-is-corroborated-on-the-first-two-alone)
  - [XingZ601 read, 2026-09-17, 18:48 UTC: INCOMPLETE at the pictures phase](translation-repair-openrouter-2026-09-03-2026-09-17-1242-to-1856-utc.md#xingz601-read-2026-09-17-1848-utc-incomplete-at-the-pictures-phase)
  - [XingZ601 launches, 2026-09-17, 18:39 UTC](translation-repair-openrouter-2026-09-03-2026-09-17-1242-to-1856-utc.md#xingz601-launches-2026-09-17-1839-utc)
  - [yingying1 read, 2026-09-17, 18:42 UTC: SETTLED, clean](translation-repair-openrouter-2026-09-03-2026-09-17-1242-to-1856-utc.md#yingying1-read-2026-09-17-1842-utc-settled-clean)
  - [yingying1 launches, 2026-09-17, 18:23 UTC](translation-repair-openrouter-2026-09-03-2026-09-17-1242-to-1856-utc.md#yingying1-launches-2026-09-17-1823-utc)
  - [shi_Yumiaoya5 read, 2026-09-17, 18:28 UTC: SETTLED, the death paragraphs on the page](translation-repair-openrouter-2026-09-03-2026-09-17-1242-to-1856-utc.md#shi_yumiaoya5-read-2026-09-17-1828-utc-settled-the-death-paragraphs-on-the-page)
  - [shi_Yumiaoya5 launches on classes fifty and fifty-one, 2026-09-17, 17:38 UTC](translation-repair-openrouter-2026-09-03-2026-09-17-1242-to-1856-utc.md#shi_yumiaoya5-launches-on-classes-fifty-and-fifty-one-2026-09-17-1738-utc)
  - [Classes fifty and fifty-one, 2026-09-17, 17:35 UTC: dark seats in the coverage denominator, and a partial claim pointing at another slice's rendering](translation-repair-openrouter-2026-09-03-2026-09-17-1242-to-1856-utc.md#classes-fifty-and-fifty-one-2026-09-17-1735-utc-dark-seats-in-the-coverage-denominator-and-a-partial-claim-pointing-at-another-slices-rendering)
  - [shi_Yumiaoya4 read, 2026-09-17, 17:30 UTC: SETTLED, but the death paragraphs shipped as a recorded gap](translation-repair-openrouter-2026-09-03-2026-09-17-1242-to-1856-utc.md#shi_yumiaoya4-read-2026-09-17-1730-utc-settled-but-the-death-paragraphs-shipped-as-a-recorded-gap)
  - [shi_Yumiaoya4 launches on class forty-nine, 2026-09-17, 16:30 UTC](translation-repair-openrouter-2026-09-03-2026-09-17-1242-to-1856-utc.md#shi_yumiaoya4-launches-on-class-forty-nine-2026-09-17-1630-utc)
  - [Class forty-nine, 2026-09-17, 16:25 UTC: a candidate defines a note its passage only refers to, and the assembly withdraws both carriers](translation-repair-openrouter-2026-09-03-2026-09-17-1242-to-1856-utc.md#class-forty-nine-2026-09-17-1625-utc-a-candidate-defines-a-note-its-passage-only-refers-to-and-the-assembly-withdraws-both-carriers)
  - [shi_Yumiaoya3 read, 2026-09-17, 16:20 UTC: INCOMPLETE at publish, the opening section withdrawn with its link](translation-repair-openrouter-2026-09-03-2026-09-17-1242-to-1856-utc.md#shi_yumiaoya3-read-2026-09-17-1620-utc-incomplete-at-publish-the-opening-section-withdrawn-with-its-link)
  - [shi_Yumiaoya3 launches on classes forty-seven and forty-eight, 2026-09-17, 14:56 UTC](translation-repair-openrouter-2026-09-03-2026-09-17-1242-to-1856-utc.md#shi_yumiaoya3-launches-on-classes-forty-seven-and-forty-eight-2026-09-17-1456-utc)
  - [Classes forty-seven and forty-eight, 2026-09-17, 14:50 UTC: a bilingual quote fails the line floor unless doubled, and a split nobody anchored drops a passage](translation-repair-openrouter-2026-09-03-2026-09-17-1242-to-1856-utc.md#classes-forty-seven-and-forty-eight-2026-09-17-1450-utc-a-bilingual-quote-fails-the-line-floor-unless-doubled-and-a-split-nobody-anchored-drops-a-passage)
  - [shi_Yumiaoya2 read, 2026-09-17, 14:40 UTC: INCOMPLETE at the farewell, the will's paragraph a silent gap](translation-repair-openrouter-2026-09-03-2026-09-17-1242-to-1856-utc.md#shi_yumiaoya2-read-2026-09-17-1440-utc-incomplete-at-the-farewell-the-wills-paragraph-a-silent-gap)
  - [shi_Yumiaoya2 launches on class forty-six, 2026-09-17, 13:45 UTC](translation-repair-openrouter-2026-09-03-2026-09-17-1242-to-1856-utc.md#shi_yumiaoya2-launches-on-class-forty-six-2026-09-17-1345-utc)
  - [Class forty-six, 2026-09-17, 13:42 UTC: a link inside the original's HTML comment is a destination the page owes](translation-repair-openrouter-2026-09-03-2026-09-17-1242-to-1856-utc.md#class-forty-six-2026-09-17-1342-utc-a-link-inside-the-originals-html-comment-is-a-destination-the-page-owes)
  - [shi_Yumiaoya1 read, 2026-09-17, 13:40 UTC: INCOMPLETE at publish over a commented-out link, the lanes' page whole](translation-repair-openrouter-2026-09-03-2026-09-17-1242-to-1856-utc.md#shi_yumiaoya1-read-2026-09-17-1340-utc-incomplete-at-publish-over-a-commented-out-link-the-lanes-page-whole)
  - [shi_Yumiaoya1 launches, 2026-09-17, 12:42 UTC](translation-repair-openrouter-2026-09-03-2026-09-17-1242-to-1856-utc.md#shi_yumiaoya1-launches-2026-09-17-1242-utc)
  - [hulicaijia7 read, 2026-09-17, 12:50 UTC: the page is whole, the headings distinct, nothing restored](translation-repair-openrouter-2026-09-03-2026-09-17-1242-to-1856-utc.md#hulicaijia7-read-2026-09-17-1250-utc-the-page-is-whole-the-headings-distinct-nothing-restored)
- [OpenRouter and the corpus passes: 2026-09-17, 00:15 to 10:10 UTC](translation-repair-openrouter-2026-09-03-2026-09-17-0015-to-1010-utc.md):
  What was launched,
  read and decided on 2026-09-17 00:15 to 10:10 UTC.
  Entries:
  hulicaijia7,
  hulicaijia6,
  Mio27,
  Mio26,
  Mio25,
  Mio24.
  Sections:
  - [hulicaijia7 launches on class forty-five, 2026-09-17, 10:10 UTC](translation-repair-openrouter-2026-09-03-2026-09-17-0015-to-1010-utc.md#hulicaijia7-launches-on-class-forty-five-2026-09-17-1010-utc)
  - [Class forty-five, 2026-09-17, 10:05 UTC: a lane renders one section under another section's heading and the page dies at publish](translation-repair-openrouter-2026-09-03-2026-09-17-0015-to-1010-utc.md#class-forty-five-2026-09-17-1005-utc-a-lane-renders-one-section-under-another-sections-heading-and-the-page-dies-at-publish)
  - [hulicaijia6 read, 2026-09-17, 09:54 UTC: ERROR at publish, thirteen headings rendered as twelve](translation-repair-openrouter-2026-09-03-2026-09-17-0015-to-1010-utc.md#hulicaijia6-read-2026-09-17-0954-utc-error-at-publish-thirteen-headings-rendered-as-twelve)
  - [hulicaijia6 launches on class forty-four, 2026-09-17, 06:15 UTC](translation-repair-openrouter-2026-09-03-2026-09-17-0015-to-1010-utc.md#hulicaijia6-launches-on-class-forty-four-2026-09-17-0615-utc)
  - [Class forty-four, 2026-09-17, 06:10 UTC: the writers copy the sheet's picture transcript into their candidates](translation-repair-openrouter-2026-09-03-2026-09-17-0015-to-1010-utc.md#class-forty-four-2026-09-17-0610-utc-the-writers-copy-the-sheets-picture-transcript-into-their-candidates)
  - [Mio27 read, 2026-09-17, 06:05 UTC: the sister clause ships end to end, the poem keeps its five breaks](translation-repair-openrouter-2026-09-03-2026-09-17-0015-to-1010-utc.md#mio27-read-2026-09-17-0605-utc-the-sister-clause-ships-end-to-end-the-poem-keeps-its-five-breaks)
  - [Mio27 launches on class forty-three, 2026-09-17, 04:07 UTC](translation-repair-openrouter-2026-09-03-2026-09-17-0015-to-1010-utc.md#mio27-launches-on-class-forty-three-2026-09-17-0407-utc)
  - [Mio26 killed at 04:07 UTC in its pictures phase](translation-repair-openrouter-2026-09-03-2026-09-17-0015-to-1010-utc.md#mio26-killed-at-0407-utc-in-its-pictures-phase)
  - [Class forty-three, 2026-09-17, 04:00 UTC: the attestation's open question is answered empty by voices that never looked](translation-repair-openrouter-2026-09-03-2026-09-17-0015-to-1010-utc.md#class-forty-three-2026-09-17-0400-utc-the-attestations-open-question-is-answered-empty-by-voices-that-never-looked)
  - [Mio26 launches on classes forty-one and forty-two, 2026-09-17, 03:25 UTC](translation-repair-openrouter-2026-09-03-2026-09-17-0015-to-1010-utc.md#mio26-launches-on-classes-forty-one-and-forty-two-2026-09-17-0325-utc)
  - [Classes forty-one and forty-two, 2026-09-17, 03:20 UTC: the polish strips the attested clause, the verse loses its breaks under a substitute paragraph](translation-repair-openrouter-2026-09-03-2026-09-17-0015-to-1010-utc.md#classes-forty-one-and-forty-two-2026-09-17-0320-utc-the-polish-strips-the-attested-clause-the-verse-loses-its-breaks-under-a-substitute-paragraph)
  - [Mio25 read, 2026-09-17, 03:00 UTC: settled on the second attempt, the sister clause lost at the polish, the poem without its breaks](translation-repair-openrouter-2026-09-03-2026-09-17-0015-to-1010-utc.md#mio25-read-2026-09-17-0300-utc-settled-on-the-second-attempt-the-sister-clause-lost-at-the-polish-the-poem-without-its-breaks)
  - [Mio25 launches on class forty, 2026-09-17, 00:40 UTC](translation-repair-openrouter-2026-09-03-2026-09-17-0015-to-1010-utc.md#mio25-launches-on-class-forty-2026-09-17-0040-utc)
  - [Class forty, 2026-09-17, 00:35 UTC: a declined contest leaves the slate blind to the lane texts](translation-repair-openrouter-2026-09-03-2026-09-17-0015-to-1010-utc.md#class-forty-2026-09-17-0035-utc-a-declined-contest-leaves-the-slate-blind-to-the-lane-texts)
  - [Mio24 read, 2026-09-17, 00:15 UTC: the attestation and the repair lane hold, the entry stops at slice 17](translation-repair-openrouter-2026-09-03-2026-09-17-0015-to-1010-utc.md#mio24-read-2026-09-17-0015-utc-the-attestation-and-the-repair-lane-hold-the-entry-stops-at-slice-17)
- [OpenRouter and the corpus passes: 2026-09-18](translation-repair-openrouter-2026-09-03-2026-09-18.md):
  What was launched,
  read and decided on 2026-09-18.
  Entries:
  XingZ607,
  XingZ606,
  XingZ605,
  XingZ604,
  XingZ603,
  XingZ602.
  Sections:
  - [Class fifty-seven, 2026-09-18, 23:55 UTC: a container half ships without its partner and the translate lane withdraws every slice](translation-repair-openrouter-2026-09-03-2026-09-18.md#class-fifty-seven-2026-09-18-2355-utc-a-container-half-ships-without-its-partner-and-the-translate-lane-withdraws-every-slice)
  - [XingZ607 attempt 2 read, 2026-09-18, 23:20 UTC: INCOMPLETE at 1h41m, one source destination dropped, the translate lane empty](translation-repair-openrouter-2026-09-03-2026-09-18.md#xingz607-attempt-2-read-2026-09-18-2320-utc-incomplete-at-1h41m-one-source-destination-dropped-the-translate-lane-empty)
  - [XingZ607 attempt 1, 2026-09-18, 21:39 UTC: the seven-hour entry deadline in consolidation, both pronoun paragraphs shipped](translation-repair-openrouter-2026-09-03-2026-09-18.md#xingz607-attempt-1-2026-09-18-2139-utc-the-seven-hour-entry-deadline-in-consolidation-both-pronoun-paragraphs-shipped)
  - [XingZ607 launches on class fifty-six, 2026-09-18, 14:40 UTC](translation-repair-openrouter-2026-09-03-2026-09-18.md#xingz607-launches-on-class-fifty-six-2026-09-18-1440-utc)
  - [Class fifty-six, 2026-09-18, 14:36 UTC: the gate keeps an ineligible standing it was never told cannot ship, and logs no reasons](translation-repair-openrouter-2026-09-03-2026-09-18.md#class-fifty-six-2026-09-18-1436-utc-the-gate-keeps-an-ineligible-standing-it-was-never-told-cannot-ship-and-logs-no-reasons)
  - [XingZ606 read, 2026-09-18, 14:30 UTC: INCOMPLETE at 3h04m on slice 33, the gate keeping the teachers paragraph's standing](translation-repair-openrouter-2026-09-03-2026-09-18.md#xingz606-read-2026-09-18-1430-utc-incomplete-at-3h04m-on-slice-33-the-gate-keeping-the-teachers-paragraphs-standing)
  - [XingZ606 launches on class fifty-five, 2026-09-18, 11:20 UTC](translation-repair-openrouter-2026-09-03-2026-09-18.md#xingz606-launches-on-class-fifty-five-2026-09-18-1120-utc)
  - [Class fifty-five, 2026-09-18, 11:15 UTC: a tied consolidation slate over an ineligible standing declines without the challenge round](translation-repair-openrouter-2026-09-03-2026-09-18.md#class-fifty-five-2026-09-18-1115-utc-a-tied-consolidation-slate-over-an-ineligible-standing-declines-without-the-challenge-round)
  - [XingZ605 read, 2026-09-18, 11:12 UTC: INCOMPLETE at 4h08m on slice 13, the slate tied](translation-repair-openrouter-2026-09-03-2026-09-18.md#xingz605-read-2026-09-18-1112-utc-incomplete-at-4h08m-on-slice-13-the-slate-tied)
  - [XingZ605 launches on class fifty-four, 2026-09-18, 07:03 UTC](translation-repair-openrouter-2026-09-03-2026-09-18.md#xingz605-launches-on-class-fifty-four-2026-09-18-0703-utc)
  - [Class fifty-four, 2026-09-18, 06:58 UTC: the consolidate gate's indecision keeps an ineligible standing over the proposal the slate chose](translation-repair-openrouter-2026-09-03-2026-09-18.md#class-fifty-four-2026-09-18-0658-utc-the-consolidate-gates-indecision-keeps-an-ineligible-standing-over-the-proposal-the-slate-chose)
  - [XingZ604 read, 2026-09-18, 06:55 UTC: INCOMPLETE at 4h53m on slice 13, the first pass with a typed-decision judge](translation-repair-openrouter-2026-09-03-2026-09-18.md#xingz604-read-2026-09-18-0655-utc-incomplete-at-4h53m-on-slice-13-the-first-pass-with-a-typed-decision-judge)
  - [XingZ604 launches on the Jev seat, 2026-09-18, 01:56 UTC](translation-repair-openrouter-2026-09-03-2026-09-18.md#xingz604-launches-on-the-jev-seat-2026-09-18-0156-utc)
  - [XingZ603 killed, 2026-09-18, 01:56 UTC: 60 minutes into the repair lane, the seat landed](translation-repair-openrouter-2026-09-03-2026-09-18.md#xingz603-killed-2026-09-18-0156-utc-60-minutes-into-the-repair-lane-the-seat-landed)
  - [Jev 1.13 seated as a select judge, 2026-09-18, 01:47 UTC: 12 of 12 through the production probe](translation-repair-openrouter-2026-09-03-2026-09-18.md#jev-113-seated-as-a-select-judge-2026-09-18-0147-utc-12-of-12-through-the-production-probe)
  - [XingZ603 launches on class fifty-three, 2026-09-18, 00:55 UTC](translation-repair-openrouter-2026-09-03-2026-09-18.md#xingz603-launches-on-class-fifty-three-2026-09-18-0055-utc)
  - [Jev 1.13 approved on OpenRouter, 2026-09-18, 00:29 UTC: a typed-decision model, probed live](translation-repair-openrouter-2026-09-03-2026-09-18.md#jev-113-approved-on-openrouter-2026-09-18-0029-utc-a-typed-decision-model-probed-live)
  - [Class fifty-three, 2026-09-18, 00:45 UTC: a tied slate with nothing to fall back on is challenged over the same slate and stops the entry](translation-repair-openrouter-2026-09-03-2026-09-18.md#class-fifty-three-2026-09-18-0045-utc-a-tied-slate-with-nothing-to-fall-back-on-is-challenged-over-the-same-slate-and-stops-the-entry)
  - [XingZ602 read, 2026-09-18, 00:56 UTC: ERROR at 2h38m on slice 14, the reattempt killed at 5h57m in consolidation](translation-repair-openrouter-2026-09-03-2026-09-18.md#xingz602-read-2026-09-18-0056-utc-error-at-2h38m-on-slice-14-the-reattempt-killed-at-5h57m-in-consolidation)
- [OpenRouter and the corpus passes: 2026-09-19, 14:55 to 23:20 UTC](translation-repair-openrouter-2026-09-03-2026-09-19-1455-to-2320-utc.md):
  What was launched,
  read and decided on 2026-09-19 14:55 to 23:20 UTC.
  Entries:
  mikaela5,
  mikaela4,
  mikaela3,
  mikaela2,
  XingZ619,
  XingZ60,
  XingZ618,
  XingZ616.
  Sections:
  - [mikaela5 read and the abstention log, 2026-09-19, 23:20 UTC: class seventy-two on the page, the slate settled slice 28 on its first round](translation-repair-openrouter-2026-09-03-2026-09-19-1455-to-2320-utc.md#mikaela5-read-and-the-abstention-log-2026-09-19-2320-utc-class-seventy-two-on-the-page-the-slate-settled-slice-28-on-its-first-round)
  - [mikaela4 read and class seventy-three, 2026-09-19, 22:50 UTC: the glossary reading lands in the lane, the entry stops at a run-off two judges decided, mikaela5 launched](translation-repair-openrouter-2026-09-03-2026-09-19-1455-to-2320-utc.md#mikaela4-read-and-class-seventy-three-2026-09-19-2250-utc-the-glossary-reading-lands-in-the-lane-the-entry-stops-at-a-run-off-two-judges-decided-mikaela5-launched)
  - [mikaela3 read and class seventy-two, 2026-09-19, 22:10 UTC: classes seventy and seventy-one live, 炸柜 seeded in the community glossary, mikaela4 launched](translation-repair-openrouter-2026-09-03-2026-09-19-1455-to-2320-utc.md#mikaela3-read-and-class-seventy-two-2026-09-19-2210-utc-classes-seventy-and-seventy-one-live-炸柜-seeded-in-the-community-glossary-mikaela4-launched)
  - [Class seventy-one, 2026-09-19, 21:30 UTC: a page's names as the archive renders them reach every sheet; mikaela2 killed, mikaela3 launched](translation-repair-openrouter-2026-09-03-2026-09-19-1455-to-2320-utc.md#class-seventy-one-2026-09-19-2130-utc-a-pages-names-as-the-archive-renders-them-reach-every-sheet-mikaela2-killed-mikaela3-launched)
  - [The three shapes read and class seventy, 2026-09-19, 21:25 UTC](translation-repair-openrouter-2026-09-03-2026-09-19-1455-to-2320-utc.md#the-three-shapes-read-and-class-seventy-2026-09-19-2125-utc)
  - [The last three unmet shapes launch together, 2026-09-19, 20:23 UTC](translation-repair-openrouter-2026-09-03-2026-09-19-1455-to-2320-utc.md#the-last-three-unmet-shapes-launch-together-2026-09-19-2023-utc)
  - [XingZ619 read, 2026-09-19, 20:40 UTC: XingZ60 SETTLED with classes sixty-seven to sixty-nine live](translation-repair-openrouter-2026-09-03-2026-09-19-1455-to-2320-utc.md#xingz619-read-2026-09-19-2040-utc-xingz60-settled-with-classes-sixty-seven-to-sixty-nine-live)
  - [Class sixty-nine, 2026-09-19, 17:20 UTC: the block deficit decides a split; XingZ618 killed, XingZ619 launched](translation-repair-openrouter-2026-09-03-2026-09-19-1455-to-2320-utc.md#class-sixty-nine-2026-09-19-1720-utc-the-block-deficit-decides-a-split-xingz618-killed-xingz619-launched)
  - [The V4.1 Flash calibration is read, V4.1 Flash writes, Mercury leaves the translator seat, 2026-09-19, 16:55 UTC](translation-repair-openrouter-2026-09-03-2026-09-19-1455-to-2320-utc.md#the-v41-flash-calibration-is-read-v41-flash-writes-mercury-leaves-the-translator-seat-2026-09-19-1655-utc)
  - [Classes sixty-seven and sixty-eight, 2026-09-19, 15:35 UTC: one name one way, one series one style](translation-repair-openrouter-2026-09-03-2026-09-19-1455-to-2320-utc.md#classes-sixty-seven-and-sixty-eight-2026-09-19-1535-utc-one-name-one-way-one-series-one-style)
  - [XingZ616 read, 2026-09-19, 14:55 UTC: XingZ60 SETTLED](translation-repair-openrouter-2026-09-03-2026-09-19-1455-to-2320-utc.md#xingz616-read-2026-09-19-1455-utc-xingz60-settled)
- [OpenRouter and the corpus passes: 2026-09-19, 00:05 to 12:46 UTC](translation-repair-openrouter-2026-09-03-2026-09-19-0005-to-1246-utc.md):
  What was launched,
  read and decided on 2026-09-19 00:05 to 12:46 UTC.
  Entries:
  XingZ616,
  XingZ615,
  XingZ614,
  XingZ613,
  XingZ612,
  XingZ611,
  XingZ610,
  XingZ608,
  v4.
  Sections:
  - [XingZ616 launches on classes sixty-five and sixty-six, 2026-09-19, 12:46 UTC](translation-repair-openrouter-2026-09-03-2026-09-19-0005-to-1246-utc.md#xingz616-launches-on-classes-sixty-five-and-sixty-six-2026-09-19-1246-utc)
  - [Classes sixty-five and sixty-six, 2026-09-19, 12:40 UTC: a respaced copy of the original, and a refused candidate on the translate slate](translation-repair-openrouter-2026-09-03-2026-09-19-0005-to-1246-utc.md#classes-sixty-five-and-sixty-six-2026-09-19-1240-utc-a-respaced-copy-of-the-original-and-a-refused-candidate-on-the-translate-slate)
  - [XingZ615 launches on class sixty-four, 2026-09-19, 09:50 UTC](translation-repair-openrouter-2026-09-03-2026-09-19-0005-to-1246-utc.md#xingz615-launches-on-class-sixty-four-2026-09-19-0950-utc)
  - [Class sixty-four, 2026-09-19, 09:45 UTC: the rate-limited seat is asked again on every round](translation-repair-openrouter-2026-09-03-2026-09-19-0005-to-1246-utc.md#class-sixty-four-2026-09-19-0945-utc-the-rate-limited-seat-is-asked-again-on-every-round)
  - [XingZ614 launches on class sixty-three, 2026-09-19, 09:38 UTC](translation-repair-openrouter-2026-09-03-2026-09-19-0005-to-1246-utc.md#xingz614-launches-on-class-sixty-three-2026-09-19-0938-utc)
  - [Class sixty-three, 2026-09-19, 09:30 UTC: the run-off over every backed candidate is the same question](translation-repair-openrouter-2026-09-03-2026-09-19-0005-to-1246-utc.md#class-sixty-three-2026-09-19-0930-utc-the-run-off-over-every-backed-candidate-is-the-same-question)
  - [XingZ613 launches on class sixty-two, 2026-09-19, 06:22 UTC](translation-repair-openrouter-2026-09-03-2026-09-19-0005-to-1246-utc.md#xingz613-launches-on-class-sixty-two-2026-09-19-0622-utc)
  - [Class sixty-two, 2026-09-19, 06:15 UTC: one model's upstream rate limit holds the whole provider out](translation-repair-openrouter-2026-09-03-2026-09-19-0005-to-1246-utc.md#class-sixty-two-2026-09-19-0615-utc-one-models-upstream-rate-limit-holds-the-whole-provider-out)
  - [XingZ612 launches on class sixty-one, 2026-09-19, 05:39 UTC](translation-repair-openrouter-2026-09-03-2026-09-19-0005-to-1246-utc.md#xingz612-launches-on-class-sixty-one-2026-09-19-0539-utc)
  - [Class sixty-one, 2026-09-19, 05:30 UTC: the interior budget refuses a paragraph the archive's block is short of](translation-repair-openrouter-2026-09-03-2026-09-19-0005-to-1246-utc.md#class-sixty-one-2026-09-19-0530-utc-the-interior-budget-refuses-a-paragraph-the-archives-block-is-short-of)
  - [XingZ611 launches on class sixty, 2026-09-19, 03:55 UTC](translation-repair-openrouter-2026-09-03-2026-09-19-0005-to-1246-utc.md#xingz611-launches-on-class-sixty-2026-09-19-0355-utc)
  - [Class sixty, 2026-09-19, 03:50 UTC: a split verdict inside the admitted tail stays unresolved](translation-repair-openrouter-2026-09-03-2026-09-19-0005-to-1246-utc.md#class-sixty-2026-09-19-0350-utc-a-split-verdict-inside-the-admitted-tail-stays-unresolved)
  - [XingZ610 launches on the tail budget, 2026-09-19, 03:24 UTC](translation-repair-openrouter-2026-09-03-2026-09-19-0005-to-1246-utc.md#xingz610-launches-on-the-tail-budget-2026-09-19-0324-utc)
  - [The untranslated tail is budgeted from the pairing, 2026-09-19, 03:20 UTC: the owner's "Do #1"](translation-repair-openrouter-2026-09-03-2026-09-19-0005-to-1246-utc.md#the-untranslated-tail-is-budgeted-from-the-pairing-2026-09-19-0320-utc-the-owners-do-1)
  - [Class fifty-nine, 2026-09-19, 02:45 UTC: the shortfall budget refuses the definitions the shipped markers reference](translation-repair-openrouter-2026-09-03-2026-09-19-0005-to-1246-utc.md#class-fifty-nine-2026-09-19-0245-utc-the-shortfall-budget-refuses-the-definitions-the-shipped-markers-reference)
  - [XingZ608 read, 2026-09-19, 02:35 UTC: INCOMPLETE at 2h16m at publish, the run time verified](translation-repair-openrouter-2026-09-03-2026-09-19-0005-to-1246-utc.md#xingz608-read-2026-09-19-0235-utc-incomplete-at-2h16m-at-publish-the-run-time-verified)
  - [deepseek-v4.1-flash routed off DeepInfra and Wafer, 2026-09-19, 00:05 UTC: the endpoint was the slow seat](translation-repair-openrouter-2026-09-03-2026-09-19-0005-to-1246-utc.md#deepseek-v41-flash-routed-off-deepinfra-and-wafer-2026-09-19-0005-utc-the-endpoint-was-the-slow-seat)
  - [Class fifty-eight, 2026-09-19, 00:10 UTC: two structural breaks defeat the single-withdrawal proof and empty the lane](translation-repair-openrouter-2026-09-03-2026-09-19-0005-to-1246-utc.md#class-fifty-eight-2026-09-19-0010-utc-two-structural-breaks-defeat-the-single-withdrawal-proof-and-empty-the-lane)
- [OpenRouter and the corpus passes: 2026-09-21](translation-repair-openrouter-2026-09-03-2026-09-21.md):
  What was launched,
  read and decided on 2026-09-21.
  Entries:
  CuspariaKLSY5,
  CuspariaKLSY4,
  d70087757,
  CuspariaKLSY2,
  c9e267d3a,
  c5470163d,
  CuspariaKLSY3,
  hulicaijia9,
  XingZ620,
  yingying2,
  shi_Yumiaoya7,
  shi_Yumiaoya6,
  hulicaijia8.
  Sections:
  - [CuspariaKLSY5 read, 2026-09-21, 23:20 UTC: class seventy-six live, the life in the past on every line, the slice 0 contest won by the translate lane 3 of 5 with two ballots citing the house rule](translation-repair-openrouter-2026-09-03-2026-09-21.md#cuspariaklsy5-read-2026-09-21-2320-utc-class-seventy-six-live-the-life-in-the-past-on-every-line-the-slice-0-contest-won-by-the-translate-lane-3-of-5-with-two-ballots-citing-the-house-rule)
  - [CuspariaKLSY4 read, 2026-09-21, 22:40 UTC: class seventy-five live, but the archive's present tense shipped on a split contest; class seventy-six (the judge tail named the replaced text's tense as the authority) fixed in d70087757; CuspariaKLSY5 launched](translation-repair-openrouter-2026-09-03-2026-09-21.md#cuspariaklsy4-read-2026-09-21-2240-utc-class-seventy-five-live-but-the-archives-present-tense-shipped-on-a-split-contest-class-seventy-six-the-judge-tail-named-the-replaced-texts-tense-as-the-authority-fixed-in-d70087757-cuspariaklsy5-launched)
  - [CuspariaKLSY2 read, 2026-09-21, 21:50 UTC: the life is told in the past tense, class seventy-five (a lane's trailing newline doubles the block separator) fixed in c9e267d3a and c5470163d; CuspariaKLSY3 killed for the second half, CuspariaKLSY4 launched](translation-repair-openrouter-2026-09-03-2026-09-21.md#cuspariaklsy2-read-2026-09-21-2150-utc-the-life-is-told-in-the-past-tense-class-seventy-five-a-lanes-trailing-newline-doubles-the-block-separator-fixed-in-c9e267d3a-and-c5470163d-cuspariaklsy3-killed-for-the-second-half-cuspariaklsy4-launched)
  - [hulicaijia9 read, 2026-09-21, 21:00 UTC: 左右 ships as the archive's Conflict, no candidate proposed the literal heading, SETTLED in 82 minutes; CuspariaKLSY2 launched for the tense line](translation-repair-openrouter-2026-09-03-2026-09-21.md#hulicaijia9-read-2026-09-21-2100-utc-左右-ships-as-the-archives-conflict-no-candidate-proposed-the-literal-heading-settled-in-82-minutes-cuspariaklsy2-launched-for-the-tense-line)
  - [The owner answers the three observations, 2026-09-21, 19:35 UTC: the judges keep the headings with the archive's wording as evidence, the em-dashes stay, the life is told in the past tense; hulicaijia9 launched](translation-repair-openrouter-2026-09-03-2026-09-21.md#the-owner-answers-the-three-observations-2026-09-21-1935-utc-the-judges-keep-the-headings-with-the-archives-wording-as-evidence-the-em-dashes-stay-the-life-is-told-in-the-past-tense-hulicaijia9-launched)
  - [XingZ620 read, 2026-09-21, 19:00 UTC: the queue is run through on the current build; three observations put to the owner](translation-repair-openrouter-2026-09-03-2026-09-21.md#xingz620-read-2026-09-21-1900-utc-the-queue-is-run-through-on-the-current-build-three-observations-put-to-the-owner)
  - [yingying2 read, 2026-09-21, 14:50 UTC: clean, XingZ620 launched as the queue's last entry](translation-repair-openrouter-2026-09-03-2026-09-21.md#yingying2-read-2026-09-21-1450-utc-clean-xingz620-launched-as-the-queues-last-entry)
  - [shi_Yumiaoya7 read, 2026-09-21, 14:25 UTC: the attribution once, the source footnote glossed inline under the house rule, yingying2 launched](translation-repair-openrouter-2026-09-03-2026-09-21.md#shi_yumiaoya7-read-2026-09-21-1425-utc-the-attribution-once-the-source-footnote-glossed-inline-under-the-house-rule-yingying2-launched)
  - [shi_Yumiaoya6 read and class seventy-four, 2026-09-21, 13:25 UTC: a repeated line on a line-structured slice, shi_Yumiaoya7 launched](translation-repair-openrouter-2026-09-03-2026-09-21.md#shi_yumiaoya6-read-and-class-seventy-four-2026-09-21-1325-utc-a-repeated-line-on-a-line-structured-slice-shi_yumiaoya7-launched)
  - [hulicaijia8 read, 2026-09-21, 12:15 UTC: the queue resumes on the current build, shi_Yumiaoya6 launched](translation-repair-openrouter-2026-09-03-2026-09-21.md#hulicaijia8-read-2026-09-21-1215-utc-the-queue-resumes-on-the-current-build-shi_yumiaoya6-launched)
- [OpenRouter and the corpus passes: 2026-09-22, 13:30 to 21:55 UTC](translation-repair-openrouter-2026-09-03-2026-09-22-1330-to-2155-utc.md):
  What was launched,
  read and decided on 2026-09-22 13:30 to 21:55 UTC.
  Entries:
  XingZ623,
  da02857b1,
  XingZ624,
  yingying4,
  yingying3,
  zheermao4,
  CuspariaKLSY7,
  CuspariaKLSY6,
  hulicaijia13,
  hulicaijia12,
  hulicaijia11,
  hulicaijia10,
  fa014fc55.
  Sections:
  - [XingZ623 read, 2026-09-22, 21:55 UTC: INCOMPLETE at publish after 3h01m on the bare TextRing slice, nothing standing behind a declined contest at an absent archive (class eighty-seven); fixed in da02857b1, XingZ624 launched](translation-repair-openrouter-2026-09-03-2026-09-22-1330-to-2155-utc.md#xingz623-read-2026-09-22-2155-utc-incomplete-at-publish-after-3h01m-on-the-bare-textring-slice-nothing-standing-behind-a-declined-contest-at-an-absent-archive-class-eighty-seven-fixed-in-da02857b1-xingz624-launched)
  - [yingying4 read, 2026-09-22, 18:40 UTC: SETTLED in 23 min, class eighty-six live, the blog title names "Yingying"; XingZ623 launched for class eighty-three](translation-repair-openrouter-2026-09-03-2026-09-22-1330-to-2155-utc.md#yingying4-read-2026-09-22-1840-utc-settled-in-23-min-class-eighty-six-live-the-blog-title-names-yingying-xingz623-launched-for-class-eighty-three)
  - [yingying3 read, 2026-09-22, 18:15 UTC: SETTLED in 14 min, but the blog title names the person "Sakura" against the declared "Yingying" (class eighty-six); fixed in 40679244d, yingying4 launched](translation-repair-openrouter-2026-09-03-2026-09-22-1330-to-2155-utc.md#yingying3-read-2026-09-22-1815-utc-settled-in-14-min-but-the-blog-title-names-the-person-sakura-against-the-declared-yingying-class-eighty-six-fixed-in-40679244d-yingying4-launched)
  - [zheermao4 read, 2026-09-22, 17:50 UTC: SETTLED in 32 min, no class; the reply email revised from the transcript, the Arts letter kept as the archive; yingying3 launched](translation-repair-openrouter-2026-09-03-2026-09-22-1330-to-2155-utc.md#zheermao4-read-2026-09-22-1750-utc-settled-in-32-min-no-class-the-reply-email-revised-from-the-transcript-the-arts-letter-kept-as-the-archive-yingying3-launched)
  - [CuspariaKLSY7 read, 2026-09-22, 17:15 UTC: SETTLED in 22 min, class eighty-five live (the gloss on the page and every line of the life in the past); zheermao4 launched](translation-repair-openrouter-2026-09-03-2026-09-22-1330-to-2155-utc.md#cuspariaklsy7-read-2026-09-22-1715-utc-settled-in-22-min-class-eighty-five-live-the-gloss-on-the-page-and-every-line-of-the-life-in-the-past-zheermao4-launched)
  - [CuspariaKLSY6 read, 2026-09-22, 16:50 UTC: SETTLED in 16 min, but slice 1 flipped back to the archive's present tense to keep a translator's gloss line (class eighty-five); fixed in 7ad1b8ec7, CuspariaKLSY7 launched](translation-repair-openrouter-2026-09-03-2026-09-22-1330-to-2155-utc.md#cuspariaklsy6-read-2026-09-22-1650-utc-settled-in-16-min-but-slice-1-flipped-back-to-the-archives-present-tense-to-keep-a-translators-gloss-line-class-eighty-five-fixed-in-7ad1b8ec7-cuspariaklsy7-launched)
  - [hulicaijia13 read, 2026-09-22, 16:25 UTC: SETTLED in 68 min, class eighty-four live (two removals of the Nayuki note withheld, the note on the page with its marker); CuspariaKLSY6 launched](translation-repair-openrouter-2026-09-03-2026-09-22-1330-to-2155-utc.md#hulicaijia13-read-2026-09-22-1625-utc-settled-in-68-min-class-eighty-four-live-two-removals-of-the-nayuki-note-withheld-the-note-on-the-page-with-its-marker-cuspariaklsy6-launched)
  - [hulicaijia12 read, 2026-09-22, 15:10 UTC: SETTLED in 79 min, but a translator's note removed and its marker left dangling (class eighty-four); fixed in 59fe52b51, hulicaijia13 launched](translation-repair-openrouter-2026-09-03-2026-09-22-1330-to-2155-utc.md#hulicaijia12-read-2026-09-22-1510-utc-settled-in-79-min-but-a-translators-note-removed-and-its-marker-left-dangling-class-eighty-four-fixed-in-59fe52b51-hulicaijia13-launched)
  - [Class eighty-three amended, 2026-09-22, 13:40 UTC: syllable groups, never one joined word (owner's veto); hulicaijia11 killed, hulicaijia12 launched on 4e5cf65e0](translation-repair-openrouter-2026-09-03-2026-09-22-1330-to-2155-utc.md#class-eighty-three-amended-2026-09-22-1340-utc-syllable-groups-never-one-joined-word-owners-veto-hulicaijia11-killed-hulicaijia12-launched-on-4e5cf65e0)
  - [Class eighty-three built, 2026-09-22, 13:30 UTC: an undeclared handle is pinyin one way across the page, with its literal meaning in parentheses (owner's answer); hulicaijia10 killed, hulicaijia11 launched on fa014fc55](translation-repair-openrouter-2026-09-03-2026-09-22-1330-to-2155-utc.md#class-eighty-three-built-2026-09-22-1330-utc-an-undeclared-handle-is-pinyin-one-way-across-the-page-with-its-literal-meaning-in-parentheses-owners-answer-hulicaijia10-killed-hulicaijia11-launched-on-fa014fc55)
- [OpenRouter and the corpus passes: 2026-09-22, 04:30 to 13:40 UTC](translation-repair-openrouter-2026-09-03-2026-09-22-0430-to-1340-utc.md):
  What was launched,
  read and decided on 2026-09-22 04:30 to 13:40 UTC.
  Entries:
  XingZ622,
  hulicaijia10,
  XingZ621,
  mikaela7,
  shi_Yumiaoya13,
  shi_Yumiaoya12,
  shi_Yumiaoya11.
  Sections:
  - [XingZ622 read, 2026-09-22, 13:40 UTC: SETTLED in 3h39m, class eighty-one live on the page ("They once said"), class eighty-two not exercised; the undeclared-handle question open; hulicaijia10 launched](translation-repair-openrouter-2026-09-03-2026-09-22-0430-to-1340-utc.md#xingz622-read-2026-09-22-1340-utc-settled-in-3h39m-class-eighty-one-live-on-the-page-they-once-said-class-eighty-two-not-exercised-the-undeclared-handle-question-open-hulicaijia10-launched)
  - [XingZ621 read, 2026-09-22, 09:35 UTC: INCOMPLETE at slice 14 on a pronoun rule that contradicts itself (class eighty-one) and a run-off that tied again after narrowing (class eighty-two), both fixed in 6c701485d; XingZ622 launched](translation-repair-openrouter-2026-09-03-2026-09-22-0430-to-1340-utc.md#xingz621-read-2026-09-22-0935-utc-incomplete-at-slice-14-on-a-pronoun-rule-that-contradicts-itself-class-eighty-one-and-a-run-off-that-tied-again-after-narrowing-class-eighty-two-both-fixed-in-6c701485d-xingz622-launched)
  - [mikaela7 read, 2026-09-22, 06:15 UTC: SETTLED, classes seventy to seventy-seven holding on the newest build, the pair bound silent on a slice with no pair; XingZ621 launched](translation-repair-openrouter-2026-09-03-2026-09-22-0430-to-1340-utc.md#mikaela7-read-2026-09-22-0615-utc-settled-classes-seventy-to-seventy-seven-holding-on-the-newest-build-the-pair-bound-silent-on-a-slice-with-no-pair-xingz621-launched)
  - [shi_Yumiaoya13 read, 2026-09-22, 05:30 UTC: SETTLED, class eighty live (the block bound refused ten candidates and the slate chose the two-line quote on the clause); mikaela7 launched](translation-repair-openrouter-2026-09-03-2026-09-22-0430-to-1340-utc.md#shi_yumiaoya13-read-2026-09-22-0530-utc-settled-class-eighty-live-the-block-bound-refused-ten-candidates-and-the-slate-chose-the-two-line-quote-on-the-clause-mikaela7-launched)
  - [shi_Yumiaoya12 read, 2026-09-22, 05:05 UTC: SETTLED with the closing quote as the archive, but by an unendorsed standing, not by the bound; the pair bound rebuilt to compare the block holding the English line, fixed in 93b11aef9; shi_Yumiaoya13 launched](translation-repair-openrouter-2026-09-03-2026-09-22-0430-to-1340-utc.md#shi_yumiaoya12-read-2026-09-22-0505-utc-settled-with-the-closing-quote-as-the-archive-but-by-an-unendorsed-standing-not-by-the-bound-the-pair-bound-rebuilt-to-compare-the-block-holding-the-english-line-fixed-in-93b11aef9-shi_yumiaoya13-launched)
  - [shi_Yumiaoya11 read, 2026-09-22, 04:30 UTC: SETTLED, the class eighty clause on every sheet and the judges still chose the second English wording; the page's own line count now bounds the pair, fixed in 873104d8f; shi_Yumiaoya12 launched](translation-repair-openrouter-2026-09-03-2026-09-22-0430-to-1340-utc.md#shi_yumiaoya11-read-2026-09-22-0430-utc-settled-the-class-eighty-clause-on-every-sheet-and-the-judges-still-chose-the-second-english-wording-the-pages-own-line-count-now-bounds-the-pair-fixed-in-873104d8f-shi_yumiaoya12-launched)
- [OpenRouter and the corpus passes: 2026-09-22, 00:20 to 03:50 UTC](translation-repair-openrouter-2026-09-03-2026-09-22-0020-to-0350-utc.md):
  What was launched,
  read and decided on 2026-09-22 00:20 to 03:50 UTC.
  Entries:
  shi_Yumiaoya10,
  shi_Yumiaoya11,
  shi_Yumiaoya9,
  d29feecd5,
  shi_Yumiaoya8,
  mikaela6,
  zheermao3,
  zheermao2.
  Sections:
  - [shi_Yumiaoya10 read, 2026-09-22, 03:50 UTC: SETTLED, class seventy-nine live, class eighty found (the bilingual film quote rendered twice in English because every line-structure wording said never drop a line), fixed in 8f5e28ad4; shi_Yumiaoya11 launched](translation-repair-openrouter-2026-09-03-2026-09-22-0020-to-0350-utc.md#shi_yumiaoya10-read-2026-09-22-0350-utc-settled-class-seventy-nine-live-class-eighty-found-the-bilingual-film-quote-rendered-twice-in-english-because-every-line-structure-wording-said-never-drop-a-line-fixed-in-8f5e28ad4-shi_yumiaoya11-launched)
  - [shi_Yumiaoya9 read, 2026-09-22, 02:55 UTC: SETTLED, class seventy-eight live, class seventy-nine found (a survived suicide attempt rendered as a death by the reader-protection rule's own sample sentence), fixed in d29feecd5; shi_Yumiaoya10 launched](translation-repair-openrouter-2026-09-03-2026-09-22-0020-to-0350-utc.md#shi_yumiaoya9-read-2026-09-22-0255-utc-settled-class-seventy-eight-live-class-seventy-nine-found-a-survived-suicide-attempt-rendered-as-a-death-by-the-reader-protection-rules-own-sample-sentence-fixed-in-d29feecd5-shi_yumiaoya10-launched)
  - [shi_Yumiaoya8 read, 2026-09-22, 02:20 UTC: SETTLED, class seventy-eight found (a handle the corpus declares in another entry rendered as a pinyin of nothing), fixed in 545bff7b9; shi_Yumiaoya9 launched](translation-repair-openrouter-2026-09-03-2026-09-22-0020-to-0350-utc.md#shi_yumiaoya8-read-2026-09-22-0220-utc-settled-class-seventy-eight-found-a-handle-the-corpus-declares-in-another-entry-rendered-as-a-pinyin-of-nothing-fixed-in-545bff7b9-shi_yumiaoya9-launched)
  - [mikaela6 read, 2026-09-22, 01:35 UTC: SETTLED, classes seventy to seventy-seven holding on the mikaela page, three translator additions dropped by the bench as observations; shi_Yumiaoya8 launched](translation-repair-openrouter-2026-09-03-2026-09-22-0020-to-0350-utc.md#mikaela6-read-2026-09-22-0135-utc-settled-classes-seventy-to-seventy-seven-holding-on-the-mikaela-page-three-translator-additions-dropped-by-the-bench-as-observations-shi_yumiaoya8-launched)
  - [zheermao3 read, 2026-09-22, 00:55 UTC: SETTLED, class seventy-seven live (the email label revised as a label, a bulleted-list revision refused on shape); the Arts letter revised this time from the photo transcripts; queue continues](translation-repair-openrouter-2026-09-03-2026-09-22-0020-to-0350-utc.md#zheermao3-read-2026-09-22-0055-utc-settled-class-seventy-seven-live-the-email-label-revised-as-a-label-a-bulleted-list-revision-refused-on-shape-the-arts-letter-revised-this-time-from-the-photo-transcripts-queue-continues)
  - [zheermao2 read, 2026-09-22, 00:20 UTC: SETTLED, but the archive block review replaced the email intro label with a four-block letter cut mid-sentence (class seventy-seven, fixed in 0f9cd44da); zheermao3 launched](translation-repair-openrouter-2026-09-03-2026-09-22-0020-to-0350-utc.md#zheermao2-read-2026-09-22-0020-utc-settled-but-the-archive-block-review-replaced-the-email-intro-label-with-a-four-block-letter-cut-mid-sentence-class-seventy-seven-fixed-in-0f9cd44da-zheermao3-launched)
- [OpenRouter and the corpus passes: 2026-09-23, 22:10 to 23:55 UTC](translation-repair-openrouter-2026-09-03-2026-09-23-2210-to-2355-utc.md):
  What was launched,
  read and decided on 2026-09-23 22:10 to 23:55 UTC.
  Entries:
  CuspariaKLSY9,
  d2e39e095,
  CuspariaKLSY10,
  zheermao7,
  zheermao6,
  yingying7,
  yingying6,
  shi_Yumiaoya17,
  shi_Yumiaoya16,
  shi_Yumiaoya15,
  mikaela11.
  Sections:
  - [CuspariaKLSY9 read, 2026-09-23, 23:55 UTC: SETTLED in 4 min, no hold so class one hundred three not exercised, but the polish gate refused the house tense on slice 0 (class one hundred four) and the gloss line lost a third time on slice 1 (class one hundred five); both fixed in d2e39e095, CuspariaKLSY10 launched](translation-repair-openrouter-2026-09-03-2026-09-23-2210-to-2355-utc.md#cuspariaklsy9-read-2026-09-23-2355-utc-settled-in-4-min-no-hold-so-class-one-hundred-three-not-exercised-but-the-polish-gate-refused-the-house-tense-on-slice-0-class-one-hundred-four-and-the-gloss-line-lost-a-third-time-on-slice-1-class-one-hundred-five-both-fixed-in-d2e39e095-cuspariaklsy10-launched)
  - [zheermao7 read, 2026-09-23, 23:40 UTC: SETTLED in 17 min, classes seventy-seven, ninety-six, one hundred one and one hundred two holding, every check as zheermao6; but Synthetic ran dry two minutes into the repair lane and the checker bench ran ten of twelve rounds on one voice (class one hundred three); fixed in 334c02f23, CuspariaKLSY9 launched](translation-repair-openrouter-2026-09-03-2026-09-23-2210-to-2355-utc.md#zheermao7-read-2026-09-23-2340-utc-settled-in-17-min-classes-seventy-seven-ninety-six-one-hundred-one-and-one-hundred-two-holding-every-check-as-zheermao6-but-synthetic-ran-dry-two-minutes-into-the-repair-lane-and-the-checker-bench-ran-ten-of-twelve-rounds-on-one-voice-class-one-hundred-three-fixed-in-334c02f23-cuspariaklsy9-launched)
  - [yingying7 read, 2026-09-23, 23:03 UTC: SETTLED in 19 min, every check as yingying6, classes eighty-six and ninety-seven holding, no class; Synthetic wet again with GLM-5.3-Flash at 50 s a stream; zheermao7 launched](translation-repair-openrouter-2026-09-03-2026-09-23-2210-to-2355-utc.md#yingying7-read-2026-09-23-2303-utc-settled-in-19-min-every-check-as-yingying6-classes-eighty-six-and-ninety-seven-holding-no-class-synthetic-wet-again-with-glm-53-flash-at-50-s-a-stream-zheermao7-launched)
  - [shi_Yumiaoya17 read, 2026-09-23, 22:43 UTC: SETTLED in 6 min, the closing quote as the archive's two lines, class one hundred one holding, no class; yingying7 launched](translation-repair-openrouter-2026-09-03-2026-09-23-2210-to-2355-utc.md#shi_yumiaoya17-read-2026-09-23-2243-utc-settled-in-6-min-the-closing-quote-as-the-archives-two-lines-class-one-hundred-one-holding-no-class-yingying7-launched)
  - [shi_Yumiaoya16 read, 2026-09-23, 22:45 UTC: SETTLED in 6 min, class one hundred one live, but the closing quote shipped its Chinese line beside the English because the lane offer's floor ran without the line-structured flag (class one hundred two); fixed in 6361d74d0, shi_Yumiaoya17 launched](translation-repair-openrouter-2026-09-03-2026-09-23-2210-to-2355-utc.md#shi_yumiaoya16-read-2026-09-23-2245-utc-settled-in-6-min-class-one-hundred-one-live-but-the-closing-quote-shipped-its-chinese-line-beside-the-english-because-the-lane-offers-floor-ran-without-the-line-structured-flag-class-one-hundred-two-fixed-in-6361d74d0-shi_yumiaoya17-launched)
  - [shi_Yumiaoya15 read, 2026-09-23, 22:30 UTC: TALLY ERROR twice at publish, the footnote definition's wording "not in the page" because the insertion composer cuts the two spaces its lane text ended in while the wording check searched them (class one hundred one); fixed in 314ca4b8c, shi_Yumiaoya16 launched](translation-repair-openrouter-2026-09-03-2026-09-23-2210-to-2355-utc.md#shi_yumiaoya15-read-2026-09-23-2230-utc-tally-error-twice-at-publish-the-footnote-definitions-wording-not-in-the-page-because-the-insertion-composer-cuts-the-two-spaces-its-lane-text-ended-in-while-the-wording-check-searched-them-class-one-hundred-one-fixed-in-314ca4b8c-shi_yumiaoya16-launched)
  - [mikaela11 read, 2026-09-23, 22:10 UTC: SETTLED in 11 min, the two page-assembly passes silent, classes seventy to seventy-seven and ninety-six holding, no class; shi_Yumiaoya15 launched](translation-repair-openrouter-2026-09-03-2026-09-23-2210-to-2355-utc.md#mikaela11-read-2026-09-23-2210-utc-settled-in-11-min-the-two-page-assembly-passes-silent-classes-seventy-to-seventy-seven-and-ninety-six-holding-no-class-shi_yumiaoya15-launched)
- [OpenRouter and the corpus passes: 2026-09-23, 17:46 to 22:00 UTC](translation-repair-openrouter-2026-09-03-2026-09-23-1746-to-2200-utc.md):
  What was launched,
  read and decided on 2026-09-23 17:46 to 22:00 UTC.
  Entries:
  hulicaijia16,
  mikaela11,
  XingZ631,
  XingZ630,
  XingZ629,
  yingying6,
  yingying4,
  CuspariaKLSY8.
  Entries:
  CuspariaKLSY8,
  XingZ629 to XingZ631,
  hulicaijia16,
  yingying6.
- [OpenRouter and the corpus passes: 2026-09-23, 15:45 to 17:35 UTC](translation-repair-openrouter-2026-09-03-2026-09-23-1545-to-1735-utc.md):
  What was launched,
  read and decided on 2026-09-23 15:45 to 17:35 UTC.
  Entries:
  zheermao6,
  zheermao4,
  CuspariaKLSY8,
  yingying5,
  e78515f56,
  zheermao5,
  shi_Yumiaoya14,
  mikaela10,
  mikaela9,
  c62ce70b3,
  hulicaijia15,
  hulicaijia13,
  hulicaijia14,
  b26ea1990.
  Sections:
  - [zheermao6 read, 2026-09-23, 17:35 UTC: SETTLED in 12 min on the class ninety-seven build, class ninety-seven not exercised (the bench kept the address itself), every check as zheermao4; no class, CuspariaKLSY8 launched](translation-repair-openrouter-2026-09-03-2026-09-23-1545-to-1735-utc.md#zheermao6-read-2026-09-23-1735-utc-settled-in-12-min-on-the-class-ninety-seven-build-class-ninety-seven-not-exercised-the-bench-kept-the-address-itself-every-check-as-zheermao4-no-class-cuspariaklsy8-launched)
  - [yingying5 read, 2026-09-23, 17:22 UTC: SETTLED in 6 min, classes eighty-six, ninety-two, ninety-five and ninety-six holding, but the farewell in the third person (class ninety-seven); fixed in e78515f56, zheermao5 killed, zheermao6 launched](translation-repair-openrouter-2026-09-03-2026-09-23-1545-to-1735-utc.md#yingying5-read-2026-09-23-1722-utc-settled-in-6-min-classes-eighty-six-ninety-two-ninety-five-and-ninety-six-holding-but-the-farewell-in-the-third-person-class-ninety-seven-fixed-in-e78515f56-zheermao5-killed-zheermao6-launched)
  - [shi_Yumiaoya14 read, 2026-09-23, 17:06 UTC: SETTLED in 9 min, classes seventy-eight to eighty holding on the class ninety-six build; no class, yingying5 launched](translation-repair-openrouter-2026-09-03-2026-09-23-1545-to-1735-utc.md#shi_yumiaoya14-read-2026-09-23-1706-utc-settled-in-9-min-classes-seventy-eight-to-eighty-holding-on-the-class-ninety-six-build-no-class-yingying5-launched)
  - [mikaela10 read, 2026-09-23, 16:57 UTC: SETTLED in 15 min, class ninety-six live, classes seventy to seventy-seven holding; no class, shi_Yumiaoya14 launched](translation-repair-openrouter-2026-09-03-2026-09-23-1545-to-1735-utc.md#mikaela10-read-2026-09-23-1657-utc-settled-in-15-min-class-ninety-six-live-classes-seventy-to-seventy-seven-holding-no-class-shi_yumiaoya14-launched)
  - [mikaela9 read, 2026-09-23, 16:40 UTC: SETTLED in 18 min, classes seventy to seventy-seven holding, but the mathematical-script handle's possessive shipped straight (class ninety-six, fixed in c62ce70b3); mikaela10 launched](translation-repair-openrouter-2026-09-03-2026-09-23-1545-to-1735-utc.md#mikaela9-read-2026-09-23-1640-utc-settled-in-18-min-classes-seventy-to-seventy-seven-holding-but-the-mathematical-script-handles-possessive-shipped-straight-class-ninety-six-fixed-in-c62ce70b3-mikaela10-launched)
  - [hulicaijia15 read, 2026-09-23, 16:17 UTC: SETTLED in 37 min, the relabel closed on the definitions alone, no trim or revert, every check as hulicaijia13; no class, mikaela9 launched](translation-repair-openrouter-2026-09-03-2026-09-23-1545-to-1735-utc.md#hulicaijia15-read-2026-09-23-1617-utc-settled-in-37-min-the-relabel-closed-on-the-definitions-alone-no-trim-or-revert-every-check-as-hulicaijia13-no-class-mikaela9-launched)
  - [hulicaijia14 read, 2026-09-23, 15:45 UTC: INCOMPLETE at 22 min on slice 41 after the footnote relabel stayed open (class ninety-five, fixed in b26ea1990), hulicaijia15 launched](translation-repair-openrouter-2026-09-03-2026-09-23-1545-to-1735-utc.md#hulicaijia14-read-2026-09-23-1545-utc-incomplete-at-22-min-on-slice-41-after-the-footnote-relabel-stayed-open-class-ninety-five-fixed-in-b26ea1990-hulicaijia15-launched)
- [OpenRouter and the corpus passes: 2026-09-23, 02:25 to 15:15 UTC](translation-repair-openrouter-2026-09-03-2026-09-23-0225-to-1515-utc.md):
  What was launched,
  read and decided on 2026-09-23 02:25 to 15:15 UTC.
  Entries:
  XingZ628,
  hulicaijia14,
  XingZ627,
  XingZ626,
  XingZ625,
  XingZ624,
  d8f847e3c.
  Entries:
  XingZ624 to XingZ628.
- [OpenRouter and the corpus passes: 2026-09-24, 07:40 to 23:35 UTC](translation-repair-openrouter-2026-09-03-2026-09-24-0740-to-2335-utc.md):
  What was launched,
  read and decided on 2026-09-24 07:40 to 23:35 UTC.
  Entries:
  XingZ6010,
  shi_Yumiaoya22,
  hulicaijia17,
  hulicaijia16,
  shi_Yumiaoya19,
  shi_Yumiaoya18,
  CuspariaKLSY14,
  CuspariaKLSY13,
  c796c5085,
  zheermao12,
  zheermao11,
  zheermao9,
  yingying10,
  a7b7f3934.
  Sections:
  - [XingZ6010 read, 2026-09-24, 23:35 UTC: SETTLED in 64.9 min; class one hundred twenty](translation-repair-openrouter-2026-09-03-2026-09-24-0740-to-2335-utc.md#xingz6010-read-2026-09-24-2335-utc-settled-in-649-min-class-one-hundred-twenty)
  - [shi_Yumiaoya22 read, 2026-09-24, 22:29 UTC: SETTLED in 31.9 min, class one hundred nineteen live on the page; no class](translation-repair-openrouter-2026-09-03-2026-09-24-0740-to-2335-utc.md#shi_yumiaoya22-read-2026-09-24-2229-utc-settled-in-319-min-class-one-hundred-nineteen-live-on-the-page-no-class)
  - [hulicaijia17 read, 2026-09-24, 11:00 UTC: SETTLED in 28.4 min, every hulicaijia16 check holding; no class; class one hundred nineteen built on the owner's answer about 小药娘](translation-repair-openrouter-2026-09-03-2026-09-24-0740-to-2335-utc.md#hulicaijia17-read-2026-09-24-1100-utc-settled-in-284-min-every-hulicaijia16-check-holding-no-class-class-one-hundred-nineteen-built-on-the-owners-answer-about-小药娘)
  - [shi_Yumiaoya19 read, 2026-09-24, 10:32 UTC: SETTLED in 20.9 min, every shi_Yumiaoya18 check holding; 小药娘 and 药娘 shipped in Han, the third rendering of the term in three runs (open question for the owner)](translation-repair-openrouter-2026-09-03-2026-09-24-0740-to-2335-utc.md#shi_yumiaoya19-read-2026-09-24-1032-utc-settled-in-209-min-every-shi_yumiaoya18-check-holding-小药娘-and-药娘-shipped-in-han-the-third-rendering-of-the-term-in-three-runs-open-question-for-the-owner)
  - [CuspariaKLSY14 read, 2026-09-24, 10:08 UTC: SETTLED in 21.6 min, GLM-5.3-Flash on Hyper with Synthetic wet (class one hundred eighteen live), the life list loose as the archive; no class](translation-repair-openrouter-2026-09-03-2026-09-24-0740-to-2335-utc.md#cuspariaklsy14-read-2026-09-24-1008-utc-settled-in-216-min-glm-53-flash-on-hyper-with-synthetic-wet-class-one-hundred-eighteen-live-the-life-list-loose-as-the-archive-no-class)
  - [CuspariaKLSY13 read, 2026-09-24, 09:45 UTC: SETTLED in 31.9 min with the death paragraph faithful; the archive's loose list shipped tight (class one hundred seventeen, built in c796c5085) and GLM-5.3-Flash on Synthetic cost two grace windows (class one hundred eighteen, built in 0d8e463ef)](translation-repair-openrouter-2026-09-03-2026-09-24-0740-to-2335-utc.md#cuspariaklsy13-read-2026-09-24-0945-utc-settled-in-319-min-with-the-death-paragraph-faithful-the-archives-loose-list-shipped-tight-class-one-hundred-seventeen-built-in-c796c5085-and-glm-53-flash-on-synthetic-cost-two-grace-windows-class-one-hundred-eighteen-built-in-0d8e463ef)
  - [zheermao12 read, 2026-09-24, 08:45 UTC: SETTLED in 21.6 min, class one hundred sixteen live (1 translate overrun against 22, the translate lane 62 s against 616), every zheermao11 check holding; no class](translation-repair-openrouter-2026-09-03-2026-09-24-0740-to-2335-utc.md#zheermao12-read-2026-09-24-0845-utc-settled-in-216-min-class-one-hundred-sixteen-live-1-translate-overrun-against-22-the-translate-lane-62-s-against-616-every-zheermao11-check-holding-no-class)
  - [zheermao11 read, 2026-09-24, 08:30 UTC: SETTLED in 30 min, every zheermao9 check holding, classes one hundred fourteen and fifteen silent by design; the translate lane 616 s against 77 on picture transcripts cut by the volume bound (class one hundred sixteen, built in 3bb47c918)](translation-repair-openrouter-2026-09-03-2026-09-24-0740-to-2335-utc.md#zheermao11-read-2026-09-24-0830-utc-settled-in-30-min-every-zheermao9-check-holding-classes-one-hundred-fourteen-and-fifteen-silent-by-design-the-translate-lane-616-s-against-77-on-picture-transcripts-cut-by-the-volume-bound-class-one-hundred-sixteen-built-in-3bb47c918)
  - [yingying10 read, 2026-09-24, 07:40 UTC: SETTLED with "Yingying" in the [^2] title, class one hundred fourteen live, class one hundred fifteen found and built in a7b7f3934](translation-repair-openrouter-2026-09-03-2026-09-24-0740-to-2335-utc.md#yingying10-read-2026-09-24-0740-utc-settled-with-yingying-in-the-2-title-class-one-hundred-fourteen-live-class-one-hundred-fifteen-found-and-built-in-a7b7f3934)
- [OpenRouter and the corpus passes: 2026-09-24, 03:45 to 07:15 UTC](translation-repair-openrouter-2026-09-03-2026-09-24-0345-to-0715-utc.md):
  What was launched,
  read and decided on 2026-09-24 03:45 to 07:15 UTC.
  Entries:
  yingying9,
  mikaela16,
  mikaela15,
  a43c5d88d,
  mikaela14,
  caac222a6,
  mikaela13,
  mikaela12,
  shi_Yumiaoya18,
  shi_Yumiaoya17.
  Sections:
  - [yingying9 read, 2026-09-24, 07:15 UTC: SETTLED with the [^2] title in the archive's "Sakura", class one hundred fourteen found and built in 645ed9d62](translation-repair-openrouter-2026-09-03-2026-09-24-0345-to-0715-utc.md#yingying9-read-2026-09-24-0715-utc-settled-with-the-2-title-in-the-archives-sakura-class-one-hundred-fourteen-found-and-built-in-645ed9d62)
  - [mikaela16 read, 2026-09-24, 06:50 UTC: SETTLED with the HRT passage once, class one hundred twelve not exercised, class one hundred thirteen found and built in 375a9b3e8; yingying9 launched](translation-repair-openrouter-2026-09-03-2026-09-24-0345-to-0715-utc.md#mikaela16-read-2026-09-24-0650-utc-settled-with-the-hrt-passage-once-class-one-hundred-twelve-not-exercised-class-one-hundred-thirteen-found-and-built-in-375a9b3e8-yingying9-launched)
  - [mikaela15 read, 2026-09-24, 05:45 UTC: SETTLED with the fold live, the HRT passage shipped twice, class one hundred twelve found and built in a43c5d88d; mikaela16 launched](translation-repair-openrouter-2026-09-03-2026-09-24-0345-to-0715-utc.md#mikaela15-read-2026-09-24-0545-utc-settled-with-the-fold-live-the-hrt-passage-shipped-twice-class-one-hundred-twelve-found-and-built-in-a43c5d88d-mikaela16-launched)
  - [mikaela14 read, 2026-09-24, 05:15 UTC: class one hundred eleven live at the admission, its chain arm found and built in caac222a6; killed at 05:05 UTC; mikaela15 launched](translation-repair-openrouter-2026-09-03-2026-09-24-0345-to-0715-utc.md#mikaela14-read-2026-09-24-0515-utc-class-one-hundred-eleven-live-at-the-admission-its-chain-arm-found-and-built-in-caac222a6-killed-at-0505-utc-mikaela15-launched)
  - [mikaela13 read, 2026-09-24, 04:50 UTC: INCOMPLETE carried-evidence-lost in 15 min again, class one hundred eleven (the carried passage rendered across both neighbours), fixed in 0ab0abdd8; mikaela14 launched](translation-repair-openrouter-2026-09-03-2026-09-24-0345-to-0715-utc.md#mikaela13-read-2026-09-24-0450-utc-incomplete-carried-evidence-lost-in-15-min-again-class-one-hundred-eleven-the-carried-passage-rendered-across-both-neighbours-fixed-in-0ab0abdd8-mikaela14-launched)
  - [mikaela12 read, 2026-09-24, 04:20 UTC: INCOMPLETE carried-evidence-lost in 15 min, class one hundred ten (a carried passage inside the neighbour's archive span), fixed in 457765221; mikaela13 launched](translation-repair-openrouter-2026-09-03-2026-09-24-0345-to-0715-utc.md#mikaela12-read-2026-09-24-0420-utc-incomplete-carried-evidence-lost-in-15-min-class-one-hundred-ten-a-carried-passage-inside-the-neighbours-archive-span-fixed-in-457765221-mikaela13-launched)
  - [shi_Yumiaoya18 read, 2026-09-24, 03:45 UTC: SETTLED in 7 min, the first page on class one hundred nine (not exercised: no hold began inside the lane), every check as shi_Yumiaoya17, no class; mikaela12 launched](translation-repair-openrouter-2026-09-03-2026-09-24-0345-to-0715-utc.md#shi_yumiaoya18-read-2026-09-24-0345-utc-settled-in-7-min-the-first-page-on-class-one-hundred-nine-not-exercised-no-hold-began-inside-the-lane-every-check-as-shi_yumiaoya17-no-class-mikaela12-launched)
- [OpenRouter and the corpus passes: 2026-09-24, 00:15 to 03:20 UTC](translation-repair-openrouter-2026-09-03-2026-09-24-0015-to-0320-utc.md):
  What was launched,
  read and decided on 2026-09-24 00:15 to 03:20 UTC.
  Entries:
  zheermao9,
  shi_Yumiaoya18,
  CuspariaKLSY12,
  CuspariaKLSY11,
  yingying8,
  yingying7,
  zheermao8,
  zheermao7,
  CuspariaKLSY10.
  Entries:
  CuspariaKLSY10 to CuspariaKLSY12,
  yingying8,
  zheermao8 to zheermao9.
- [OpenRouter and the corpus passes: 2026-09-25, 06:30 to 08:50 UTC](translation-repair-openrouter-2026-09-03-2026-09-25-0630-to-0850-utc.md):
  What was launched,
  read and decided on 2026-09-25 06:30 to 08:50 UTC.
  Entries:
  hulicaijia20,
  shi_Yumiaoya35,
  hulicaijia19,
  shi_Yumiaoya34,
  shi_Yumiaoya32,
  shi_Yumiaoya31,
  shi_Yumiaoya30.
  Sections:
  - [hulicaijia20 read, 2026-09-25: class one hundred thirty-four live; douhua, a name's casing and a pinyin tone (classes one hundred thirty-five to one hundred thirty-seven)](translation-repair-openrouter-2026-09-03-2026-09-25-0630-to-0850-utc.md#hulicaijia20-read-2026-09-25-class-one-hundred-thirty-four-live-douhua-a-names-casing-and-a-pinyin-tone-classes-one-hundred-thirty-five-to-one-hundred-thirty-seven)
  - [shi_Yumiaoya35 and hulicaijia19 read, 2026-09-25: class one hundred thirty-three live, Canadian dates and spellings on untouched slices (class one hundred thirty-four)](translation-repair-openrouter-2026-09-03-2026-09-25-0630-to-0850-utc.md#shi_yumiaoya35-and-hulicaijia19-read-2026-09-25-class-one-hundred-thirty-three-live-canadian-dates-and-spellings-on-untouched-slices-class-one-hundred-thirty-four)
  - [shi_Yumiaoya34 read, 2026-09-25, 08:50 UTC: classes one hundred thirty-one and thirty-two live, two life-event calques (class one hundred thirty-three)](translation-repair-openrouter-2026-09-03-2026-09-25-0630-to-0850-utc.md#shi_yumiaoya34-read-2026-09-25-0850-utc-classes-one-hundred-thirty-one-and-thirty-two-live-two-life-event-calques-class-one-hundred-thirty-three)
  - [Canadian English, 2026-09-25, 08:17 UTC: class one hundred thirty-two](translation-repair-openrouter-2026-09-03-2026-09-25-0630-to-0850-utc.md#canadian-english-2026-09-25-0817-utc-class-one-hundred-thirty-two)
  - [shi_Yumiaoya32 read, 2026-09-25, 08:05 UTC: class one hundred thirty live, seven wording calques (class one hundred thirty-one)](translation-repair-openrouter-2026-09-03-2026-09-25-0630-to-0850-utc.md#shi_yumiaoya32-read-2026-09-25-0805-utc-class-one-hundred-thirty-live-seven-wording-calques-class-one-hundred-thirty-one)
  - [shi_Yumiaoya31 read, 2026-09-25, 07:35 UTC: class one hundred twenty-nine live, nine idiom calques (class one hundred thirty)](translation-repair-openrouter-2026-09-03-2026-09-25-0630-to-0850-utc.md#shi_yumiaoya31-read-2026-09-25-0735-utc-class-one-hundred-twenty-nine-live-nine-idiom-calques-class-one-hundred-thirty)
  - [shi_Yumiaoya30 read, 2026-09-25, 06:55 UTC: classes one hundred twenty-six to twenty-eight live, five phrasing calques (class one hundred twenty-nine)](translation-repair-openrouter-2026-09-03-2026-09-25-0630-to-0850-utc.md#shi_yumiaoya30-read-2026-09-25-0655-utc-classes-one-hundred-twenty-six-to-twenty-eight-live-five-phrasing-calques-class-one-hundred-twenty-nine)
  - [The page's register, 2026-09-25, 06:30 UTC: plain written English on every sheet (class one hundred twenty-eight)](translation-repair-openrouter-2026-09-03-2026-09-25-0630-to-0850-utc.md#the-pages-register-2026-09-25-0630-utc-plain-written-english-on-every-sheet-class-one-hundred-twenty-eight)
- [OpenRouter and the corpus passes: 2026-09-25, 00:22 to 06:00 UTC](translation-repair-openrouter-2026-09-03-2026-09-25-0022-to-0600-utc.md):
  What was launched,
  read and decided on 2026-09-25 00:22 to 06:00 UTC.
  Entries:
  shi_Yumiaoya27,
  shi_Yumiaoya26,
  shi_Yumiaoya25,
  shi_Yumiaoya24,
  shi_Yumiaoya23,
  zheermao13,
  yingying11,
  mikaela17.
  Sections:
  - [shi_Yumiaoya27 read, 2026-09-25, 06:00 UTC: classes one hundred twenty-two to twenty-five live, five more renderings (class one hundred twenty-six)](translation-repair-openrouter-2026-09-03-2026-09-25-0022-to-0600-utc.md#shi_yumiaoya27-read-2026-09-25-0600-utc-classes-one-hundred-twenty-two-to-twenty-five-live-five-more-renderings-class-one-hundred-twenty-six)
  - [shi_Yumiaoya26 void, 2026-09-25, 05:30 UTC: the freeze nested the build, so the pass ran a stale one](translation-repair-openrouter-2026-09-03-2026-09-25-0022-to-0600-utc.md#shi_yumiaoya26-void-2026-09-25-0530-utc-the-freeze-nested-the-build-so-the-pass-ran-a-stale-one)
  - [shi_Yumiaoya25 read, 2026-09-25, 03:30 UTC: class one hundred twenty-four live, two calques on the page (class one hundred twenty-five)](translation-repair-openrouter-2026-09-03-2026-09-25-0022-to-0600-utc.md#shi_yumiaoya25-read-2026-09-25-0330-utc-class-one-hundred-twenty-four-live-two-calques-on-the-page-class-one-hundred-twenty-five)
  - [shi_Yumiaoya24 read, 2026-09-25, 03:05 UTC: classes one hundred twenty-two and twenty-three live, a survived attempt's method on the page (class one hundred twenty-four)](translation-repair-openrouter-2026-09-03-2026-09-25-0022-to-0600-utc.md#shi_yumiaoya24-read-2026-09-25-0305-utc-classes-one-hundred-twenty-two-and-twenty-three-live-a-survived-attempts-method-on-the-page-class-one-hundred-twenty-four)
  - [Classes one hundred twenty-two and one hundred twenty-three, 2026-09-25, 02:40 UTC: the owner's reading of shi_Yumiaoya23](translation-repair-openrouter-2026-09-03-2026-09-25-0022-to-0600-utc.md#classes-one-hundred-twenty-two-and-one-hundred-twenty-three-2026-09-25-0240-utc-the-owners-reading-of-shi_yumiaoya23)
  - [shi_Yumiaoya23 read, 2026-09-25, 01:49 UTC: SETTLED in 20.9 min; the sedative detail and two calques on the page (classes one hundred twenty-two and twenty-three, found by the owner)](translation-repair-openrouter-2026-09-03-2026-09-25-0022-to-0600-utc.md#shi_yumiaoya23-read-2026-09-25-0149-utc-settled-in-209-min-the-sedative-detail-and-two-calques-on-the-page-classes-one-hundred-twenty-two-and-twenty-three-found-by-the-owner)
  - [zheermao13 read, 2026-09-25, 01:08 UTC: SETTLED in 21.0 min; no class](translation-repair-openrouter-2026-09-03-2026-09-25-0022-to-0600-utc.md#zheermao13-read-2026-09-25-0108-utc-settled-in-210-min-no-class)
  - [yingying11 read, 2026-09-25, 00:46 UTC: SETTLED in 15.4 min; no class](translation-repair-openrouter-2026-09-03-2026-09-25-0022-to-0600-utc.md#yingying11-read-2026-09-25-0046-utc-settled-in-154-min-no-class)
  - [mikaela17 read, 2026-09-25, 00:22 UTC: SETTLED in 40.8 min; class one hundred twenty-one](translation-repair-openrouter-2026-09-03-2026-09-25-0022-to-0600-utc.md#mikaela17-read-2026-09-25-0022-utc-settled-in-408-min-class-one-hundred-twenty-one)
- [OpenRouter and the corpus passes: 2026-09-26, classes 160 to 178](translation-repair-openrouter-2026-09-03-2026-09-26-classes-160-to-178.md):
  What was launched,
  read and decided on 2026-09-26 classes 160 to 178.
  Entries:
  TianqiChen66610,
  TianqiChen6669,
  TianqiChen6668,
  TianqiChen6667,
  TianqiChen6665,
  TianqiChen6664,
  TianqiChen6663,
  TianqiChen6662.
  Entries:
  TianqiChen6662 to TianqiChen6665,
  TianqiChen6667 to TianqiChen6669,
  TianqiChen66610.
- [OpenRouter and the corpus passes: 2026-09-26, classes 150 to 159](translation-repair-openrouter-2026-09-03-2026-09-26-classes-150-to-159.md):
  What was launched,
  read and decided on 2026-09-26 classes 150 to 159.
  Entries:
  Xu_Yushu1,
  aiyysk4,
  aiyysk2,
  aiyysk1,
  shihai4h2,
  shihai4h1,
  shi_Yumiaoya39,
  shi_Yumiaoya38,
  XingZ6014,
  yingying12,
  shi_Yumiaoya37,
  shi_Yumiaoya36.
  Sections:
  - [Xu_Yushu1 read, 2026-09-26: INCOMPLETE at the pictures phase, the padded-reading log fixed](translation-repair-openrouter-2026-09-03-2026-09-26-classes-150-to-159.md#xu_yushu1-read-2026-09-26-incomplete-at-the-pictures-phase-the-padded-reading-log-fixed)
  - [aiyysk4 read, 2026-09-26: classes one hundred fifty-eight and fifty-nine live](translation-repair-openrouter-2026-09-03-2026-09-26-classes-150-to-159.md#aiyysk4-read-2026-09-26-classes-one-hundred-fifty-eight-and-fifty-nine-live)
  - [aiyysk2 read, 2026-09-26: class one hundred fifty-seven live, class one hundred fifty-eight](translation-repair-openrouter-2026-09-03-2026-09-26-classes-150-to-159.md#aiyysk2-read-2026-09-26-class-one-hundred-fifty-seven-live-class-one-hundred-fifty-eight)
  - [aiyysk1 read, 2026-09-26: class one hundred fifty-six against a correct archive, class one hundred fifty-seven](translation-repair-openrouter-2026-09-03-2026-09-26-classes-150-to-159.md#aiyysk1-read-2026-09-26-class-one-hundred-fifty-six-against-a-correct-archive-class-one-hundred-fifty-seven)
  - [shihai4h2 read, 2026-09-26: class one hundred fifty-six live](translation-repair-openrouter-2026-09-03-2026-09-26-classes-150-to-159.md#shihai4h2-read-2026-09-26-class-one-hundred-fifty-six-live)
  - [shihai4h1 read, 2026-09-26: class one hundred fifty-four against a correct archive, class one hundred fifty-six](translation-repair-openrouter-2026-09-03-2026-09-26-classes-150-to-159.md#shihai4h1-read-2026-09-26-class-one-hundred-fifty-four-against-a-correct-archive-class-one-hundred-fifty-six)
  - [shi_Yumiaoya39 read, 2026-09-26: classes one hundred fifty-four and fifty-five live](translation-repair-openrouter-2026-09-03-2026-09-26-classes-150-to-159.md#shi_yumiaoya39-read-2026-09-26-classes-one-hundred-fifty-four-and-fifty-five-live)
  - [shi_Yumiaoya38 read, 2026-09-26: class one hundred fifty-one live, classes one hundred fifty-four and fifty-five](translation-repair-openrouter-2026-09-03-2026-09-26-classes-150-to-159.md#shi_yumiaoya38-read-2026-09-26-class-one-hundred-fifty-one-live-classes-one-hundred-fifty-four-and-fifty-five)
  - [XingZ6014 read, 2026-09-26: class one hundred fifty-two live, class one hundred fifty-three](translation-repair-openrouter-2026-09-03-2026-09-26-classes-150-to-159.md#xingz6014-read-2026-09-26-class-one-hundred-fifty-two-live-class-one-hundred-fifty-three)
  - [yingying12 read, 2026-09-26: class one hundred fifty-two](translation-repair-openrouter-2026-09-03-2026-09-26-classes-150-to-159.md#yingying12-read-2026-09-26-class-one-hundred-fifty-two)
  - [shi_Yumiaoya37 read, 2026-09-26: the attempt said, and class one hundred fifty-one](translation-repair-openrouter-2026-09-03-2026-09-26-classes-150-to-159.md#shi_yumiaoya37-read-2026-09-26-the-attempt-said-and-class-one-hundred-fifty-one)
  - [shi_Yumiaoya36 read, 2026-09-26: class one hundred fifty](translation-repair-openrouter-2026-09-03-2026-09-26-classes-150-to-159.md#shi_yumiaoya36-read-2026-09-26-class-one-hundred-fifty)
- [OpenRouter and the corpus passes: 2026-09-26, classes 138 to 149](translation-repair-openrouter-2026-09-03-2026-09-26-classes-138-to-149.md):
  What was launched,
  read and decided on 2026-09-26 classes 138 to 149.
  Entries:
  hulicaijia31,
  hulicaijia30,
  hulicaijia29,
  XingZ6013,
  hulicaijia28,
  XingZ6012,
  XingZ6011.
  Sections:
  - [hulicaijia31 read, 2026-09-26: class one hundred forty-nine not exercised, and the en_CA scan corrected](translation-repair-openrouter-2026-09-03-2026-09-26-classes-138-to-149.md#hulicaijia31-read-2026-09-26-class-one-hundred-forty-nine-not-exercised-and-the-en_ca-scan-corrected)
  - [hulicaijia30 read, 2026-09-26: class one hundred forty-eight quiet, and class one hundred forty-nine](translation-repair-openrouter-2026-09-03-2026-09-26-classes-138-to-149.md#hulicaijia30-read-2026-09-26-class-one-hundred-forty-eight-quiet-and-class-one-hundred-forty-nine)
  - [hulicaijia29 read, 2026-09-26: class one hundred forty-four not exercised, and class one hundred forty-eight](translation-repair-openrouter-2026-09-03-2026-09-26-classes-138-to-149.md#hulicaijia29-read-2026-09-26-class-one-hundred-forty-four-not-exercised-and-class-one-hundred-forty-eight)
  - [XingZ6013 read, 2026-09-26: classes 139 to 142 live, and classes 145 to 147](translation-repair-openrouter-2026-09-03-2026-09-26-classes-138-to-149.md#xingz6013-read-2026-09-26-classes-139-to-142-live-and-classes-145-to-147)
  - [hulicaijia28 read, 2026-09-26: the note kept, class one hundred forty-four not exercised](translation-repair-openrouter-2026-09-03-2026-09-26-classes-138-to-149.md#hulicaijia28-read-2026-09-26-the-note-kept-class-one-hundred-forty-four-not-exercised)
  - [Class one hundred forty-four, 2026-09-26: one panel's acceptance made final on every later sheet](translation-repair-openrouter-2026-09-03-2026-09-26-classes-138-to-149.md#class-one-hundred-forty-four-2026-09-26-one-panels-acceptance-made-final-on-every-later-sheet)
  - [Class one hundred forty-three, 2026-09-26: a translator note on wordplay voted out as an addition, and who voted](translation-repair-openrouter-2026-09-03-2026-09-26-classes-138-to-149.md#class-one-hundred-forty-three-2026-09-26-a-translator-note-on-wordplay-voted-out-as-an-addition-and-who-voted)
  - [Class one hundred forty-two, 2026-09-26: straight quotes on archive paragraphs of a page written curly](translation-repair-openrouter-2026-09-03-2026-09-26-classes-138-to-149.md#class-one-hundred-forty-two-2026-09-26-straight-quotes-on-archive-paragraphs-of-a-page-written-curly)
  - [Class one hundred forty-one, 2026-09-26: an English title left in the Chinese title marks 《》](translation-repair-openrouter-2026-09-03-2026-09-26-classes-138-to-149.md#class-one-hundred-forty-one-2026-09-26-an-english-title-left-in-the-chinese-title-marks-)
  - [XingZ6012 read, 2026-09-26: a bracketed album read into the signer's name, and a credit's 作者 as "author" (classes one hundred thirty-nine and one hundred forty)](translation-repair-openrouter-2026-09-03-2026-09-26-classes-138-to-149.md#xingz6012-read-2026-09-26-a-bracketed-album-read-into-the-signers-name-and-a-credits-作者-as-author-classes-one-hundred-thirty-nine-and-one-hundred-forty)
  - [XingZ6011 read, 2026-09-26: a source credit read as a signature (class one hundred thirty-eight)](translation-repair-openrouter-2026-09-03-2026-09-26-classes-138-to-149.md#xingz6011-read-2026-09-26-a-source-credit-read-as-a-signature-class-one-hundred-thirty-eight)
- [OpenRouter and the corpus passes: 2026-09-27 to 2026-09-28](translation-repair-openrouter-2026-09-03-2026-09-27-to-2026-09-28.md):
  What was launched,
  read and decided on 2026-09-27 to 2026-09-28.
  Entries:
  TianqiChen66619,
  TianqiChen66616,
  TianqiChen66614,
  TianqiChen66613,
  TianqiChen66612,
  TianqiChen66611,
  TianqiChen66621.
  Sections:
  - [Whole-package audit, 2026-09-27: class one hundred eighty-six onward](translation-repair-openrouter-2026-09-03-2026-09-27-to-2026-09-28.md#whole-package-audit-2026-09-27-class-one-hundred-eighty-six-onward)
  - [TianqiChen66619 read, 2026-09-27: INCOMPLETE at slice 9, class one hundred eighty-five](translation-repair-openrouter-2026-09-03-2026-09-27-to-2026-09-28.md#tianqichen66619-read-2026-09-27-incomplete-at-slice-9-class-one-hundred-eighty-five)
  - [Glossary audit and TianqiChen66616 read, 2026-09-27: class one hundred eighty-four](translation-repair-openrouter-2026-09-03-2026-09-27-to-2026-09-28.md#glossary-audit-and-tianqichen66616-read-2026-09-27-class-one-hundred-eighty-four)
  - [TianqiChen66614 read, 2026-09-27: SETTLED, class one hundred eighty-two](translation-repair-openrouter-2026-09-03-2026-09-27-to-2026-09-28.md#tianqichen66614-read-2026-09-27-settled-class-one-hundred-eighty-two)
  - [TianqiChen66613 read, 2026-09-27: SETTLED, classes one hundred eighty and eighty-one](translation-repair-openrouter-2026-09-03-2026-09-27-to-2026-09-28.md#tianqichen66613-read-2026-09-27-settled-classes-one-hundred-eighty-and-eighty-one)
  - [TianqiChen66612 killed and TianqiChen66613 launched on class one hundred seventy-nine, 2026-09-27](translation-repair-openrouter-2026-09-03-2026-09-27-to-2026-09-28.md#tianqichen66612-killed-and-tianqichen66613-launched-on-class-one-hundred-seventy-nine-2026-09-27)
  - [TianqiChen66611 read, 2026-09-27: SETTLED, quote 2 twice, class one hundred seventy-nine](translation-repair-openrouter-2026-09-03-2026-09-27-to-2026-09-28.md#tianqichen66611-read-2026-09-27-settled-quote-2-twice-class-one-hundred-seventy-nine)
  - [TianqiChen66621 read, 2026-09-28: SETTLED, class one hundred eighty-seven](translation-repair-openrouter-2026-09-03-2026-09-27-to-2026-09-28.md#tianqichen66621-read-2026-09-28-settled-class-one-hundred-eighty-seven)
- [OpenRouter and the corpus passes: 2026-10-11 to 2026-10-12](translation-repair-openrouter-2026-09-03-2026-10-11-to-2026-10-12.md):
  What was launched,
  read and decided on 2026-10-11 to 2026-10-12.
  Entries:
  TianqiChen66622,
  TianqiChen66623.
- [OpenRouter and the corpus passes: 2026-10-15](translation-repair-openrouter-2026-09-03-2026-10-15.md):
  What was launched,
  read and decided on 2026-10-15.
  Entries:
  TianqiChen66624,
  TianqiChen66625.
- [OpenRouter and the corpus passes: 2026-10-18](translation-repair-openrouter-2026-09-03-2026-10-18.md):
  What was launched,
  read and decided on 2026-10-18.
  Entries:
  TianqiChen66626.
- [OpenRouter and the corpus passes: 2026-10-19](translation-repair-openrouter-2026-09-03-2026-10-19.md):
  What was launched,
  read and decided on 2026-10-19.
  Entries:
  zheermao14,
  CuspariaKLSY16,
  CuspariaKLSY17,
  XingZ6015.
- [OpenRouter and the corpus passes: 2026-10-20](translation-repair-openrouter-2026-09-03-2026-10-20.md):
  What was read and decided on 2026-10-20.
  Entries:
  XingZ6015,
  zheermao15,
  CuspariaKLSY18,
  zheermao16,
  CuspariaKLSY19,
  zheermao17,
  zheermao19.
- [OpenRouter and the corpus passes: 2026-10-21](translation-repair-openrouter-2026-09-03-2026-10-21.md):
  What was read and decided on 2026-10-21.
  Entries:
  zheermao20,
  CuspariaKLSY20.
- [OpenRouter and the corpus passes: the rotation restarted on build 3637592bd4 (2026-10-21)](translation-repair-openrouter-2026-09-03-2026-10-21-rotation-restart.md):
  What was read on 2026-10-21 of the first three passes after the rotation restarted;
  two entries settled and the third stopped on a picture only one model had read.
  Entries:
  zheermao21,
  CuspariaKLSY21,
  XingZ6016.
  Files:
  - [The run of zheermao21](translation-repair-openrouter-2026-09-03-2026-10-21-rotation-restart-zheermao21-run.md)
  - [The page of zheermao21](translation-repair-openrouter-2026-09-03-2026-10-21-rotation-restart-zheermao21-page.md)
  - [The run of CuspariaKLSY21](translation-repair-openrouter-2026-09-03-2026-10-21-rotation-restart-cuspariaklsy21-run.md)
  - [The page of CuspariaKLSY21](translation-repair-openrouter-2026-09-03-2026-10-21-rotation-restart-cuspariaklsy21-page.md)
  - [The XingZ6016 picture stop](translation-repair-openrouter-2026-09-03-2026-10-21-rotation-restart-xingz6016.md)
  - [The causes of the rotation 21 findings](translation-repair-openrouter-2026-09-03-2026-10-21-rotation-restart-causes.md)
- [OpenRouter and the corpus passes: the rotation restarted on build 2a336603bf after the Gemma readers were lifted (2026-10-21)](translation-repair-openrouter-2026-09-03-2026-10-21-gemma-lift-restart.md):
  What was read on 2026-10-21 and 2026-10-22 of the first passes after the rotation restarted a second time;
  one entry settled with its letter regressed,
  one settled with a mixed page,
  and one settled after 268.7 min with three providers dry in sequence.
  Entries:
  zheermao22,
  CuspariaKLSY22,
  XingZ6017.
  Files:
  - [The run of zheermao22](translation-repair-openrouter-2026-09-03-2026-10-21-gemma-lift-restart-zheermao22-run.md)
  - [The letter of zheermao22](translation-repair-openrouter-2026-09-03-2026-10-21-gemma-lift-restart-zheermao22-letter.md)
  - [The page of zheermao22](translation-repair-openrouter-2026-09-03-2026-10-21-gemma-lift-restart-zheermao22-page.md)
  - [The run of CuspariaKLSY22](translation-repair-openrouter-2026-09-03-2026-10-21-gemma-lift-restart-cuspariaklsy22-run.md)
  - [The providers of CuspariaKLSY22](translation-repair-openrouter-2026-09-03-2026-10-21-gemma-lift-restart-cuspariaklsy22-providers.md)
  - [The page of CuspariaKLSY22](translation-repair-openrouter-2026-09-03-2026-10-21-gemma-lift-restart-cuspariaklsy22-page.md)
  - [The run of XingZ6017](translation-repair-openrouter-2026-09-03-2026-10-21-gemma-lift-restart-xingz6017-run.md)
  - [The providers of XingZ6017](translation-repair-openrouter-2026-09-03-2026-10-21-gemma-lift-restart-xingz6017-providers.md)
  - [The page of XingZ6017](translation-repair-openrouter-2026-09-03-2026-10-21-gemma-lift-restart-xingz6017-page.md)
- [OpenRouter and the corpus passes: the rotation on build 8864316eb2 after the owner's answers (2026-10-22)](translation-repair-openrouter-2026-09-03-2026-10-22-answers-restart.md):
  What was read on 2026-10-22 of the passes on the build that landed the owner's answers of that day,
  and of the two passes on the build after it;
  zheermao23 settled with NeuralWatt dry and the Mercury window never reached,
  CuspariaKLSY23 settled in 17.1 min with Mercury clean and tense left open by the repair lane again,
  XingZ6018 ended after 62.4 min with no page while every provider ran dry,
  XingZ6019 ended after 19.2 min on a short writing bench with NeuralWatt dry,
  XingZ6020 was relaunched,
  and XingZ6021 settled in 2 h 5 min 45 s on the build `bcff23b8ce` with the Shinonome section shipping archive text
  that the source does not contain.
  Entries:
  zheermao23,
  CuspariaKLSY23,
  XingZ6018,
  XingZ6019,
  XingZ6020,
  XingZ6021.
  Files:
  - [The run of zheermao23](translation-repair-openrouter-2026-09-03-2026-10-22-answers-restart-zheermao23-run.md)
  - [The stages of zheermao23](translation-repair-openrouter-2026-09-03-2026-10-22-answers-restart-zheermao23-stages.md)
  - [The evidence of zheermao23](translation-repair-openrouter-2026-09-03-2026-10-22-answers-restart-zheermao23-evidence.md)
  - [The run of CuspariaKLSY23](translation-repair-openrouter-2026-09-03-2026-10-22-answers-restart-cuspariaklsy23-run.md)
  - [The stages of CuspariaKLSY23](translation-repair-openrouter-2026-09-03-2026-10-22-answers-restart-cuspariaklsy23-stages.md)
  - [The evidence of CuspariaKLSY23](translation-repair-openrouter-2026-09-03-2026-10-22-answers-restart-cuspariaklsy23-evidence.md)
  - [The run of XingZ6018](translation-repair-openrouter-2026-09-03-2026-10-22-answers-restart-xingz6018-run.md)
  - [The providers of XingZ6018](translation-repair-openrouter-2026-09-03-2026-10-22-answers-restart-xingz6018-providers.md)
  - [The defects of XingZ6018](translation-repair-openrouter-2026-09-03-2026-10-22-answers-restart-xingz6018-defects.md)
  - [XingZ6019 and XingZ6020](translation-repair-openrouter-2026-09-03-2026-10-22-answers-restart-xingz6019.md)
  - [The run of XingZ6021](translation-repair-openrouter-2026-09-03-2026-10-22-answers-restart-xingz6021-run.md)
  - [The stages of XingZ6021](translation-repair-openrouter-2026-09-03-2026-10-22-answers-restart-xingz6021-stages.md)
  - [The evidence of XingZ6021](translation-repair-openrouter-2026-09-03-2026-10-22-answers-restart-xingz6021-evidence.md)
  - [The defects of XingZ6021](translation-repair-openrouter-2026-09-03-2026-10-22-answers-restart-xingz6021-defects.md)
