/**
 Stand-in `mise`, `git` and `journalctl` for the install command's tests, with a throwaway checkout to install into.

 The install is a command that writes a real lockfile, makes a real commit and reads a real Git directory, so its fixture is
 a real checkout under the scratch parent: the stand-in `git` answers `--version` with the wrapper's own line, which is the
 one thing a throwaway checkout's real Git cannot say, and passes every other command to the first `git` after its own
 directory, so the commit, the diff and the Git directory queries are real.

 The questions a test asks of the checkout afterwards are in `install-fixture-checkout-reads-for-tests.ts`, which this file
 uses and which uses nothing of this one.
 */
import {
  mkdir,
  mkdtemp,
  rm,
  writeFile,
} from 'node:fs/promises';
import { homedir } from 'node:os';
import { join } from 'node:path';

import { fixtureGit } from './install-fixture-checkout-reads-for-tests.ts';

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
 Environment variable that names a release another process writes into the lockfile while the install runs.

 The stand-in `git` writes it when it is asked for `--version`, which is the moment between the install of the tools and
 the commit of the lock: a concurrent `toolchain:update` of another tool reaches the lock there.
 */
export const installStandInConcurrentLockVersionVariable = 'INSTALL_STAND_IN_CONCURRENT_LOCK_VERSION';

/**
 Environment variable that, when it is `1`, makes the stand-in `git` commit this release's lock itself at `--version`.

 That is another install of the same release finishing its commit first, which leaves this install with nothing to
 record and must not be an error.
 */
export const installStandInConcurrentCommitVariable = 'INSTALL_STAND_IN_CONCURRENT_COMMIT';

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

 Two things another process does to the lockfile are modelled here, at the moment between installing the tools and
 committing the lock: writing another release into it, and committing this release's lock itself.
 */
const standInGit = `#!/usr/bin/env node
const fs = require('node:fs');
const path = require('node:path');
const { spawnSync } = require('node:child_process');
const directory = process.env.${installStandInDirectoryVariable};
const version = process.env.${installStandInVersionVariable} ?? '';
const args = process.argv.slice(2);
fs.appendFileSync(path.join(directory, 'git.jsonl'), JSON.stringify({ args, cwd: process.cwd() }) + '\\n');
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
const runReal = (realArgs) => spawnSync(real, realArgs, { stdio: 'inherit' });
if (args.includes('--version')) {
  const concurrentLockVersion = process.env.${installStandInConcurrentLockVersionVariable} ?? '';
  if (concurrentLockVersion !== '') {
    fs.writeFileSync(path.join(process.cwd(), 'toolchain.lock.json'), '{"tools":{"cargo:monochromatic-git-policy-cli":"' + concurrentLockVersion + '"}}\\n');
  }
  if (process.env.${installStandInConcurrentCommitVariable} === '1') {
    const done = runReal(['commit', '--message=chore(toolchain): monochromatic-git-policy-cli ' + version, '--', 'toolchain.lock.json']);
    if (done.status !== 0) process.exit(done.status === null ? 1 : done.status);
  }
  process.stdout.write('git version 2.56.0\\nmonochromatic-git-policy-cli ' + version + ', shadowing git at /usr/bin/git\\n');
  process.exit(0);
}
const done = runReal(args);
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
 The lockfile text that records one release.

 @param version - the release the lockfile names

 @returns the whole lockfile, one line

 @example
 ```ts
 lockRecordText({ version: '0.6.1-alpha.10' });
 ```
 */
export function lockRecordText({ version }: { readonly version: string; }): string {
  return `{"tools":{"cargo:monochromatic-git-policy-cli":"${version}"}}\n`;
}

/**
 Creates a throwaway checkout holding a committed lockfile that names the release before the one installed.

 @param version - the release the stand-in tool reports and writes into the lockfile

 @param options - what the fixture's stand-ins are told: `lockUnchanged` leaves the lockfile as it is, which is an
   install of a release already recorded, `stagedLockVersion` writes that release into the lockfile and stages it before
   the install begins, `concurrentLockVersion` is a release another process writes into the lockfile while the install
   runs, `concurrentCommit` makes another process commit this release's lock first, `failStep` names the step whose
   stand-in fails, and `timingLine` is one recorded timing line to write beside the common Git directory

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
    readonly stagedLockVersion?: string;
    readonly concurrentLockVersion?: string;
    readonly concurrentCommit?: boolean;
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
  await fixtureGit({
    directory: repository,
    args: [
      'init',
      '--quiet',
      '--initial-branch=main',
      '.',
    ],
  });
  await fixtureGit({
    directory: repository,
    args: [
      'add',
      'toolchain.lock.json',
    ],
  });
  await fixtureGit({
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
  if (options.stagedLockVersion !== undefined) {
    await writeFile(
      join(
        repository,
        'toolchain.lock.json'
      ),
      lockRecordText({ version: options.stagedLockVersion, })
    );
    await fixtureGit({
      directory: repository,
      args: [
        'add',
        'toolchain.lock.json',
      ],
    });
  }
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
      // The identity a stand-in commit is made with, so a fixture never depends on this machine's Git configuration.
      GIT_AUTHOR_NAME: 'Install Test',
      GIT_AUTHOR_EMAIL: 'install-test@example.invalid',
      GIT_COMMITTER_NAME: 'Install Test',
      GIT_COMMITTER_EMAIL: 'install-test@example.invalid',
      [installStandInDirectoryVariable]: directory,
      [installStandInVersionVariable]: version,
      ...(options.failStep === undefined ? {} : { [installStandInFailStepVariable]: options.failStep }),
      ...(options.lockUnchanged === true ? { [installStandInLockUnchangedVariable]: '1' } : {}),
      ...(options.concurrentLockVersion === undefined ? {} : { [installStandInConcurrentLockVersionVariable]: options.concurrentLockVersion }),
      ...(options.concurrentCommit === true ? { [installStandInConcurrentCommitVariable]: '1' } : {}),
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
