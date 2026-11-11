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
  type GateCommandOutcome,
  runGateCommand,
  runGateCommandOrThrow,
} from './native-gate-run.ts';
import {
  type GateStandIn,
  createGateScratchDirectory,
  createGateStandIn,
  standInFailureVariable,
  standInStarts,
} from './stand-in-gate-programs-for-tests.ts';

/**
 Milliseconds one step of a test is given, which is far more than a repository of three files needs.
 */
const testDeadlineMilliseconds = 180_000;

/**
 Path of the command under test, which runs as TypeScript.
 */
const mainTipPath = join(
  import.meta.dirname,
  'native-gate-main-tip.ts',
);

/**
 Digits `git rev-parse --short=<digits>` prints, which the label and every assertion below name a commit by.
 */
const shortShaDigits = 10;

/**
 A throwaway checkout with a clean gate worktree one commit behind `main`, which is the state the command moves.

 Nothing in this fixture is the real repository: the worktree is made by the wrapper's own `git worktree add` in a
 directory under the scratch parent, so a test can move and dirty it without touching anyone's work.
 */
export type ThrowawayCheckout = {
  /**
   Absolute path of the checkout.
   */
  readonly repository: string;
  /**
   The gate worktree's name under the checkout's worktrees folder.
   */
  readonly worktree: string;
  /**
   Absolute path of the gate worktree.
   */
  readonly worktreePath: string;
  /**
   The commit `main` points at, in the ten digits the command names it by.
   */
  readonly mainSha: string;
  /**
   The commit the gate worktree is at, before the command moves it.
   */
  readonly gateSha: string;
};

/**
 Run one git command in the fixture and answer what it printed.

 @param directory - the checkout or worktree to run in

 @param args - arguments after the program name

 @returns what git printed on standard output
 */
async function git({
  directory,
  args,
}: {
  readonly directory: string;
  readonly args: readonly string[];
}): Promise<string> {
  return (await runGateCommandOrThrow({
    program: 'git',
    args: [
      '-C',
      directory,
      ...args,
    ],
    deadlineMilliseconds: testDeadlineMilliseconds,
  })).stdout;
}

/**
 Make a checkout whose `main` holds the package a gate works on, one commit ahead of a clean gate worktree.

 @param parent - directory to make the checkout in

 @returns the checkout, its gate worktree and the two commits

 @example
 ```ts
 const checkout = await createThrowawayCheckout({ parent: scratch.path });
 ```
 */
export async function createThrowawayCheckout({ parent }: { readonly parent: string; }): Promise<ThrowawayCheckout> {
  /**
   The checkout's own folder.
   */
  const repository = join(
    parent,
    'repo',
  );
  /**
   The worktree the gate runs in, under the folder the gate commands look in.
   */
  const worktree = 'gate-under-test';
  /**
   Absolute path of that worktree.
   */
  const worktreePath = join(
    repository,
    '.claude',
    'worktrees',
    worktree,
  );
  await mkdir(
    join(
      repository,
      'package',
      'git-policy',
      'cli',
    ),
    { recursive: true, },
  );
  await Promise.all([
    writeFile(
      join(
        repository,
        'package',
        'git-policy',
        'cli',
        'package.json',
      ),
      '{}\n',
    ),
    writeFile(
      join(
        repository,
        'tracked.txt',
      ),
      'tracked\n',
    ),
  ]);
  /**
   The commit identity, given on every commit so the fixture never depends on this machine's git configuration.
   */
  const identity = [
    '-c',
    'user.email=gate-test@example.invalid',
    '-c',
    'user.name=Gate Test',
  ];
  await git({
    directory: repository,
    args: ['init', '--quiet', '--initial-branch=main', '.',],
  });
  await git({
    directory: repository,
    args: [
      'add',
      'tracked.txt',
      'package/git-policy/cli/package.json',
    ],
  });
  await git({
    directory: repository,
    args: [
      ...identity,
      'commit',
      '--quiet',
      '-m',
      'first',
      'tracked.txt',
      'package/git-policy/cli/package.json',
    ],
  });
  await git({
    directory: repository,
    args: [
      'worktree',
      'add',
      '--quiet',
      '--detach',
      worktreePath,
    ],
  });
  /**
   The commit the gate worktree is left at.
   */
  const gateSha = (await git({
    directory: worktreePath,
    args: [
      'rev-parse',
      `--short=${String(shortShaDigits)}`,
      'HEAD',
    ],
  })).trim();
  await writeFile(
    join(
      repository,
      'later.txt',
    ),
    'later\n',
  );
  await git({
    directory: repository,
    args: ['add', 'later.txt',],
  });
  await git({
    directory: repository,
    args: [
      ...identity,
      'commit',
      '--quiet',
      '-m',
      'second',
      'later.txt',
    ],
  });
  return {
    repository,
    worktree,
    worktreePath,
    mainSha: (await git({
      directory: repository,
      args: [
        'rev-parse',
        `--short=${String(shortShaDigits)}`,
        'main',
      ],
    })).trim(),
    gateSha,
  };
}

/**
 Run the command under test.

 @param standIn - the fixture whose stand-in `systemd-run` goes first on `PATH`

 @param checkout - the throwaway checkout to point it at

 @param logDirectory - where the gate's log belongs

 @param extraEnvironment - further environment values, such as the one that refuses the start

 @returns how the command ended, with its output
 */
async function relaunch({
  standIn,
  checkout,
  logDirectory,
  extraEnvironment = {},
}: {
  readonly standIn: GateStandIn;
  readonly checkout: ThrowawayCheckout;
  readonly logDirectory: string;
  readonly extraEnvironment?: Readonly<Record<string, string>>;
}): Promise<GateCommandOutcome> {
  return await runGateCommand({
    program: process.execPath,
    args: [
      mainTipPath,
      '--repository',
      checkout.repository,
      '--worktree',
      checkout.worktree,
      '--log-directory',
      logDirectory,
    ],
    deadlineMilliseconds: testDeadlineMilliseconds,
    environment: {
      ...standIn.environment,
      ...extraEnvironment,
    },
  });
}

/**
 The unit name the stand-in `systemd-run` was asked to start.

 @param standIn - the fixture whose calls are read

 @returns the unit, without its `.service` suffix

 @throws Error when no start was recorded, so a test never reads an empty name as a pass
 */
async function startedUnit({ standIn }: { readonly standIn: GateStandIn; }): Promise<string> {
  /**
   Every start the stand-in recorded.
   */
  const starts = await standInStarts({ directory: standIn.directory, });
  /**
   The unit argument of the first start.
   */
  const unitArgument = starts.at(0)
    ?.find(function isUnitArgument(argument) {
      return argument.startsWith('--unit=');
    });
  if (unitArgument === undefined)
    throw new Error('the stand-in systemd-run was never asked to start a unit');
  return unitArgument.slice('--unit='.length);
}

/**
 Tells whether text is the shape a stamp carries: eight digits, a `T`, six digits, a sign and four digits.

 @param stamp - the text the test read

 @returns whether it is a second-precise ISO 8601 basic timestamp with its offset
 */
function isStampShaped(stamp: string): boolean {
  /**
   The digits of the day, the time and the offset's own digits.
   */
  const digits = `${stamp.slice(0, 8)}${stamp.slice(9, 15)}${stamp.slice(16)}`;
  if ((stamp.length !== 20) || (stamp.charAt(8) !== 'T'))
    return false;
  if ((stamp.charAt(15) !== '+') && (stamp.charAt(15) !== '-'))
    return false;
  for (let index = 0; index < digits.length; index += 1) {
    /**
     The character at this position.
     */
    const character = digits.charAt(index);
    if ((character < '0') || (character > '9'))
      return false;
  }
  return true;
}

await describe({
  name: 'native:gate:main-tip, which gates main\'s tip',
  children: [
    it({
      name: 'detaches the gate worktree at main\'s tip and starts a gate labelled with that commit',
      fn: async () => {
        await using scratch = await createGateScratchDirectory({ prefix: 'git-policy-gate-main-tip-', });
        await using standIn = await createGateStandIn();
        /**
         The throwaway checkout the command is pointed at.
         */
        const checkout = await createThrowawayCheckout({ parent: scratch.path, });
        /**
         How the relaunch ended.
         */
        const launched = await relaunch({
          standIn,
          checkout,
          logDirectory: join(
            scratch.path,
            'logs',
          ),
        });
        expect(launched.status,).toBe(0,);
        expect(checkout.gateSha,).not.toBe(checkout.mainSha,);
        expect(launched.stdout,).toContain(`gate worktree at ${checkout.mainSha}`,);
        expect(launched.stdout,).toContain('launch status 0',);
        expect((await git({
          directory: checkout.worktreePath,
          args: [
            'rev-parse',
            `--short=${String(shortShaDigits)}`,
            'HEAD',
          ],
        })).trim(),).toBe(checkout.mainSha,);
        /**
         The unit the launch started.
         */
        const unit = await startedUnit({ standIn, });
        /**
         What the unit name carries after the worktree's label.
         */
        const stamp = unit.slice(`gpc-gate-under-test-on-${checkout.mainSha}-`.length);
        expect(unit.startsWith(`gpc-gate-under-test-on-${checkout.mainSha}-`),).toBe(true,);
        expect(isStampShaped(stamp),).toBe(true,);
      },
    }),
    it({
      name: 'leaves a worktree holding uncommitted work alone, prints what it holds and exits 2',
      fn: async () => {
        await using scratch = await createGateScratchDirectory({ prefix: 'git-policy-gate-main-tip-', });
        await using standIn = await createGateStandIn();
        /**
         The throwaway checkout the command is pointed at.
         */
        const checkout = await createThrowawayCheckout({ parent: scratch.path, });
        await writeFile(
          join(
            checkout.worktreePath,
            'uncommitted.txt',
          ),
          'not to be thrown away\n',
        );
        /**
         How the relaunch ended.
         */
        const launched = await relaunch({
          standIn,
          checkout,
          logDirectory: join(
            scratch.path,
            'logs',
          ),
        });
        expect(launched.status,).toBe(2,);
        expect(launched.stdout,).toContain('the gate worktree is not clean, so it was not moved',);
        expect(launched.stdout,).toContain('uncommitted.txt',);
        expect((await git({
          directory: checkout.worktreePath,
          args: [
            'rev-parse',
            `--short=${String(shortShaDigits)}`,
            'HEAD',
          ],
        })).trim(),).toBe(checkout.gateSha,);
        expect(await standInStarts({ directory: standIn.directory, }),).toHaveLength(0,);
      },
    }),
    it({
      name: 'starts nothing and says so when the move is fine but the start is refused',
      fn: async () => {
        await using scratch = await createGateScratchDirectory({ prefix: 'git-policy-gate-main-tip-', });
        await using standIn = await createGateStandIn();
        /**
         The throwaway checkout the command is pointed at.
         */
        const checkout = await createThrowawayCheckout({ parent: scratch.path, });
        /**
         How the relaunch ended.
         */
        const launched = await relaunch({
          standIn,
          checkout,
          logDirectory: join(
            scratch.path,
            'logs',
          ),
          extraEnvironment: { [standInFailureVariable]: '1', },
        });
        expect(launched.status,).toBe(1,);
        expect(launched.stdout,).toContain(`gate worktree at ${checkout.mainSha}`,);
        expect(launched.stdout,).toContain('launch status 1',);
        expect(launched.stderr,).toContain('systemd-run --user failed with status 1',);
      },
    }),
    it({
      name: 'refuses a positional argument, because every value this command takes is a named option',
      fn: async () => {
        await using scratch = await createGateScratchDirectory({ prefix: 'git-policy-gate-main-tip-', });
        await using standIn = await createGateStandIn();
        /**
         The throwaway checkout the command is pointed at.
         */
        const checkout = await createThrowawayCheckout({ parent: scratch.path, });
        /**
         How the relaunch ended.
         */
        const launched = await runGateCommand({
          program: process.execPath,
          args: [
            mainTipPath,
            '--repository',
            checkout.repository,
            '--log-directory',
            join(
              scratch.path,
              'logs',
            ),
            'stray',
          ],
          deadlineMilliseconds: testDeadlineMilliseconds,
          environment: standIn.environment,
        });
        expect(launched.status,).toBe(1,);
        expect(launched.stderr,).toContain('takes no positional arguments',);
      },
    }),
    it({
      name: 'refuses when git cannot name main, rather than moving the worktree to nothing',
      fn: async () => {
        await using scratch = await createGateScratchDirectory({ prefix: 'git-policy-gate-main-tip-', });
        await using standIn = await createGateStandIn();
        /**
         How the refused relaunch ended.
         */
        const launched = await runGateCommand({
          program: process.execPath,
          args: [
            mainTipPath,
            '--repository',
            scratch.path,
            '--log-directory',
            join(
              scratch.path,
              'logs',
            ),
          ],
          deadlineMilliseconds: testDeadlineMilliseconds,
          environment: standIn.environment,
        });
        expect(launched.status,).toBe(1,);
        expect(launched.stderr,).toContain('git -C failed with status 128',);
      },
    }),
  ],
});
