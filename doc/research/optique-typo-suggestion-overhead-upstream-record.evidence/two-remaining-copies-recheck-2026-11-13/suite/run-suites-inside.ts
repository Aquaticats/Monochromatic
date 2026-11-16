// Run inside the work container, with the suite fixture mounted at /work and the staged arms at /arms: for each arm
// named on the command line, puts that arm's packages/core/src into the fixture clone, builds the package, and runs
// upstream's packages/core suite on Deno, Node and Bun. Then, on the arm named `both`, makes deliberate breaks of
// the view path one at a time and runs the test files that should notice. One JSON line per step.
//
//   node /suite/run-suites-inside.ts base both views-always

import { spawnSync, } from 'node:child_process';
import { cpSync, existsSync, mkdirSync, readFileSync, rmSync, writeFileSync, } from 'node:fs';

const CORE = '/work/optique/packages/core';
const LOGS = '/work/logs';
const STRIP_COLOUR = /\u001b\[[0-9;]*m/gu;
// Names that must appear in a log for a green run of that tree to count: one test upstream had before the diffs,
// and, where the diffs are applied, one test from each of them.
const EXPECTED_ALWAYS = ['should parse an integer argument',];
const EXPECTED_WITH_STATE_DIFF = ['should keep a state that is extended twice from mixing items', 'refuses to write an element or the length',];
const EXPECTED_WITH_BUFFER_DIFF = ['should keep a buffer consumed twice from mixing tokens',];
const EXPECTED_IN_FINAL = ['should replace the last item of a long state and keep the state before',];
// Files the diffs add or change, for upstream's own format, lint and type checks.
const TOUCHED = [
  'src/modifiers.ts',
  'src/modifiers.test.ts',
  'src/primitives.ts',
  'src/primitives.test.ts',
  'src/internal/array-view.ts',
  'src/internal/array-view.test.ts',
  'src/internal/persistent-vector.ts',
  'src/internal/persistent-vector.test.ts',
  'src/internal/input-buffer-view.ts',
  'src/internal/input-buffer-view.test.ts',
];

function expectedNames(arm: string,): string[] {
  if (arm === 'base' || arm === 'concat' || arm === 'slice-push') return EXPECTED_ALWAYS;

  if (arm.startsWith('multiple-final',)) return [...EXPECTED_ALWAYS, ...EXPECTED_WITH_STATE_DIFF, ...EXPECTED_IN_FINAL,];

  return [...EXPECTED_ALWAYS, ...EXPECTED_WITH_STATE_DIFF, ...EXPECTED_WITH_BUFFER_DIFF,];
}

mkdirSync(LOGS, { recursive: true, },);

function step({ name, command, args, }: { name: string; command: string; args: string[]; },): { exit: number | null; output: string; seconds: number; } {
  const startedAt = Date.now();
  const child = spawnSync(command, args, { cwd: CORE, encoding: 'utf8', maxBuffer: 1024 * 1024 * 1024, },);
  const output = `${child.stdout}${child.stderr}`.replace(STRIP_COLOUR, '',);

  writeFileSync(`${LOGS}/${name}.log`, output,);

  return { exit: child.status, output, seconds: Math.round((Date.now() - startedAt) / 1000,), };
}

function failingLines(output: string,): string[] {
  const lines = output.split('\n',).filter((line,) => /^\s*(✖|not ok|\(fail\))|\.\.\. FAILED|=> \.\//u.test(line,));

  return [...new Set(lines.map((line,) => line.trim().replace(/\s[[(][0-9.]+\s?ms[\])]$/u, '',).slice(0, 220,)),),];
}

function summaryLines(output: string,): string[] {
  return output.split('\n',).filter((line,) => /^(ok|FAILED) \| |^ℹ (tests|suites|pass|fail|skipped|cancelled|todo) |^\s*\d+ (pass|fail|skip|todo)$|^Ran \d+ tests/u.test(line,)).map((line,) => line.trim());
}

function putSource(arm: string,): void {
  rmSync(`${CORE}/src`, { recursive: true, force: true, },);
  cpSync(`/arms/${arm}/packages/core/src`, `${CORE}/src`, { recursive: true, },);
}

const arms = process.argv.slice(2,);
const versions = spawnSync('sh', ['-c', 'node --version; deno --version | head --lines=1; bun --version',], { encoding: 'utf8', },);

console.log(JSON.stringify({ runtimes: versions.stdout.trim().split('\n',), arms, },),);

for (const arm of arms) {
  putSource(arm,);

  const build = step({ name: `${arm}-build`, command: 'pnpm', args: ['build',], },);

  console.log(JSON.stringify({ arm, step: 'build', exit: build.exit, seconds: build.seconds, tail: build.output.trim().split('\n',).slice(-2,).map((line,) => line.slice(0, 200,)), },),);

  const touched = TOUCHED.filter((file,) => existsSync(`${CORE}/${file}`,));

  for (const [check, args,] of [['fmt-check', ['fmt', '--check',],], ['lint', ['lint',],], ['check', ['check',],],] as const) {
    const run = step({ name: `${arm}-${check}`, command: 'deno', args: [...args, ...touched,], },);

    console.log(JSON.stringify({ arm, step: check, files: touched.length, exit: run.exit, tail: run.output.trim().split('\n',).slice(-12,).map((line,) => line.slice(0, 200,)), },),);
  }

  for (
    const [runtime, command, args,] of [
      ['deno', 'deno', ['test', '--allow-read', '--allow-env',],],
      ['node', 'node', ['--test',],],
      ['bun', 'bun', ['test',],],
    ] as const
  ) {
    const run = step({ name: `${arm}-${runtime}`, command, args: [...args,], },);
    const expected = expectedNames(arm,);

    console.log(JSON.stringify({
      arm,
      step: 'suite',
      runtime,
      exit: run.exit,
      seconds: run.seconds,
      summary: summaryLines(run.output,),
      expectedNamesMissing: expected.filter((name,) => !run.output.includes(name,)),
      failing: failingLines(run.output,).slice(0, 60,),
    },),);
  }
}

// region Deliberate breaks of the view path, on the arm with both fixes

const MUTATIONS = [
  {
    name: 'state-view-keeps-the-dropped-item',
    file: 'internal/persistent-vector.ts',
    find: 'const shorter = vector.dropLast();',
    replace: 'const shorter = vector;',
    tests: ['src/modifiers.test.ts', 'src/internal/persistent-vector.test.ts',],
  },
  {
    name: 'multiple-never-drops-the-replaced-item',
    file: 'modifiers.ts',
    find: 'added ? context.state : dropLastFrom(context.state),',
    replace: 'context.state,',
    tests: ['src/modifiers.test.ts',],
  },
  {
    name: 'buffer-view-reads-the-next-token',
    file: 'internal/input-buffer-view.ts',
    find: 'get: (index) => base[offset + index],',
    replace: 'get: (index) => base[offset + index + 1],',
    tests: ['src/primitives.test.ts', 'src/internal/input-buffer-view.test.ts',],
  },
  {
    name: 'array-view-reports-the-length-of-its-empty-target',
    file: 'internal/array-view.ts',
    find: ': { ...descriptor, value: elements.length };',
    replace: ': descriptor;',
    tests: ['src/internal/array-view.test.ts',],
  },
  {
    name: 'array-view-lets-a-write-past-its-end-through',
    file: 'internal/array-view.ts',
    find: 'indexOfProperty(property, Number.POSITIVE_INFINITY) !== undefined',
    replace: 'indexOfProperty(property, elementsOf(target).length) !== undefined',
    tests: ['src/internal/array-view.test.ts',],
  },
  {
    name: 'array-view-lets-itself-be-made-non-extensible',
    file: 'internal/array-view.ts',
    find: '    return false;\n  },',
    replace: '    return Reflect.preventExtensions(arguments[0]);\n  },',
    tests: ['src/internal/array-view.test.ts',],
  },
  {
    name: 'state-view-reads-the-wrong-item',
    file: 'internal/persistent-vector.ts',
    find: 'get: (index) => vector.get(index),',
    replace: 'get: (index) => vector.get(index === 0 ? 1 : index),',
    tests: ['src/modifiers.test.ts', 'src/internal/persistent-vector.test.ts',],
  },
] as const;

const mutated = ['both', 'multiple-final',].find((arm,) => arms.includes(arm,));

if (mutated !== undefined) {
  putSource(mutated,);

  for (const mutation of MUTATIONS) {
    const path = `${CORE}/src/${mutation.file}`;

    if (!existsSync(path,)) {
      console.log(JSON.stringify({ mutation: mutation.name, arm: mutated, skipped: `${mutation.file} is not in this arm`, },),);
      continue;
    }

    const original = readFileSync(path, 'utf8',);
    const found = original.split(mutation.find,).length - 1;

    if (found === 0) {
      console.log(JSON.stringify({ mutation: mutation.name, arm: mutated, skipped: `${mutation.file} does not hold the text this break replaces`, },),);
      continue;
    }

    writeFileSync(path, original.replaceAll(mutation.find, mutation.replace,),);

    const perFile = mutation.tests.map((test,) => {
      const run = step({ name: `mutation-${mutation.name}-${test.split('/',).at(-1,)}`, command: 'deno', args: ['test', '--allow-read', '--allow-env', test,], },);

      return { test, exit: run.exit, summary: summaryLines(run.output,), failing: failingLines(run.output,).slice(0, 12,), };
    },);

    writeFileSync(path, original,);
    console.log(JSON.stringify({ mutation: mutation.name, arm: mutated, sites: found, perFile, },),);
  }

  const restored = step({ name: 'after-mutations-modifiers-and-primitives', command: 'deno', args: ['test', '--allow-read', '--allow-env', 'src/modifiers.test.ts', 'src/primitives.test.ts', 'src/internal/',], },);

  console.log(JSON.stringify({ step: 'after-mutations', exit: restored.exit, summary: summaryLines(restored.output,), },),);
}

// endregion Deliberate breaks of the view path

console.log(JSON.stringify({ done: true, },),);
