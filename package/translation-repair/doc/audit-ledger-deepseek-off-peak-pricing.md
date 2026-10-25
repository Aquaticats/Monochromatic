# Audit ledger: DeepSeek off-peak pricing

Findings in how the provider package books the cost of a DeepSeek call
against the hours DeepSeek's page prices differently.

Part of [Audit ledger](audit-ledger.md),
which lists every file of this record.

## DeepSeek off-peak pricing

Provenance:
the XingZ6021 read (its defect N8),
the fix of 2026-10-25 (UTC),
and the second and third rounds after independent re-checks of that fix.

### B920: every DeepSeek call was booked at peak rates, so an off-peak run booked twice what the balance fell

Status:
fixed on 2026-10-25 (UTC) in three rounds,
found by the read of the XingZ6021 run (its defect N8)
and decided by the lead that day (the id is the lead's allocation and may be renumbered at merge time).
The entry moved from `audit-ledger-providers-routing-and-seating.md` to this file in the second round,
because the extension would have taken that file past its 1000 line budget.

The defect:
`deepSeekCostOf` (`provider/src/direct-cost.ts`) priced every call at the card's peak rates.
The XingZ6021 run went from Saturday 22:49 UTC to Sunday 00:55 UTC,
off peak throughout,
and booked 0.147093 USD on 15 DeepSeek lines while the balance fell 0.07.
The page (`https://api-docs.deepseek.com/quick_start/pricing`,
read 2026-10-25 by the lead and by the reviewer)
says peak hours are 01:00 to 04:00 UTC and 06:00 to 10:00 UTC,
Monday through Friday,
excluding Chinese public holidays,
that off-peak rates are half of the peak rates,
and that all other hours are off peak,
including weekends and Chinese public holidays in full.

The first round:
a call was booked at the card's peak rates when its request started inside a weekday peak window in UTC,
and at half of them otherwise (`DEEPSEEK_OFF_PEAK_MULTIPLIER` and `isDeepSeekPeakAt`).
Each window is half-open:
01:00:00 UTC and 06:00:00 UTC are peak,
04:00:00 UTC and 10:00:00 UTC are off peak,
because the page does not say whether a window's last instant belongs to it.
The weekday is the UTC weekday,
which is the Beijing weekday at every instant a window covers.
An instant that is not a time is peak.
The direct client had no clock,
so `createDirectClient` and `createDeepSeekClient` take `now` (the system clock by default),
read it once before the first attempt of a call,
and hand it to the route as `startedAt`.
The red commit is `1ebf4ca7e0` (seven cases fail),
fixed by `57c772b8ab`.

The re-check of the first round
found the rule and the recomputation correct,
and eight findings.
The ones that changed code or tests:
the case that pinned the clock read to the start of the call passed when the read moved after the reply;
Tuesday and Thursday were never tested,
so a change that made either off peak passed every case;
and the card and the decision record said a booked figure is never an under-read,
which the start-only rule could not promise,
since the page does not say whether the start or the end of a request decides its rate.
The ones that changed words:
the region comment still said the page was not fetched,
a decision-record line was over 120 characters,
the earlier SPEND lines were not flagged as peak figures,
and some commit bodies did not name every package they touched.

The second round:
a call is booked at peak when its request started,
or its reply completed,
inside a weekday peak window,
and at half the peak rates only when both instants lie outside every window
(`isDeepSeekPeakDuring`,
a pure function of the two instants).
The client reads the clock a second time once the retry ladder has returned,
so a retried call is bracketed by the first attempt's start and the successful attempt's completion,
and hands both to the route as `startedAt` and `completedAt`.
A call that starts or completes inside a window is therefore booked at peak whichever instant DeepSeek bills by,
and a call that spans a whole window with both instants outside is booked at half,
which is wrong only if DeepSeek bills by an instant between the two.

The red cases of the second round:
`83cb81981c` (eight cases fail:
four in `deepseek-client.unit.test.ts`
and four in `direct-cost.unit.test.ts`),
fixed by `1edf0002db`.
The client cases let the fake clock answer the off-peak instant only after the transport has been called,
and the peak instant only after it,
so a read moved across the reply fails them;
moving the start read after the reply made two of them fail,
the order case and the retried call whose first instant is peak.
The case for each weekday from Monday to Friday at 02:00 UTC and 07:00 UTC,
and Saturday and Sunday at the same hours,
was green on arrival,
because the code was right and the suite had a gap,
which is what the re-check's own mutants showed.

The third round,
after a second independent re-check found no major finding,
two minor ones and six nits:
the sentence "a call that crosses a window edge is booked at peak" was wider than the code,
since a call starting at 00:30 UTC and completing at 04:30 UTC on a weekday crosses both edges of the first window,
has both instants outside it,
and books half,
as the not-bounded sentences of the same files said.
It is reworded in the region comment and the TSDoc of `isDeepSeekPeakDuring` in `provider/src/direct-cost.ts`,
in the card TSDoc in `shared-contract/src/direct-card-sides.ts`,
in the Prices section of the decision record and in this entry:
a call that starts or completes inside a window books peak,
and a call that spans a whole window with both instants outside books half,
which is wrong only if DeepSeek bills by an instant between the two.
No test pinned that half:
a mutant that books peak for any peak minute between the two instants passed both test files.
The case Wednesday 00:30 to 04:30 UTC and 05:00 to 10:30 UTC now expects half,
and the mutant fails it
(`mutant J`,
a minute-by-minute probe between the instants;
two cases fail,
the spanning case and the new direct case of `isDeepSeekPeakDuring`).
The nits:
the module header of `direct-cost.unit.test.ts` names both instants,
two test names of 121 columns are shortened,
`isDeepSeekPeakDuring` has its own suite,
the multiplier TSDoc is rewrapped,
and a retried-call case lets the clock answer by how many attempts the transport has been handed,
so a start read taken between attempts and not before the first books another rate
(a mutant that reads the start at every attempt fails it on the booked figure,
where the earlier retried-call case fails only on the read count).
The statements on `main` that went stale,
in the XingZ6020 run record,
the XingZ6021 defects record and the state-after-k4 handover,
carry dated notes beside the original text.
The commits are `496f09c0aa` (cases),
`f5aa75f982` (wording) and `e7e7a82d15` (notes),
after the merge of `origin/main` at `9d234a763e` in `f3be7fd0ce`.

What the booked figure bounds:
it is an upper bound as to holidays,
which only make a weekday window cheaper and which the run reads no calendar for,
and as to which instant decides.
It is not bounded as to a Saturday that China makes a working day,
which the page's wording books off peak and DeepSeek may bill at peak,
and not as to a call that starts before a window and completes after it
(three hours at least),
which has both instants outside and books half.
Figures written before 2026-10-25 are peak figures for every call,
and over-read off-peak calls by up to twice.

The readers of the booked cost:
the run spend meter,
the per-run spend ceiling,
the spend report and the SPEND lines
add up the figure as written and assumed no peak pricing.
The dry floor of the DeepSeek meter and the decision record
reason from peak as the worst case,
which stays true.
An off-peak night now reaches the 20 USD ceiling later in real time at the same real dollars.

Recomputed from the XingZ6021 log,
the 15 lines come to 0.073547 USD against the balance fall of 0.07 (printed to two places).

The rule for next time:
a price that varies by the hour is priced from an injected clock,
at the dearer rate when either end of the call lies in the dear hours and the page does not say which end decides,
and the page's wording at an edge is stated where the code pins it.

Cache versions:
none moved.
The booked cost reaches the SPEND line and the run meter only,
and no cache key reads either.
