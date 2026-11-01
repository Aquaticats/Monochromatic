# Translation repair history, first handover document: corpus passes one to three, three pipeline iterations and the translation policy directive, 2026-07-23

The first corpus pass driver and its first runs,
the user directive to fix and restart,
and the three iterations that followed:
screening false non-translation votes,
paragraph-bound slices,
and a wider union of claims,
with the sentinel probes and gates that validated each.
The same day the user stated the first translation policy rules:
a third language is kept beside its gloss,
and emotional completeness outranks word-for-word correspondence;
the policy gates on Anilovr and the ruling on the original's own language are here too.

Part of [Translation repair history: the first handover document, to 2026-08-15](translation-repair-history-first-handover-to-2026-08-15.md),
which lists every file of this record.

Historical evidence only:
no next-action list here is a current instruction,
and the stop condition in the index applies.

## First corpus pass driver, runs 001 and 002 and the first false block

DRIVER BUILT AND LAUNCHED (2026-07-23):
`~/temp/translation-repair-corpus/corpus-pass-driver.ts` (home temp,
not the session scratchpad,
  because milestone-two's scratchpad
drivers evaporated with their sessions and the user grades from
these artifacts later).
  It imports the built dist as a real
consumer,
  lists people at the pinned SHA,
  excludes `tdor` (measured:
neither `page.md` nor `page.en.md` at the pin,
  so the universe is
exactly the 92 pairs),
  skips entries with existing artifacts,
orders by fewest attempts then listing order,
  dispatches while
elapsed < 25 min with 4-minute per-call deadlines and a 45-minute
plain-timer outer net (never `AbortSignal.timeout` composition on
Node 26),
  roster all seven critics and panelists,
  GLM-5.2 editor,
GLM-5.2/Qwen/Kimi checkers,
  `perModelConcurrency: 1`,
  and writes
one full artifact JSON per entry (completion marker) plus a TALLY
stdout line per entry (status,
  issue counts,
  findings,
  wall).
Plan mode (`--plan`,
  zero quota) verified:
  92 pending.
Run 001 launched 2026-07-23 ~14:40 local,
  log
`~/temp/translation-repair-corpus/run-001.log`.
Per-run procedure:
  read TALLY lines,
  append content-free run record
here,
  commit,
  push,
  relaunch until every entry has an artifact.
Corpus-pass run log (all counts,
  no content):
run 001 (2026-07-23,
  2009 s):
  3 dispatched,
  3 completed,
  0 failed;
Acheron repaired (46 issues,
  45 accepted,
  45 resolved,
  5 findings,
911 s);
  AkiraComplex repaired (9/9/9,
  1 finding,
  479 s);
AmbeR_the_anpa repaired (23 issues,
  22 accepted,
  21 resolved,
3 findings,
  620 s).
  Remaining 89.
run 002 (2026-07-23,
  2064 s):
  2 dispatched,
  2 completed,
  0 failed;
Anilovr repaired (78 issues,
  70 accepted,
  69 resolved,
  12 findings,
1420 s,
  largest issue count of the pass so far);
  Aniloviraw
blocked-non-translation (34 issues,
  31 accepted,
  resolution never
ran,
  644 s),
  and a zero-quota probe rules this the FIRST FALSE
BLOCK:
  unlike XIEPT2/shi_Yumiaoya the en page carries zero CJK,
sizes match (~1.5 KB both sides),
  front matter and body translate
line for line on inspection.
  The log shows 4 of 7 critics cast
critical non-translation votes on the single chunk while the same
stage produced 59 claims;
  so the block logic worked as designed
(ensemble agreement) but the critic-level non-translation
classification is noisy on divergence-heavy quote-fragment diary
content.
  Decision:
  measurement continuity,
  pipeline unchanged
through the pass;
  false blocks get probed and tallied per entry,
and block calibration (for example requiring voters to file no
substantive claims themselves) is a named post-pass workstream.
Remaining 87.

## Directive to improve the pipeline and restart every pass

USER DIRECTIVE (2026-07-23,
  supersedes the measurement-continuity
freeze recorded above):
  iteratively improve the system whenever a
change is highly confident to improve it,
  and RESTART ALL PASSES
after each such change;
  prior pass artifacts are discarded.
  Standing
procedure from here:
  discovery -> high-confidence fix -> unit tests
-> live sentinel validation where prior behavior is known -> commit
-> wipe artifacts -> restart pass numbering.

## First iteration: screening non-translation votes against contradicting claims

FIRST ITERATION (commit `6f11683fd`):
  deterministic contradiction
screening for non-translation votes.
  Threshold ideas failed
measurement first (length ratio:
  genuine pair Zha_Ke runs 16x en
over zh while correctly blocked XIEPT2 sits at 6.4x;
  CJK residue:
genuine shihai4h carries 6.8 percent versus correctly blocked
shi_Yumiaoya's 5.1).
  The surviving discriminator comes from the
category's own definition (wholly unrelated pair):
  validated claims
that critique translated content (category leaf outside omission/
untranslated/non-translation) and anchor at least one span into the
TARGET side contradict the votes deterministically.
  Aniloviraw
measured 44 such claims (and its panel had ACCEPTED a critical
non-translation issue,
  so panel routing alone would not have saved
it);
  floor set at 8 (ensemble-scale margin over a seven-critic
roster).
  New module `non-translation-evidence.ts`:
`assessNonTranslationEvidence` (verdict + count) and
`screenNonTranslationVotes` (dismisses contradicted votes together
with their non-translation claims pre-aggregation,
  emitting a
finding);
  `repairChunk` screens after the critic stage and exposes
`nonTranslationContradicted`;
  `repairTranslation` blocks only on
uncontradicted votes.
  NON_TRANSLATION_BLOCK_VOTES moved to the new
module (barrel path updated).
  Verification:
  87 suites pass against
dist including the new contradicted-path end-to-end test,
  oxlint
0/0 (max-lines remediated by moving screening logic into the new
module,
  never by raising the limit),
  lint:types exit 0.
LIVE SENTINEL PROBE:
  ALL PASS (2026-07-23,
  log
`~/temp/translation-repair-corpus/sentinel-probe.log`,
  artifacts
kept under `probe/`).
  XIEPT2 stayed blocked (blocked chunk:
  5
claims,
  4 votes,
  165 s);
  shi_Yumiaoya stayed blocked (blocked
chunk:
  7 claims,
  5 votes;
  its translated chunk drew 35 claims with
zero votes,
  983 s);
  Aniloviraw REPAIRED (4 votes reproduced across
independent runs,
  dismissed against 37 content-critique claims,
655 s).
  The floor of 8 sits in a wide gap:
  correct blocks at 5 and
7 total claims,
  the false block at 37 content-critique claims.

## Pass 2 restarted from zero and its first run

Run 004 had been stopped mid-flight;
  PASS 2 started from zero
(all pass-1 artifacts and attempts wiped,
  92 pending) under the
fixed pipeline,
  logs `pass2-run-NNN.log`.
Pass 2 run log (all counts,
  no content):
pass2 run 001 (2026-07-23,
  1502 s):
  2 dispatched,
  2 completed,
  0
failed;
  Acheron repaired (48 issues,
  46 accepted,
  45 resolved,
  7
findings,
  1012 s);
  AkiraComplex repaired (9 issues,
  8 accepted,
  6
resolved,
  3 findings,
  491 s).
  Both reproduce their pass-1 statuses
with nondeterministic count drift.
  Remaining 90.

## Second iteration: paragraph-bound slices

SECOND ITERATION (2026-07-23,
  commit `c3ea27b23`),
  root cause named
by the user:
  multi-LLM value is OVERLAPPING coverage (A finds a b c,
B finds b c d),
  and each critic satisfices at 10 to 14 claims per
call regardless of defect density (the user counts 200+ issues on
Anilovr at first glance versus 33 found).
  Measured corpus-wide:
67 to 84 percent singleton issues across all pass-2 artifacts,
  so
losing a voice loses its findings nearly one-for-one;
  the earlier
full-roster-retry idea survives only as secondary hardening,
  not
the fix.
  Root-cause fix:
  PARAGRAPH-BOUND SLICES (user decision:
paragraph-bound,
  never sentence-bound,
  because sentence windows
reward mechanical one-to-one rendering over meaning and emotion).
`slice-pair.ts` subdivides each aligned section pair into
budget-bound node runs (never splitting a block node;
SLICE_CHAR_BUDGET 400 target chars;
  source budget scales by
character share so the denser zh side cannot collapse pairing back
to section scale,
  a flaw the unit test caught);
  the whole loop runs
per slice.
  Evidence base:
  on DarlinChit-scale units the ensemble
produced a 28-member agreement cluster,
  so small units yield both
thoroughness and overlap.
  CONTRACT CHANGE,
  dominance block:
  a
2-vote tiny slice must not block a document,
  so
`blocked-non-translation` now fires only when standing-vote slices
dominate target characters (`assessNonTranslationDominance`);
minority standing slices ship unchanged with findings (per-slice
degradation,
  matching the settled architecture's never-document-wide
rule).
  Consequence recorded:
  shi_Yumiaoya's expected production
outcome changes from blocked to not-blocked with its untranslated
region degraded per slice.
  `repairChunk` early-exits standing-vote
slices before panel and editor spend (types moved to
`repair-contract.ts` for the line budget).
  Secondary:
  critic stage
retries to FULL ROSTER (`retryTarget: 'full-roster'` in
`stage-quorum.ts`),
  voting stages keep quorum;
`stage-roster-incomplete` finding records shortfalls.
  Verification:
89 suites pass against dist (slicing byte-exactness,
  dominance,
per-slice degradation end to end),
  oxlint 0/0,
  types clean.
Sentinel probe 2 launched (four entries:
  XIEPT2 must block via
dominance,
  shi_Yumiaoya must NOT block under the new contract,
Aniloviraw repaired,
  Anilovr measures thoroughness against its
33-issue section-scale baseline with the user's 200+ as reference;
log `sentinel-probe-2.log`).
  Pass 3 restarts from zero on ALL PASS
plus a decisive Anilovr thoroughness gain.

## Third iteration: widening the union of claims

THIRD ITERATION (2026-07-23,
  commit `666d87602`),
  user question "is
our union algorithm good" then directive "LLMs must act as part of
union algorithms".
  Findings from code plus measurement over pass-2
artifacts (192 issues,
  3496 same-chunk issue pairs):
  exact dedupe
can never merge cross-critic claims (free-text summaries differ),
so clustering is the only cross-critic union;
  the LLM half already
exists (panel ballots carry a sameDefect opinion per multi-member
cluster,
  majority merges,
  silence and ties split conservatively
because a wrong merge hides a defect),
  and the 243 same-family
overlapping-but-separate issue pairs are the panel's judged splits
working as designed.
  The real gap:
  the family gate in
`claimsShareDefect` kept 62 overlapping cross-family pairs
(accuracy vs terminology 23,
  accuracy vs fluency 12,
  accuracy vs
extension 9...) from ever reaching panel judgment.
  Fix:
  proposals
now arise from same-side span overlap alone;
  neither family nor
severity pre-decides,
  the panel disposes every proposal.
  Also
measured:
  only 3 of 192 issues carry single-side evidence,
  so the
same-side overlap requirement is not a material union gap.
  KNOWN
LIMIT recorded:
  disposal is binary per cluster (merge all members
or split to singletons),
  so a widened mixed cluster judged "not one
defect" splinters exactly as today,
  no regression;
  per-sub-group
disposal is the recorded refinement if graded evidence demands it.
Verification:
  89 suites pass,
  oxlint 0/0,
  types clean.
  The running
sentinel probe loaded the pre-union dist at process start and stays
internally consistent;
  Anilovr gets one re-run on the
current-tip pipeline for the thoroughness gate before pass 3.

## Naming rule for pipelines, gates and artifacts

NAMING RULE (user directive 2026-07-23):
  never call any pipeline,
gate,
  or artifact "final";
  the system is early in polishing and
every pass is one iteration among many.
  Say current-tip,
this-iteration,
  or name the commit.

## Full-roster critic retries reverted and the second sentinel probe

FULL-ROSTER CRITIC RETRIES REVERTED (2026-07-23,
  commit
`78317a93c`,
  user decision "we shouldn't retry everything until ALL
respond" after the probe ran 70 minutes).
  Measurement vindicated
the concern only partially:
  critics answered 7/7 first-round in
nearly every probe slice (one critic retry fired in the whole log),
so the revert costs little;
  the REAL wall-time sink is the panel,
where the same four voices hit the full 240 s deadline slice after
slice before the retry recovers them.
  The full-roster mechanism
stays in stage-quorum.ts,
  tested but unused.
  SENTINEL PROBE 2:
  ALL PASS
(2026-07-23,
  pre-union dist,
  log `sentinel-probe-2.log`,
  artifacts
under `probe/`).
  XIEPT2 blocked in 565 s via dominance early exit
(0 issues,
  24 findings);
  shi_Yumiaoya REPAIRED with 79 issues under
the new contract (untranslated-region slices degraded per slice);
Aniloviraw REPAIRED,
  69 issues (up from 44 at section scale);
Anilovr REPAIRED,
  130 issues in 2681 s.
THOROUGHNESS GATE MET:
  Anilovr 130 issues versus its 33-issue
section-scale baseline,
  a 3.9x gain moving decisively toward the
user's first-glance estimate of 200+;
  per-entry wall time roughly
doubled (1358 s to 2681 s),
  the expected slicing cost.

## Union gate on Anilovr, the under-merge check and the halted third pass

CURRENT-TIP GATE (2026-07-23,
  Anilovr re-run on the union-widened,
quorum-retry tip,
  log `gate-anilovr-union.log`):
  PASS,
  repaired,
121 issues in 2547 s.
  Consolidation from 130 is the union working
(3 MIXED-FAMILY merged issues,
  structurally impossible before the
widening),
  not lost findings.
UNDER-MERGE CHECK,
  offline and zero quota:
  singleton share rose to
89 percent at slice scale,
  so near-miss anchoring was measured
directly.
  Same-slice issue pairs by nearest-span gap:
  361
overlapping (proposed and SPLIT by the panel,
  its conservative
disposal working),
  309 within 1 to 20 chars,
  324 within 21 to 60.
Sampling the 1-to-20 band showed every pair is a genuinely
DISTINCT defect on adjacent text (untranslated Esperanto phrase
beside an added sentence;
  preposition error beside a CJK
quotation-mark convention issue;
  heading mistranslation beside a
nuance shift),
  so proximity inside a small slice is adjacency,
  not
duplication.
  Widening the merge neighborhood would OVER-merge.
Verdict:
  the union algorithm is sound at slice scale and the high
singleton share reflects real defect density;
  no further union
iteration warranted on current evidence.
PASS 3 STARTED then STOPPED after two entries (2026-07-23):
  the
translation-policy directive below landed while it ran,
  so its
artifacts were discarded rather than spend quota on a superseded
prompt.

## Translation policy directive: foreign phrases and naturalness

TRANSLATION POLICY,
USER DIRECTIVE (2026-07-23,
commit
`4bab4412c`).
Two standing rules,
now baseline prompt policy (the
architecture always held that policy files are optional and the
system must work without them;
these are the first policy rules
the user has stated directly):
1. A phrase the ORIGINAL writes in a language other than its own
   keeps that original wording in the TRANSLATION and carries its
   meaning ALONGSIDE,
   side by side.
   Never replaced by meaning
   alone,
   never left bare.
   New category
   `policy/foreign-phrase-gloss`,
   distinct from
   `accuracy/untranslated` because the remedy differs:
   gloss beside
   preserved wording,
   not replacement.
   Prompted by a real finding
   on Anilovr,
   where a critic flagged an Esperanto line as
   `accuracy/untranslated`,
   whose remedy would have destroyed the
   original wording.
2. Prioritize emotional completeness and naturalness over
   one-to-one meaning correspondence.
   Critics must not report
   non-literal renderings as defects,
   and must report flattened
   voice,
   warmth,
   humor,
   irony,
   grief,
   or intimacy as the new
   `style/emotional-flattening`;
   stiff literal renderings are
   `style/awkward-phrasing` even when every word matches.
   Editors
   recast wording,
   sentence boundaries,
   and clause order freely to
   serve the feeling.
   Two guards keep this from becoming license:
   "Naturalness never licenses dropping content" pairs with the
   existing clause-enumeration rule,
   and "Never introduce content
   the ORIGINAL does not support" stays (milestone two measured
   embellishment as a real failure mode).
Taxonomy growth is safe:
   `remapCategoryLeaf` derives owners from
the category list,
   so no exhaustive map needed updating.
Verification:
   91 suites pass,
   oxlint 0/0,
   types clean;
   the prompt
suites now assert both policies in both directions.
POLICY GATE,
   Anilovr (2026-07-23,
   log
`gate-anilovr-policy.log`):
   PASS,
   repaired,
   114 issues,
   2608 s.
The policy fired as designed:
   5 `policy/foreign-phrase-gloss` and
5 `style/emotional-flattening` claims,
   ZERO `accuracy/untranslated`
(the miscategorization that prompted the directive is gone),
   and
the Esperanto line ships preserved with its meaning beside it:
`//La homa mondo devus esti detruita (The human world should be
destroyed)//`.
   Exactly the requested side-by-side shape.
ONE OVER-APPLICATION MEASURED:
   the same stylized quote kept its
CJK clause with a gloss (`我会在参宿四上等你 (I will wait for you at
Betelgeuse)`),
   but Chinese is the ORIGINAL's own language,
   not a
foreign phrase,
   so the rule does not cover it.
   Scope is bounded:
whole-page CJK went 6 to 15 chars,
   one phrase,
   one line;
   the only
other CJK line holds proper names already present in the input.
Risk if systematic:
   preserved CJK in English pages is exactly the
`accuracy/untranslated` signal the non-translation detector reads,
so unchecked spread could interact with blocking.
   Resolution is a
values question about memorial presentation (the quote is the
person's own last words),
   not a measurable one,
   so it went to the
user.
USER PICK (2026-07-23):
   render source-language text into English
like ordinary prose;
   only genuinely third languages get
preserve-plus-gloss,
   including inside quotations and stylized
multilingual lines.
   Rejected alternatives recorded:
   preserve CJK
with a gloss everywhere,
   and a quoted-blockquote-only carve-out.
SCOPING FIX (commit `4b8fd64c8`):
   both prompts now carry the
explicit negative,
   "The ORIGINAL's own language is never such a
phrase",
   with the quotation and stylized-line cases named so the
exception cannot spread;
   prompt tests assert it in both files.
Verification:
   91 suites,
   oxlint 0/0,
   types clean.
   Second policy
gate on Anilovr (log `gate-anilovr-policy2.log`):
   PASS,
   repaired,
92 issues,
   2795 s.
   Both goals confirmed and no over-correction:
the Esperanto stays glossed (4 gloss claims,
   gloss retained
`//La homa mondo devus esti detruita// (The human world should be
destroyed)`),
   and the stylized quote's Chinese clause now renders
to English ("I will wait for you at Betelgeuse").
   Whole-page CJK
went 6 to 0:
   the editor also romanized two proper names the input
carried in characters (方方 to Fang Fang,
   铃木真依 to Mai Suzuki),
flagged as one `accuracy/untranslated` and resolved.
   That is
consistent with the render-source-into-English pick,
   not a new
over-application,
   and it clears the non-translation-detector
interaction entirely (no residual CJK to read as untranslated).
PASS 4 starting from zero on this tip.

## Pass 2 runs 002 and 003

pass2 run 002 (2026-07-23,
  1737 s):
  2 dispatched,
  2 completed,
  0
failed;
  Aniloviraw REPAIRED (44 issues,
  40 accepted,
  40 resolved,
14 findings,
  379 s):
  the false block reproduced a THIRD time (4
votes,
  48 content-critique claims) and the screen dismissed it in
production,
  exactly as designed;
  Anilovr repaired (33 issues,
  25
accepted,
  25 resolved,
  13 findings,
  1358 s).
  The pass-1 78 versus
pass-2 33 issue swing decomposes mechanically from the logs (user
challenged the first "variance" label;
  measured on challenge):
pass-1 big chunk ran 7/7 critics (95 claims) into a
quorum-degraded 4/7 panel (72 issues,
  acceptance at 3 of 4 heard),
pass-2 ran 5/7 critics after a forfeit-retry (57 claims,
  per-critic
volume nearly identical) into a full 7/7 panel (27 issues).
  Claims
barely converge across critics on this entry (95 claims to 72
distinct issues,
  mostly singletons),
  so each lost critic removes
its singletons,
  and the thin pass-1 panel is the outlier side.
Screening uninvolved:
  zero non-translation votes on Anilovr chunks
in both passes.
  No iteration triggered:
  quorum design worked as
built,
  and thin-panel volatility errs toward fewer accepted issues,
the safe direction for a precision gate;
  revisit only if the graded
sample shows precision misses clustering in degraded-panel entries.
Remaining 88.
run 003 (2026-07-23,
  1913 s):
  3 dispatched,
  3 completed,
  0 failed;
Arita repaired (20 issues,
  18 accepted,
  17 resolved,
  6 findings,
644 s);
  ArtsEpiphany unchanged (0 claims from 7/7 critics,
  8 s;
measured:
  the pair is a 120-byte front-matter stub each side,
  so
zero claims is correct,
  not a silent failure);
  BI4PBV REPAIRED
(31 issues,
  30 accepted,
  28 resolved,
  3 findings,
  1260 s):
  the
milestone-two quarantine entry that forfeited all seven models in
two independent benchmark runs completed 7/7 on all three chunks
at first attempt;
  the spiral was provider weather plus seeded-text
conditions,
  and the production pass holds no quarantine list.
Remaining 84.
