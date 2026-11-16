// Coordinator's own check of two claims from the scaling round, sharing no code with the agent's harness.
//
// Claim A: with one option parser, the time of a successful parse is close to quadratic in the number of
// positional tokens on every version, because the input trace copies its whole map for each recorded token.
// Claim B: a parse with paths after "--" behaves the same way, since the trace still records each token.
//
// Run: node token-quadratic-check.ts   (needs only @optique/core beside it)

import { object } from "@optique/core/constructs";
import { multiple, optional } from "@optique/core/modifiers";
import { parseSync } from "@optique/core/parser";
import { argument, option } from "@optique/core/primitives";
import { string } from "@optique/core/valueparser";

const parser = object({
  name: optional(option("--name", string())),
  paths: multiple(argument(string())),
});

const tokenCounts = [1000, 2000, 4000, 8000];
const timedParses = 5;
const results: { tokens: number; medianMs: number; msPerToken: number; parsed: number; }[] = [];

for (const count of tokenCounts) {
  const args = Array.from({ length: count }, (_, index) => `p${index.toString(36)}`);

  parseSync(parser, args);

  const samples: number[] = [];
  let parsed = -1;

  for (let index = 0; index < timedParses; index += 1) {
    const startedAt = performance.now();
    const result = parseSync(parser, args);

    samples.push(performance.now() - startedAt);
    parsed = result.success ? result.value.paths.length : -1;
  }

  samples.sort((left, right) => left - right);
  const medianMs = samples[Math.floor(samples.length / 2)]!;

  results.push({ tokens: count, medianMs: Number(medianMs.toFixed(1)), msPerToken: Number((medianMs / count).toFixed(4)), parsed });
}

const first = results[0]!;
const last = results.at(-1)!;

console.log(JSON.stringify({
  results,
  // 1 means linear, 2 means quadratic.
  exponentFirstToLast: Number((Math.log(last.medianMs / first.medianMs) / Math.log(last.tokens / first.tokens)).toFixed(2)),
  parsedPaths: last.parsed,
  medianMs: last.medianMs,
}));
