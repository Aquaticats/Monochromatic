# module-numeric-format

Ready to publish.

Numeric formatters for the gaps left by `Intl.DurationFormat` and `Intl.NumberFormat`.
This package mirrors `@monochromatic-dev/const-core`:
one file per category,
 re-exported from `index.ts`,
consuming the matching constants category (`package/const/core/src/time.ts` etc.) instead of redefining ratios.

The charter is narrow.
Only formatters that the platform `Intl` APIs explicitly do not cover go here.
Anything `Intl` already handles should be called directly at the consumer.

## Exports

<table>
<thead>
<tr>
<th>Function</th>
<th>Source</th>
<th>Description</th>
</tr>
</thead>
<tbody>
<tr>
<td>`formatBytes`</td>
<td>`src/byte.ts`</td>
<td>IEC binary byte formatter (`B` / `KiB` / `MiB` / `GiB`); sub-KiB values render as raw bytes (`422 B`).</td>
</tr>
<tr>
<td>`formatDuration`</td>
<td>`src/duration.ts`</td>
<td>Magnitude-adaptive sub-ms / ms / s formatter for `performance.now()` deltas.</td>
</tr>
<tr>
<td>`formatTrackedDuration`</td>
<td>`src/duration.ts`</td>
<td>Ultra-compact seconds-to-years ladder for productivity-app chip text (`1h30m`, `3d1h`, `1y2m`).</td>
</tr>
</tbody>
</table>

## Why not `Intl.DurationFormat`

`Intl.DurationFormat` shipped in Baseline 2025 (Firefox 136+,
 well within the project's Firefox ESR 140 baseline).
It does not cover developer-facing perf output for two reasons grounded in TC39 design decisions,
 not oversights.

### 1. Sub-millisecond precision is not representable

The Duration Record holds `[[Milliseconds]]`,
 `[[Microseconds]]`,
 `[[Nanoseconds]]` as three separate integer slots,
sync'd with `Temporal.Duration` ([proposal-intl-duration-format#157][i157],
 [#199][i199]).
Fractional milliseconds (e.g. `0.34ms` from `performance.now()`) cannot be expressed.

A `"fractional"` style exists for seconds,
 milliseconds,
 microseconds ([#205][i205])
but fractional minutes / hours / days were deferred ([proposal-intl-duration-format#65][i65] -> [ecma402#980][e980],
 still open).
@ryzokuken's reason in ecma402#980:

> In the case of days,
>  it's hard to say how many hours constitute 0.5 days,
>  since not all days are equally long.
> In the case of smaller units,
>  Temporal assumes all units are of fixed size for now but this might not hold in the future
> when (and if at all) we decide to add support for alternate time-keeping systems
> like we currently support alternate calendars.

### 2. Magnitude-driven unit selection is out of scope

The auto-select-best-unit feature
(`"0.3ms" -> "51ms" -> "1.2s"` depending on input magnitude)
was redirected to a hypothetical
`Intl.RelativeTimeFormat` v2 ([ecma402#498][e498],
 still open).
The proposal team's framing in [proposal-intl-duration-format#174][i174],
 @sffc:

> I believe the thinking is that relative time format has generally been considered to be an approximation of time,
> so a single unit is sufficient ... I believe the thinking was that if you need more precision then you should just
> format the timestamp directly.

And @FrankYFTang on the charter:

> Intl.
> DurationFormat is chartered to format the "amount" of time,
>  not a "particular time in the timeline" ...
> A duration itself is a "amount of time",
>  without any reference point of start or end.

[#32][i32] is the longest thread on adaptive unit-set selection.
The options debated
(`smallestUnit` / `largestUnit` / `hideZeroValued` / `requiredFields`)
all let the caller fix the field set;
none picks the field set from input magnitude.
No `bestFit`-style option was added.

### Implication

`formatDuration` (sub-ms fractional,
 magnitude-adaptive) is blocked by both points and is not on a TC39 roadmap.
The custom implementation here is justified.
Use `Intl.DurationFormat` directly for any integer-seconds multi-unit case (e.g. `1h30min` clocks);
 they are not in scope for this package.

### `formatTrackedDuration` (productivity-app tracked time)

`formatTrackedTime` was first migrated to `Intl.DurationFormat` with `style: 'digital'`,
 producing `H:MM:SS`.
That shape forces mental arithmetic past 24h (a 3-day task renders `73:00:00`),
 so a second migration
hand-rolled an ultra-compact ladder.
The hand-roll is justified by three Intl gaps that compose:

1. `Intl.DurationFormat` with `style: 'narrow'` produces `Xh Ym` (space-separated).
   The app wants no space (`Xh0m` for stopwatch-style stability under live ticks).
   Stripping the space post-format is locale-fragile.
2. Narrow suffixes are locale data,
    not constants:
    `週` and `日` in Japanese,
    `세` and `일` in Korean.
   The app wants ASCII single-letter suffixes everywhere so chip widths stay predictable
   and the visual scan does not depend on locale.
3. The decomposition uses 30 days per month and 365 days per year.
   `Intl.DurationFormat` does not normalize across variable-length units;
   the caller has to decompose anyway,
    and `30d ≈ 1 month` is an app-domain convention,
   not an Intl default.

Output rule:
 strict top-2 in adjacency.
The biggest non-zero unit pairs with the immediately smaller unit,
 even if the smaller one is zero.
Seconds-only renders as top-1 (`Xs`),
 all-zero renders as `0s`.

<table>
<thead>
<tr>
<th>Magnitude</th>
<th>Shape</th>
<th>Example</th>
</tr>
</thead>
<tbody>
<tr>
<td>zero</td>
<td>`0s`</td>
<td>`formatTrackedDuration(0)` -> `'0s'`</td>
</tr>
<tr>
<td>< 1 minute</td>
<td>`Xs`</td>
<td>`formatTrackedDuration(45)` -> `'45s'`</td>
</tr>
<tr>
<td>minutes</td>
<td>`XmYs`</td>
<td>`formatTrackedDuration(90)` -> `'1m30s'`</td>
</tr>
<tr>
<td>hours</td>
<td>`XhYm`</td>
<td>`formatTrackedDuration(5_400)` -> `'1h30m'`</td>
</tr>
<tr>
<td>days</td>
<td>`XdYh`</td>
<td>`formatTrackedDuration(263_400)` -> `'3d1h'`</td>
</tr>
<tr>
<td>weeks</td>
<td>`XwYd`</td>
<td>`formatTrackedDuration(1_468_800)` -> `'2w3d'`</td>
</tr>
<tr>
<td>months</td>
<td>`XmYw`</td>
<td>`formatTrackedDuration(30 * 86400)` -> `'1m0w'`</td>
</tr>
<tr>
<td>years</td>
<td>`XyYm`</td>
<td>`formatTrackedDuration(425 * 86400)` -> `'1y2m'`</td>
</tr>
</tbody>
</table>

Single-letter `m` is reused for both months and minutes;
 the secondary suffix always disambiguates.
A standalone `Xm` chip never occurs because seconds-only renders as `Xs`.

Trade-offs from strict adjacency and the 30 / 365 approximations:

- `35d` of tracked time renders as `1m0w` (months pairs with weeks=0;
   the trailing 5 days are dropped).
  Acceptable cost for visual stability during live timer ticks.
- 12 months in this system equals 360 days (12 × 30),
   not 365.
  A task at 360 days renders `12m0w`;
   at 365 days it crosses into `1y0m`;
  at 390 days it renders `1y0m` (the 25-day remainder is below the 30-day month threshold).
  Same task class,
   different shape near the year boundary,
   because the two approximations are not internally consistent.

## Why not `Intl.NumberFormat` for bytes

`Intl.NumberFormat` with `style: "unit"` supports the SI byte units (`byte`,
 `kilobyte`,
 `megabyte`,
 `gigabyte`,
 `terabyte`,
 `petabyte`),
sourced from CLDR's unit list.
It has no entries for the IEC binary units (`kibibyte`,
 `mebibyte`,
 `gibibyte`),
so any consumer that needs `KiB` / `MiB` / `GiB` display must hand-roll the format.
That formatter (`formatBytes`) belongs in this package alongside `formatDuration` once it lands;
SI byte display should go directly through `Intl.NumberFormat`.

[i32]: https://github.com/tc39/proposal-intl-duration-format/issues/32
[i65]: https://github.com/tc39/proposal-intl-duration-format/issues/65
[i157]: https://github.com/tc39/proposal-intl-duration-format/issues/157
[i174]: https://github.com/tc39/proposal-intl-duration-format/issues/174
[i199]: https://github.com/tc39/proposal-intl-duration-format/issues/199
[i205]: https://github.com/tc39/proposal-intl-duration-format/issues/205
[e498]: https://github.com/tc39/ecma402/issues/498
[e980]: https://github.com/tc39/ecma402/issues/980
