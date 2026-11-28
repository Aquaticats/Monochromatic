// Checks every quotation the proposal uses against the file it names.
// Whitespace is normalised in both, because a quotation may wrap across lines in its source.
// Run from the worktree root: node package/linter/monochromatic-lint/doc/dylint-driver-proposal-evidence-2026-11-28/scripts/check-quotations.ts
import { readFileSync } from "node:fs";

const here: string = new URL(".", import.meta.url).pathname;
const lines: string[] = readFileSync(`${here}quotations.tsv`, "utf8").split("\n").filter((line: string): boolean => line !== "");
const normalise = (text: string): string => text.replace(/^> ?/gmu, "").replace(/\s+/gu, " ").trim();
let failures: number = 0;
for (const line of lines) {
  const separator: number = line.indexOf("\t");
  const source: string = line.slice(0, separator);
  const quote: string = line.slice(separator + 1);
  let text: string = "";
  try {
    text = readFileSync(source, "utf8");
  } catch (error) {
    process.stderr.write(`MISSING FILE ${source}\n`);
    failures += 1;
    continue;
  }
  if (normalise(text).includes(normalise(quote))) {
    process.stdout.write(`ok   ${source}: ${quote.slice(0, 60)}\n`);
  } else {
    process.stderr.write(`MISS ${source}: ${quote}\n`);
    failures += 1;
  }
}
process.stdout.write(`${lines.length - failures} of ${lines.length} quotations matched\n`);
if (failures > 0) {
  process.exitCode = 1;
}
