# Synthetic `/v2/quotas` renamed its weekly block to `planWeeklyLimit` and the meter read unreadable for hours

## Symptom

A corpus pass started on 2026-10-22 (`XingZ6020`) parsed Synthetic's quota body at its first read, 19:32:44 UTC
(421 characters), and at every read up to 22:20:53 UTC (413 to 422 characters).
Every read the log shows after 22:21:58 UTC was refused with this warning,
copied from line 12621 of `/var/mnt/encrypted/temp/agent/corpus-pass-XingZ6020.log`
and wrapped here at spaces to fit the width (the log line is one line):

```text
[warn] [2026-10-22T22:21:58.964Z] [translation-repair] [XingZ60] [repair slice 73] [meterRecordOf]
synthetic: budget unreadable, treating as spendable
(Synthetic /quotas body violated expectations: weeklyTokenLimit is not an object;
received subscription:object, search:object, planWeeklyLimit:object)
```

At 2026-10-23T04:12:28Z, `rg --count` of that message over the log counted 291 lines, and the pass was still running,
so the count grows.
The `METERS` line then printed `synthetic=unreadable`,
the seat reader printed `synthetic=unreadable-treated-as-spendable`,
and the router sent Synthetic calls as if it had budget.
By the keyed reading of 03:28:16 UTC on 2026-10-23 the weekly credit was spent (0 percent remaining).
The last credit the log shows is 34.3 percent at 22:20:54 UTC, before the first refusal,
so when the credit fell to 0 is unknown, and so is how long a dry provider was routed as spendable.

## Root cause

Synthetic renamed the weekly block of the `GET https://api.synthetic.new/v2/quotas` body.
The endpoint carries no version, so the evidence is dated readings.
A keyed read at 2026-10-23 03:28:16 UTC answered HTTP 200 with 269 characters.
Its top-level keys and their JSON types were:

- `subscription`: object with `limit` (number), `requests` (number), `renewsAt` (string)
- `search`: object with `hourly` (object)
- `planWeeklyLimit`: object with `nextRegenAt` (string), `percentRemaining` (number), `nextRegenPercent` (number)

The values of `planWeeklyLimit` were
`{"nextRegenAt":"2026-10-23T05:03:47.000Z","percentRemaining":0,"nextRegenPercent":2}`.
The body held no `weeklyTokenLimit` and no `freeToolCalls`.
The `x-synthetic-quotas` response header of a keyed chat call at 2026-10-23 05:06:33 UTC (HTTP 200) has the same shape:
`subscription`, `search` and `planWeeklyLimit` with `nextRegenAt`, `percentRemaining` and `nextRegenPercent`,
and no `weeklyTokenLimit`.
This reader parses the `/v2/quotas` body only; a reader of the header meets the same name.
The reading of 2026-10-19 at 14:35 UTC had `weeklyTokenLimit` and `freeToolCalls`
(`doc/troubleshooting/translation-repair-provider-concurrency.md`, section on the removed limits).

The rename happened after 22:20:53 UTC and before 22:21:58 UTC on 2026-10-22.
The pass log shows both ends as fresh reads, not cached ones.
The meter cache trusts a reading for 60 seconds and stamps it when the read begins
(`package/translation-repair/provider/src/provider-budget.ts:70` and `:736`):

```ts
// provider-budget.ts:70 and :736
const BUDGET_FRESH_MS = 60_000;
    if ((now() - cache.startedAt) >= freshForMs)
      await readNow({ signal, },);
```

`takeReading` awaits `readEveryMeter` and only then logs the `METERS` line (`provider-budget.ts:641`),
so a `METERS` line is printed after a read that finished, never from the cache.
The `METERS` line at line 12568 of the pass log, 22:20:54.193 UTC, follows the line
`stream quotas: completed, elapsed 232ms ... 420 raw chars` of 22:20:53.109 UTC (line 12561),
a body the old reader parsed, and it reads `synthetic=wet syntheticWeekly=34.31539395918367%`.
No further `METERS` or `stream quotas` line appears before line 12620, the `stream quotas: completed,
elapsed 221ms ... 285 raw chars` read that ended at 22:21:58.963 UTC and was refused (the warning is line 12621).
The refused read began at 22:21:58.742 UTC, about 66 seconds after the wet one began (22:20:52.877 UTC),
which the 60-second cache permits.

The credit read 34.3 percent at the last parsed read, so a deploy is likelier than the credit reaching 0 percent
as the cause of the rename, but the log shows no credit after that read, so it is not proved either way.

The reader looked the weekly block up under one key only.
At `0fa4950b88`, `package/translation-repair/provider-stream/src/synthetic-quota.ts`, in `parseQuotaSnapshot`:

```ts
// synthetic-quota.ts:549 and :558 at 0fa4950b88
    weeklyTokenLimit: weekly,
  } = parsed;
  ...
  if (!isJsonRecord(weekly,))
    throw shapeRefusal({
      detail: 'weeklyTokenLimit is not an object',
```

`QuotaShapeError` is a provider protocol failure, so the meter layer turns it into `unreadable`
and logs the warning of the Symptom section
(`package/translation-repair/provider/src/provider-meters.ts:201`):

```ts
      `${name}: budget unreadable, treating as spendable (${
```

An unreadable meter routes as spendable, because only `dry` holds a provider out
(`provider-meters.ts:82-85`):

```ts
export function routesAsDry(
  { state, }: { readonly state: MeterState; },
): boolean {
  return state === 'dry';
}
```

So the refusal would hide a dry provider, as it did by 2026-10-23 03:28:16 UTC at the latest.
`syntheticIsDry` (`package/translation-repair/provider/src/budget-routing.ts:369-378`) is true at zero,
and `provider-meters.ts:315` feeds it the snapshot, so a body that had been read would have been `dry`:

```ts
  const { percentRemaining, } = quota.weekly;

  if (percentRemaining <= 0)
    return true;
```

## Verification

Version under test: the endpoint has no version, so the subject is the parser,
at these commits of this repository,
which were not on main when this was written
and have since landed on main by a merge,
which kept them:
`0fa4950b88` (before), `a4a53c010d` (first fix),
`c388130106` (second fix, which adds instant validation, null-as-absent and drops `nextRegenPercent`),
and `61528e1180` (red) with `90a480dcea` (third fix, which keeps the window and marks the return unreadable
where `nextRegenAt` is no instant, instead of refusing the body),
and `4e1db8a35a` (red) with `bc367f792f` (fourth fix, which reads two blocks that agree on `percentRemaining`
and differ in `nextRegenAt` with the return marked unreadable, instead of refusing the body),
and `218443fc90` (red) with `9371655ffd` (fifth fix, which logs one warning, not two, for such a pair).
The reading used is the one of 2026-10-23 03:28:16 UTC described in the Root cause section,
with invented numbers in the blocks other than `planWeeklyLimit`.

Reproduce the failure keyless, with no provider call, by running the red test file of `2376d5eb37` against the parser of
`0fa4950b88` in a scratch directory `$D` (the test file imports the built output, so its import is pointed at the copy):

```sh
# run from a checkout of this repository
D="$(mktemp --directory "${HOME}/temp/agent/synthetic-repro.XXXXXXXX")"
git show 0fa4950b88:package/translation-repair/provider-stream/src/synthetic-quota.ts \
  > "$D/synthetic-quota.ts"
git show 2376d5eb37:package/translation-repair/provider-stream/src/synthetic-quota.unit.test.ts \
  | sed "s#'../dist/final/node/index.mjs'#'./synthetic-quota.ts'#" \
  > "$D/synthetic-quota.unit.test.ts"
ln --symbolic "$PWD/package/translation-repair/provider-stream/node_modules" "$D/node_modules"
env --ignore-environment PATH="$PATH" HOME="$HOME" node "$D/synthetic-quota.unit.test.ts"
rm --recursive --force -- "$D"
```

Output at `0fa4950b88`: exit 1, `[FAIL]` 9 (eight cases and the suite line).
The first failing case parses the measured body and fails with the message the pass logged:

```text
QuotaShapeError: Synthetic /quotas body violated expectations: weeklyTokenLimit is not an object;
received subscription:object, search:object, planWeeklyLimit:object
```

At `9371655ffd` the cases pass under `mise run //package/translation-repair/provider-stream:test:unit`
and `mise run //package/translation-repair/provider:test:unit`.

Bodies that read cleanly:

- the 2026-07-16 shape with both blocks,
- the 2026-10-19 shape with `weeklyTokenLimit` and no five-hour block,
- the 2026-10-23 shape with `planWeeklyLimit`, which a real client reads as `dry` at 0 percent
  (with `syntheticWeeklyReturn=2026-10-23T05:03:47.000Z`) and as `wet` with budget left (no return field),
- a deprecated key kept as `null` beside a valid block under the other name, in either direction,
- a block carrying `nextRegenPercent` of any type, which is ignored,
- both names present with the same `percentRemaining` and a different `nextRegenAt` (two instants, or an instant and a
  value that is none): the window keeps `percentRemaining`, carries no `nextRegenAt`, the meter prints
  `syntheticWeeklyReturn=unreadable` for a spent block, and one warning names both blocks and the field
  (`Synthetic /quotas weeklyTokenLimit and planWeeklyLimit agree on percentRemaining and differ in nextRegenAt;
  the return reads unreadable`), never a value,
- a block whose `nextRegenAt` is missing, not a string, or a string that is no instant spelled the way
  `Date.prototype.toISOString` writes one (a value with a space and a newline, an empty one, a date-time with no zone,
  and one with no milliseconds): the window keeps `percentRemaining`, so a spent block still reads `dry`,
  the window carries no `nextRegenAt` at all, the meter prints `syntheticWeeklyReturn=unreadable`,
  and one warning names the block and the field, never the value
  (`Synthetic /quotas planWeeklyLimit.nextRegenAt is missing or no ISO-8601 instant as the logger writes one;
  the return reads unreadable`).

Bodies that are refused, each naming the field and the block's own keys and never a value:

- no weekly key under either name, or only `null` ones,
- a weekly key that holds neither `null` nor an object,
- a `percentRemaining` that is missing or not a number,
- both names present with a different `percentRemaining`.

## Verified workarounds

The reader takes both names (`weeklyWindowFromBody`, `synthetic-quota.ts:608`):

```ts
  const readings = WEEKLY_BLOCK_NAMES.flatMap(function readCarried(blockName,): readonly WeeklyReading[] {
```

It carries `nextRegenAt` only as an instant in `weeklyWindowOf` (`synthetic-quota.ts:545`), and otherwise warns and
keeps the window without it:

```ts
  if (((typeof nextRegenAt) === 'string') && isIsoStampText({ text: nextRegenAt, },))
```

The meter prints the return of a spent weekly budget as `syntheticWeeklyReturn=<nextRegenAt>` in its `METERS` fields
(`syntheticMeterLevel`, `budget-routing.ts:753-761`), and `unreadable` where the window has no instant:

```ts
        ((weekly.nextRegenAt !== undefined) && isIsoStampText({ text: weekly.nextRegenAt, },))
```

The printed instant passes a spelling check, not a plausibility check:
a past instant such as `1970-01-01T00:00:00.000Z` or a far-future one such as `+275760-09-13T00:00:00.000Z`
prints as given, and the comment of `syntheticMeterLevel` says so.

The maintainer's working rulings of 2026-10-23, open to the owner's veto:

- `nextRegenAt` is carried only as a validated instant because the meter prints it into a log line,
  where a space or a newline would forge a field or a line.
  A value that is no instant does not refuse the body, because the field decides nothing:
  the window keeps `percentRemaining`, marks the return unreadable and warns.
- `nextRegenPercent` is not modeled, because nothing reads it and the parser models only the fields a decision reads.
- A `null` weekly key counts as absent, so a deprecated key kept as `null` does not blind the meter.

Tradeoffs:

- A `nextRegenAt` that is a valid time in another spelling (no milliseconds, an offset) loses only the stated return:
  the meter still reads `dry` or `wet` from `percentRemaining`,
  prints `syntheticWeeklyReturn=unreadable` for a spent block,
  and the warning names the block and the field, so the next reading shows it.
  The first ruling of that day refused the whole body, which read `unreadable` and routed as spendable,
  the failure this document is about, for a field whose only use is a log line.
- Two blocks of one body that agree on `percentRemaining` and differ in `nextRegenAt` lose only the stated return,
  because which instant is right is unknowable; two that differ in `percentRemaining` are refused,
  since the decision field itself is then in doubt. No body is known to carry both names.
- The stated return is a field of the `METERS` line only.
  The router and the seat reader still decide on the dry bit and on refusal holds;
  a dry meter does not create a hold, so the seat reader says `no provider has named its return` for Synthetic.
  Deriving a hold from a meter's `nextRegenAt` would make a phase wait up to a regeneration period (202 minutes)
  and would change when a run ends on every provider dry, which the owner has not decided.

## What does not work

- Reading only `planWeeklyLimit` would break on the older shape, which an account may still receive.
- Treating a missing weekly block as dry would turn a renamed key into an outage, the opposite error.
- Waiting for the rename to be undone is not a plan: the body of 2026-10-23 still had the new name.

## Upstream filing decision

Filing upstream is out of scope: the owner ruled that upstream filing for this provider is not done.
That ruling is not recorded in a file of this repository.
The `.out-of-scope/` directory was checked and no entry names Synthetic,
so the exemption rests on the owner's ruling alone.
The six constraints and the duplicate search were therefore not walked, and no draft issue is kept.
Synthetic's `/v2/quotas` shape is not documented in a contract that promises key names,
and the repository records it from live readings only.
The next live reading that changes a key name again is refused with the body's keys and types,
which is what this reader keeps for the next investigator.
