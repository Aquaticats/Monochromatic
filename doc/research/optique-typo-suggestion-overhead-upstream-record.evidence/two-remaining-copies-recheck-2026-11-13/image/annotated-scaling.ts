// The parse of token-scaling.ts (one optional option and a run of positional arguments), with and without
// annotations passed to the parse. A parse that carries annotations copies them onto states as it goes, and some of
// those copies clone an array state, so this is a surface the published script does not reach.
//
// Prints exactly one JSON line. A parse that does not return every value throws.
//
// Run: deno run --no-config --import-map <map> annotated-scaling.ts --counts 2000,4000 --runs 2 --warmup 1

import { object } from "@optique/core/constructs";
import { formatMessage } from "@optique/core/message";
import { multiple, optional } from "@optique/core/modifiers";
import { parseSync } from "@optique/core/parser";
import { argument, option } from "@optique/core/primitives";
import { string } from "@optique/core/valueparser";

function valueOf(name: string, fallback: string): string {
  const argv = (globalThis as { process?: { argv?: string[] } }).process?.argv ?? [];
  const index = argv.indexOf(name);

  return index === -1 ? fallback : argv[index + 1] ?? fallback;
}

const counts = valueOf("--counts", "2000,4000,8000").split(",").map(Number);
const runs = Number(valueOf("--runs", "2"));
const warmup = Number(valueOf("--warmup", "1"));

for (const value of [...counts, runs, warmup]) {
  if (!Number.isSafeInteger(value) || value < 0) throw new Error(`not a whole number: ${value}`);
}

const parser = object({
  name: optional(option("--name", string())),
  paths: multiple(argument(string())),
});
const annotations = { [Symbol.for("@recheck/annotated-scaling")]: { present: true } };

function medianOf(samples: readonly number[]): number {
  const sorted = [...samples].sort((left, right) => left - right);

  return sorted[Math.floor(sorted.length / 2)]!;
}

function measure(tokens: number, annotated: boolean): { tokens: number; annotated: boolean; medianMs: number; minMs: number; maxMs: number } {
  const input = Array.from({ length: tokens }, (_, index) => `p${index.toString(36)}`);

  function parseOnce(): number {
    const startedAt = performance.now();
    const result = annotated ? parseSync(parser, input, { annotations }) : parseSync(parser, input);
    const elapsed = performance.now() - startedAt;

    if (!result.success) throw new Error(`parse of ${tokens} tokens failed: ${formatMessage(result.error)}`);

    const parsed = result.value.paths;

    if (parsed.length !== tokens || parsed[0] !== input[0] || parsed[tokens - 1] !== input[tokens - 1]) {
      throw new Error(`the ${tokens} parsed tokens are not the input`);
    }

    return elapsed;
  }

  for (let index = 0; index < warmup; index += 1) parseOnce();

  const samples = Array.from({ length: runs }, () => parseOnce());

  return {
    tokens,
    annotated,
    medianMs: Number(medianOf(samples).toFixed(3)),
    minMs: Number(Math.min(...samples).toFixed(3)),
    maxMs: Number(Math.max(...samples).toFixed(3)),
  };
}

console.log(JSON.stringify({
  script: "annotated-scaling",
  library: (import.meta as { resolve?: (id: string) => string }).resolve?.("@optique/core/primitives") ?? "unresolved",
  plain: counts.map((tokens) => measure(tokens, false)),
  annotated: counts.map((tokens) => measure(tokens, true)),
}));
