All parts are measured,
the image is final,
and no containers or processes of mine are left running.
Here is the report.

# Optique measurement, round two: scaling, one search, and what 1.4 added

## What ran

Same scratch folder,
`results-scaling/` for this round's raw records.
Image `localhost/optique-measure:2026-11-12`,
id `1d3ec42796786d…f25f44ebf`,
base `docker.io/cachyos/cachyos-v4:latest` (`sha256:1dcd1350f4b8…8c370`),
Node v26.13.1,
every run `podman run --rm --cpus 2 --memory 4g --pids-limit 512 --blkio-weight 10` with nothing mounted.
This session's cgroup allows 3 CPUs and 6 GiB;
the host has 32 CPUs and its load average ran 95 to 100 during the final collection (it had reached 160 earlier in the hour).
Final collection:
16:41:27Z to 16:56:17Z.

Rebuild the image:

```text
cd -- /home/user/temp/agent/optique-measure-2026-11-12 && podman build --tag localhost/optique-measure:2026-11-12 --file Containerfile .
```

Rerun all three parts plus both extra probes,
then every host-side fit and summary:

```text
cd -- /home/user/temp/agent/optique-measure-2026-11-12 && node scripts/measure-scaling.ts
```

One part at a time,
if wanted:

```text
cd -- /home/user/temp/agent/optique-measure-2026-11-12 && podman run --rm --cpus 2 --memory 4g --pids-limit 512 --blkio-weight 10 localhost/optique-measure:2026-11-12 scripts/scaling-all.ts
cd -- /home/user/temp/agent/optique-measure-2026-11-12 && podman run --rm --cpus 2 --memory 4g --pids-limit 512 --blkio-weight 10 localhost/optique-measure:2026-11-12 arms/0.7.0/bench/search-cost.ts
cd -- /home/user/temp/agent/optique-measure-2026-11-12 && podman run --rm --cpus 2 --memory 4g --pids-limit 512 --blkio-weight 10 localhost/optique-measure:2026-11-12 scripts/separator-all.ts --mode=spread --rounds=6 --reps=8 --warmup=2 --timed=5
cd -- /home/user/temp/agent/optique-measure-2026-11-12 && podman run --rm --cpus 2 --memory 4g --pids-limit 512 --blkio-weight 10 localhost/optique-measure:2026-11-12 scripts/separator-all.ts --mode=matrix --rounds=7 --warmup=2 --timed=5
cd -- /home/user/temp/agent/optique-measure-2026-11-12 && podman run --rm --cpus 2 --memory 4g --pids-limit 512 --blkio-weight 10 localhost/optique-measure:2026-11-12 scripts/profile-run2.ts
cd -- /home/user/temp/agent/optique-measure-2026-11-12 && podman run --rm --cpus 2 --memory 4g --pids-limit 512 --blkio-weight 10 localhost/optique-measure:2026-11-12 scripts/scaling-token-probe.ts
cd -- /home/user/temp/agent/optique-measure-2026-11-12 && podman run --rm --cpus 2 --memory 4g --pids-limit 512 --blkio-weight 10 localhost/optique-measure:2026-11-12 scripts/profile-separator-sizes.ts
```

Raw files:
`results-scaling/{scaling,separator,separator-spread,profiles,search,token-probe,profile-sizes}.jsonl`;
derived:
`fit.json`,
`separator-summary.json`,
`search-summary.json`,
`probes-summary.json`,
and the `summarise-*.txt` / `fit-scaling.txt` printouts.
`results/` from round one is untouched.

## Part 1: scaling, one factor at a time

Grammar:
`object()` with P × `optional(option("--option-NNN", string()))`,
each name exactly 12 characters,
plus `multiple(argument(string()))`,
no `passThrough`.
Tokens are distinct `p<base36>` names,
never option-like.
Arms `1.1.1`,
`1.3.6`,
`1.4.0-dev.2664`,
`1.4.0-dev.2971`;
5 rounds;
each round runs every point as a fresh process (2 warmup parses,
then 5 timed,
25 timed parses per point);
versions rotate within each round.

Levels,
with the other two factors held fixed:

- tokens:
  P=16,
  L=12,
  N at 100,
  200,
  500,
  1000,
  2000,
  5000
- options:
  N=100,
  L=12,
  P at 4,
  8,
  16,
  32,
  64,
  128
- length:
  N=500,
  P=16,
  L at 4,
  8,
  12,
  16,
  32,
  64,
  128,
  256

### Probe counts (deterministic, one parse per point, round 0)

Every one of the 20 points gives exactly N × P mismatch probes on `1.1.1`,
`1.3.6` and `1.4.0-dev.2664`,
and exactly 0 on `1.4.0-dev.2971`.
Examples from the raw records:
tokens N=5000,
P=16 gives 80,000 on the three and 0 on 2971;
options P=128,
N=100 gives 12,800 on the three and 0 on 2971;
every length level gives 8,000 on the three and 0 on 2971.
The counting parser is a separate build from the timed one (function-valued `errors.noMatch`),
so the timings below are all on the default error path.

### Timings, median / minimum in ms

Tokens,
P=16,
L=12:

- N=100:
  1.1.1 16.59/14.83,
  1.3.6 18.23/14.49,
  2664 18.14/17.04,
  2971 5.07/3.70
- N=200:
  32.13/29.33,
  32.98/30.39,
  38.01/33.62,
  11.42/7.82
- N=500:
  81.78/76.16,
  82.69/77.02,
  92.43/84.15,
  28.36/22.43
- N=1000:
  194.92/169.22,
  188.90/161.18,
  209.78/184.35,
  67.62/51.87
- N=2000:
  377.91/361.48,
  425.01/370.91,
  452.94/407.33,
  158.28/145.91
- N=5000:
  1399.41/1237.43,
  1388.57/1229.45,
  1555.75/1408.68,
  774.99/687.94

Options,
N=100,
L=12:

- P=4:
  2.12/1.71,
  2.67/1.95,
  3.60/2.61,
  2.76/1.97
- P=8:
  6.39/4.99,
  6.14/4.99,
  8.09/6.16,
  4.66/2.77
- P=16:
  16.29/14.45,
  15.70/14.59,
  17.81/16.51,
  5.74/3.83
- P=32:
  57.32/54.21,
  58.99/53.84,
  62.25/58.39,
  10.84/7.89
- P=64:
  230.14/203.63,
  232.08/217.16,
  230.61/214.10,
  16.32/12.98
- P=128:
  898.33/827.96,
  919.75/843.82,
  968.74/889.21,
  27.64/25.10

Length,
N=500,
P=16:

- L=4:
  42.83/40.76,
  22.31/20.01,
  29.85/27.14,
  26.84/20.39
- L=8:
  72.46/58.77,
  22.27/19.42,
  30.83/28.53,
  27.02/20.36
- L=12:
  78.59/75.40,
  84.41/77.07,
  96.41/83.79,
  28.20/21.58
- L=16:
  101.45/93.61,
  21.89/19.73,
  30.73/27.99,
  27.20/21.18
- L=32:
  182.92/162.95,
  21.82/19.84,
  30.39/27.34,
  26.36/20.74
- L=64:
  341.84/300.76,
  22.98/20.14,
  33.58/29.48,
  28.40/22.30
- L=128:
  652.12/580.18,
  22.60/20.04,
  32.12/27.97,
  27.97/22.24
- L=256:
  1209.09/1115.94,
  22.01/19.91,
  35.90/29.53,
  30.75/21.90

### Fitted exponents

Log-log slope per round (5 values),
reported as mean ± sample sd [min,
max];
the second figure is the slope through each level's minimum across rounds.
I used the per-round medians for the uncertainty and the level minima for the single-figure slope,
because the host load was high and minima are the more conservative estimate of working-set cost.

- tokens:
  1.1.1 1.085 ± 0.100 [0.909,
  1.159],
  min-fit 1.122;
  1.3.6 1.110 ± 0.059,
  min-fit 1.122;
  2664 1.123 ± 0.032,
  min-fit 1.118;
  2971 1.252 ± 0.048 [1.188,
  1.316],
  min-fit 1.318
- options:
  1.1.1 1.753 ± 0.051 [1.693,
  1.832],
  min-fit 1.788;
  1.3.6 1.708 ± 0.046,
  min-fit 1.772;
  2664 1.607 ± 0.030,
  min-fit 1.693;
  2971 0.641 ± 0.089 [0.517,
  0.718],
  min-fit 0.745
- length:
  1.1.1 0.820 ± 0.012 [0.804,
  0.838],
  min-fit 0.814;
  1.3.6 -0.068 ± 0.042,
  min-fit -0.078;
  2664 -0.021 ± 0.030,
  min-fit -0.052;
  2971 0.040 ± 0.041,
  min-fit 0.021

Predicted-shape fits,
per round:

- options,
  `t = a + b·P²`:
  R² 0.9975 to 0.9998 for 1.1.1,
  0.9991 to 0.99999 for 1.3.6,
  0.9993 to 0.9999 for 2664,
  with b ≈ 0.052 to 0.065 ms per P².
  A straight line in P fits the same data at R² 0.92 to 0.95,
  so the quadratic wins clearly.
  For 2971,
  `a + b·P²` fits at R² 0.68 to 0.94 with b ≈ 0.0013,
  i.e. quadratic does not describe it.
- length,
  straight line in L:
  1.1.1 slope 4.4 to 4.9 ms per character with R² 0.96 to 0.9997;
  minima fit gives 4.50 ms per character and an intercept of about 22 ms.
- tokens,
  straight line in N:
  not a usable model over this range (negative intercepts),
  because the curve is close to a power law,
  not affine;
  the log-log slope is the right statistic here.

### Did the predictions hold

- **1.1.1 linear in tokens**:
  yes,
  1.085 ± 0.100.
  Per-doubling ratios of the minima are 2.0,
  2.6,
  2.2,
  2.1,
  3.4,
  so slight superlinearity at the largest size,
  but the exponent sits at 1.09 to 1.12.
- **1.1.1 quadratic in options**:
  yes,
  by the `a + b·P²` fit (R² ≥ 0.9975) and by the marginal increments:
  each doubling of P multiplies the time by 2.9,
  2.9,
  3.75,
  3.76,
  4.07,
  approaching the 4× a pure quadratic predicts.
  The log-log slope is 1.75 rather than 2.00 only because about 5 ms of fixed per-parse cost sits in the curve;
  the slope of a quadratic with an additive constant is always below 2.
- **1.1.1 linear in token length**:
  yes,
  as a line with an intercept:
  4.4 to 4.9 ms per added character (R² 0.96 to 0.9997).
  The log-log slope of 0.82 is depressed by the roughly 22 ms intercept,
  not by sublinear behaviour;
  successive per-character increments are 4.4,
  4.7,
  4.5,
  4.5,
  4.4,
  4.5,
  5.0 ms.
- **1.3.6 linear in tokens,
  quadratic in options,
  flat in length**:
  yes for all three (1.110 ± 0.059;
  R² 0.999 for `a + b·P²`;
  -0.068 ± 0.042).
  With a caveat the source makes sharp:
  flatness holds only outside a window around the candidate-name length.
  At L=12,
  where every candidate has the same length as the token,
  the length prefilter cannot fire and 1.3.6 jumps from about 20 ms to 77 ms,
  a 3.5× spike at one level.
  The prefilter is `minDistance > maxDistance` or `minDistance / input.length > maxDistanceRatio` with `maxDistance` 3 and `maxDistanceRatio` 0.5,
  so names of 12 characters only search at 9 to 15 characters of token.
  2664 shows the same spike (83.8 ms at L=12 against about 28 ms elsewhere).
- **2971 no search,
  linear in tokens,
  flat in length,
  options open**:
  probes are 0 at every point,
  length is flat (+0.040 ± 0.041),
  and options are linear,
  not quadratic:
  the marginal cost per option parser is 0.159 ms (P 32 to 64) and 0.189 ms (P 64 to 128) per 100 tokens,
  i.e. 1.6 to 1.9 µs per (token,
  option) probe,
  constant.
  So option-scope bookkeeping does not grow with the grammar;
  tokens are the exception,
  at 1.252 ± 0.048.
- **2971 slightly superlinear in tokens**:
  real,
  and traced below.

### The token exponent above 1 is not suggestion work

With one option parser instead of sixteen,
both versions are strongly quadratic in tokens:
1.1.1 23.9,
95.5,
343.9,
1365.6 ms at N = 1000,
2000,
4000,
8000 (exponent 1.936);
2971 26.8,
101.7,
368.8,
1404.5 ms (exponent 1.900).
Per-token cost doubles with every doubling of N in both,
and the effect is identical in versions with and without suggestion work.

The profile names it.
On the separator shape with 1.3.6,
`input-trace.js` holds 51.6% of the parse at N=1000 (19.4 ms per parse,
`set` alone 51.0%) and 78.6% at N=4000 (351.7 ms per parse,
`set` 78.3%);
across the two sizes the total scales with exponent 1.785 while `set` scales with 2.089.
The source is `InputTraceImpl.set`,
which does `const copy = new Map(this.#entries)` on every recorded input (`packages/core/src/input-trace.ts`,
line 141 at the clone's `1d6e4594`),
so each of N recorded tokens copies every earlier entry.
`InputTrace` landed in `b2137fed` (2026-03-29),
before 1.1.1,
so every measured version carries this O(N²) floor,
and no version removes it.

## Part 2: what one search costs at the design note's parameters

`@optique/core` 0.7.0 is in the image as its own arm.
Parser:
50 × `optional(option("--opt-NNNxxxxxxxxxxx", string()))`,
names exactly 20 characters;
one mistyped 20-character token `--opt-000xxxxxxxxxxy`.
The search itself is `findSimilar` from `dist/suggestion.js`,
reached by file path because 0.7.0's exports map has no `./suggestion` subpath (it lists `.`,
`./completion`,
`./constructs`,
`./doc`,
`./facade`,
`./message`,
`./modifiers`,
`./parser`,
`./primitives`,
`./usage`,
`./valueparser`).
20,000 isolated calls,
200 warmup.

- **One search:
  0.0607 ms** (median of 20,000 in-process calls;
  3.04 ns per edit-distance cell over 50 × 20 × 20 = 20,000 cells).
  The note's prediction was about 0.07 ms at 3.4 ns per cell,
  so the prediction holds.
- **Margin against "less than 1 ms":
  16.5×.**
  For one search the note is comfortable.
- **But one mistyped option is not one search.**
  A single failing parse of that token against those 50 option parsers costs **3.359 ms** (median of 2,000 parses),
  which is 55.3 searches,
  because each option parser probes the token and each probe runs its own search with its own error message.
  Against the note's 1 ms,
  the realistic single-typo overhead is 3.4× over budget.
  A successful parse of the same parser with a valid option costs 0.0104 ms.
- **One thousand-path successful parse on 0.7.0 with those 50 options performs 50,000 searches**,
  counted,
  not derived:
  50,000 probe callbacks from the function-valued `errors.noMatch` (equal to 1000 tokens × 50 parsers),
  1000 paths accepted,
  8.7 s wall time for that one parse.
  Each probe runs exactly one `findSimilar` in 0.7.0's `primitives.js` (the search is executed before the callback at both option sites),
  so 50,000 probes means 50,000 searches.

## Part 3: what the 1.4 line added, and the separator regression

### 1.3.2 to 1.4.0-dev.2664 on the thousand-real-path commit parser

Per parse,
from `node --cpu-prof-interval=100` runs (40 parses for 1.3.2,
30 for 2664;
absolute per-parse figures run above the timed medians because of profiler overhead,
but the difference closes against the timed difference):

- parse:
  215.18 ms on 1.3.2,
  248.78 ms on 2664,
  +33.6 ms (+15.6%)
- `short-option.js`:
  absent on 1.3.2,
  **31.95 ms on 2664** (`withParserOptionScope` 8.96,
  `withOptionScopeChild` 7.66,
  `forkOptionScope` 7.12,
  `bindScope` 2.61,
  `value` 2.24,
  anonymous 1.90)
- `constructs.js` 10.12 to 13.64 (+3.5),
  `primitives.js` 6.97 to 9.16 (+2.2),
  `modifiers.js` 5.52 to 7.35 (+1.8)
- `suggestion.js` 98.24 to 97.06 (-1.2),
  `usage.js` 57.95 to 53.22 (-4.7),
  `input-trace.js` 24.37 to 22.97 (-1.4)

The positives sum to +39.5 ms against a measured +33.6 ms,
i.e. `short-option.js` alone is about 95% of the gross addition and the delta closes.
The timed matrix from round one agrees in size (180.0 ms to 229.0 ms,
+49 ms).

So yes,
`short-option.js` explains it.
It is also the only compiled file present in 2664 and absent in 1.3.2 (`dist/short-option.js` and `.cjs`;
source `packages/core/src/short-option.ts`,
added by `fe63bb3f`,
"Support attached short-option values",
2026-09-27T23:43:34+09:00 = 2026-09-27T14:43:34Z,
part of pull request #981,
merged as `4dd892c0` at 2026-09-27T16:56:44Z).
`withOptionScopeChild` and `bindScope` arrived two commits later in `acfbd6f3`,
"Isolate option routes by parser occurrence",
same second,
and `47d729f9`,
`f6b50628`,
`9b1f0fa0`,
`79efe48a`,
`cf50e959` (2026-09-27T16:23:31Z) and `867c4f91` (2026-09-28T06:26:01Z) extended it.
`2664`'s own commit is `6f258296` (2026-09-29T16:42:03Z),
the merge of pull request #996 "Share parser child selection rules" (`8add1ff9`,
2026-09-29T16:25:00Z),
which rewrote child selection in `constructs.ts`;
that is the plausible source of the +3.5 ms on `constructs.js`.

I confirmed the coordinator's framing:
`6f258296` contains tag `1.3.2` and not tag `1.3.3` (checked with `merge-base --is-ancestor` against both tags).

Mechanism,
from the source:
`withParserOptionScope` returns a fresh context object with a fresh `Set` of scope sources on every call,
appending `bindScope(...)` to a spread copy of the inherited set;
`forkOptionScope` copies the selections `Map` per probe.
Every option probe in a grammar now allocates a context,
a set,
a path array,
and a map copy,
where before it allocated nothing.
With a flat grammar the inherited set stays small,
which is why the synthetic options sweep is linear for 2971,
but the constant per-probe cost is paid on every one of the N × P probes.

### The separator case, measured properly

Shape:
the parser of `coordinator-check/separator-variant.ts` (the old commit parser's option set,
30 fields,
`passThrough`),
`["--message","x","--", ...1000 paths]` passed straight to `parseSync`,
so the library itself sees all 1000 tokens.
Spread measured first:
8 independent copies per arm over 6 rounds,
then a 7-round matrix with versions rotating.
Medians with minimum and maximum,
in ms:

- `1.1.1`:
  41.51 (min 33.19,
  max 68.87)
- `1.3.6`:
  42.93 (min 36.75,
  max 86.17)
- `1.4.0-dev.2664`:
  75.92 (min 67.63,
  max 99.52)
- `1.4.0-dev.2971`:
  79.67 (min 68.96,
  max 144.06)
- unchanged-arm spread:
  1.3.6 range 4.69 ms (11.7%);
  2971 range 10.98 ms (14.3%)
- 2971 over 1.3.6:
  **1.86×**,
  bounds from the spread 1.60× to 2.07×;
  2664 over 1.3.6 is 1.77×,
  so the regression is already there,
  and 2971 adds only about 5% on this shape.

This reproduces the coordinator's two rounds and replaces them:
1.1.1 and 1.3.6 are indistinguishable (difference 1.03×,
inside the 11.7% range),
and the 1.4 line is about 1.8 to 1.9 times slower.

Profiles on this shape,
per parse:

- `1.3.6`:
  total 41.23 ms;
  `input-trace.js` 21.07,
  `constructs.js` 7.73,
  `execution-context.js` 3.70,
  `modifiers.js` 3.20,
  `primitives.js` 1.39,
  `passthrough.js` 1.24;
  suggestion share 0.4%
- `2664`:
  total 72.07 ms;
  **`short-option.js` 25.85** (`withParserOptionScope` 8.00,
  `forkOptionScope` 6.07,
  `withOptionScopeChild` 6.06,
  `bindScope` 1.83,
  `value` 1.48),
  `input-trace.js` 19.34,
  `constructs.js` 11.11,
  `modifiers.js` 4.64,
  `execution-context.js` 4.10
- `2971`:
  total 86.25 ms;
  **`short-option.js` 26.96** (8.16,
  6.22,
  6.18,
  1.99,
  1.98),
  `input-trace.js` 26.16,
  `constructs.js` 11.78,
  `execution-context.js` 5.89,
  `modifiers.js` 5.65

Attribution closes:
1.3.6 to 2664 is +30.8 ms per parse,
made of +25.9 `short-option.js`,
+3.4 `constructs.js`,
+1.4 `modifiers.js`,
+0.6 `primitives.js`,
-1.7 `input-trace.js`.
2664 to 2971 is +14.2 ms,
spread over `input-trace.js` +6.8,
`execution-context.js` +1.8,
`short-option.js` +1.1,
`primitives.js` +1.2,
`modifiers.js` +1.0.
So the separator slowdown is the short-option scoping series,
with a smaller second helping of per-child bookkeeping in 2971,
and none of it is suggestion work.

## Departures from the suggested design

- **Added `1.4.0-dev.2664` to the separator matrix and its profiles**,
  beyond the three arms asked for.
  Without it the regression looks like a 2971 change;
  with it,
  1.77× is already present at 2664 and 2971 adds about 5%.
- **Added L=12 to the length sweep** (8 levels,
  not 5 to 7).
  The prefilter's window for 12-character names is 9 to 15 characters,
  so a log-spaced sweep without L=12 would have missed the 3.5× spike entirely and reported "flat" without its mechanism.
- **Different fixed values per sweep**:
  N=100 for the options sweep and N=500 for the length sweep,
  because 1.1.1 is quadratic in options and linear in length,
  so N=1000 everywhere would have made the P=128 and L=256 points multi-second.
- **5 rounds,
  2 warmups,
  5 timed parses** for Part 1 (the earlier round used 7 rounds and 7 timed).
  The fits use per-round medians for the uncertainty and per-level minima for the headline exponents;
  the brief allowed either,
  and minima are the more robust choice under this host load.
- **Two probes outside the brief**:
  the one-option token sweep (N up to 8000) and the separator profile at N=1000 and 4000.
  They decide whether the token exponent above 1 belongs to option probing or to per-token bookkeeping,
  and the answer changes the story.
- **Part 2 beyond the isolated search**:
  because 0.7.0 does not export `findSimilar`,
  the search is timed through the shipped file path,
  and I added the failing-parse and the counted 1000-path measurements,
  since the note's claim is about a CLI's worst case rather than about one function call.
- **The 0.7.0 arm carries only the search-cost probe**:
  its API predates the commit parser (no `parseSync`;
  `parse` instead),
  so it cannot run the thousands-of-paths commit workload.

## Could not do, and what surprised me

- **Could not measure 0.7.0 on the real commit parser.**
  The library at 0.7.0 has a different API,
  so the 1000-path count uses the 50-option parser and the issue-988 path shape instead.
- **Could not get a quieter host.**
  The final collection ran at load 95 to 100 on 32 CPUs,
  with earlier rounds up to 160.
  Absolute figures here sit above the first round's (the commit parser's 2971 arm measured 90 to 95 ms in round one;
  the same point here is slower),
  so I report minima and per-round slopes for shape,
  and treat the round-one matrix as the better absolute source.
  Ratios agree across both rounds.
- **Surprise,
  and a correction to my previous report**:
  the standard-deviation helper I used in round one was wrong (a `reduce` without an initial value,
  so the first sample's raw value was added into the sum of squared deviations).
  Ranges and medians were unaffected;
  the sd figures I quoted were inflated.
  Corrected numbers for the unchanged arms:
  1.3.6 at 1000 real paths sd 2.25 ms (I said 5.59),
  2971 at the same point 4.13 ms (5.45),
  1.3.6 at one path 0.065 ms (0.373),
  the owned parser 0.011 ms (0.154).
  The helper is fixed in `scripts/spread.ts` and `scripts/fit-scaling.ts`,
  and `results/spread-summary.json` has been regenerated.
- **Surprise:
  the O(N²) token floor.**
  Every version,
  including 1.1.1 and the best 1.4 dev build,
  is quadratic in the number of positional tokens because `InputTrace.set` copies its whole entry map per recorded input.
  With one option parser,
  8000 tokens cost 1.37 s on 1.1.1 and 1.40 s on 2971,
  against 24 and 27 ms at 1000.
  This is not a suggestion cost,
  it is not fixed by the 992 patch or by 997,
  and it is invisible in the earlier round because suggestion work dominated there.
- **Surprise:
  the 992 prefilter is not monotone in token length.**
  It makes long tokens cheap and short tokens cheap,
  and it makes tokens whose length matches the option names expensive.
  At the one level where they match,
  1.3.6 is 3.5× slower than at any other length,
  and 2664 is 3× slower.
- **Surprise:
  on the separator shape the versions invert.**
  2971 is about twice as fast as 1.3.6 on the thousand-path workload without a separator (90 to 95 ms against 184 ms) and about 1.9 times slower with one (79.7 ms against 42.9 ms),
  because the second shape has no suggestion work to save and pays the new scope bookkeeping on every probe.
- **Mild surprise:
  2664 is slower than 1.3.6 on the synthetic grammar too** (18.1 ms against 14.5 ms at 100 tokens,
  407 ms against 371 ms at 2000),
  so the short-option series cost is not specific to the commit parser.
