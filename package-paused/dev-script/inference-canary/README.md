# inference-canary

Detects LLM inference degradation by running code-generation probes against multiple models and scoring the output
 on correctness, lint quality, and type safety.

## Motivation

Inference degradation (increased latency, truncated outputs, reduced code quality) is hard to detect
 from inside an agent session.
This package provides objective, reproducible probes that establish a baseline for each model
 and flag statistical outliers.

## How it works

Each probe asks a model to write a single-file TypeScript CLI that solves a non-trivial problem.
The generated code is:

1. Executed in a locked-down podman container (no network, read-only FS, 256MB memory, 15s timeout)
2. Linted with the monorepo's oxlint config
3. Type-checked with tsgo under the monorepo's strict tsconfig

Scoring combines three dimensions:

- **Correctness** (hard gate): does the code execute and produce the expected output?
- **Lint quality**: oxlint violations, errors penalize 0.1 each, warnings 0.05 each
- **Type safety**: tsgo type errors, 0.1 penalty each

Correctness is a hard gate: anything below a perfect 1.0 zeroes the entire score.
Code that produces the right output but has lint errors scores 0, the same as code that crashes.
This is intentional: a submission full of lint violations and type errors is not production-quality code,
regardless of whether it happens to work.
The system prompt gives the model the exact oxlint and tsgo configs it will be graded against,
so there is no ambiguity about what the rules are.

When correctness is perfect, the score starts at 1.0 and each quality issue applies a flat deduction.
A typical first-pass score of 0 with a fix-pass score of 0.7-0.9 means the model wrote correct code
with many lint issues, then cleaned up most of them when shown the diagnostics.

**Performance multiplier**: code-gen probes optionally run the generated source against a larger input
to measure throughput. The perf score (0-1, linear decay between fast/slow thresholds) is applied as
a direct multiplier on the combined score. Slow implementations degrade the full score proportionally.

A second "fix" pass sends the model its code plus diagnostics and measures improvement.

### Overall score aggregation

The overall model score is the **arithmetic mean** of all individual scores,
treating initial and fix pass scores as equal participants.
A probe that produced a fix pass contributes two scores to the mean;
a probe without a fix pass contributes one.

Geometric mean was evaluated and rejected for this use case.
Because probes vary significantly in difficulty (stak-interpreter routinely scores 0.00
while stak-simulation scores 1.00 for most models),
geometric mean collapses scores toward zero; a single hard probe tanks the entire result
regardless of performance elsewhere.
With a floor of 0.01, every model in a test run scored below 0.25 under geometric mean,
destroying differentiation between strong and weak models.
Arithmetic mean preserves proportional contribution from each probe
and produces scores in a range where the pass/fail thresholds (0.7 WARN, 0.9 PASS) can differentiate.

Simulation probes like stak-simulation score 1.00 for most models on most days,
which looks like a freebie, but this is intentional.
They give a nonzero baseline to models that score nothing on code-gen probes,
and even strong models occasionally fail them, so they still carry signal.

### Numeric precision

Scores are stored as raw IEEE 754 doubles with no rounding.
Probe ratios like `13/20` are exact in decimal but not in binary,
and accumulation in `mean()` (`src/math.ts`) and `combinedScore()` (`src/codegen/scoring.ts`)
can shift the final value by a few ULPs.
The same `13/20` may serialize as `0.65` in one entry and `0.6499999999999999` in another,
depending on the order arithmetic was performed.

This is intentional: the viewer rounds to two decimals at display
(`src/runner-probe-core.ts`, `src/report-model.ts`),
and the pass/fail thresholds (0.7 WARN, 0.9 PASS) have margins large enough that ULP-level drift is irrelevant.
Comparators that consume scores directly should use tolerance, not equality.

## Probes

### Code-gen probes

- **csv-rfc4180**: RFC 4180 CSV parser with escaped quotes and multiline fields
- **expr-eval**: expression evaluator with operator precedence and recursive descent parsing
- **css-mixin-transpiler**: CSS native `@mixin`/`@apply` transpiler with recursive expansion, nested rules,
 and top-level apply; no-regex constraint
- **stak-interpreter**: stack-based language interpreter with floor division, floored modulo, variables, labels,
 and jumps
- **task-scheduler** (slow, off by default): concurrent task scheduler with dependency resolution and timing constraints

### Simulation probes

- **stak-simulation**: gives the model the Stak interpreter source and asks it to trace five programs mentally;
 tests careful code reading over pattern matching

Probes are intentionally hard: a healthy model scores around 0.7-0.8, not 1.0.
This makes subtle degradation detectable: a drop from 0.7 to 0.4 is clear signal.

## Usage

```bash
mise run canary
```

### Options

- `--model <id>`: test a single OpenRouter model instead of all registered models
- `--runs <n>`: consistency runs per probe (default: 2)
- `--simple`: run cheap text-only probes instead of code-gen
- `--slow`: include slow probes (e.g. task-scheduler)
- `--retest-all`: retest all models even if recent (<24h) results exist

### Environment

- `INFERENCE_VALIDATION_OPENROUTER_API_KEY`: OpenRouter API key (read from `.env.local` via mise)

## Models

All models are tested in parallel via OpenRouter:

- Anthropic Claude Opus 4.6
- Anthropic Claude Sonnet 4.6
- Anthropic Claude Haiku 4.5
- MiniMax M2.5
- Moonshot Kimi K2.5
- Z-AI GLM 5
- Qwen 3.5 OSS (397B MoE)
- ~~OpenAI GPT 5.2~~ (dropped 2026-02-28; see comment in `models.ts`)

## Artifacts

### `canary-history.jsonl`

Append-only JSONL file with one entry per model per run.
Used for statistical threshold computation (mean - 2 * stddev).
Gitignored, local to each machine.

### `src/canary-lint/`

Generated model outputs organized as `<model-slug>/<probe>-<pass>/`:

- `canary.ts`: the TypeScript source the model generated
- `meta.json`: model ID, probe name, pass (initial/fix), timestamp

Gitignored but preserved locally across runs for post-run inspection.
Directory structure is designed for a future viewer tool that displays score graphs with clickable data points.

## Architecture

```
src/
  index.ts               CLI entry point, multi-model orchestration
  runner.ts              Per-model canary orchestrator
  runner-probe.ts        Per-probe execution with timeout
  runner-stream.ts       OpenAI SDK streaming with timing data
  runner-client.ts       Client creation and single-turn probe execution
  runner-second-pass.ts  Fix-pass loop
  probes.ts              Probe type, simple probes, probe registry
  probe-types.ts         Shared Probe and ScoreContext types
  codegen/
    probe-factory.ts     Factory eliminating per-probe boilerplate
    perf.ts              Performance scoring (timed container, linear decay)
    scoring.ts           Combined score: correctness gate + lint/type penalties
    fix-prompt.ts        Second-pass fix prompt builder
    extract-code.ts      Markdown fence stripping
    system-prompt.ts     System prompt with embedded project configs
    csv-rfc4180.ts       CSV parser probe
    expr-eval.ts         Expression evaluator probe
    css-mixin.ts         CSS mixin transpiler probe
    stak.ts              Stak interpreter probe
    task-scheduler.ts    Task scheduler probe
    perf-test-data/      Generated large inputs for perf tests
  simulation/
    stak-simulation.ts   Stak simulation probe
    system-prompt.ts     Simulation system prompt
  stak/                  Stak interpreter source, test cases, perf data
  container*.ts          Podman container executor (base, runtime, exec)
  linter*.ts             Oxlint + tsgo runner, artifact writing
  models.ts              Model registry
  history*.ts            JSONL history storage, threshold computation
  report*.ts             Report formatting
  math.ts                Shared numeric utilities
  paths.ts               Package-level path constants
```
