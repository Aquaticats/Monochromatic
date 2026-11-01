# Roster calibration: the XIEPT2 launches, 2026-09-02 night to 04:14 UTC on 2026-09-03

XIEPT2 refused whole over one unfilled passage;
the run that exhausted Synthetic's week and the 429 hold that stalled Hyper;
the request-rate limit that led to paced calls,
and the landing at 04:14 UTC on the paced client.

Part of [Roster calibration under the owner blocklist, 2026-09-01](translation-repair-roster-calibration-2026-09-01.md),
which lists every file of this record.

## XIEPT2 ran 35 minutes at overlap 4 and was refused whole over one unfilled passage

`TALLY XIEPT2 status=INCOMPLETE ms=2106786 error=entry XIEPT2 retains 1 unfilled source passage(s) at
slices 15` on `ae39bf22e`,
23:10 to 23:45 UTC,
0 refusals,
0 holds,
the archive-block review retained
its block,
no page,
no artifact.
Slice 15 (409 source characters,
no archive text) went through the
translate lane's judged round and its one follow-up;
both tied at weight 1 and 1.5 with three of eight
judges cut at the 60-second window each time,
so no candidate reached the minimum weight
(`no-candidate-backed`) and the passage stayed unfilled.
Then `assertPublishableTranslation`
(2026-08-27,
"refuse incomplete published pages") refused the entry.
The no-loop design of 2026-09-01
(`7e702e075`,
"Insertion placement,
single round") says the page ships without such a passage with the
gap recorded,
and the artifact already records it (`lanes.translate.unfilled`);
the publish test's own
comment records an earlier XIEPT2 attempt lost the same way after four hours forty-eight minutes.
Fixed
under `QDF` (the later stated policy winning over the earlier guard,
as with the naturalness floor):
the pass logs each unfilled passage as `source-passage-unfilled` and publishes;
the refusal stays
exported for callers that want to fail closed.
Recorded for veto.

The window cuts are runaway streams,
measured across today's runs:
398 replies abandoned at the
60-second window had delivered a median 177,741 characters (p90 1,429,821,
max 2,367,634) where a
ballot is a few thousand;
minimax-m3 exceeded 200,000 raw characters on 345 of its 539 completed
replies,
gpt-oss-120b on 77 of 353,
Kimi-K3 on 28 of 285,
Qwen3.8-27B on 23 of 232.
Task 31:
cut a
stream early once it passes a size the stage's reply cannot legitimately reach.

## XIEPT2's fourth launch ran Synthetic's week dry, and the 429 hold then stalled Hyper

Launched 23:52 UTC on `c1db6f729` at overlap 4 into `~/temp/agent/xiept2-rerun3-20260902`.
Synthetic's
weekly meter read 2.34% at launch and 0% by 00:49;
from then on every seat routed to Hyper (balance
4,847).
Hyper answered with 429 bursts of 80 to 131 hits lasting 14 to 21 seconds,
one burst about
every 80 seconds (00:46:22,
00:47:46,
00:49:08,
00:50:24),
and the pass stopped advancing:
72 slice
costs at 00:49 and at 00:51,
in consolidation.
The mechanism,
read off the log:
a 429 on a wet meter
holds the whole provider for the 60-second backoff (`RATE_LIMIT_BACKOFF_MS = BUDGET_FRESH_MS`),
95 calls
waited out a 60-second hold,
and when it ended every pending call fired at once into Hyper's
concurrency limit,
which produced the next burst;
44 calls hit a second both-dry reading inside a new
hold and were lost as `EveryProviderDryError` (then under its two-provider name) (the consolidation polish gate lost Kimi-K3 and
gpt-oss-120b that way at 00:47:33).
A provider-wide hold is the wrong answer to a concurrency limit
when the provider is the only one left:
it converts a per-call retry into a synchronised herd.

By 00:53 the count was 579 429 lines and 137 both-dry losses,
still 72 slice costs,
consolidation
slices settling at about 512 seconds each;
the pass was stopped at 00:54 rather than left to limp for
hours.
Fixed by `57745afcf` (test shown to fail without it):
a refusal on a wet meter holds the provider only when the
other provider is wet (the hold then moves traffic,
which is what it is for);
with the other provider
dry there is no hold,
and each call keeps its own jittered retry ladder in the transport layer.
The
refuser stays routable at once;
the test shows the hold at zero with the other provider dry and at the
backoff with both wet.
XIEPT2 relaunched on that tree,
Hyper alone from the start (Synthetic's week is
spent),
which is the shape `QPW` calls normal.

## XIEPT2's fifth attempt hit Hyper's request-rate limit with no hold at all, so calls are paced now

Launched 00:55 UTC on `57745afcf` at overlap 4 into `~/temp/agent/xiept2-rerun4-20260903`,
Hyper alone
from the start.
It ended at 01:39 (43 minutes) `INCOMPLETE ... provider-unavailable` with 8,632 HTTP
429 lines and no hold anywhere:
with the hold gone,
nothing stood between the seats and the limit.
The 429 bodies all read `You've hit your hourly rate limit. Please try again in 1s` (also 2s,
3s,
4s).
Calls in flight at the moment of a refusal had a median of 4 (max 28),
so it is not the concurrency
limit the earlier section named.
A first reading of the log counted "completed streams" and reached
9,628 successes in 43 minutes,
which made the owner's figure in `hyper-client.ts` ("limits this account
to 1,000 requests per hour") look wrong and led to a per-minute pacer at 600 (`50da3787a`);
the advisor
asked for a per-model clustering,
and the recount off the SPEND lines,
which record only successful
calls,
overturned it:
a refused exchange completes its stream too,
so the first count was successes
plus refusals.
The real numbers:
612 successful Hyper calls in the whole run;
every seat's first
five-times-refused call between 01:00:33 and 01:00:37 (27 successes each before it,
257 in this run,
the rest of the hour's thousand in the run before it,
whose calls from 00:00 were still in the window);
successes per ten minutes after that of 141,
134,
104 and 11,
the trickle at which requests from an hour
earlier left the window;
6,441 retry attempts and 1,487 calls refused five times over,
spread across the
nine seats in proportion to their traffic (218 minimax-m3 down to 73 glm-5.3),
which is one shared
window,
not a per-model one.
The owner's 1,000 requests per rolling hour holds (a full run at that rate
drew no refusal;
whether the ceiling sits higher is unmeasured).
Every seat lost
about half its calls because the transport ladder (four retries at 1,
2,
4,
8 s equal-jitter,
honouring
the body's 1 to 4 s) cannot outwait a window that frees a place every five seconds,
and a call whose
five attempts were all refused reached the router as a provider outage.

Fixed by `50da3787a` and its rework (tests shown to fail without each):
`request-pace.ts`,
a sliding-window
pacer in front of the Hyper transport that lets 1,000 request starts into any rolling hour,
retries and
credit reads included,
and queues the rest in arrival order,
with `TRANSLATION_REPAIR_HYPER_REQUESTS_PER_HOUR`
as the dial;
and the retry ladder now reads the refusal's own "try again in Ns" and waits at least that
long.
The pacer's window starts empty,
so a launch within an hour of a heavy run is refused until that
run's calls leave the window;
rerun4's last calls (01:10 to 01:39) leave it by 02:39.

What 1,000 an hour means for XIEPT2 on Hyper alone:
Carena0442's landed pass made 1,938 requests
(1,648 Hyper) and keyword233's 241;
XIEPT2 is about five times Toka_ls's source,
and rerun4 had made
612 successful calls plus 1,487 lost ones without leaving the translate stage.
A full XIEPT2 pass is
some thousands of requests,
so at the cap it runs several hours,
outside `FIT` (about an hour),
until
Synthetic's week resets or a smaller shape is chosen.
Not a design decision:
the owner's "Run it now at
overlap 4" and `QPW` cover a one-provider launch,
so rerun5 launches paced and the duration is reported
for the veto.

Rerun5 launched 02:04 UTC on `4b5c8b438` (the pacer at 1,000 per rolling hour,
its test rewritten so the
queued-abort guard is the one under test) at overlap 4 into `~/temp/agent/xiept2-rerun5-20260903`,
Hyper
alone;
17 successful calls and no refusal in the first 45 seconds.
Launched before rerun4's last calls
had left Hyper's window (they do by 02:39),
on the reasoning that the pacer's own thousand is spent
only after some ten minutes of the preparation stage,
by which time most of rerun4's calls have left,
and that a refusal in between costs one call's five short retries,
not the run.
A watcher reports
Hyper successes,
refusals,
cuts and slice costs every five minutes and ends on the tally.

The cut-stream cap (the runaway-reply item) is closed on measurement rather than built.
Rerun4's 41
cuts (qwen3.8-27b 20,
kimi-k3 17,
minimax-m3 3,
glm-5.3-flash 1) all had zero content characters,
15,000 to 45,000 reasoning characters and about 70 seconds elapsed:
the round's 60-second post-quorum
grace cutting slow reasoners,
not runaway output.
Raw stream bytes run seventeen times the text
(one SSE frame per few characters),
so the day's "1.43 MB at p90" measured framing.
Answer overrun
already has its own ending (`overrun`,
`maxAnswerChars` on the exchange).
A cap on reasoning would end
the same lost calls seconds earlier and save no requests,
and under a window of 1,000 an hour the
request is the scarce unit,
not the second.

What the cuts are,
read per seat and round kind (both lanes' select rounds log as `judgeTranslateSlate`;
the lane tag splits them):
rerun5 cut Qwen3.8-27B in 30 of 34 translate-lane select rounds and 21 of
24 consolidation-slate select rounds (in the first 25 minutes the only late seat in 7 of 16,
so those
rounds waited the whole 60 s window for nothing),
Kimi-K3 in 17 of 34 and 18 of 24,
minimax-m3 in 7
and 6;
Carena0442's landed pass (1,648 of 1,938 calls on Hyper) cut Qwen in 14 of 19 translate-lane
select,
11 of 19 consolidation-slate select,
21 of 59 repair-select,
13 of 25 critic,
15 of 19 panel,
17 of 19 lane-contest and 7 of 14 consolidation-gate rounds,
113 abandoned rounds against 106
successful calls;
Toka_ls on Synthetic cut it in 3 of 28 select rounds.
Under the owner's standing authorisation to drop
a model from a role on evidence,
and on the GLM-5.3-Flash precedent (12 of 21 translate-select rounds),
Qwen3.8-27B leaves every judge seat (`WIDE_SEAT_DROPPED`,
six seats,
quorum 3;
`LATE_JUDGE_DROPPED`,
seven) and keeps its writer seats;
the seating decision carries the 2026-09-03 addendum.
The seat is
lost to Hyper's serving speed for this model rather than to the model,
so a provider-aware seat is the
open design question.
Kimi-K3 (17 of 34 and 18 of 24 select on rerun5;
11 of 19,
10 of 19 select and
10 of 19 contest on Carena,
under half in critic and panel) is recorded as the next candidate,
not
dropped.
Rerun5 keeps the old seats;
the change reaches the next launch.

At 25 minutes rerun5 had 438 successes,
12 retried refusals and 3 lost calls:
rerun4's calls were
still inside Hyper's window (they leave by 02:39) while the pacer's own count stood well under 1,000,
which is the launch-timing cost named at launch.

## XIEPT2 landed at 04:14 UTC in 129 minutes on the paced client

`TALLY XIEPT2 status=SETTLED slices=35 ... translateChanged=27 pageChanged=27 pageSilent=0
alignmentFindings=13 selection=contested ms=7761380`;
`verify-published` 1 of 1 pages,
every wording,
at
the promised length.
1,852 successful Hyper calls,
196 cuts,
40 credit reads;
the pacer queued 135
times and nothing was refused while it held the count at 1,000;
the 12 retried and 3 lost calls all fell
before 02:39.
Burst then hold:
the first thousand in 48 minutes,
then holds 02:52 to 03:04 and 03:52 to
04:04 while the bursts left the window.
The four-entry pass is complete:
keyword233,
Toka_ls,
Carena0442 (fixture) and XIEPT2 each landed on the tree of their launch.
Reading:
`doc/planning/translation-repair-xiept2-reading-2026-09-03.md` (death line and euthanasia as decided,
"To Live" and italic "Unease" from the editor comments over the lookup's "Disquiet",
four findings for
the owner:
the postscript's "took medication and breathed in gas",
the archive's `(To-Do)` placeholder
kept as the first paragraph,
inline glosses,
per-slice wrap).
