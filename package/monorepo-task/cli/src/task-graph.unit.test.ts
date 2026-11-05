import { execFile, } from 'node:child_process';
import {
  cp,
  mkdir,
  mkdtemp,
  readFile,
  realpath,
  rm,
  writeFile,
} from 'node:fs/promises';
import {
  join,
  resolve,
} from 'node:path';
import { promisify, } from 'node:util';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';

//region The actual source entrypoint consumed by task-cli-setup

/**
 GitHub Actions invokes src/run.ts directly before repository artifacts exist.
 These consumer tests copy that source entry and its configuration into disposable repositories.
 Built library behavior is tested separately when an exported execution interface is available.
 */
const repositoryRoot = resolve(import.meta.dirname, '../../../..',);

/**
 Subprocess adapter for the consumer command.
 */
const executeFile = promisify(execFile,);

/**
 Task definitions accepted by the disposable repository.
 */
type FixtureTasks = Readonly<Record<string, {
  readonly depends?: readonly string[];
  readonly extends?: string;
  readonly run?: string | {
    readonly shell: string;
    readonly run: string;
  };
}>>;

/**
 Builds a terminal task that records its identity without running a real build.

 @param name - Identity whose execution the consumer observes.

 @returns Node task body writing one marker per execution.
 */
function marker(name: string,) {
  const encoded = JSON.stringify(name,);
  const line = JSON.stringify(`${encoded}\n`,);
  return {
    shell: 'node --input-type=module-typescript -e',
    run: `const { appendFileSync } = await import('node:fs');
if (!process.env.TASK_TEST_FIXTURE_ROOT) throw new Error('Missing fixture root');
appendFileSync(process.env.TASK_TEST_FIXTURE_ROOT + '/calls', ${line});`,
  };
}

/**
 Runs the configured source consumer against isolated task files.

 @param tasks - Dependency graph under test.
 @param target - Entry task in the fixture package.

 @returns Identities actually executed by the task CLI.
 */
async function recordedExecutions(
  { tasks, target, consumer = 'source', argv = [], taskSource, rootTarget = false, }: {
    readonly tasks: FixtureTasks;
    readonly target: string;
    readonly consumer?: 'source' | 'built';
    readonly argv?: readonly string[];
    readonly taskSource?: string;
    readonly rootTarget?: boolean;
  },
): Promise<readonly unknown[]> {
  const directory = await mkdtemp(join(scratchRoot(), 'task-graph-',),);
  const root = await realpath(directory,);
  try {
    const fixturePackage = join(root, 'package/fixture/subject',);
    const cliSource = join(root, 'package/monorepo-task/cli/src',);
    await mkdir(fixturePackage, { recursive: true, },);
    await cp(import.meta.dirname, cliSource, { recursive: true, },);
    await cp(join(repositoryRoot, 'task.ts',), join(root, 'task.ts',),);
    await writeFile(join(root, 'pnpm-workspace.yaml',), 'packages: ["package/*/*"]\n',);
    await writeFile(join(root, 'package.json',), '{"type":"module"}\n',);
    await writeFile(join(fixturePackage, 'package.json',), '{"name":"fixture-subject","type":"module"}\n',);
    const serialized = JSON.stringify(JSON.stringify(tasks,),);
    await writeFile(join(fixturePackage, 'task.ts',), taskSource ?? `export default JSON.parse(${serialized});\n`,);
    const entry = consumer === 'source'
      ? join(cliSource, 'run.ts',)
      : join(repositoryRoot, 'package/monorepo-task/cli/dist/final/node/run.mjs',);
    if (consumer === 'built') {
      await writeFile(join(cliSource, 'run.ts',), "throw new Error('The built command delegated to the fixture source entry');\n",);
    }
    await executeFile(process.execPath, [
      entry,
      rootTarget ? target : `//package/fixture/subject:${target}`,
      '--',
      ...argv,
    ], { cwd: root, env: { ...process.env, TASK_TEST_FIXTURE_ROOT: root, }, },);
    return (await readFile(join(root, 'calls',), 'utf8',)).trim().split('\n',).map((line,): unknown => JSON.parse(line,));
  }
  finally {
    await rm(root, { recursive: true, force: true, },);
  }
}

//endregion The actual source entrypoint consumed by task-cli-setup

//region Shared dependency regression cases

await describe({
  name: 'task CLI dependency graph',
  children: [
    ...(['source', 'built',] as const).flatMap(consumer => [
    it({
      name: `${consumer} executes a shared prerequisite once across sequential dependency branches`,
      fn: async () => {
        const tasks = {
          all: { depends: ['left', 'right',], },
          left: { depends: ['shared',], run: marker('left',), },
          right: { depends: ['shared',], run: marker('right',), },
          shared: { run: marker('shared',), },
        } satisfies FixtureTasks;
        const actual = await recordedExecutions({ tasks, target: 'all', consumer, },);
        const expected = Object.entries(tasks,).filter(([, task,]) => 'run' in task).map(([name,]) => name).toSorted();
        expect(actual.toSorted(),).toEqual(expected,);
      },
    },),
    it({
      name: `${consumer} executes a shared prerequisite once across parallel template children`,
      fn: async () => {
        const tasks = {
          build: { extends: 'build', },
          'build:left': { depends: ['shared',], run: marker('build:left',), },
          'build:right': { depends: ['shared',], run: marker('build:right',), },
          shared: { run: marker('shared',), },
        } satisfies FixtureTasks;
        const actual = await recordedExecutions({ tasks, target: 'build', consumer, },);
        const expected = Object.entries(tasks,).filter(([, task,]) => 'run' in task).map(([name,]) => name).toSorted();
        expect(actual.toSorted(),).toEqual(expected,);
      },
    },),
    ],),
    it({
      name: 'built command preserves literal arguments through a shell body',
      fn: async () => {
        const argv = ['space inside', "quote'inside", 'line\nbreak', '$(printf unused)', '../path', '--literal',];
        const tasks = {
          capture: {
            run: `node --input-type=module -e 'import { appendFileSync } from "node:fs"; if (!process.env.TASK_TEST_FIXTURE_ROOT) throw new Error("Missing fixture root"); appendFileSync(process.env.TASK_TEST_FIXTURE_ROOT + "/calls", JSON.stringify(process.argv.slice(1)) + "\\n");' --`,
          },
        } satisfies FixtureTasks;
        const actual = await recordedExecutions({ tasks, target: 'capture', consumer: 'built', argv, },);
        expect(actual,).toEqual([argv,],);
      },
    },),
    it({
      name: 'built command provides function bodies with explicit cwd, environment and arguments',
      fn: async () => {
        const probe = 'literal environment value';
        const argv = ['argument with spaces',];
        const taskSource = String.raw`export default {
          capture: {
            dir: 'nested', env: { PROBE: ${JSON.stringify(probe,)} },
            run: async ({ cwd, env, argv }) => {
              const { appendFile, mkdir } = await import('node:fs/promises');
              const { existsSync } = await import('node:fs');
              const { join, relative } = await import('node:path');
              const root = process.env.TASK_TEST_FIXTURE_ROOT;
              if (!root || cwd !== join(root, 'package/fixture/subject/nested')) throw new Error('Unexpected fixture cwd');
              await mkdir(cwd, { recursive: true });
              await appendFile(join(root, 'calls'), JSON.stringify({
                bodyPath: relative(root, cwd), argv, probe: env.PROBE,
                rootUnchanged: process.cwd() === root && existsSync('pnpm-workspace.yaml')
              }) + '\n');
            }
          }
        };`;
        const actual = await recordedExecutions({ tasks: {}, target: 'capture', consumer: 'built', argv, taskSource, },);
        expect(actual,).toEqual([{
          bodyPath: join('package', 'fixture', 'subject', 'nested',), argv, probe, rootUnchanged: true,
        },],);
      },
    },),
    it({
      name: 'built root fanout uses the same shared dependency execution',
      fn: async () => {
        const tasks = {
          build: { extends: 'build', },
          'build:left': { depends: ['shared',], run: marker('build:left',), },
          'build:right': { depends: ['shared',], run: marker('build:right',), },
          shared: { run: marker('shared',), },
        } satisfies FixtureTasks;
        const actual = await recordedExecutions({ tasks, target: 'build', consumer: 'built', rootTarget: true, },);
        const expected = Object.entries(tasks,).filter(([, task,]) => 'run' in task).map(([name,]) => name).toSorted();
        expect(actual.toSorted(),).toEqual(expected,);
      },
    },),
    ...[false, true,].map(rootTarget => it({
      name: `built command rejects an undeclared prototype name in ${rootTarget ? 'root' : 'package'} tasks`,
      fn: async () => {
        let caught: unknown;
        try {
          await recordedExecutions({ tasks: {}, target: 'constructor', consumer: 'built', rootTarget, },);
        }
        catch (error) {
          caught = error;
        }
        if (!Error.isError(caught,) || !('stderr' in caught))
          throw new Error('Expected the task command itself to reject the undeclared name',);
        expect(String(caught.stderr,),).toContain('TaskSelectionError',);
        expect(String(caught.stderr,),).toContain('constructor',);
      },
    },),),
  ],
},);

//endregion Shared dependency regression cases
