// Runs the two behaviour probes once per build of @optique/core inside the recheck image and says, per build and
// probe, whether its output is identical to the first build's. The first build is the reference.
//
// Deliberately broken copies are always included as controls: a probe proves nothing unless it reports them as
// different. One control breaks the plain-array path of the reference; two break the view path of the build named
// `both`, which only inputs past the prototype's thresholds reach, so the short probe must not notice those two and
// the long probe must.
//
// Run inside the image: node compare-builds-in-container.ts <name>=<checkout folder> ...
// A checkout folder holds packages/core/src.

import { spawnSync, } from 'node:child_process';
import { cpSync, mkdirSync, readFileSync, writeFileSync, } from 'node:fs';

const SUBPATHS = ['constructs', 'message', 'modifiers', 'parser', 'primitives', 'valueparser',] as const;
const WORK = '/tmp/behaviour-probe';
const PROBES = ['behaviour-probe.ts', 'long-input-probe.ts',] as const;

const CONTROLS = [
  {
    name: 'control-plain-path-loses-items',
    from: 'base',
    file: 'modifiers.ts',
    find: '...(added ? context.state : context.state.slice(0, -1)),',
    replace: '...(added ? context.state.slice(0, 2) : context.state.slice(0, -1)),',
    mustDiffer: ['behaviour-probe.ts', 'long-input-probe.ts',],
  },
  {
    name: 'control-buffer-view-reads-the-next-token',
    from: 'both',
    file: 'internal/input-buffer-view.ts',
    find: 'get: (index) => base[offset + index],',
    replace: 'get: (index) => base[offset + index + 1],',
    mustDiffer: ['long-input-probe.ts',],
  },
  {
    name: 'control-state-view-keeps-the-dropped-item',
    from: 'both',
    file: 'internal/persistent-vector.ts',
    find: 'const shorter = vector.dropLast();',
    replace: 'const shorter = vector;',
    mustDiffer: ['long-input-probe.ts',],
  },
] as const;

function mapFor(name: string, checkout: string,): string {
  const path = `${WORK}/${name}.json`;
  const imports = Object.fromEntries(SUBPATHS.map((subpath,) => [`@optique/core/${subpath}`, `file://${checkout}/packages/core/src/${subpath}.ts`,]),);

  writeFileSync(path, JSON.stringify({ imports, },),);

  return path;
}

function probe({ name, checkout, script, }: { name: string; checkout: string; script: string; },): { exit: number | null; output: string; stderr: string; seconds: number; } {
  const startedAt = Date.now();
  const child = spawnSync('deno', ['run', '--no-config', '--import-map', mapFor(name, checkout,), `${import.meta.dirname}/${script}`,], {
    encoding: 'utf8',
    maxBuffer: 512 * 1024 * 1024,
    env: { ...process.env, NO_COLOR: '1', },
  },);

  return { exit: child.status, output: child.stdout.trim(), stderr: child.stderr.slice(0, 1500,), seconds: Math.round((Date.now() - startedAt) / 1000,), };
}

// Paths of the values that differ between two JSON documents, at most `limit` of them.
function differences({ left, right, path, found, limit, }: { left: unknown; right: unknown; path: string; found: string[]; limit: number; },): void {
  if (found.length >= limit) return;

  if (typeof left !== 'object' || typeof right !== 'object' || left === null || right === null) {
    if (left !== right) found.push(`${path}: ${JSON.stringify(left,)?.slice(0, 140,)} != ${JSON.stringify(right,)?.slice(0, 140,)}`,);

    return;
  }

  const keys = new Set([...Object.keys(left,), ...Object.keys(right,),],);

  for (const key of keys) {
    differences({ left: (left as Record<string, unknown>)[key], right: (right as Record<string, unknown>)[key], path: `${path}/${key}`, found, limit, },);
  }
}

mkdirSync(WORK, { recursive: true, },);

const builds = process.argv.slice(2,).map((argument,) => {
  const [name, checkout,] = argument.split('=',);

  if (name === undefined || checkout === undefined) throw new Error(`expected <name>=<checkout folder>, got ${argument}`,);

  return { name, checkout, mustDiffer: [] as readonly string[], };
},);
const reference = builds[0];

if (reference === undefined) throw new Error('name at least one build',);

const controls = CONTROLS.map((control,) => {
  // When the kept patch is among the builds, the break of the state view is made in it, so that the control shows
  // the long probe reaching the view code of the very build being judged.
  const from = control.file === 'internal/persistent-vector.ts' && builds.some((build,) => build.name === 'multiple-final') ? 'multiple-final' : control.from;
  const origin = builds.find((build,) => build.name === from);

  if (origin === undefined) throw new Error(`control ${control.name} needs the build named ${from}`,);

  const checkout = `${WORK}/${control.name}`;

  cpSync(origin.checkout, checkout, { recursive: true, },);

  const path = `${checkout}/packages/core/src/${control.file}`;
  const source = readFileSync(path, 'utf8',);

  if (!source.includes(control.find,)) throw new Error(`control ${control.name} cannot be made: ${path} does not hold the text the break replaces`,);

  writeFileSync(path, source.replaceAll(control.find, control.replace,),);

  return { name: control.name, checkout, mustDiffer: control.mustDiffer as readonly string[], };
},);

let controlsBehaved = true;

for (const script of PROBES) {
  const referenceRun = probe({ name: reference.name, checkout: reference.checkout, script, },);

  console.log(JSON.stringify({ probe: script, build: reference.name, role: 'reference', exit: referenceRun.exit, bytes: referenceRun.output.length, seconds: referenceRun.seconds, stderr: referenceRun.stderr, },),);

  if (referenceRun.exit !== 0) throw new Error(`${script} failed on the reference build`,);

  const referenceRecord: unknown = JSON.parse(referenceRun.output,);

  for (const build of [...controls, ...builds.slice(1,),]) {
    const run = probe({ name: build.name, checkout: build.checkout, script, },);
    const found: string[] = [];

    if (run.exit === 0) differences({ left: referenceRecord, right: JSON.parse(run.output,), path: '', found, limit: 12, },);

    const identical = run.exit === 0 && run.output === referenceRun.output;
    const isControl = build.name.startsWith('control-',);
    const expectation = !isControl ? 'compared' : build.mustDiffer.includes(script,) ? 'control, must differ' : 'control, must not differ';

    if (isControl && identical === build.mustDiffer.includes(script,)) controlsBehaved = false;

    console.log(JSON.stringify({ probe: script, build: build.name, role: expectation, exit: run.exit, bytes: run.output.length, seconds: run.seconds, identical, differences: found, stderr: run.stderr, },),);
  }

  // What the reference run exercised, so that an identical result is known not to be identical emptiness.
  const sections = referenceRecord as Record<string, unknown>;

  console.log(JSON.stringify({
    probe: script,
    referenceSections: Object.keys(sections,),
    threw: referenceRun.output.split('"threw"',).length - 1,
    failed: referenceRun.output.split('"failed',).length - 1,
    successes: referenceRun.output.split('"success":true',).length - 1,
    errors: referenceRun.output.split('"success":false',).length - 1,
    stepsTaken: sections['stepsTaken'],
    replaceLast: JSON.stringify(sections['replaceLast'] ?? sections['branchingReplaceLast'],).slice(0, 1200,),
  },),);
}

console.log(JSON.stringify({ done: true, controlsBehaved, },),);
