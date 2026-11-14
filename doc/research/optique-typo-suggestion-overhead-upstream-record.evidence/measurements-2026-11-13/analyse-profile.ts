// Reflects a `node --cpu-prof` profile as self time per function, so the
// remaining cost of a token parse can be named instead of guessed.
//
// Run: node analyse-profile.ts <profile.cpuprofile> [topCount]

import { readFileSync } from "node:fs";
import { basename } from "node:path";

interface CallFrame {
  readonly functionName: string;
  readonly url: string;
  readonly lineNumber: number;
}

interface ProfileNode {
  readonly id: number;
  readonly callFrame: CallFrame;
  readonly children?: readonly number[];
}

interface CpuProfile {
  readonly nodes: readonly ProfileNode[];
  readonly samples: readonly number[];
  readonly timeDeltas: readonly number[];
  readonly startTime: number;
  readonly endTime: number;
}

const profilePath = process.argv[2];
const topCount = Number(process.argv[3] ?? "25");
if (profilePath == null) throw new Error("Give a .cpuprofile file.");

const profile = JSON.parse(readFileSync(profilePath, "utf8")) as CpuProfile;
const nodesById = new Map<number, ProfileNode>(
  profile.nodes.map((node) => [node.id, node]),
);
const parents = new Map<number, number>();
for (const node of profile.nodes) {
  for (const child of node.children ?? []) parents.set(child, node.id);
}

// Only samples under a parseSync() frame count, so module loading and the
// harness itself stay out of the attribution.
const parseRoots = new Set<number>(
  profile.nodes
    .filter((node) => node.callFrame.functionName === "parseSync")
    .map((node) => node.id),
);

function isInsideParse(nodeId: number): boolean {
  let current: number | undefined = nodeId;
  while (current !== undefined) {
    if (parseRoots.has(current)) return true;
    current = parents.get(current);
  }
  return false;
}

function shortFile(url: string): string {
  if (url === "") return "(v8)";
  if (url.startsWith("node:")) return url;
  return basename(url);
}

const selfMicros = new Map<string, number>();
const microsByFile = new Map<string, number>();
let parseMicros = 0;
let parseSamples = 0;

for (const [index, nodeId] of profile.samples.entries()) {
  const delta = profile.timeDeltas[index] ?? 0;
  const node = nodesById.get(nodeId);
  if (node === undefined) continue;
  if (!isInsideParse(nodeId)) continue;
  parseMicros += delta;
  parseSamples += 1;
  const file = shortFile(node.callFrame.url);
  const key = `${file}#${node.callFrame.functionName}:${node.callFrame.lineNumber}`;
  selfMicros.set(key, (selfMicros.get(key) ?? 0) + delta);
  microsByFile.set(file, (microsByFile.get(file) ?? 0) + delta);
}

const totalMs = (profile.endTime - profile.startTime) / 1000;
const sorted = [...selfMicros.entries()].sort((left, right) => right[1] - left[1]);
const files = [...microsByFile.entries()].sort((left, right) => right[1] - left[1]);

console.log(JSON.stringify({
  profile: basename(profilePath),
  wallMs: Number(totalMs.toFixed(2)),
  parseMs: Number((parseMicros / 1000).toFixed(2)),
  sampleCount: profile.samples.length,
  parseSamples,
  samplingUs: Number((parseMicros / Math.max(parseSamples, 1)).toFixed(1)),
}, null, 2));
console.log("");
console.log("self time per file, in ms of parse time");
for (const [file, micros] of files.slice(0, 15)) {
  console.log(`  ${(micros / 1000).toFixed(2)}\t${((micros / parseMicros) * 100).toFixed(1)}%\t${file}`);
}
console.log("");
console.log(`self time per function, in ms of parse time (top ${topCount})`);
for (const [key, micros] of sorted.slice(0, topCount)) {
  console.log(`  ${(micros / 1000).toFixed(2)}\t${((micros / parseMicros) * 100).toFixed(1)}%\t${key}`);
}
