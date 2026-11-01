# Translation repair history, first handover document: pass 6 coverage bar, resume-first ordering and runs 011 to 019, 2026-07-25

The change of the completion bar to stratified size bands,
the resume-first ordering and its first production validation,
and the run records from 013 to 019,
including the externally killed launches that resumability absorbed.

Part of [Translation repair history: the first handover document, to 2026-08-15](translation-repair-history-first-handover-to-2026-08-15.md),
which lists every file of this record.

Historical evidence only:
no next-action list here is a current instruction,
and the stop condition in the index applies.

## Coverage bar changed to stratified size bands

COVERAGE-BAR DECISION (2026-07-25,
   user-approved):
   task 30's completion bar
changed from "all 92 settled" to STRATIFIED REPRESENTATIVE COVERAGE
(~10/10/10 across small/medium/large size bands).
   Rationale:
   the M3 gate is
human-graded precision on a uniform 50-issue sample;
   12 settled entries
already yield hundreds of accepted issues,
   so sample SIZE was satisfied long
ago and the full-92 pass was never load-bearing for it.
   What the gate needs
is a REPRESENTATIVE sample,
   and the pool was skewed small -- 9 small / 2
medium / 1 large by page.md byte-size tertiles (small <=1.8KB,
   medium
1.8-3.6KB,
   large >=3.6KB up to 40.7KB),
   with the single large entry sitting
at the bottom of its band.
   The heavily-sliced large tail -- exactly where
the lockstep slicing fix most affects precision -- was un-sampled.
   Target:
~10 settled per band (need ~1 small,
   ~8 medium,
   ~9 large),
   large picks
spread across the band;
   then draw the 50-issue sample STRATIFIED by band.
The skew is a throughput artifact:
   small entries finish inside one run while
large ones consume it,
   so equal wall-clock settles many small + few large.
DRIVER FIX (same day,
   DRIVER-ONLY so NO restart/wipe -- repair outputs per
entry are identical):
   corpus-pass now sorts the small band LAST in `pending`
(`SMALL_PAGE_BYTES=1843`,
   measuring page-source bytes via TextEncoder),
then fewest-attempts within a band,
   so run budget flows to medium+large;
small still settles once larger bands are served (deprioritize,
   not
exclude).
   Verified format/lint/types 0/0/0 and `--plan` first-5 pending all
non-small (Everythings99 1859B .. Huasheng 7397B).
   Run 011 was already in
flight on the old order and finishes normally;
   run 012 onward uses the new
order.
   Task 30 subject/description updated to match.

## Pass 6 runs 011 and 012: resume-first ordering validated

PASS 6 RUN 011 (2026-07-25,
   tip `e032fa453`,
   hit the per-entry 90-min cap):
0 settled,
   still 12/92.
   Dethelly (6171B,
   large band) processed 13
chunks/slices then aborted at the hard ceiling (TALLY status=ERROR
aborted=true,
   Timeout).
   FIRST cap-abort since resumability landed:
   all 13
finished slices persisted to `slice-cache/Dethelly/` and the cache was
correctly RETAINED on abort (discard is success-only).
   This exposed an
ordering flaw -- `attempts[Dethelly]` incremented to 1 and the within-band
tiebreak was fewest-attempts-first,
   so Dethelly sorted BEHIND every
0-attempt non-small entry;
   run 012 would have started a fresh entry and
left the 13 cached slices idle,
   and every big-large entry would take one
partial attempt with none finishing,
   starving exactly the entries that most
need resume and defeating the "large spread across band" goal.
RESUME-FIRST FIX (driver-only,
   no restart):
   added `listResumableEntries`
(slice-cache-store.ts) returning ids whose cache dir holds >=1 finished
slice;
   corpus-pass now sorts those FIRST (before band,
   before attempts) so
an in-flight large document finishes before a fresh one starts.
   Safe against
livelock because `repairChunk` never throws -- every failure path (votes
stand / no claims / no envelopes / no surviving ops / lost voices) returns
an unchanged outcome that gets persisted,
   and a cap-abort always completes
>=1 new slice;
>the only residual (a deterministic pure-function throw at
some slice) would surface as a repeated same-entry ERROR and is caught by
per-run inspection,
>not silently absorbed.
>Verified format/lint/types 0/0/0
and `--plan` first=Dethelly,Everythings99,... (resumable sorts first).
VERIFICATION CAVEAT (do NOT overstate):
>only the PERSIST-on-abort half of
resumability is proven in production (the 13 files exist).
>The RESUME half
-- next run reads them back,
>skips them with ZERO model calls,
>continues on
new slices,
>settles,
>and DISCARDS the cache -- has never run.
>Run 012 is its
first real test.
>Watch run 012 for:
>Dethelly starts near-instantly,
>no
critic/panel/editor/checker calls on the 13 cached chunks,
>continuation on
chunk 13+,
>a settle,
>then `slice-cache/Dethelly/` GONE.
>Only after seeing
that is resumability end-to-end validated.
>Run 012 launched.
PASS 6 RUN 012 (2026-07-25,
>tip `90809eeed`,
>3958s wall ~66 min):
>1 settled,
13/92.
>RESUMABILITY END-TO-END VALIDATED IN PRODUCTION.
>Resume-first put
Dethelly at the front (START/--plan confirmed);
>the log shows `6 chunk
pairs, 24 slices`,
>the 13 cached slices (chunks 0-12) skipped with ZERO
stage/model logs,
>the FIRST critic stage firing at chunk 13,
>then
continuation through slice 23,
>then TALLY status=repaired (391 issues,
>373
accepted,
>364 resolved,
>100 findings).
>Post-settle the `Dethelly.json`
artifact is present AND `slice-cache/Dethelly/` is GONE (cache root empty):
the full skip->continue->settle->discard cycle observed.
>Both halves now
proven -- persist-on-abort (run 011) and resume+discard (run 012).
>The
resume-first ordering also proved correct:
>run 012 processed only Dethelly
(it started past the 25-min soft budget) and ended.
>Dethelly is large-band,
so large settled 1->2 (Chinatsu_Suzuki ~5.2KB,
>Dethelly ~6.2KB -- both
mid-large;
>bigger large entries still needed for band spread).
>Run 013
launched (no resumable entries remain,
>so it picks the first non-small
0-attempt entry by band order).

## Pass 6 runs 013 to 019

PASS 6 RUN 013 (2026-07-25,
>tip `54eb8c323`,
>1529s wall ~25 min):
>1 settled,
14/92.
>Everythings99 repaired (32 issues,
>31 accepted,
>31 resolved,
>1
finding);
>settled in one run,
>no cap,
>cache empty after.
>First non-small
0-attempt entry by the new band order (1859B,
>bottom of the medium band),
so medium settled 2->3.
>Bands now 9 small / 3 medium / 2 large.
>Run 014
launched.
PASS 6 RUN 014 (2026-07-25,
>tip `a70cb3592`,
>4867s wall ~81 min):
>1 settled,
15/92.
>Futajuhuacha repaired (5448B/large,
>248 issues,
>243 accepted,
>238
resolved,
>59 findings);
>settled in ONE run just under the 90-min cap,
>no
abort,
>cache empty after.
>Large settled 2->3 (a large entry that fits the
single-run budget -- only the biggest large entries, >~6-12KB,
>need resume).
Bands now 9 small / 3 medium / 3 large.
>Run 015 launched.
PASS 6 RUN 015 (2026-07-25,
>tip `3a119f095`,
>5194s wall ~87 min):
>1 settled,
16/92.
>Huasheng repaired (7397B/large,
>235 issues,
>227 accepted,
>226
resolved,
>40 findings);
>settled in ONE run right at the 90-min cap ceiling.
Large settled 3->4 (all four are mid-large 5.2-7.4KB;
>entries above ~7.4KB
will start capping and needing resume).
>Bands now 9 small / 3 medium / 4
large.
>OPERATIONAL NOTE:
>run 015's FIRST launch was chained onto the commit
with a bare `&` (untracked) -- no completion notification,
>and the tracked
watcher I added was itself killed after ~1 min.
>Fixed by killing the
untracked run (0 slices lost,
>killed mid-first-chunk on GLaDOSister;
>its
empty cache dir removed) and relaunching run 015 as its own
`run_in_background` task.
>GLaDOSister carries a wasted attempt=1 from that
kill,
>so it sorts one slot behind the 0-attempt entries (harmless,
>still a
needed medium).
>RULE:
>launch corpus-pass ONLY as a standalone tracked
background command,
>never a bare `&` chained after another command.
>Run 016
launched.
PASS 6 RUN 016 (2026-07-25,
>tip `8eddd3906`,
>4016s wall ~67 min):
>1 settled,
17/92.
>Jennife80677612 repaired (3859B/large-bottom,
>84 issues,
>78 accepted,
76 resolved,
>11 findings);
>one run,
>no cap.
>Large settled 4->5 (all five
still lower-large 3.9-7.4KB;
>bigger large entries still ahead).
>Bands now 9
small / 3 medium / 5 large.
>Run 017 launched.
PASS 6 RUN 017 (2026-07-25,
>tip `7719f975f`,
>~77 min effective on the third
launch):
>2 settled,
>19/92.
>KILL SAGA + resume-across-external-kill validated.
Run 017's first launch (tracked) ran ~16 min on Katerina,
>finishing chunks
0-2,
>then was KILLED (external,
>non-resource:
>load ~1.3,
>33Gi free,
>no OOM);
Katerina's 3 slices persisted.
>Relaunch died at START (~1 min,
>another
transient kill),
>zero progress,
>3 slices intact.
>THIRD launch survived ~77
min and settled TWO entries:
>Katerina RESUMED (`8 slices`,
>first completion
`chunk 3` -- the 3 cached chunks skipped with zero model calls,
>so resume is
now proven across an EXTERNAL KILL,
>not just a cap-abort;
>~18 min,
>70 issues
66 accepted 65 resolved) then Kotori fresh (~59 min,
>138 issues 129 accepted
126 resolved).
>Both are medium (Katerina 2.2KB,
>Kotori 2.4KB),
>so medium
3->5.
>LESSON:
>the intermittent background-task kills seen this session (runs
010/011 first launches,
>a watcher,
>run 017 x2) are HARMLESS under
resumability -- each costs at most the in-flight slice;
>on a `killed`
notification just relaunch and resume-first continues the entry.
>Recomputed
band totals over all 19 settled:
>small 9/30 (need ~1),
>medium 5/31 (need
~5),
>large 5/31 (need ~5);
>large still all lower-band 3.8-7.2KB,
>bigger
large entries (9-40KB,
>will cap and need multi-run resume) still ahead.
>Run
018 launched.
PASS 6 RUN 018 (2026-07-25,
>tip `0fa06bd7e`,
>3096s wall ~52 min):
>1 settled,
20/92.
>LCG_Akiball repaired (large,
>89 issues,
>83 accepted,
>82 resolved,
>8
findings);
>one run,
>no cap.
>Large settled 5->6.
>Bands now 9 small / 5 medium
/ 6 large (need ~1 small,
>~5 medium,
>~4 large).
>Run 019 launched.
PASS 6 RUN 019 (2026-07-25,
>tip `5063a3740`,
>4254s wall ~71 min effective;
first launch killed at START,
>0 loss,
>empty MTF_0615 cache dir removed):
1 settled,
>21/92.
>MeowBot233 repaired (medium,
>206 issues,
>198 accepted,
>198
resolved,
>23 findings);
>one run,
>no cap.
>Medium settled 5->6.
>Bands now 9
small / 6 medium / 6 large (need ~1 small,
>~4 medium,
>~4 large).
>Run 020
launched.
