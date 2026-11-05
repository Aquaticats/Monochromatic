import { execFile, } from 'node:child_process';
import {
  cp,
  mkdir,
  mkdtemp,
  readFile,
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
  readonly run?: {
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
  return {
    shell: 'node --input-type=module-typescript -e',
    run: `const { appendFileSync } = await import('node:fs');
appendFileSync(process.env.TASK_REPO_ROOT + '/calls', ${JSON.stringify(`${name}\n`,)});`,
  };
}

/**
 Runs the configured source consumer against isolated task files.

 @param tasks - Dependency graph under test.
 @param target - Entry task in the fixture package.

 @returns Identities actually executed by the task CLI.
 */
async function recordedExecutions(
  { tasks, target, }: { readonly tasks: FixtureTasks; readonly target: string; },
): Promise<readonly string[]> {
  const root = await mkdtemp(join(scratchRoot(), 'task-graph-',),);
  try {
    const fixturePackage = join(root, 'package/fixture/subject',);
    const cliSource = join(root, 'package/monorepo-task/cli/src',);
    await mkdir(fixturePackage, { recursive: true, },);
    await cp(import.meta.dirname, cliSource, { recursive: true, },);
    await cp(join(repositoryRoot, 'task.ts',), join(root, 'task.ts',),);
    await writeFile(join(root, 'pnpm-workspace.yaml',), 'packages: ["package/*/*"]\n',);
    await writeFile(join(root, 'package.json',), '{"type":"module"}\n',);
    await writeFile(join(fixturePackage, 'package.json',), '{"name":"fixture-subject","type":"module"}\n',);
    await writeFile(join(fixturePackage, 'task.ts',), `export default ${JSON.stringify(tasks,)};\n`,);
    await executeFile(process.execPath, [
      join(cliSource, 'run.ts',),
      `//package/fixture/subject:${target}`,
    ], { cwd: root, },);
    return (await readFile(join(root, 'calls',), 'utf8',)).trim().split('\n',).toSorted();
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
    it({
      name: 'executes a shared prerequisite once across sequential dependency branches',
      fn: async () => {
        const tasks = {
          all: { depends: ['left', 'right',], },
          left: { depends: ['shared',], run: marker('left',), },
          right: { depends: ['shared',], run: marker('right',), },
          shared: { run: marker('shared',), },
        } satisfies FixtureTasks;
        const actual = await recordedExecutions({ tasks, target: 'all', },);
        const expected = Object.entries(tasks,).filter(([, task,]) => 'run' in task).map(([name,]) => name).toSorted();
        expect(actual,).toEqual(expected,);
      },
    },),
    it({
      name: 'executes a shared prerequisite once across parallel template children',
      fn: async () => {
        const tasks = {
          build: { extends: 'build', },
          'build:left': { depends: ['shared',], run: marker('build:left',), },
          'build:right': { depends: ['shared',], run: marker('build:right',), },
          shared: { run: marker('shared',), },
        } satisfies FixtureTasks;
        const actual = await recordedExecutions({ tasks, target: 'build', },);
        const expected = Object.entries(tasks,).filter(([, task,]) => 'run' in task).map(([name,]) => name).toSorted();
        expect(actual,).toEqual(expected,);
      },
    },),
  ],
},);

//endregion Shared dependency regression cases
