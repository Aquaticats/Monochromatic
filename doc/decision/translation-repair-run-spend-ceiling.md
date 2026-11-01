# A run stops starting entries once it has spent its allowance on OpenRouter, and, since 2026-10-22, on DeepSeek or Xiaomi

Decided by the owner on 2026-09-04 ("Add a per-run USD ceiling"),
asked with two options after the
OpenRouter fallback landed and the only spend guard was the credits meter.
The section "The rule" is that decision as made,
for OpenRouter alone;
the amendment of 2026-10-22 extends it to DeepSeek and Xiaomi.

## The rule

- Every call's `cost=` (the USD the OpenRouter wire reports,
  the same figure the `SPEND` line carries and
    `spend-report` sums) is added to a process-wide meter (`run-spend-meter.ts`) as the line is written.
- Before the scheduler starts each entry it asks whether the run's OpenRouter spend is at or past the
    ceiling (`corpus-run/spend-ceiling.ts`,
  through the queue's `stopBeforeNext`).
  Past it,
  the run prints
    `SPEND CEILING reached: <spent> of <ceiling> USD spent on openrouter this run; not starting new
    entries` and starts nothing more;
  entries already running finish.
- The built-in ceiling is 20 USD:
  the 2026-09-03 planning record priced a day of three to four entries on
    OpenRouter alone without Kimi-K3 at about that,
  and the recommended auto top-up threshold is the same.
- `TRANSLATION_REPAIR_RUN_SPEND_CEILING_USD` overrides it for one launch and prints `SPEND CEILING
    OVERRIDDEN`;
  unset and blank are the built-in;
  anything else unreadable or negative is refused at
    launch,
  as the entry ceiling's override is (`cap-override.ts`);
  zero is allowed and means start nothing,
    which is how the guard is shown to fire live at no cost.
- The total is a floor,
  as the spend report's is:
  a cut stream reports no cost.

## What landed

- `6a1b4262e`:
  `run-spend-meter.ts`,
  `corpus-run/spend-ceiling.ts`,
  `corpus-run/pass-stop-before-next.ts`
    (the attempt queue's stop rule,
  split out of `corpus-pass.ts` at the line cap),
  fed from `spend-line.ts`.
- Shown live the same day at no cost:
  a keyword233 launch with the ceiling at zero printed
    `SPEND CEILING OVERRIDDEN ... 0 USD ... rather than the built-in 20`,
  then
    `SPEND CEILING reached: 0 of 0 USD spent on openrouter this run; not starting new entries`,
  and ended
    `DONE processed=0 of pending=1` with no `SPEND` line (`~/temp/agent/ceiling-zero-20260904.log`).
- Guards shown to fail with the predicate and the meter feed neutralised,
  restored and passing.

## Option rejected

- Meter and top-up amount only:
  no code,
  but auto top-up repeats,
  so a runaway spends every top-up until
    someone notices,
  and the meter only stops a pass at zero.

## Amendment of 2026-10-22: each USD-billed provider, on its own

Decided by the owner on 2026-10-22.
The owner was asked whether the 20 USD stop applies per provider,
as a separate Xiaomi figure,
or summed across providers,
and chose "20 USD each".

The question put to the owner stated three facts:
one run could spend up to about 60 USD in total
(each of the three providers just under 20 USD when another reaches it),
plus the entries in flight;
DeepSeek is bounded in practice by its prepaid balance (9.96 USD on 2026-10-22);
and Xiaomi has no balance meter,
so this ceiling is its only stop.

The direct DeepSeek and Xiaomi routes sit ahead of OpenRouter in the spending order,
so a ceiling that read only OpenRouter left the MiMo seat that it used to bound unbounded:
Xiaomi documents no balance (`doc/decision/translation-repair-direct-providers.md`),
and DeepSeek's prepaid balance can be topped up.

- The same 20 USD applies to each of OpenRouter,
  DeepSeek and Xiaomi (`SPEND_CEILING_PROVIDERS`),
  each against its own total and never a sum.
  The run stops starting entries when any one reaches it,
  and the `SPEND CEILING reached` line names that provider.
  No new number was chosen.
- The worst case is about 60 USD across the three providers (20 USD each),
  plus the entries already in flight,
  which finish.
- The mechanism is unchanged:
  the same process-wide meter,
  the same scheduler stop before each entry,
  the same override variable.
  This is a stop on starting entries and not a change to what the router reads:
  Xiaomi's meter still answers wet,
  and at the ceiling the run ends rather than diverting its seat to OpenRouter.
- The override note now reads `on any one of openrouter, deepseek, xiaomi`.

## What this is not

- Not a per-entry bound:
  an entry in flight finishes,
  so a run's worst case is the ceiling plus the entries
    that had already started.
  The entry ceiling in minutes (`TRANSLATION_REPAIR_HARD_CAP_MINUTES`) bounds
    those.
- Not a bound on NeuralWatt,
  Synthetic,
  Bedrock or Hyper:
  none of them is metered by this ceiling.
  NeuralWatt and Synthetic run on allowances,
  and Hyper and Bedrock spend prepaid balances that have their own meters
  (Bedrock's is never topped up).
