// Coordinator's own check of upstream issue 38's estimate on the first release with the feature:
// "For a typical CLI with 50 options, the worst-case overhead is less than 1ms on modern hardware."
// One mistyped option is parsed against 50 options whose names are 20 characters long.
//
// Run with @optique/core 0.7.0 beside it: node one-typo-on-0.7.0.ts

import { object } from "@optique/core/constructs";
import { optional } from "@optique/core/modifiers";
import { parse } from "@optique/core/parser";
import { option } from "@optique/core/primitives";
import { string } from "@optique/core/valueparser";

const optionCount = 50;
const names = Array.from({ length: optionCount }, (_, index) => `--opt-${String(index).padStart(3, "0")}${"x".repeat(11)}`);
const fields = Object.fromEntries(names.map((name, index) => [`field${index}`, optional(option(name as any, string()))]));
const parser = object(fields as any);

const typo = `${names[0]!.slice(0, -1)}y`;
const warmup = 200;
const timed = 2000;

function medianOf(run: () => unknown): number {
  for (let index = 0; index < warmup; index += 1) run();
  const samples: number[] = [];

  for (let index = 0; index < timed; index += 1) {
    const startedAt = performance.now();

    run();
    samples.push(performance.now() - startedAt);
  }

  samples.sort((left, right) => left - right);

  return samples[Math.floor(samples.length / 2)]!;
}

const failing = parse(parser as any, [typo, "value"]);
const succeeding = parse(parser as any, [names[0]!, "value"]);
const failingMs = medianOf(() => parse(parser as any, [typo, "value"]));
const succeedingMs = medianOf(() => parse(parser as any, [names[0]!, "value"]));

console.log(JSON.stringify({
  nameLength: names[0]!.length,
  typoLength: typo.length,
  failingParseSucceeded: failing.success,
  succeedingParseSucceeded: succeeding.success,
  failingParseMs: Number(failingMs.toFixed(4)),
  succeedingParseMs: Number(succeedingMs.toFixed(4)),
  overheadMs: Number((failingMs - succeedingMs).toFixed(4)),
  parsedPaths: 0,
  medianMs: Number(failingMs.toFixed(4)),
}));
