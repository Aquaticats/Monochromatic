import {
  mkdir,
  writeFile,
} from 'node:fs/promises';
import { join } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  type GateStandIn,
  createGateStandIn,
  standInFailureVariable,
  standInStarts,
} from './stand-in-gate-programs-for-tests.ts';
import {
  type GateCommandOutcome,
  runGateCommand,
} from './native-gate-run.ts';

/**
 Milliseconds a launch of the command under test is given, which is far more than a `node` start needs.
 */
const testDeadlineMilliseconds = 60_000;

/**
 Path of the command under test, which runs as TypeScript.
 */
const launchPath = join(
  import.meta.dirname,
  'native-gate-launch.ts',
);

/**
 A checkout with one gate worktree in it, which is what the launcher looks for.

 `package/git-policy/cli` is what a gate works on, so a worktree without it has nothing to gate.
 */
export type GateRepository = {
  /**
   Absolute path of the checkout.
   */
  readonly repository: string;
  /**
   The worktree's name under the checkout's worktrees folder.
   */
  readonly worktree: string;
  /**
   Absolute path of the worktree.
   */
  readonly worktreePath: string;
};

/**
 Create a checkout with one worktree that holds the package, and a second that holds nothing.

 @param parent - directory to make the checkout in

 @returns the checkout and the worktree that can be gated

 @example
 ```ts
 const repository = await createGateRepository({ parent: standIn.directory });
 ```
 */
export async function createGateRepository({ parent }: { readonly parent: string; }): Promise<GateRepository> {
  /**
   The checkout's own folder.
   */
  const repository = join(
    parent,
    'repo',
  );
  /**
   The worktree that holds the package a gate works on.
   */
  const worktreePath = join(
    repository,
    '.claude',
    'worktrees',
    'gate-under-test',
  );
  await mkdir(
    join(
      worktreePath,
      'package',
      'git-policy',
      'cli',
    ),
    { recursive: true, },
  );
  await writeFile(
    join(
      worktreePath,
      'package',
      'git-policy',
      'cli',
      'package.json',
    ),
    '{}\n',
  );
  await mkdir(
    join(
      repository,
      '.claude',
      'worktrees',
      'nothing-to-gate',
    ),
    { recursive: true, },
  );
  return {
    repository,
    worktree: 'gate-under-test',
    worktreePath,
  };
}

/**
 Run the command under test.

 @param standIn - the fixture whose stand-ins go first on `PATH`

 @param args - arguments after the script path

 @param extraEnvironment - further environment values, such as the one that refuses the start

 @returns how the command ended, with its output
 */
async function launch({
  standIn,
  args,
  extraEnvironment = {},
}: {
  readonly standIn: GateStandIn;
  readonly args: readonly string[];
  readonly extraEnvironment?: Readonly<Record<string, string>>;
}): Promise<GateCommandOutcome> {
  return await runGateCommand({
    program: process.execPath,
    args: [
      launchPath,
      ...args,
    ],
    deadlineMilliseconds: testDeadlineMilliseconds,
    environment: {
      ...standIn.environment,
      ...extraEnvironment,
    },
  });
}

await describe({
  name: 'native:gate, which starts one gate as its own unit',
  children: [
    it({
      name: 'starts the unit with the caps, the worktree and the log and prints where both are',
      fn: async () => {
        await using standIn = await createGateStandIn();
        /**
         The checkout the stand-in launcher is pointed at.
         */
        const repository = await createGateRepository({ parent: standIn.directory, });
        /**
         Where the gate's log belongs.
         */
        const logDirectory = join(
          standIn.directory,
          'logs',
        );
        /**
         How the launch ended.
         */
        const launched = await launch({
          standIn,
          args: [
            '--repository',
            repository.repository,
            '--log-directory',
            logDirectory,
            repository.worktree,
            'wrapper-next-alpha',
          ],
        });
        expect(launched.status,).toBe(0,);
        /**
         The last line the launch printed, which names the unit and its log.
         */
        const answer = launched.stdout.trim()
          .split('\n')
          .at(-1)
          ?? '';
        /**
         Where `.service=` splits that line.
         */
        const separatorIndex = answer.indexOf('.service=');
        /**
         The unit the launch started.
         */
        const unit = answer.slice(0, separatorIndex);
        /**
         The log path the launch printed.
         */
        const log = answer.slice(separatorIndex + '.service='.length);
        expect(unit.startsWith('gpc-gate-wrapper-next-alpha-'),).toBe(true,);
        expect(unit.slice('gpc-gate-wrapper-next-alpha-'.length).length,).toBe(20,);
        expect(log,).toBe(join(
          logDirectory,
          `${unit}.log`,
        ),);
        /**
         The start the stand-in was asked for.
         */
        const [start] = await standInStarts({ directory: standIn.directory, });
        expect(start,).toContain(`--unit=${unit}`,);
        expect(start,).toContain(`--working-directory=${repository.worktreePath}`,);
        expect(start,).toContain(`--setenv=GIT_POLICY_NATIVE_IMAGE_TAG=${unit}`,);
        expect(start,).toContain('--property=CPUQuota=1000%',);
        expect(start,).toContain('--property=MemoryMax=20G',);
        expect(start,).toContain('--property=TasksMax=4000',);
        expect(start,).toContain('--property=IOWeight=10',);
        expect(start,).toContain(`--property=StandardOutput=truncate:${log}`,);
        expect(start?.at(-1),).toContain('--pids-limit 512',);
      },
    }),
    it({
      name: 'makes the log folder before the unit starts, because systemd writes the file and never the folder',
      fn: async () => {
        await using standIn = await createGateStandIn();
        /**
         The checkout the stand-in launcher is pointed at.
         */
        const repository = await createGateRepository({ parent: standIn.directory, });
        /**
         A log folder that does not exist yet, two levels deep.
         */
        const logDirectory = join(
          standIn.directory,
          'not-yet',
          'gate',
        );
        await launch({
          standIn,
          args: [
            '--repository',
            repository.repository,
            '--log-directory',
            logDirectory,
            repository.worktree,
            'wrapper-next-alpha',
          ],
        });
        expect(await standInStarts({ directory: standIn.directory, }),).toHaveLength(1,);
      },
    }),
    it({
      name: 'refuses a failed start with the status and the last line the start printed, and no stack trace',
      fn: async () => {
        await using standIn = await createGateStandIn();
        /**
         The checkout the stand-in launcher is pointed at.
         */
        const repository = await createGateRepository({ parent: standIn.directory, });
        /**
         How the refused launch ended.
         */
        const launched = await launch({
          standIn,
          args: [
            '--repository',
            repository.repository,
            '--log-directory',
            join(
              standIn.directory,
              'logs',
            ),
            repository.worktree,
            'wrapper-next-alpha',
          ],
          extraEnvironment: { [standInFailureVariable]: '1', },
        });
        expect(launched.status,).toBe(1,);
        expect(launched.stderr,).toContain('systemd-run --user failed with status 1',);
        expect(launched.stderr,).toContain('the stand-in was asked to refuse this start',);
        expect(launched.stderr,).not.toContain('    at ',);
      },
    }),
    it({
      name: 'refuses a label that cannot name a unit, before anything starts',
      fn: async () => {
        await using standIn = await createGateStandIn();
        /**
         The checkout the stand-in launcher is pointed at.
         */
        const repository = await createGateRepository({ parent: standIn.directory, });
        /**
         How the refused launch ended.
         */
        const launched = await launch({
          standIn,
          args: [
            '--repository',
            repository.repository,
            '--log-directory',
            join(
              standIn.directory,
              'logs',
            ),
            repository.worktree,
            'Wrapper',
          ],
        });
        expect(launched.status,).toBe(1,);
        expect(launched.stderr,).toContain('the label "Wrapper" must use only lowercase letters',);
        expect(await standInStarts({ directory: standIn.directory, }),).toHaveLength(0,);
      },
    }),
    it({
      name: 'refuses a worktree that holds nothing to gate, naming the folder it looked in',
      fn: async () => {
        await using standIn = await createGateStandIn();
        /**
         The checkout the stand-in launcher is pointed at.
         */
        const repository = await createGateRepository({ parent: standIn.directory, });
        /**
         How the refused launch ended.
         */
        const launched = await launch({
          standIn,
          args: [
            '--repository',
            repository.repository,
            '--log-directory',
            join(
              standIn.directory,
              'logs',
            ),
            'nothing-to-gate',
            'wrapper-next-alpha',
          ],
        });
        expect(launched.status,).toBe(1,);
        expect(launched.stderr,).toContain('package/git-policy/cli',);
        expect(launched.stderr,).toContain('nothing-to-gate',);
      },
    }),
    it({
      name: 'refuses a command line with one positional, and says how the command is called',
      fn: async () => {
        await using standIn = await createGateStandIn();
        /**
         How the refused launch ended.
         */
        const launched = await launch({
          standIn,
          args: ['gate-under-test',],
        });
        expect(launched.status,).toBe(1,);
        expect(launched.stderr,).toContain('usage: native:gate',);
      },
    }),
    it({
      name: 'refuses a third positional instead of ignoring it, so a misread option never reads as a worktree',
      fn: async () => {
        await using standIn = await createGateStandIn();
        /**
         How the refused launch ended.
         */
        const launched = await launch({
          standIn,
          args: [
            'gate-under-test',
            'wrapper-next-alpha',
            'ignored',
          ],
        });
        expect(launched.status,).toBe(1,);
        expect(launched.stderr,).toContain('takes one worktree and one label',);
      },
    }),
    it({
      name: 'refuses caps systemd cannot read, before the unit is queued',
      fn: async () => {
        await using standIn = await createGateStandIn();
        /**
         The checkout the stand-in launcher is pointed at.
         */
        const repository = await createGateRepository({ parent: standIn.directory, });
        /**
         How the refused launch ended.
         */
        const launched = await launch({
          standIn,
          args: [
            '--repository',
            repository.repository,
            '--unit-io-weight',
            '11',
            repository.worktree,
            'wrapper-next-alpha',
          ],
        });
        expect(launched.status,).toBe(1,);
        expect(launched.stderr,).toContain('--unit-io-weight 11 is above 10',);
        expect(await standInStarts({ directory: standIn.directory, }),).toHaveLength(0,);
      },
    }),
  ],
});
