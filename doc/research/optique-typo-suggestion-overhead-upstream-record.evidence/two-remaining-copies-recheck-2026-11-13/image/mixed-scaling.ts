// How a successful parse scales when options and positional arguments alternate for the whole command line:
// `-v v0 p0 -v v1 p1 ...`. The published token-scaling.ts covers a run of positional arguments only, where the
// buffer is cut by argument() alone; here every third step is an option, whose parser cuts the buffer with
// Array.prototype.slice, so a buffer that is not a plain array is read element by element.
//
// Prints exactly one JSON line. A parse that does not return every value throws.
//
// Run: deno run --no-config --import-map <map> mixed-scaling.ts --counts 6000,12000 --runs 2 --warmup 1

import { object } from "@optique/core/constructs";
import { formatMessage } from "@optique/core/message";
import { multiple } from "@optique/core/modifiers";
import { parseSync } from "@optique/core/parser";
import { argument, option } from "@optique/core/primitives";
import { string } from "@optique/core/valueparser";

const TOKENS_PER_GROUP = 3;

function valueOf(name: string, fallback: string): string {
  const argv = (globalThis as { process?: { argv?: string[] } }).process?.argv ?? [];
  const index = argv.indexOf(name);

  return index === -1 ? fallback : argv[index + 1] ?? fallback;
}

const counts = valueOf("--counts", "3000,6000,12000").split(",").map(Number);
const runs = Number(valueOf("--runs", "3"));
const warmup = Number(valueOf("--warmup", "1"));

for (const value of [...counts, runs, warmup]) {
  if (!Number.isSafeInteger(value) || value < 0) throw new Error(`not a whole number: ${value}`);
}

const parser = object({
  values: multiple(option("-v", "--value", string())),
  rest: multiple(argument(string())),
});

function medianOf(samples: readonly number[]): number {
  const sorted = [...samples].sort((left, right) => left - right);

  return sorted[Math.floor(sorted.length / 2)]!;
}

const results = counts.map((tokens) => {
  const groups = Math.floor(tokens / TOKENS_PER_GROUP);
  const input = Array.from({ length: groups }, (_, index) => ["-v", `v${index.toString(36)}`, `p${index.toString(36)}`]).flat();

  function parseOnce(): number {
    const startedAt = performance.now();
    const result = parseSync(parser, input);
    const elapsed = performance.now() - startedAt;

    if (!result.success) throw new Error(`parse of ${input.length} tokens failed: ${formatMessage(result.error)}`);

    const { values, rest } = result.value;

    if (values.length !== groups || rest.length !== groups) throw new Error(`parsed ${values.length} values and ${rest.length} arguments of ${groups}`);

    if (values[groups - 1] !== input[input.length - 2] || rest[groups - 1] !== input[input.length - 1] || rest[0] !== input[2]) {
      throw new Error("the parsed values are not the input");
    }

    return elapsed;
  }

  for (let index = 0; index < warmup; index += 1) parseOnce();

  const samples = Array.from({ length: runs }, () => parseOnce());

  return {
    tokens: input.length,
    runs,
    medianMs: Number(medianOf(samples).toFixed(3)),
    minMs: Number(Math.min(...samples).toFixed(3)),
    maxMs: Number(Math.max(...samples).toFixed(3)),
  };
});

console.log(JSON.stringify({
  script: "mixed-scaling",
  library: (import.meta as { resolve?: (id: string) => string }).resolve?.("@optique/core/primitives") ?? "unresolved",
  results,
}));
