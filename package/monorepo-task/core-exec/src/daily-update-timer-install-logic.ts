/**
 Logic of `mise run toolchain:timer:install`: renders the versioned systemd unit files of the daily update into
 `~/.config/systemd/user` and enables the timer (never the service).

 Run it from the main checkout: the units name that checkout's absolute path. Options: `--no-enable` (write and reload,
 leave the timer disabled), `--unit-directory <path>` (default: `$XDG_CONFIG_HOME/systemd/user`), `--repository <path>`
 (default: the current directory). The entry that calls {@link main} is `daily-update-timer-install.ts`.

 @module
 */

import spawn from 'nano-spawn';
import {
  mkdir,
  readFile,
  realpath,
  writeFile,
} from 'node:fs/promises';
import { homedir, } from 'node:os';
import { join, } from 'node:path';
import { parseArgs, } from 'node:util';

/**
 Directory of the unit sources, relative to the repository.
 */
export const UNIT_SOURCE_DIRECTORY = 'package/monorepo-task/core-exec/systemd';

/**
 Name of the service unit, shared by the source folder and the installed folder.
 */
export const SERVICE_UNIT_NAME = 'toolchain-daily-update.service';

/**
 Name of the timer unit.
 */
export const TIMER_UNIT_NAME = 'toolchain-daily-update.timer';

/**
 Resource caps and time bound of the service, sized from the measured runs in `doc/planning/toolchain-always-latest.md`
 ("Measured run" and "Checking every package after the update").
 The start timeout covers the update (about 21 minutes in the rehearsal), the baseline pass and the second pass, each
 bounded at 90 minutes by `CHECK_PASS_DEADLINE_MS`: 3.4 hours at most, so four hours.
 */
export const SERVICE_LIMITS: Readonly<Record<string, string>> = {
  CPU_QUOTA: '200%',
  MEMORY_MAX: '8G',
  TASKS_MAX: '2048',
  TIMEOUT_START: '4h',
};

/**
 Replaces every `@NAME@` placeholder of a unit template.

 @param template - unit text with placeholders

 @param values - placeholder name (without the at signs) to its value

 @returns the rendered text

 @throws Error when the at signs do not pair up or a placeholder has no value, so a unit is never installed half rendered

 @example
 ```ts
 renderUnit({ template: 'WorkingDirectory=@REPOSITORY@', values: { REPOSITORY: '/r', }, },); // 'WorkingDirectory=/r'
 ```
 */
export function renderUnit({
  template,
  values,
}: {
  readonly template: string;
  readonly values: Readonly<Record<string, string>>;
},): string {
  /**
   Text between at signs: even positions are plain text, odd positions are placeholder names.
   */
  const pieces = template.split('@',);
  if ((pieces.length % 2) === 0) {
    throw new Error('the unit template has an at sign that does not pair with another',);
  }
  /**
   Pieces with each placeholder replaced by its value.
   */
  const substituted = pieces.map(function substitute(
    piece,
    index,
  ) {
    if ((index % 2) === 0) {
      return piece;
    }
    /**
     Value for this placeholder name.
     */
    const value = values[piece];
    if (value === undefined) {
      throw new Error(`the unit template has a placeholder @${piece}@ with no value`,);
    }
    return value;
  },);
  return substituted.join('',);
}

/**
 Finds a program on `PATH`.

 @param program - program name

 @returns its absolute path

 @throws Error when `PATH` has no executable of that name

 @example
 ```ts
 await findOnPath('mise',); // '/home/user/.local/bin/mise'
 ```
 */
export async function findOnPath(program: string,): Promise<string> {
  try {
    return (await spawn(
      'which',
      [program,],
    )).stdout
      .trim();
  }
  catch (error) {
    throw new Error(
      `${program} is not on PATH: ${String(error,)}`,
      { cause: error },
    );
  }
}

/**
 Runs a command and returns its trimmed standard output.

 @param program - program name

 @param args - arguments

 @returns trimmed standard output

 @example
 ```ts
 await outputOf({ program: 'git', args: ['--version',], },);
 ```
 */
export async function outputOf({
  program,
  args,
}: {
  readonly program: string;
  readonly args: readonly string[];
},): Promise<string> {
  return (await spawn(
    program,
    args,
  )).stdout
    .trim();
}

/**
 Checks that a directory is the main checkout, not a linked worktree.

 @param repository - absolute repository path

 @throws Error when the directory is a linked worktree, whose path would not outlive the worktree

 @example
 ```ts
 await requireMainCheckout('/var/home/user/Monochromatic',);
 ```
 */
export async function requireMainCheckout(repository: string,): Promise<void> {
  /**
   Git directory of the checkout the command runs in.
   */
  const gitDirectory = await outputOf({
    program: 'git',
    args: [
      '-C',
      repository,
      'rev-parse',
      '--absolute-git-dir',
    ],
  },);
  /**
   Git directory shared by every worktree of the repository.
   */
  const commonDirectory = join(
    repository,
    await outputOf({
      program: 'git',
      args: [
        '-C',
        repository,
        'rev-parse',
        '--git-common-dir',
      ],
    },),
  );
  if (await realpath(gitDirectory,) !== await realpath(commonDirectory,)) {
    throw new Error(`${repository} is a linked worktree; run this from the main checkout, which the units will name`,);
  }
}

/**
 Installs the units.

 @param argv - command-line arguments after the program name

 @returns the process exit code

 @throws Error when the directory is a linked worktree, a template cannot be rendered, or systemd rejects a unit

 @example
 ```ts
 process.exitCode = await main(process.argv.slice(2,),);
 ```
 */
export async function main(argv: readonly string[],): Promise<number> {
  /**
   Options of the command line.
   */
  const { values, } = parseArgs({
    args: [...argv,],
    options: {
      'no-enable': { type: 'boolean', },
      'unit-directory': { type: 'string', },
      repository: { type: 'string', },
    },
    strict: true,
  },);
  /**
   Absolute path of the main checkout the units will name.
   */
  const repository = await realpath(values.repository ?? process.cwd(),);
  await requireMainCheckout(repository,);
  /**
   Folder the rendered units are written to.
   */
  const unitDirectory = values['unit-directory']
    ?? join(
      process.env
        .XDG_CONFIG_HOME
        ?? join(
          homedir(),
          '.config',
        ),
      'systemd',
      'user',
    );
  /**
   Values of the unit templates' placeholders.
   */
  const placeholders: Readonly<Record<string, string>> = {
    REPOSITORY: repository,
    HOME: homedir(),
    MISE: await findOnPath('mise',),
    ...SERVICE_LIMITS,
  };

  await mkdir(
    unitDirectory,
    { recursive: true, },
  );
  // The two units are separate files, so they are rendered side by side.
  await Promise.all([
    SERVICE_UNIT_NAME,
    TIMER_UNIT_NAME,
  ].map(async function writeUnit(unitName,) {
    /**
     Text of the unit source.
     */
    const template = await readFile(
      join(
        repository,
        UNIT_SOURCE_DIRECTORY,
        unitName,
      ),
      'utf8',
    );
    await writeFile(
      join(
        unitDirectory,
        unitName,
      ),
      renderUnit({
        template,
        values: placeholders,
      },),
    );
  },),);
  await spawn(
    'systemd-analyze',
    [
      '--user',
      'verify',
      join(
        unitDirectory,
        SERVICE_UNIT_NAME,
      ),
      join(
        unitDirectory,
        TIMER_UNIT_NAME,
      ),
    ],
    { stdio: 'inherit', },
  );
  await spawn(
    'systemctl',
    [
      '--user',
      'daemon-reload',
    ],
    { stdio: 'inherit', },
  );
  if (values['no-enable'] === true) {
    process.stderr
      .write('installed the units; the timer is not enabled (--no-enable)\n',);
    return 0;
  }
  await spawn(
    'systemctl',
    [
      '--user',
      'enable',
      '--now',
      TIMER_UNIT_NAME,
    ],
    { stdio: 'inherit', },
  );
  return 0;
}
