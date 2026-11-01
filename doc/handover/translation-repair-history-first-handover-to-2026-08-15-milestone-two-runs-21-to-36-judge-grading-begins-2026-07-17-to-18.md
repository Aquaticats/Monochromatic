# Translation repair history, first handover document: milestone two runs 21 to 36, when judge grading began, 2026-07-17 to 18

The run records from the last lexical-only run through the first judge-graded runs,
including the cases where the judge and the word counter disagreed in each direction,
and the third quarantined entry.

Part of [Translation repair history: the first handover document, to 2026-08-15](translation-repair-history-first-handover-to-2026-08-15.md),
which lists every file of this record.

Historical evidence only:
no next-action list here is a current instruction,
and the stop condition in the index applies.

## Runs 21 to 36: the last lexical run and the first judge-graded runs

run 21 (2026-07-17,
2413 s,
LAST lexical-only run):
lexical repair
2/4,
detection 4/4;
a2581911655's 18-worder missed a FIFTH time
(5/18) and Acheron's 9-worder at 1/9,
exactly the terse-faithful
cases the judge is expected to re-credit,
so run 21 is the natural
before/after datum.
FINAL LEXICAL ACCUMULATED:
57/97 (0.59) over 21 runs;
detection
72/78 (0.92).
The judge rate starts fresh at run 22.
run 22 (2026-07-17,
2271 s,
FIRST judge-graded run):
judge repair
2/4 (0.50) versus lexical 0/4 on the SAME seeds,
and the split is
the validation,
not noise:
- ttttsuuukikoo_ both seeds JUDGE=restored but LEXICAL=absent
  (2/6 and 2/7 words):
  the editor faithfully re-translated the zh
  with different English wording,
  vocabulary overlap missed it,
  the
  bilingual judge caught it.
  This is the under-crediting the user
  predicted,
  now corrected.
- wangzihao980 both seeds JUDGE=absent AND LEXICAL=absent while the
  run still resolved 37/37 OTHER issues:
  a genuine repair failure
  the judge agrees on.
  Detection was true (issue accepted at the
  region) but the editor did not actually restore the content,
  so
  detection-yet-unrepaired is real and the judge exposes it.
The judge discriminates (credits faithful rewording,
  fails genuine
misses) rather than inflating;
  that is the whole point.
run 23 (2026-07-17,
  2069 s):
  judge 4/4 (1.0),
  lexical also 4/4;
noname3031 and MioCardMeow both fully restored,
  both graders
agreeing this run (agreement is common;
  disagreement appears on the
terse-rewording cases like run 22's ttttsuuukikoo_).
run 24 (2026-07-17,
  2469 s,
  first post-compaction run):
  judge 3/4
(0.75),
  lexical also 3/4;
  detection 4/4.
  Acheron seed 0 (9 words)
JUDGE=partial with 2/9 lexical words returned;
  its sibling seed 1
(10 words) fully restored.
  MioCardMeow both restored again (repeat
draw from run 23).
  Four entries budget-skipped including
luxuanwen3,
  so the anti-compression prompt retest did not draw.
PATTERN NOTE:
  Acheron breaks the "missed seed is always the LONGEST
needle of its entry" streak;
  the partial (9 words) is shorter than
its fully restored sibling (10 words).
run 25 (2026-07-17,
  1795 s):
  judge 6/6 (1.0),
  lexical 6/6,
detection 6/6;
  yingying,
  lxy,
  SU5ZI2MO1 all fully repaired (three
dispatched,
  three budget-skipped).
  SU5ZI2MO1 seed 0 was a 20-word
needle restored fully (11/20 lexical words,
  judge unanimous):
  the
longest needle restored to date,
  evidence the anti-compression
editor prompt (edit-prompt.ts) is working on exactly the long-needle
compression failure it was written for.
run 26 (2026-07-17,
  1608 s):
  judge 6/6 (1.0),
  lexical 4/6,
detection 6/6;
  coin,
  Mizuki_Yuuki,
  AkiraComplex all fully repaired
(three dispatched,
  three budget-skipped).
  Two more judge-vs-lexical
disagreements,
  both unanimous judge=restored on low word overlap
(coin seed 0:
  3/11 words;
  AkiraComplex seed 0:
  4/10):
  the
faithful-rewording under-credit pattern run 22 first exposed.
Mizuki_Yuuki seed 0 (18 words) restored fully,
  a second long
needle credited since the anti-compression prompt.
run 27 (2026-07-17,
  2414 s):
  judge 5/6 (0.83,
  one partial),
lexical 3/6,
  detection 6/6;
  CuspariaKLSY,
  Hangmster,
CutOceanHeyFis1 all repaired (three dispatched,
  three
budget-skipped).
  Three more unanimous judge=restored on low overlap
(1/7,
  6/14,
  3/11 words).
  FIRST INVERSION:
  CuspariaKLSY seed 1 is
lexical=restored (4/8 words) but judge=partial against the zh
source;
  the judge discriminates in BOTH directions,
  catching a
half-restoration the word counter credited.
  Lenient rate still 1.0.
run 28 (2026-07-17,
  1462 s):
  raw scorecard judge 2/4,
  detection
2/4;
  but shi_Yumiaoya came back status=blocked-non-translation.
Probe confirmed the block is CORRECT:
  its en page holds a genuinely
untranslated CJK region (six lines at 33 to 83 percent CJK;
  zh 3935
chars vs en 1458),
  4/7 critics voted non-translation on that chunk,
whole entry returned unchanged by design.
  Like XIEPT2,
  seeding it
grades nothing about repair,
  so shi_Yumiaoya is now QUARANTINED in
the driver (third entry).
  Its two run-28 seeds are excluded from
the accumulated tallies below.
JUDGE CAVEAT found:
  on the unchanged blocked text the judges
unanimously called the 231-char seed 0 "restored" though the needle
occurs 0 times;
  only 2 of its content words uniquely disappeared,
so near-duplicate needles (vocabulary still present elsewhere) can
fool the judge.
  Rare (first in 28 runs),
  quarantine prevents this
instance;
  benchmark-side fix (exclude blocked entries from the
judge universe) noted as a calibration follow-up.
The repairable entry CuspariaKLSY,
  a repeat draw from run 27,
reproduced its run 27 judgments exactly (seed 0 restored on 0/7
lexical words,
  seed 1 partial):
  judge verdicts are stable across
independent runs.
  Detection 2/2 there.
run 29 (2026-07-17,
  2405 s):
  judge 2/4 (0.50,
  two partials),
lexical 2/4,
  detection 4/4;
  Anilovr and Acheron repaired (two
dispatched,
  four budget-skipped).
  Acheron is a repeat draw from run
24 and its seed 0 judged PARTIAL both times (independent runs):
second reproducibility case;
  that specific needle consistently
comes back half-restored from the editor.
  Anilovr seed 0 is a
second inversion (lexical=restored on 4/7 words,
  judge=partial
against the zh source).
  Lenient rate stays 1.0:
  every judged seed
in the repairable universe is at least partial.
run 30 (2026-07-18,
  1598 s):
  judge 6/6 (1.0),
  lexical 4/6,
detection 6/6;
  Mizuki_Yuuki,
  Barron12312,
  mone all fully repaired
(three dispatched,
  three budget-skipped).
  Mizuki_Yuuki is a repeat
draw from run 26 and both seeds judged restored BOTH times
(including its 18-word needle):
  reproducibility now shown on the
restored side as well as the partial side.
  mone adds two more
unanimous judge=restored on low overlap (5/12,
  3/11 words).
run 31 (2026-07-18,
  1818 s):
  judge 3/4 (0.75,
  one partial),
lexical 4/4,
  detection 4/4;
  Anilovr and yingying repaired (two
dispatched,
  four budget-skipped).
  Anilovr seed 0 judged PARTIAL
again (runs 29 and 31,
  lexical over-crediting it both times):
  the
inversion reproduces,
  third stable-partial seed alongside Acheron
seed 0 and CuspariaKLSY seed 1.
  yingying reproduced run 25's
double-restored.
run 32 (2026-07-18,
  2044 s):
  judge 6/6 (1.0),
  lexical 5/6,
detection 6/6;
  Mizuki_Yuuki,
  SU5ZI2MO1,
  a2581911655 all fully
repaired (three dispatched,
  three budget-skipped).
  Mizuki_Yuuki is
three-for-three across independent draws;
  SU5ZI2MO1's 20-word
needle restored again on repeat;
  a2581911655 seed 0 is another
under-credit (8/18 words,
  unanimous judge=restored).
run 33 (2026-07-18,
  1444 s):
  judge 4/4 (1.0),
  lexical 3/4,
detection 4/4;
  MioCardMeow and AkiraComplex repaired (two
dispatched,
  four budget-skipped).
  MioCardMeow three-for-three
across draws;
  AkiraComplex reproduced run 26 exactly including the
same 4/10 lexical under-credit on seed 0.
  First enriched artifact
(carries repairedText) but no partial verdicts to analyze this run.
run 34 (2026-07-18,
  1542 s):
  judge 5/6 (0.83,
  one partial),
lexical 3/6,
  detection 6/6;
  keyword233,
  mone,
  Mizuki_Yuuki repaired
(three dispatched,
  three budget-skipped).
  mone reproduced run 30's
double-restored double-under-credit;
  Mizuki_Yuuki restored on its
FOURTH draw.
  New partial:
  keyword233 seed 0,
  15 content words,
6 returned.
FIRST ENRICHED-ARTIFACT ANALYSIS (partial-needle-analysis.ts,
  zero
quota):
  keyword233's partial needle is a SINGLE sentence;
  its
ordered coverage bitmap is 000100101110001,
  scattered mid-sentence
coverage,
  so the residual failure mode is within-sentence
paraphrase of one long sentence,
  NOT head-truncation and NOT
dropped trailing sentences.
  OPEN HYPOTHESIS:
  if the deleted EN
sentence embellished beyond the zh source,
  a faithful zh-anchored
editor cannot restore those words and "partial" is the correct
ceiling for that seed;
  testable later by asking the judge ensemble
whether each needle is fully derivable from zh.
run 35 (2026-07-18,
  2404 s):
  judge 4/4 (1.0),
  lexical 2/4,
detection 4/4;
  ttttsuuukikoo_ and CutOceanHeyFis1 repaired (two
dispatched,
  four budget-skipped).
  Both are repeats reproducing
earlier verdicts:
  ttttsuuukikoo_ (run 22's original under-credit
case) restored again;
  CutOceanHeyFis1 matched run 27 including the
same judge=restored-lexical=absent split on seed 0.
run 36 (2026-07-18,
  1554 s):
  judge 2/4 (0.50,
  both akasa_musha
seeds partial),
  lexical 2/4,
  detection 4/4;
  akasa_musha (new entry)
and CutOceanHeyFis1 (third draw,
  reproduced again) repaired.
akasa_musha seed 0 is the largest needle yet (23 content words over
three sentences);
  seed 1 is another inversion (lexical=restored
8/13,
  judge=partial).
ENRICHED ANALYSIS:
  akasa_musha seed 0's three sentences covered
0.45/0.57/0.60 (none dropped whole);
  seed 1 single sentence at
0.69 yet still judged partial.
  All three enriched partials to date
show the same signature:
  scattered within-sentence paraphrase,
  no
head-truncation,
  no sentence-dropping;
  the judges hold a high bar
even at 0.69 word coverage.
JUDGE ACCUMULATED (repairable universe):
  59/70 (0.84) over 15 runs
(22 to 36).
  Lexical over the same runs 45/70 (0.64).
  Detection
142/148 (0.96).
