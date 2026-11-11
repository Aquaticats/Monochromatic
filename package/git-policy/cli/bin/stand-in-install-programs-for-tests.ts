/**
 Stand-in `mise`, `git` and `journalctl` for the install command's tests, with a throwaway checkout to install into.

 The install is a command that writes a real lockfile, makes a real commit and reads a real Git directory, so its fixture is
 a real checkout under the scratch parent: the stand-in `git` answers `--version` with the wrapper's own line, which is the
 one thing a throwaway checkout's real Git cannot say, and passes every other command to the first `git` after its own
 directory, so the commit, the diff and the Git directory queries are real.
 */
import {
  mkdir,
  mkdtemp,
  readFile,
  rm,
  writeFile,
} from 'node:fs/promises';
import { homedir } from 'node:os';
import { join } from 'node:path';

import { runGateCommandOrThrow } from './native-gate-run.ts';

/**
 Permission bits that give the owner alone access.
 */
const ownerOnlyMode = 0o700;

/**
 Scratch parent of every fixture directory; never the system temporary directory (rule `ATH`).
 */
const scratchParent = join(
  homedir(),
  'temp',
  'agent',
);

/**
 Environment variable that names the fixture directory the stand-ins write their calls to.
 */
export const installStandInDirectoryVariable = 'INSTALL_STAND_IN_DIR';

/**
 Environment variable that names the release the installed wrapper reports.
 */
export const installStandInVersionVariable = 'INSTALL_STAND_IN_VERSION';

/**
 Environment variable that names the step whose stand-in fails: `update` or `install`.
 */
export const installStandInFailStepVariable = 'INSTALL_STAND_IN_FAIL_STEP';

/**
 Environment variable that, when it is `1`, makes the stand-in `mise` leave the lockfile as it was.
 */
export const installStandInLockUnchangedVariable = 'INSTALL_STAND_IN_LOCK_UNCHANGED';

/**
 Milliseconds one git command of a fixture is given.
 */
const fixtureGitDeadlineMilliseconds = 120_000;

/**
 The commit identity every fixture commit is made with, so a fixture never depends on this machine's Git configuration.
 */
const fixtureIdentity: readonly string[] = [
  '-c',
  'user.email=install-test@example.invalid',
  '-c',
  'user.name=Install Test',
];

/**
 The stand-in `mise`: records the call, moves the lockfile the way the repository's toolchain tasks do, and answers with
 the summary lines the install prints.
 */
const standInMise = `#!/usr/bin/env node
const fs = require('node:fs');
const path = require('node:path');
const directory = process.env.${installStandInDirectoryVariable};
const version = process.env.${installStandInVersionVariable} ?? '';
const args = process.argv.slice(2);
fs.appendFileSync(path.join(directory, 'mise.jsonl'), JSON.stringify({ args, cwd: process.cwd() }) + '\\n');
const failing = process.env.${installStandInFailStepVariable} ?? '';
const lock = path.join(process.cwd(), 'toolchain.lock.json');
const record = '{"tools":{"cargo:monochromatic-git-policy-cli":"' + version + '"}}\\n';
if (args.includes('toolchain:update')) {
  if (failing === 'update') {
    process.stderr.write('error: the registry refused the resolution\\n');
    process.exit(1);
  }
  if (process.env.${installStandInLockUnchangedVariable} !== '1') {
    fs.writeFileSync(lock, record);
    process.stdout.write('moved cargo:monochromatic-git-policy-cli 0.6.1-alpha.9 -> ' + version + '\\n');
  }
  process.exit(0);
}
if (args.includes('toolchain:install')) {
  if (failing === 'install') {
    process.stderr.write('error: the archive could not be fetched\\n');
    process.exit(1);
  }
  process.stdout.write('installed cargo:monochromatic-git-policy-cli ' + version + ', 3 tools converged\\n');
  process.exit(0);
}
process.stderr.write('the stand-in mise was asked something it does not answer: ' + args.join(' ') + '\\n');
process.exit(2);
`;

/**
 The stand-in `git`: answers `--version` with the wrapper's own line, records every call, and passes everything else to
 the first `git` found after its own directory, so no absolute path to a Git binary is written anywhere here.
 */
const standInGit = `#!/usr/bin/env node
const fs = require('node:fs');
const path = require('node:path');
const { spawnSync } = require('node:child_process');
const directory = process.env.${installStandInDirectoryVariable};
const version = process.env.${installStandInVersionVariable} ?? '';
const args = process.argv.slice(2);
fs.appendFileSync(path.join(directory, 'git.jsonl'), JSON.stringify({ args, cwd: process.cwd() }) + '\\n');
if (args.includes('--version')) {
  process.stdout.write('git version 2.56.0\\nmonochromatic-git-policy-cli ' + version + ', shadowing git at /usr/bin/git\\n');
  process.exit(0);
}
const own = path.resolve(path.dirname(process.argv[1]));
const real = (process.env.PATH ?? '').split(':').filter((entry) => entry !== '').filter((entry) => path.resolve(entry) !== own).map((entry) => path.join(entry, 'git')).find((candidate) => {
  try {
    fs.accessSync(candidate, fs.constants.X_OK);
    return fs.statSync(candidate).isFile();
  } catch {
    return false;
  }
});
if (real === undefined) {
  process.stderr.write('the stand-in git found no git after its own directory\\n');
  process.exit(127);
}
const done = spawnSync(real, args, { stdio: 'inherit' });
process.exit(done.status === null ? 1 : done.status);
`;

/**
 The stand-in `journalctl`: records the call and prints one entry, so a test reads the bounds and the identifiers it was
 asked for without reading this machine's journal.
 */
const standInJournalctl = `#!/usr/bin/env node
const fs = require('node:fs');
const path = require('node:path');
const directory = process.env.${installStandInDirectoryVariable};
fs.appendFileSync(path.join(directory, 'journalctl.jsonl'), JSON.stringify(process.argv.slice(2)) + '\\n');
process.stdout.write('2026-11-11T18:37:02.100000-05:00 host monochromatic-git-policy-cli[1]: the sample commit was answered\\n');
`;

/**
 A throwaway checkout an install writes in, with the stand-in programs first on `PATH`.
 */
export type InstallFixture = AsyncDisposable & {
  /**
   The checkout, whose lockfile is committed and whose common Git directory is real.
   */
  readonly repository: string;
  /**
   The fixture's own directory, which the stand-ins write their calls into.
   */
  readonly directory: string;
  /**
   The environment an install runs with when it should use the stand-ins.
   */
  readonly environment: Readonly<Record<string, string>>;
};

/**
 Runs one git command in the fixture and answers what it printed.

 @param directory - the checkout to run in

 @param args - the arguments after the `-C <checkout>` pair

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
    deadlineMilliseconds: fixtureGitDeadlineMilliseconds,
  })).stdout;
}

/**
 Creates a throwaway checkout holding a committed lockfile that names the release before the one installed.

 @param version - the release the stand-in tool reports and writes into the lockfile

 @param options - `lockUnchanged` leaves the lockfile as it is, which is an install of a release already recorded

 @param options.lockUnchanged - whether the lockfile is left as it is

 @param options.failStep - the step whose stand-in fails

 @param options.timingLine - one recorded timing line to write beside the common Git directory

 @returns the fixture; declare it with `await using`

 @example
 ```ts
 await using fixture = await createInstallFixture({ version: '0.6.1-alpha.10' });
 ```
 */
export async function createInstallFixture({
  version,
  options = {},
}: {
  readonly version: string;
  readonly options?: {
    readonly lockUnchanged?: boolean;
    readonly failStep?: string;
    readonly timingLine?: string;
  };
}): Promise<InstallFixture> {
  await mkdir(
    scratchParent,
    {
      recursive: true,
      mode: ownerOnlyMode,
    },
  );
  /**
   Fresh directory for this fixture.
   */
  const directory = await mkdtemp(join(
    scratchParent,
    'git-policy-install-',
  ));
  /**
   The checkout the install writes in.
   */
  const repository = join(
    directory,
    'repo',
  );
  /**
   Directory that goes first on `PATH` and holds the stand-in programs.
   */
  const programs = join(
    directory,
    'path',
  );
  await mkdir(
    repository,
    { recursive: true, },
  );
  await mkdir(
    programs,
    { recursive: true, },
  );
  await Promise.all([
    writeFile(
      join(
        programs,
        'mise',
      ),
      standInMise,
      { mode: ownerOnlyMode, },
    ),
    writeFile(
      join(
        programs,
        'git',
      ),
      standInGit,
      { mode: ownerOnlyMode, },
    ),
    writeFile(
      join(
        programs,
        'journalctl',
      ),
      standInJournalctl,
      { mode: ownerOnlyMode, },
    ),
    writeFile(
      join(
        repository,
        'toolchain.lock.json',
      ),
      '{"tools":{"cargo:monochromatic-git-policy-cli":"0.6.1-alpha.9"}}\n',
    ),
  ]);
  await git({
    directory: repository,
    args: ['init', '--quiet', '--initial-branch=main', '.',],
  });
  await git({
    directory: repository,
    args: ['add', 'toolchain.lock.json',],
  });
  await git({
    directory: repository,
    args: [
      ...fixtureIdentity,
      'commit',
      '--quiet',
      '-m',
      'first',
      'toolchain.lock.json',
    ],
  });
  if (options.timingLine !== undefined)
    await writeFile(
      join(
        repository,
        '.git',
        'monochromatic-git-policy-cli-commit-timing.jsonl'
      ),
      `${options.timingLine}\n`
    );
  return {
    repository,
    directory,
    environment: {
      ...process.env,
      PATH: `${programs}:${process.env
        .PATH
        ?? ''}`,
      [installStandInDirectoryVariable]: directory,
      [installStandInVersionVariable]: version,
      ...(options.failStep === undefined ? {} : { [installStandInFailStepVariable]: options.failStep }),
      ...(options.lockUnchanged === true ? { [installStandInLockUnchangedVariable]: '1' } : {}),
    } as Readonly<Record<string, string>>,
    async [Symbol.asyncDispose](): Promise<void> {
      await rm(
        directory,
        {
          recursive: true,
          force: true,
        },
      );
    },
  };
}

/**
 The commits one fixture checkout holds, newest first.

 @param repository - the checkout to read

 @returns every commit's subject

 @example
 ```ts
 await commitSubjects({ repository: fixture.repository });
 ```
 */
export async function commitSubjects({ repository }: { readonly repository: string; }): Promise<readonly string[]> {
  return (await git({
    directory: repository,
    args: [
      'log',
      '--format=%s',
    ],
  })).split('\n')
    .filter(function hasText(line) {
      return line !== '';
    });
}

/**
 The text of one file of a fixture checkout, which is how a test reads what a step wrote.

 @param repository - the checkout to read

 @param name - the file's name relative to the checkout

 @returns its text

 @example
 ```ts
 await fileTextOf({ repository: fixture.repository, name: 'toolchain.lock.json' });
 ```
 */
export async function fileTextOf({
  repository,
  name,
}: {
  readonly repository: string;
  readonly name: string;
}): Promise<string> {
  /**
   The file's contents.
   */
  return await readFile(
    join(
      repository,
      name
    ),
    'utf8'
  );
}
