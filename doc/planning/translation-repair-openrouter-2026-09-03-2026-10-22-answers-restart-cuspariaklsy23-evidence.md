# OpenRouter and the corpus passes: the evidence of CuspariaKLSY23 on build 8864316eb2 (2026-10-22)

The log line or artifact field behind every figure of
[the run of CuspariaKLSY23](translation-repair-openrouter-2026-09-03-2026-10-22-answers-restart-cuspariaklsy23-run.md)
and
[the stages of CuspariaKLSY23](translation-repair-openrouter-2026-09-03-2026-10-22-answers-restart-cuspariaklsy23-stages.md),
and the figures no log can settle.
A second reader recounted every figure from the logs on 2026-10-22 with scripts of its own
(under `temp/agent/tr-fan/w284-read/check/`):
137 confirmed,
6 corrected in place in the run and stages files,
and none unsourceable beyond the list in "Figures that could not be sourced",
each entry of which it confirmed absent from the log and the run directory.

Part of [Translation repair: OpenRouter as the paid fallback provider](translation-repair-openrouter-2026-09-03.md),
which lists every file of this record.

## CuspariaKLSY23 evidence, 2026-10-22

### Evidence for the figures

Log line numbers are of `corpus-pass-CuspariaKLSY23.log`.
Artifact fields are of `artifacts/CuspariaKLSY.json`.
`CuspariaKLSY22` line numbers are of `corpus-pass-CuspariaKLSY22.log`.

- Tip,
  pipeline digest,
  files,
  pending:
  line 196;
  artifact `tip`,
  `pipelineDigest`,
  `corpusSha`.
- `ONLY CuspariaKLSY`:
  line 195.
  Build `Finished in 9.26s`:
  line 193.
- Writer grace built in:
  line 197.
- `PROVIDER OFF bedrock` (and the only `HTTP 401` text):
  line 198.
- First timestamped line 13:22:21.578 UTC:
  line 199.
  Last timestamped line 13:39:30.494 UTC:
  line 2210.
  Log length 2222 lines:
  `wc --lines`.
  `Finished in 1041.98s`:
  line 2222.
- `OVERLAP CuspariaKLSY value=4 source=fallback`:
  line 200.
  `ARCHIVE NOTE`:
  line 201.
- Baseline 5.9624 kWh:
  line 205.
  Day anchor:
  line 206.
- First `METERS` (neuralwatt dry,
  minus 5.188,
  555.188,
  synthetic 0 percent dry,
  hyper 193,
  openrouter 3.98):
  line 210.
  Last `METERS` (hyper 157,
  openrouter 3.93):
  line 2030.
  The 13 `METERS` lines:
  210,
  314,
  576,
  829,
  1047,
  1290,
  1366,
  1444,
  1506,
  1627,
  1750,
  1862 and 2030.
- The 14 `stream quota` lines:
  204,
  310,
  568,
  826,
  1042,
  1286,
  1362,
  1440,
  1503,
  1624,
  1747,
  1856,
  2026 and 2210.
- `JUDGE SEATS`:
  lines 211,
  227,
  228,
  1400,
  1673 and 1751.
- `FRONT MATTER`:
  line 212.
  `CORPUS NAMES`:
  line 213.
- Block pairing:
  lines 214,
  225 and 226.
- Both lanes start:
  229 and 230.
  Repair `SLICE-START`:
  231 to 234.
  Repair `SLICE-COST`:
  1202 (slice 0),
  1255 (2),
  1261 (1),
  1352 (3).
  Critic stages:
  263 (slice 2),
  280 (3),
  384 (0),
  445 (1).
  Probe rounds:
  1199 (0),
  1252 (2),
  1258 (1),
  1349 (3);
  the added-damage claim:
  1259.
  Refine decline:
  1392 and 1393.
  Repair shipped:
  1395.
  `semantic wrap` skipped:
  1394.
  `content lost`:
  1396 to 1399.
- Translate lane:
  `JUDGE SEATS` 1400,
  re-seat 1401,
  disputes 1402 and 1403,
  `SLICE-START` 1404 to 1407,
  `SLICE-COST` 1491 (slice 3),
  1578 (0),
  1617 (1),
  1669 (2),
  winners 1490,
  1577,
  1616 and 1668,
  summary 1671 and 1672.
  Publication-rule warns:
  1437,
  1472,
  1820,
  1848,
  1871,
  1872 and 1873.
- Lane contest start:
  line 1674;
  damage claim 1675;
  verdicts 1709 (slice 3),
  1726 (0),
  1735 (1),
  1742 (2);
  rounds 1703,
  1720,
  1729 and 1736.
- Consolidation start:
  line 1752.
  `SLICE-START`:
  1753 to 1757.
  Disputes:
  1755 and 1758.
  Standing lacks endorsement:
  1759.
  Run-off announcement:
  1788 and 1789.
  Slate round:
  1807 to 1813.
  Consolidate gates:
  1834 and 1835 (slice 3),
  1933 (0),
  2171 (2).
  Polish gate:
  2142 to 2148.
  Naturalness reviews:
  1942,
  2019,
  2130,
  2162,
  2191 and 2201.
  `SLICE-COST`:
  2020 (slice 3),
  2163 (1),
  2192 (0),
  2202 (2).
  Page assembly:
  2204 and 2205.
  Consistency:
  2203.
  Publish:
  2206.
- `TALLY`:
  line 2207.
  `DESTINATIONS`:
  2208.
  `DONE`:
  2209.
  End `stream quota`:
  2210.
  Plan energy:
  2211.
  `SEAT` lines:
  2212 (`gemma-4-e2b`),
  2213 (`deepseek-v4.1-flash`),
  2214 (`hf:zai-org/GLM-5.3-Flash`),
  2215 (`hf:moonshotai/Kimi-K3`),
  2216 (`glm-5.3`),
  2217 (`inception/mercury-2.5`),
  2218 (`gemma-4-26b-a4b-it`),
  2219 (`minimax-m3`),
  2220 (`typesafe/jev-1.13`),
  2221 (`xiaomi/mimo-v2.6-flash`).
- Mercury:
  maximum first content line 1530 (21,327 ms);
  the 64 first-content times:
  `mercury.ts` over the stream lines (listed in the paragraph "The 64 first-content times of Mercury");
  `SEAT` 2217;
  unusable answers 954 (with 953 `no re-ask elsewhere`) and 2187 (2186);
  JSON recoveries 244 and 1295;
  no empty stream.
  All 64 `SPEND` lines carry `endpoint=Inception` (`mercury.ts`).
- Kimi cut:
  stream line 1737,
  warn 1743,
  round 1736,
  contest ballots 1738 to 1741.
  Kimi stream 92,371 ms:
  line 1727.
- Cap-outs:
  streams 1512,
  1618,
  1851;
  `SPEND` 1513,
  1619,
  1852;
  re-asks 1514 and 1517,
  1620 and 1630,
  1853 and 1869;
  rounds 1518,
  1631,
  1870;
  `max_tokens` payloads:
  `payloads.ts`.
  `CuspariaKLSY22` limit-ended replies:
  `payloads22.ts`;
  the `SPEND` lines with 21,645 and 10,822 completion tokens in its log.
- Irregular ballots:
  lines 343,
  344,
  468,
  469,
  470,
  653,
  654 and 655.
- Final tail:
  lines 2180 to 2200 (`deepseek-v4.1-flash` Hyper stream 2180,
  re-ask 2182,
  OpenRouter stream 2197,
  answered 2199,
  round 2200).
- Re-asks:
  lines 307,
  428,
  733,
  752,
  952,
  1462,
  1514,
  1620,
  1853,
  2182 and 2185 (re-asking),
  337,
  433,
  742,
  757,
  1517,
  1630,
  1869 and 2199 (answered usably),
  953,
  1463 and 2186 (`no re-ask elsewhere`).
- Recovery rounds:
  1014 to 1033 and 1499 to 1510.
- Rounds with 0 ms in grace on a short count:
  1013,
  1498 and 2200.
- Shipped page:
  `wc --chars --bytes --lines` of `fixed/people/CuspariaKLSY/page.en.md`
  (1886 characters,
  1908 bytes,
  52 lines);
  log line 2206 prints 1886.
- Artifact fields:
  `callConfig`,
  `durationMs` (1028107),
  `lanes.repair.result.issues` (36),
  `lanes.repair.result.findings` (107),
  `lanes.repair.result.changedSliceIndices`,
  `lanes.translate.result.changedSliceIndices`,
  `laneSelection.slices`,
  `consolidation.slices`,
  `pageAssembly`,
  `preparation.alignmentFindings` (1),
  `preparation.sliceCount`.
- Payloads:
  336 files in `prompt-payloads/`,
  fields `reply.finishReason` and `reply.servedBy`
  (`payloads.ts`,
  `payloads2.ts`).
- Ledger directory:
  25 files (`ls`).
- `CuspariaKLSY22` figures:
  its `TALLY` (line 2300),
  its `SEAT` lines (2305 to 2314),
  its block pairing round (227),
  its phase times (`phases22.ts`),
  its warns and counts (`counts22.ts`),
  its Mercury streams (`mercury22b.ts`).
- `zheermao23` figures:
  the draft's run facts,
  NeuralWatt section and Mercury section.
- Sibling figures:
  `XingZ6017` providers record lines 286 to 295 and 350 to 356 (minus 5.188 and 555.188 Wh,
  54.3 Wh,
  about 3.8036 kWh),
  and `CuspariaKLSY22` providers record lines 94 to 100 (37.3 Wh)
  and 236 to 242 (`withheld=`).

The 64 first-content times of Mercury,
in log order (log line then milliseconds),
from `mercury.ts`:

- Block pairing (1):
  223 6,038.
- Repair lane (37):
  242 6,172;
  247 7,351;
  249 7,632;
  315 8,875;
  317 8,899;
  438 9,599;
  440 9,150;
  503 6,428;
  594 8,192;
  596 5,675;
  644 6,145;
  736 6,973;
  753 7,629;
  766 6,534;
  796 5,974;
  833 6,169;
  869 6,645;
  873 8,869;
  950 7,793;
  988 7,146;
  1007 6,667;
  1009 6,612;
  1015 6,012;
  1030 6,836;
  1043 6,359;
  1092 5,960;
  1107 6,504;
  1157 5,509;
  1177 6,674;
  1193 4,701;
  1203 7,656;
  1239 6,212;
  1245 6,613;
  1247 6,184;
  1293 6,784;
  1344 7,746;
  1372 5,761.
- Translate lane (4):
  1458 6,616;
  1494 10,242;
  1524 10,610;
  1530 21,327.
- Lane contest (4):
  1685 7,516;
  1689 8,003;
  1691 8,309;
  1695 8,885.
- Consolidation (18):
  1769 5,943;
  1773 6,079;
  1775 6,962;
  1795 5,638;
  1823 6,040;
  1863 7,021;
  1894 5,175;
  1901 6,417;
  1906 6,103;
  1920 8,535;
  1922 5,858;
  1952 7,254;
  2038 8,201;
  2121 7,444;
  2123 6,260;
  2125 5,511;
  2151 5,960;
  2183 8,050.

### Figures that could not be sourced

- The launch time of 13:22:07 UTC and the `DONE` time of 13:43 UTC are the brief's.
  The log carries no clock time on its `START` or `DONE` lines,
  and its own times (13:22:21.578 to 13:39:30.494 UTC) and `Finished in 1041.98s` agree with each other,
  and not with 13:43 UTC.
- The unit result `Result=success` is the brief's;
  it is in no line of the log or the run directory.
- The 90000 ms Mercury first-content window is in no line of the log or the artifact;
  it is read from the worktree card only,
  and whether the working tree matches the tip was not compared.
- The end baseline of NeuralWatt for `zheermao23` (about 5.9587 kWh) and for this run (about 6.2181 kWh)
  are computed and printed nowhere,
  and the 3.7 Wh step between them carries a rounding of about 0.2 Wh;
  the 84 s gap uses the last timestamp of the `zheermao23` draft.
- What drew the 255.7 Wh of plan energy is in no log line;
  the owner settled it on 2026-10-22
  (the NeuralWatt key of the owner's dsh harness is on the same account).
- Why the `minimax-m3` completion figure 21,645 exceeds the card's cap of 10,822.
- That a re-ask is not counted as an ask in the `SEAT` tally:
  read from the arithmetic only (344 + 8 = 352);
  the counting code was not read for it.
- That the 16 `typesafe/jev-1.13` typed decisions are not stored as payloads:
  inferred from the counts (96 OpenRouter `SPEND` lines,
  80 OpenRouter payloads).
- The cause of the translate lane's slow Mercury streams,
  the 70 s `Morph` re-ask,
  and the Kimi runaway reasoning.
- Whether the 8 irregular ballot lines are counted among the 31 `panel-packet` findings.
- Why `gemma-4-26b-a4b-it` returns group indices out of range.
- Why slice 3 drew 14 issues in the repair lane.
- That the polish step skipped slice 2 because it is a numbered list (a reading,
  not shown).
- The `CuspariaKLSY22` round counts and grace sums are from `phases22.ts`;
  its Mercury first-content times do not exist (its log has no `firstContent` field),
  so only elapsed times are compared for that run.
