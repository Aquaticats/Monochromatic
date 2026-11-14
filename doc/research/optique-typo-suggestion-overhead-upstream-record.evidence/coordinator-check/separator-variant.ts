// Separator variant of the reproducer (paths follow "--"): a successful parse of many positional paths through a parser shaped like
// a `git commit` option set. Needs only @optique/core beside it.
// Run: node reproducer.ts        (Node 24 or newer runs TypeScript directly)

import { object } from "@optique/core/constructs";
import { multiple, optional } from "@optique/core/modifiers";
import { parseSync } from "@optique/core/parser";
import { argument, flag, option, passThrough } from "@optique/core/primitives";
import { string } from "@optique/core/valueparser";

// Git accepts any unambiguous prefix of a long option, so each flag declares them all.
function prefixes(name: string, shortestStem: number): [any, ...any[]] {
  const stem = name.slice(2);
  const names = [];
  for (let length = shortestStem; length <= stem.length; length += 1) names.push(`--${stem.slice(0, length)}`);
  return names as [any, ...any[]];
}

const parser = object({
  all: multiple(flag("-a", "--all")),
  only: multiple(flag("-o", "--only")),
  noOnly: multiple(flag("--no-only")),
  include: multiple(flag("-i", ...prefixes("--include", 3))),
  interactive: multiple(flag("--interactive")),
  patch: multiple(flag("-p", "--patch")),
  amend: multiple(flag("--amend")),
  allowEmpty: multiple(flag("--allow-empty")),
  dryRun: multiple(flag(...prefixes("--dry-run", 2))),
  short: multiple(flag(...prefixes("--short", 2))),
  porcelain: multiple(flag(...prefixes("--porcelain", 3))),
  long: multiple(flag(...prefixes("--long", 1))),
  nul: multiple(flag("-z", ...prefixes("--null", 2))),
  message: optional(option("-m", "--message", string())),
  file: optional(option("-F", "--file", string())),
  reuseMessage: optional(option("-C", "--reuse-message", string())),
  reeditMessage: optional(option("-c", "--reedit-message", string())),
  squash: optional(option("--squash", string())),
  fixup: optional(option("--fixup", string())),
  author: optional(option("--author", string())),
  date: optional(option("--date", string())),
  cleanup: optional(option("--cleanup", string())),
  trailer: multiple(option("--trailer", string())),
  template: optional(option("-t", "--template", string())),
  unified: optional(option("-U", "--unified", string())),
  interHunkContext: optional(option("--inter-hunk-context", string())),
  pathspecFromFile: optional(option("--pathspec-from-file", string())),
  pathspecFileNul: multiple(flag("--pathspec-file-nul")),
  paths: multiple(argument(string())),
  unknown: passThrough({ format: "nextToken" }),
});

const pathCount = 1000;
const paths = Array.from(
  { length: pathCount },
  (_, index) => `package/group-${index % 37}/package-${index}/src/feature-${index}.unit.test.ts`,
);
const args = ["--message", "x", "--", ...paths];

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
