# Translation repair handover 2026-09-06: the takeover of 2026-09-16 and the passes to 2026-09-19

The checkpoint written when the owner took the work back on 2026-09-16,
with the teardown of the provider-free layer,
then every pass from Mio13 to XingZ619 in order,
each with the class it found and the fix,
and the model-card work built on the owner's request the same afternoon.

Part of [Translation repair handover snapshot: 2026-09-06](translation-repair-handover-2026-09-06.md),
which lists every file of this record.

## 2026-09-16, 02:00 UTC: the provider-free layer is torn down and the queue resumes

The owner took the work back from the 2026-09-10 to 2026-09-15 sessions on 2026-09-16 with
"full authorization to fix/teardown whatever GPT did and use funds using your own reasonable judgement".
What those sessions left is read from the git log and the import graph,
not from their transcript,
which the takeover session does not have.

- Passes stopped after `Mio12` (2026-09-10,
  03:45 UTC).
  Nothing was launched from 2026-09-10 to 2026-09-15 except the V4.1 Flash judge calibration of 2026-09-11.
- 282 source files were added to the package since `40aba2fdb`.
  Measured by import closure from `corpus-pass.ts` and every build entry that existed at the handoff,
  41 of them are on the pass path and stay:
  the footnote relabel family,
  source-only breaks and the source display,
  the naturalness quorum,
  the name-form policy,
  the block-pairing protocol and question key,
  archive block selection evidence,
  the rendered-break prompt,
  the panel stage,
  and the picture support the archive review reads (class twenty-five).
  143 production modules were reached by no pass or probe entry,
  only by `index.ts` and their own tests:
  the archive-naming family,
  the `producer-input-*` container runner,
  the preparation attempt,
  receipt,
  root and selection records,
  the persisted-input reader ("task63") and request capture.
  `cbedea357` removes them with their 47 tests and fixtures,
  the three build entries,
  the bootstrap and sealed rolldown configs and the seal tasks.
  Build,
  oxlint,
  types and the unit suite pass on the result.
  The decision record is
  [the teardown decision](../decision/translation-repair-provider-free-layer-teardown-2026-09-16.md).
- The task-numbered plan of those sessions (items 31 to 67 of their own tracker:
  "no paid model invocation",
  finite plans,
  journals,
  DTO readers) is abandoned.
  Its records stay in `doc/planning/` under an abandonment note;
  no step of it is next.
  The sections from "Latest checkpoint:
  V4.1 judge admission on 2026-09-11"
  to "Repository state" are those sessions' own record,
  kept as written:
  a number there,
  after the word "task" or alone as in "35 remains blocked by 38",
  names an item of their tracker,
  described in the planning record the section links,
  and is not a GitHub issue.
- Not removed,
  for want of permission in the takeover session:
  171 stopped `podman` containers named `preparation-owner-*` and `comparison-*` on the
  `monochromatic/terminal` image,
  and the uncommitted evidence directories under `node_modules/.monochromatic/`
  (`comparison-evidence` at 1.8 GB,
  `preparation-input-reader`,
  `preparation-root-order`,
  `preparation-root-order-retention`).
  Both are safe to delete.
- What those sessions did land on the pass path,
  and what it was verified on:
  classes twenty-five,
  twenty-seven,
  twenty-eight,
  twenty-nine and thirty,
  each read on a Mio page (Mio8 to Mio12);
  V4.1 Flash seated as a judge (`83e632127`,
  2026-09-11,
  14 of 14 references chosen) and still in `WRITER_UNMEASURED`.
  The changes after Mio12 (the name-form policy and the archive-review wiring of the 2026-09-10 daytime)
  ran in bounded probes and never in a full pass.
- `Mio13` launched at 01:58 UTC on frozen `cbedea357` (pid 1001520,
  runs `~/temp/agent/Mio13-20260916`,
  log `~/temp/agent/Mio13-20260916.log`),
  the first full pass since Mio12 and the first with V4.1 Flash judging.
  Meters at launch:
  Synthetic 76.3 percent of its week,
  Hyper 171 credits,
  Bedrock 185.38 USD,
  OpenRouter 267.73 USD.
  Roster 12,
  translators 8,
  readers 6,
  writers 11.
- `Mio13` ended INCOMPLETE at 02:42 UTC,
  `provider-unavailable` from the archive block review of the first chat translation with every provider
  wet:
  seven seats spent their whole completion cap reasoning and sent no content,
  Mercury answered with an empty source quote,
  deepseek-v4-pro was cut in the recovery round's grace window,
  and the four heard were read as "4/12".
  Class thirty-one:
  a cap-cut reply is an answer nobody could read,
  not silence.
  Guards in `19ad9d238`,
  fix in `db090b955`:
  the gather names its `unreadable` seats and the review throws only when heard plus unreadable is below
  the reachable quorum.
  The reading is in the pass log under the "Mio13 read" heading dated 2026-09-16 02:55 UTC.
  The overrun itself (seven of twelve seats cannot review that block within their measured caps) is not a
  defect class;
  the levers (the caps,
  the six-transcript picture support) are design questions put to the owner in "What to do next".
- `Mio14` launched at 03:04 UTC on frozen `db090b955` (pid 1045264,
  runs `~/temp/agent/Mio14-20260916`,
  log `~/temp/agent/Mio14-20260916.log`).
- `Mio14` killed at 04:05 UTC under ALWAYS KILL AND RELAUNCH once `36b79bfa0` landed;
  it had reproduced the overrun (24 cap cuts,
  one block with 6 of 12 seats unreadable,
  reviewed on 5 under class thirty-one)
  and drained Hyper from 84 to 42 credits in 61 minutes.
  `Mio15` launched at 04:06 UTC on frozen `36b79bfa0` (pid 1089368,
  runs `~/temp/agent/Mio15-20260916`,
  log `~/temp/agent/Mio15-20260916.log`).
- `Mio15` stopped INCOMPLETE at 06:46 UTC at slice 14,
  class thirty-two (the page's farewell paragraph paired with the original's poem quote,
  and the floor ceiling leaves no room for both);
  fixed in `8d058d102` with the red guard `9c06d6d63`.
  One-transcript verdict:
  fewer cap cuts on ordinary blocks,
  the first chat block unchanged (7 of 12 unreadable);
  Hyper is billed per request and ran dry.
  `Mio16` launched at 07:02 UTC on frozen `8d058d102` (pid 1135987,
  runs `~/temp/agent/Mio16-20260916`,
  log `~/temp/agent/Mio16-20260916.log`).
- `Mio16` settled at 09:42 UTC (160 minutes,
  the first settled Mio since Mio12);
  class thirty-two not exercised (slice 14 cut the Mio12 way),
  the seven checks against Mio12 all better or equal,
  but the archive block review revised the first chat translation with its opening speakers swapped:
  the transcript carried no side (class thirty-three).
  Fixed in `170b6b40c` (red guard `605e66228`),
  wording measured on both chat pictures in `ffd35334a`
  (photo6 six of six readers right,
  photo7 five of six).
  `Mio17` launched at 10:05 UTC on frozen `ffd35334a` (pid 1197118,
  runs `~/temp/agent/Mio17-20260916`,
  log `~/temp/agent/Mio17-20260916.log`).
- `Mio17` stopped INCOMPLETE at 13:29 UTC at slice 16 (203 minutes):
  class thirty-three held (the review's winning revision opens with Mio's sticker and 在嘛),
  but the pairing set the archive's second chat translation (a picture's) against the poem's blockquote,
  so the floor had room for one blockquote and the gate kept an ineligible standing (class thirty-four).
  Fixed in `e964d2f3a` (red guards `d7e514742`):
  the pairing sheet sees what the pictures say and the pass pairs again with them in hand.
  `Mio18` launched at 13:52 UTC on frozen `e964d2f3a` (pid 1260175,
  runs `~/temp/agent/Mio18-20260916`,
  log `~/temp/agent/Mio18-20260916.log`).
  Meters at launch:
  Synthetic 77.3 percent,
  Hyper 89,
  Bedrock 185.36 USD,
  OpenRouter 267.73 USD.
- Owner,
  2026-09-16 (after the Mio18 launch):
  "DeepSeek V4.1 Flash is much better than both V4 Pro and V4 Flash,
  so please remove V4 Pro and V4 Flash."
  Both left the roster and joined the blocklist;
  the editor and refiner seats `deepseek-v4-pro-0813` held passed to `deepseek-v4.1-flash`
  on the owner's words,
  unmeasured in those roles until the next `editor-calibrate`
  (the 2026-09-16 addendum of the seating decision).
  `Mio18` was killed at 14:12 UTC under always-kill-and-relaunch.
  Its 20 minutes still read on class thirty-four:
  the sighted pairing of section 2 carried 1382 characters of picture transcript
  and paired 12 of 19 translation blocks where Mio17's blind pairing paired 14 of 19,
  the two fewer being the picture-rendering chat quotes the rule now leaves out.
  `Mio19` launched at 14:14 UTC on frozen `a88f52d5b` (pid 1315881,
  runs `~/temp/agent/Mio19-20260916`,
  log `~/temp/agent/Mio19-20260916.log`);
  Hyper dry all day,
  so `glm-5.3` is unreachable as on Mio17 and Mio18.
  `Mio19` settled at 15:49 UTC in 100 minutes and its page reads clean
  (the pass log's "Mio19 read" heading dated 2026-09-16 16:05 UTC):
  class thirty-four held (section 2 paired 12 of 19 translation blocks,
  the chat quotes,
  their labels and the tweet quote unpaired,
  the poem unpaired),
  class thirty-three held (the archive's chat kept,
  two typos corrected),
  the seven checks against Mio12 all better or equal,
  the three checks passed,
  four wording notes recorded and not fixed.
  That closes the Mio reading owed for a horizontal rule in the body and a bare URL.
  `hulicaijia5` launched at 15:58 UTC on frozen `1cfd9d2fe` (pid 1470018,
  runs `~/temp/agent/hulicaijia5-20260916`,
  log `~/temp/agent/hulicaijia5-20260916.log`).
- Owner,
  2026-09-16,
  after the Mio19 report,
  on the Mio19 wording note about "older sister who is also trans":
  "It is true though?
  And the Chinese source does imply that,
  if I remember correctly?"
  Measured:
  the Chinese page says 有一个姐姐 in all seven revisions,
  the cited blog says 「Mio 的姐姐也是 MtF」,
  and four critics had filed the clause as an addition on absence alone with the panel supporting all four,
  against the rule both sheets already carry.
  Twenty of Mio19's 24 addition claims shipped an edit,
  fourteen of Mio16's.
  Class thirty-five.
  Four options put to the owner;
  the owner:
  "3,
  and we have Exa.
  Such Exa fetched results need to be cached too to avoid hitting reference links too much",
  then "Cache Exa results semi-permanently on disk since the reference links rarely change their content."
  Landed in `dff8c91d2`
  ([the cited-references decision](../decision/translation-repair-cited-references-2026-09-16.md)).
  `hulicaijia5` killed at 16:43 UTC under always-kill-and-relaunch;
  `Mio20` launched at 16:43 UTC on frozen `dff8c91d2` (pid 1551070,
  runs `~/temp/agent/Mio20-20260916`,
  log `~/temp/agent/Mio20-20260916.log`),
  its log opening `REFERENCES cited=2 cached=2 bought=0`.
- Mio20 read at 18:44 UTC
  (the pass log's "Mio20 read" heading dated 2026-09-16 18:44 UTC):
  SETTLED,
  the panel now rejects the sister addition 3 to 2
  (Mio19:
  supported 4 to 1),
  the repair lane keeps the clause,
  and the lane contest loses it:
  three of five judges call the repair candidate unsupported for it,
  translate wins,
  the consolidation writes from translate.
  Class thirty-six:
  the contest,
  consolidation and slate-judge sheets never saw the references.
  Fixed in `20a7272dc`
  (all eight guards shown red);
  `Mio21` launched at 19:04 UTC on frozen `20a7272dc` (pid 1635211,
  runs `~/temp/agent/Mio21-20260916`,
  log `~/temp/agent/Mio21-20260916.log`).
  The rest of Mio20 read clean and kept the archive's wording where Mio19 rewrote it.
- Mio21 read at 20:40 UTC
  (the pass log's "Mio21 read" heading dated 2026-09-16 20:40 UTC):
  SETTLED in 87 minutes,
  the sister clause ships only because slice 2's slate was declined and the archive stood:
  the panel supported the addition claim 3 to 2 with the references on the sheet,
  the repair lane dropped the clause,
  and four select judges wrote "true per the cited reference but not stated in the ORIGINAL".
  Class thirty-seven:
  the sheets' addition category outranks the rule block.
  Fixed in `35e9a0785` and `887e5262c`:
  reference-backed archive details attested word for word at preparation,
  addition claims on them rejected before the panel,
  attested lines under the references on every sheet.
  The archive block review revised the first chat block against the transcript
  (the pat,
  the mtf question,
  the musculine typo)
  but shipped straight quotes:
  class thirty-eight,
  fixed in `6b853e5ce`.
  `Mio22` (frozen `35e9a0785`,
  21:00 UTC) attested nothing
  (one of three sister attestations spaced the reference's Chinese)
  and was killed at 21:01 UTC under ALWAYS KILL AND RELAUNCH;
  `Mio23` launched at 21:05 UTC on frozen `887e5262c` (pid 1793041,
  runs `~/temp/agent/Mio23-20260916`,
  log `~/temp/agent/Mio23-20260916.log`).
  Mio23 read at 22:40 UTC
  (the pass log's "Mio23 read" heading):
  SETTLED in 94 minutes,
  the sister detail attested by five of six voices,
  no addition claim filed on it,
  the repair lane kept the clause,
  and the translate lane,
  written from the ORIGINAL alone,
  won the contest 3 to 2 without it,
  one ballot calling the repair candidate wrong for keeping it:
  class thirty-nine,
  the translate writer never sees the attested lines,
  fixed in `27109ca8b`
  (the writer sheet carries an ATTESTED DETAILS block and a rule to carry each,
  folded into the translate run shape).
  `Mio24` launched at 22:58 UTC on frozen `27109ca8b` (pid 2354397,
  runs `~/temp/agent/Mio24-20260916`,
  log `~/temp/agent/Mio24-20260916.log`).
  Mio24 read at 00:15 UTC on 2026-09-17
  (the pass log's "Mio24 read" heading):
  INCOMPLETE after 76.7 minutes at slice 17,
  the sister attested four of five,
  both addition claims on it rejected before the panel,
  the repair lane kept the clause and won the contest 3 to 2,
  the translate lane's writers heard the rule and one candidate of five carried the clause while the
  judges chose a dropper 3 of 5;
  slice 17 is class forty:
  the contest settled neither with the archive flawed on every ballot,
  the archive's one-line rendering stood,
  the deterministic rule withheld it (the poem's link),
  the writers' proposals flattened the blockquote,
  three of five slate judges abstained,
  the two who voted tied,
  and the two valid lane texts were on no slate.
  Fixed in `1c38e4f34`
  (lane texts that pass the rule join the slate whenever the standing is neither endorsed nor eligible,
  under a `lane` producer kind;
  guards red first at `341a540bf`).
  `Mio25` launched at 00:40 UTC on frozen `1c38e4f34` (pid 2590736,
  runs `~/temp/agent/Mio25-20260917`,
  log `~/temp/agent/Mio25-20260917.log`).
  Mio25 read at 03:00 UTC
  (the pass log's "Mio25 read" heading):
  SETTLED on the second attempt after 136 minutes
  (the first attempt died on the poem slice's translate select,
  two of four candidates the Chinese untranslated,
  the launcher re-attempted from the cache);
  class forty not exercised (the contest chose translate six of six on slice 16);
  the sister attested five of six,
  the repair lane kept the clause,
  the contest chose translate four of six over an unsupported repair-lane phrase,
  the slate gate restored the clause four of five,
  and the polish removed it with its gate confirming four of six (class forty-one);
  the poem ships as a blockquote with its link and none of its five hard breaks (class forty-two).
  Fixed in `836f90687` (the references on the refiner,
  refine-judge and polish-gate sheets and in the refine key) and `052f8eea0`
  (the break floor per block kind the page never rendered,
  plus the untranslated refusal);
  guards red first at `16f7639a1`.
  `Mio26` launched at 03:25 UTC on frozen `052f8eea0` (pid 2838543,
  runs `~/temp/agent/Mio26-20260917`,
  log `~/temp/agent/Mio26-20260917.log`).
  Mio26's attestation lost the sister
  (heard 5,
  four voices answered an empty list in 6 to 2,416 tokens,
  one attested it,
  deepseek cap-cut,
  GLM-5.3-Flash a runaway):
  class forty-three,
  fixed in `f7aa7c675`
  (every verified candidate goes back to the bench as a numbered yes-or-no question;
  guard red first at `b41ff966c`;
  the pass log's "Class forty-three" heading dated 2026-09-17 04:00 UTC).
  Mio26 killed at 04:07 UTC in its pictures phase under ALWAYS KILL AND RELAUNCH.
  `Mio27` launched at 04:07 UTC on frozen `f7aa7c675` (pid 3011126,
  runs `~/temp/agent/Mio27-20260917`,
  log `~/temp/agent/Mio27-20260917.log`).
  Mio27 read at 06:05 UTC
  (the pass log's "Mio27 read" heading):
  SETTLED in one attempt after 117 minutes,
  the sister attested four of five and confirmed five of five,
  the repair lane kept the clause and won the contest five of six,
  the polish kept it with its gate five of five,
  and the page carries the sister clause (class forty-one verified);
  the poem ships with five `<br/>` breaks and its link;
  class forty fired on slice 4 and the slate declined;
  the seven checks and the three checks clean.
  Found:
  four of 68 translate candidates carried the sheet's picture-transcript block,
  and both slices that had one chose it (class forty-four,
  fixed in `0986c871f`,
  guard red first at `d138f456b`);
  the page was clean by the contest and the consolidation.
  `hulicaijia6` launched at 06:15 UTC on frozen `0986c871f` (pid 3369011,
  runs `~/temp/agent/hulicaijia6-20260917`,
  log `~/temp/agent/hulicaijia6-20260917.log`)
  and died at publish after 205 minutes:
  the repair lane rendered 相遇 under the archive's heading for 初识
  and the page carried twelve distinct headings for thirteen
  (class forty-five,
  fixed in `978bd7d20`,
  guard red first at `b6bfd2b96`:
  the page assembly restores the archive's heading into a colliding slice).
  `hulicaijia7` launched at 10:10 UTC on frozen `978bd7d20` (pid 3491560,
  runs `~/temp/agent/hulicaijia7`,
  log `~/temp/agent/hulicaijia7.log`)
  and read clean at 12:50 UTC:
  SETTLED first attempt,
  150 minutes,
  4.41 USD,
  thirteen distinct headings with nothing restored
  (the lanes rendered the two sections apart this time,
  so class forty-five's restoration is still unmeasured live),
  ruby and Sakura as the archive,
  the original's eight footnotes relabelled,
  the archive's two translator-added notes dropped by the additions rule.
  `shi_Yumiaoya1` launched at 12:42 UTC on the same dist (pid 3508311,
  runs `~/temp/agent/shi_Yumiaoya1`,
  log `~/temp/agent/shi_Yumiaoya1.log`)
  and stopped INCOMPLETE at publish after 53 minutes:
  the bare-URL scan counted a profile link inside the original's HTML comment as a destination the page owed
  (class forty-six,
  fixed in `cde09e8bc`,
  guard red first at `9ba9520ab`;
  every lane text carried its links).
  `shi_Yumiaoya2` launched at 13:45 UTC on frozen `cde09e8bc` (pid 3531497,
  runs `~/temp/agent/shi_Yumiaoya2`,
  log `~/temp/agent/shi_Yumiaoya2.log`)
  and stopped INCOMPLETE at 48 minutes:
  the bilingual farewell quote failed the line floor unless quoted twice
  (class forty-seven)
  and the will's paragraph shipped as a silent gap on a split coverage round nobody anchored
  (class forty-eight;
  both fixed in `56307dca5`,
  guards red first at `d692d5f1d`).
  `shi_Yumiaoya3` launched at 14:56 UTC on frozen `56307dca5` (pid 3571748,
  runs `~/temp/agent/shi_Yumiaoya3`,
  log `~/temp/agent/shi_Yumiaoya3.log`)
  and stopped INCOMPLETE at publish after 78 minutes:
  the translate lane defined the footnote inside the opening section,
  the assembly withdrew both carriers,
  and the section left the page with its film link
  (class forty-nine,
  fixed in `cd789763c`,
  guard red first at `a22dcffde`).
  Classes forty-seven and forty-eight held live.
  `shi_Yumiaoya4` launched at 16:30 UTC on frozen `cd789763c` (pid 3610388,
  runs `~/temp/agent/shi_Yumiaoya4`,
  log `~/temp/agent/shi_Yumiaoya4.log`)
  and settled after 52 minutes with the death paragraphs shipped as a recorded gap:
  the coverage majority counted two router-refused seats
  (class fifty)
  and one partial claim quoted the archive's farewell line,
  another slice's paired rendering
  (class fifty-one;
  both fixed in `479bb011f`,
  guards red first at `6f7567f7e`).
  Class forty-nine held.
  `shi_Yumiaoya5` launched at 17:38 UTC on frozen `479bb011f` (pid 3740123,
  runs `~/temp/agent/shi_Yumiaoya5`,
  log `~/temp/agent/shi_Yumiaoya5.log`)
  and settled after 40 minutes with the death paragraphs on the page
  (the pass log's "shi_Yumiaoya5 read" heading dated 2026-09-17 18:28 UTC;
  three judge-quality notes there,
  "Zhongshan No. 1 Hotel" for 中山一酒店 a world-knowledge miss on an ambiguous 一,
  the owner's reading of 2026-09-17).
  `yingying1` launched at 18:23 UTC on the same frozen `479bb011f` (pid 3820813,
  runs `~/temp/agent/yingying1`,
  log `~/temp/agent/yingying1.log`)
  and settled clean after 15 minutes
  (the pass log's "yingying1 read" heading dated 2026-09-17 18:42 UTC).
  `XingZ601` launched at 18:39 UTC on the same frozen `479bb011f` (pid 3839544,
  runs `~/temp/agent/XingZ601`,
  log `~/temp/agent/XingZ601.log`)
  and stopped INCOMPLETE at the pictures phase after 7 minutes:
  three readers read the score screenshot and the corroboration compared only the first two
  (class fifty-two,
  fixed in `24372215a`,
  guard red first at `c8d9a990c`).
  `XingZ602` launched at 18:56 UTC on frozen `24372215a` (pid 3880778,
  runs `~/temp/agent/XingZ602`,
  log `~/temp/agent/XingZ602.log`):
  its first attempt stopped ERROR after 2h38m on slice 14,
  an archive text the deterministic floor refuses and two tied judge rounds
  (class fifty-three,
  fixed in `012b9ccc3`,
  guard red first at `5ae2e79ad`),
  and its second attempt was killed at 5h57m in the consolidation once the fix landed.
  `XingZ603` launched at 00:55 UTC on 2026-09-18 on frozen `012b9ccc3` (pid 4181820,
  runs `~/temp/agent/XingZ603`,
  log `~/temp/agent/XingZ603.log`)
  and was killed at 01:56 UTC 60 minutes into the repair lane,
  before the translate lane,
  once the Jev seat landed
  (the pass log's "XingZ603 killed" heading dated 2026-09-18 01:56 UTC;
  1.24 USD).
  The owner approved `typesafe/jev-1.13` on OpenRouter at 00:29 UTC
  (the pass log's "Jev 1.13 approved" heading dated 2026-09-18 00:29 UTC):
  a typed-decision model on a decisions endpoint,
  measured 28 of 28 by scratch at 01:05 UTC,
  put on the roster as a decision-only seat (`ad20acc84`),
  wired through a decisions transport into the candidate-select stage (`bc6fdc9b9`),
  measured 12 of 12 through the production probe at 01:40 UTC
  and seated as a select judge (`c14c7f176`;
  the pass log's "Jev 1.13 seated" heading dated 2026-09-18 01:47 UTC;
  the seating decision's 2026-09-18 addendum).
  `XingZ604` launched at 01:56 UTC on frozen `c14c7f176` (pid 211894,
  runs `~/temp/agent/XingZ604`,
  log `~/temp/agent/XingZ604.log`),
  JUDGE SEATS `wide=8 select=9`,
  and stopped INCOMPLETE at 4h53m on slice 13
  (class fifty-four,
  the consolidate gate's indecision keeping an ineligible standing over the proposal the slate chose;
  fixed in `0d0747200`,
  guard red first at `b8b32e28d`;
  the pass log's "XingZ604 read" heading dated 2026-09-18 06:55 UTC;
  7.97 USD).
  Jev voted live on 221 rounds,
  every call on the decisions route.
  `XingZ605` launched at 07:03 UTC on frozen `0d0747200` (pid 474604,
  runs `~/temp/agent/XingZ605`,
  log `~/temp/agent/XingZ605.log`)
  and stopped INCOMPLETE at 4h08m on slice 13 again,
  this time a tied slate over the withheld standing with no challenge round
  (class fifty-five,
  fixed in `adca69d4e`,
  guard red first at `c6ec06788`;
  the pass log's "XingZ605 read" heading dated 2026-09-18 11:12 UTC;
  5.41 USD).
  `XingZ606` launched at 11:20 UTC on frozen `adca69d4e` (pid 512931,
  runs `~/temp/agent/XingZ606`,
  log `~/temp/agent/XingZ606.log`)
  and stopped INCOMPLETE at 3h04m on slice 33,
  the teachers paragraph,
  where the gate kept the ineligible standing 4 of 5 without being told it cannot ship
  (class fifty-six,
  fixed in `139a7f99a`,
  guards red first at `90e5e2a19`;
  the pass log's "XingZ606 read" heading dated 2026-09-18 14:30 UTC;
  6.92 USD).
  `XingZ607` launched at 14:40 UTC on frozen `139a7f99a` (pid 571308,
  runs `~/temp/agent/XingZ607`,
  log `~/temp/agent/XingZ607.log`);
  no cron job on the owner's word,
  the background waiter alone reports the exit.
  Its attempt 1 ran to the seven-hour entry deadline at 21:39 UTC in consolidation with no slice stopped,
  both pronoun paragraphs shipped through informed gates three to one
  (the pass log's "XingZ607 attempt 1" heading dated 2026-09-18 21:39 UTC);
  attempt 2 stopped INCOMPLETE at 23:20 UTC,
  one source destination dropped,
  because the translate lane had withdrawn all 88 of its slices at assembly over two lone closing tags
  (class fifty-seven,
  fixed in `6036e6618`,
  guards red first at `55cd8f7e7`;
  the pass log's "XingZ607 attempt 2 read" and "Class fifty-seven" headings;
  12.42 USD over both attempts).
  The run-time work followed
  (the pass log's "deepseek-v4.1-flash routed off DeepInfra and Wafer" heading dated 2026-09-19 00:05 UTC:
  the endpoint was the slow seat,
  `ac93f7978`)
  and class fifty-eight
  (two structural breaks emptied the lane;
  `eaaf39ec3`,
  guards red first at `35b2125d2`).
  `XingZ608` (frozen `eaaf39ec3`,
  overlap 8) ran the whole pipeline in 2h16m,
  the run time verified,
  and stopped INCOMPLETE at publish on the dropped bilibili link:
  the whole-page shortfall budget refused the footnote definitions at the page's tail
  and the translate assembly reverted every carrier of their markers
  (class fifty-nine,
  the pass log's headings dated 2026-09-19 02:35 and 02:45 UTC;
  fixed in `06b395ac7`,
  guards red first at `db5f0124e`).
  `XingZ609` (frozen `06b395ac7`) confirmed class fifty-nine live at its admission
  and was killed at 03:24 UTC,
  35 minutes in,
  once the owner's "Do #1" landed:
  the untranslated tail after the pairing's last agreed pair is budgeted from the pairing
  (`5bda732de`,
  guards red first at `c6c869267`,
  the pass log's heading dated 2026-09-19 03:20 UTC;
  "The shortfall budget question" is answered).
  `XingZ610` (frozen `5bda732de`) confirmed the tail bound live at its admission
  (31 slices,
  "admitted on that bound",
  28 admitted)
  and showed class sixty:
  three tail slices split on one minority anchored claim stayed unresolved
  (the pass log's heading dated 2026-09-19 03:50 UTC;
  fixed in `de25b4ed8`,
  guard red first at `6420d96e8`);
  killed at 03:53 UTC,
  29 minutes in,
  2.08 USD by the meters.
  `XingZ611` (frozen `de25b4ed8`) confirmed class sixty live at its admission
  (all 31 tail slices admitted,
  five splits among them)
  and showed class sixty-one:
  the memorial disclosure block's third paragraph,
  absent by a majority,
  refused by an interior budget of zero while the archive's block is one paragraph short of the original's
  (the pass log's heading dated 2026-09-19 05:30 UTC;
  fixed in `75f67fbd4`,
  guard red first at `1083b9163`);
  killed at 05:37 UTC,
  1 hour and 42 minutes in,
  6.91 USD by the meters.
  `XingZ612` launched at 05:39 UTC on frozen `75f67fbd4` (pid 895538,
  runs `~/temp/agent/XingZ612`,
  log `~/temp/agent/XingZ612.log`,
  `TRANSLATION_REPAIR_SLICE_OVERLAP=8`);
  no cron job on the owner's word,
  the background waiter alone reports the exit.
  Its admission at 06:00 UTC confirmed class sixty-one live
  (slice 70 admitted inside details of slices 67 to 71,
  the original writing 5 blocks there,
  the archive 4),
  the tail admitted on the bound,
  and no `placement unresolved` on any slice.
  It stopped ERROR at 06:13 UTC,
  34 minutes in,
  three attempts "writing bench unreachable at lanes":
  Inception rate-limited Mercury 2.5,
  OpenRouter passed each 429 on as "Provider returned error",
  and the router held the whole provider out 60 s on every one
  (class sixty-two,
  the pass log's heading dated 2026-09-19 06:15 UTC;
  fixed in `c340b496b`,
  guard red first at `79b3ee845`);
  2.19 USD by the meters.
  `XingZ613` (frozen `c340b496b`) confirmed class sixty-two live
  (54 pass-on lines against Mercury 2.5,
  no provider hold,
  no bench stop)
  and stopped INCOMPLETE at 09:24 UTC,
  3h02m in,
  at the consolidation of slice 84
  (the `其九：空白` heading):
  the standing ineligible on the line rule,
  four valid proposals split 1.5 to 1 to 1 against the minimum of 2,
  the run-off over every backed candidate the same question,
  split the same way
  (class sixty-three,
  the pass log's heading dated 2026-09-19 09:30 UTC;
  fixed in `4193495fa`,
  guard red first at `9176c9819`);
  7.81 USD by the meters.
  Its repair lane ran 1h39m against XingZ608's 35m because every round a rate-limited Mercury sat on waited its ladder out in grace
  (class sixty-four,
  open,
  under the same heading).
  `XingZ614` (frozen `4193495fa`) was killed at 09:50 UTC,
  12 minutes in and still in preparation,
  once class sixty-four landed
  (fixed in `fc428eaee`,
  guard red first at `17084ca5e`:
  a model its upstream endpoint rate-limited is held out 60 s and refused at call time as unreachable);
  0.04 USD by the meters.
  `XingZ615` (frozen `fc428eaee`) ran its repair lane at 91 s a slice
  (XingZ613 295 s,
  XingZ608 62 s;
  Mercury unlimited this run,
  so class sixty-four was not exercised)
  and stopped INCOMPLETE at 12:24 UTC,
  2h41m in,
  at the consolidation of slice 91,
  the closing poem:
  the translate slate had carried a candidate the rule refused
  (the attribution moved out of the quote)
  and the judges chose it
  (class sixty-six),
  and the consolidation's only floor-valid proposal was the Chinese original respaced
  (class sixty-five;
  the pass log's heading dated 2026-09-19 12:40 UTC;
  both fixed in `6eada935a`,
  guards red first at `636d56a8c`);
  10.34 USD by the meters.
  `XingZ616` (frozen `6eada935a`) SETTLED at 14:50 UTC,
  2h04m,
  the first settled XingZ60 page
  (the pass log's "XingZ616 read" heading dated 2026-09-19 14:55 UTC):
  the poem with its attribution inside the quote,
  the four memorial paragraphs,
  both links,
  nine definitions,
  front matter equal;
  two quality observations,
  the section ordinals in seven styles and two handles translated literally,
  named classes sixty-seven and sixty-eight by the owner
  ("Fix these quality observations")
  and fixed in `c26648e2a`
  (guards red first at `034d52b42`;
  the pass log's heading dated 2026-09-19 15:35 UTC):
  the page assembly renders a signer's name one way across headings and signatures,
  the archive's signature rendering first,
  and a numbered heading series in the one style most of its headings took.
  9.19 USD by the meters.
  The V4.1 Flash producer calibration ran detached from 14:53 to 16:45 UTC
  (pid 1031770,
  log `~/temp/agent/producer-calibrate-v41flash-20260919.log`)
  and was read at once:
  V4.1 Flash at the pooled null (z +0.79) takes the translator and consolidation seats,
  Mercury 2.5 below it across the threshold (z -3.00) leaves the translator seat
  (`9f8dcccd2`,
  the seating decision's addendum dated 2026-09-19).
  XingZ617 (frozen `c26648e2a`) was killed nine minutes in for the seating;
  XingZ618 (frozen `9f8dcccd2`) reached its admission at 17:13 UTC with slice 70,
  the memorial's third paragraph,
  unresolved on a split verdict inside the block deficit
  (class sixty-nine,
  fixed in `bb4bb7556`,
  the pass log's heading dated 2026-09-19 17:20 UTC)
  and was killed at 17:20 UTC.
  `XingZ619` (frozen `bb4bb7556`) SETTLED at 20:17 UTC,
  2h57m,
  one attempt
  (the pass log's "XingZ619 read" heading dated 2026-09-19 20:40 UTC):
  classes sixty-seven to sixty-nine live
  (four names restored,
  five headings unified into `Part One` to `Part Ten`,
  slice 70 admitted on the block deficit and the memorial's four paragraphs on the page),
  fourteen headings,
  both links,
  front matter equal,
  Jev 240 of 240,
  V4.1 Flash writing in both lanes and Mercury judging only in the translate lane;
  one open observation (the Bird in a Cage attribution keeps its Chinese link text);
  10.95 USD by the meters.
  Also today:
  the Synthetic meter move between Mio20 and Mio21 was the owner's own DeepSeek V4.1 Flash use
  (`aa73b9b22`),
  and the Claude Code Monitor tool's 30-minute cap is written into the CLAUDE.md preamble
  (`2901a92ba`;
  long waits use Bash run_in_background).
- Owner,
  2026-09-16,
  same hour:
  "Adding / removing / changing models should never be this difficult.
  And given how fast the AI space is moving,
  a reusable workflow or mise task or something should be built."
  The unseat touched eight source files and fifty-eight test files.
  Asked which shape,
  the owner delegated:
  "Please make the decisions yourself as these aren't really design decisions."
  Built the same afternoon
  (`doc/decision/translation-repair-model-cards.md`,
  runbook `doc/runbook/translation-repair-roster-change.md`):
  one card per model in `model-cards.ts` from which every catalog,
  cap table and seat hold derives (`ea0024e3c`),
  the `roster-card` task that prints a card fragment off a provider's live listing
  (`527681ba4` and `e53f4c295`),
  and role-named fixture seats in 172 test files (`099b69787`).
  Adding a model is now one card,
  its spellings on the lists in `roster-id.ts`,
  one fixture seat and the checks;
  removing one is the reverse plus a blocklist line.
  Hyper's listing reported `max_output_tokens` 32768 for V4.1 Flash on 2026-09-16
  against the card's 26214 of 2026-09-11;
  left as measured,
  to re-read with the next Hyper reading.

Everything from "### Latest checkpoint" down to "## Repository state" is those sessions' own record,
kept as history and not as the current state.
