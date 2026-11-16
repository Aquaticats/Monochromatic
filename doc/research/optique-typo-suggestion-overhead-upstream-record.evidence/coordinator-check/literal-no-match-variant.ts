// Workaround probe: the reproducer's parser with a literal `errors.noMatch` message on
// every option and flag. From 1.3.2 a literal message returns before the suggestion search;
// on 1.3.1 and earlier the search runs first regardless.
// Run: node literal-no-match-variant.ts   (needs only @optique/core beside it)

import { object } from "@optique/core/constructs";
import { message } from "@optique/core/message";
import { multiple, optional } from "@optique/core/modifiers";
import { parseSync } from "@optique/core/parser";
import { argument, flag, option, passThrough } from "@optique/core/primitives";
import { string } from "@optique/core/valueparser";

const quiet = { errors: { noMatch: message`No match.` } };

function prefixes(name: string, shortestStem: number): [any, ...any[]] {
  const stem = name.slice(2);
  const names = [];
  for (let length = shortestStem; length <= stem.length; length += 1) names.push(`--${stem.slice(0, length)}`);
  return names as [any, ...any[]];
}

const parser = object({
  all: multiple(flag("-a", "--all", quiet)),
  only: multiple(flag("-o", "--only", quiet)),
  noOnly: multiple(flag("--no-only", quiet)),
  include: multiple(flag("-i", ...prefixes("--include", 3), quiet)),
  interactive: multiple(flag("--interactive", quiet)),
  patch: multiple(flag("-p", "--patch", quiet)),
  amend: multiple(flag("--amend", quiet)),
  allowEmpty: multiple(flag("--allow-empty", quiet)),
  dryRun: multiple(flag(...prefixes("--dry-run", 2), quiet)),
  short: multiple(flag(...prefixes("--short", 2), quiet)),
  porcelain: multiple(flag(...prefixes("--porcelain", 3), quiet)),
  long: multiple(flag(...prefixes("--long", 1), quiet)),
  nul: multiple(flag("-z", ...prefixes("--null", 2), quiet)),
  message: optional(option("-m", "--message", string(), quiet)),
  file: optional(option("-F", "--file", string(), quiet)),
  reuseMessage: optional(option("-C", "--reuse-message", string(), quiet)),
  reeditMessage: optional(option("-c", "--reedit-message", string(), quiet)),
  squash: optional(option("--squash", string(), quiet)),
  fixup: optional(option("--fixup", string(), quiet)),
  author: optional(option("--author", string(), quiet)),
  date: optional(option("--date", string(), quiet)),
  cleanup: optional(option("--cleanup", string(), quiet)),
  trailer: multiple(option("--trailer", string(), quiet)),
  template: optional(option("-t", "--template", string(), quiet)),
  unified: optional(option("-U", "--unified", string(), quiet)),
  interHunkContext: optional(option("--inter-hunk-context", string(), quiet)),
  pathspecFromFile: optional(option("--pathspec-from-file", string(), quiet)),
  pathspecFileNul: multiple(flag("--pathspec-file-nul", quiet)),
  paths: multiple(argument(string())),
  unknown: passThrough({ format: "nextToken" }),
});

const pathCount = 1000;
const paths = Array.from(
  { length: pathCount },
  (_, index) => `package/group-${index % 37}/package-${index}/src/feature-${index}.unit.test.ts`,
);
const args = ["--message", "x", ...paths];

const warmupParses = 2;
const timedParses = 7;
for (let index = 0; index < warmupParses; index += 1) parseSync(parser, args);

const samples: number[] = [];
let parsedPaths = -1;
for (let index = 0; index < timedParses; index += 1) {
  const startedAt = performance.now();
  const result = parseSync(parser, args);
  samples.push(performance.now() - startedAt);
  parsedPaths = result.success ? result.value.paths.length : -1;
}
samples.sort((left, right) => left - right);

console.log(JSON.stringify({
  paths: pathCount,
  parsedPaths,
  medianMs: Number(samples[Math.floor(samples.length / 2)]!.toFixed(1)),
  minMs: Number(samples[0]!.toFixed(1)),
}));
