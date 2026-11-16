/**
 Controls for the fixture Git: a fixture command runs where the caller named it, whatever the caller's environment
 holds.

 Git reads its repository, its index, its object store, its configuration, its identity, its dates and the programs it
 starts from the environment, and a fixture that inherits those from the test process can be moved, redirected or made
 to fail by a caller that set one of them (a Git hook, a rebase, an editor, another test). So each control below sets
 one of the variables Git reads to a value that would redirect or break the fixture, and asserts that the fixture is
 still built in the folder the caller named and that nothing else changed.

 The variable list is generated, not written per variable: the local names come from the installed Git itself
 (`git rev-parse --local-env-vars`, which prints the repository-selection names for the release that runs the test) and
 the rest from the installed Git's own documentation (`man git`, ENVIRONMENT VARIABLES; `git-config(1)` for the
 `GIT_CONFIG_*` family; `git-init(1)` for `GIT_TEMPLATE_DIR`, `GIT_DEFAULT_HASH` and `GIT_DEFAULT_REF_FORMAT`), read on
 2026-11-15 from `git version 2.56.0`. A later release that prints a further local name gets its row from the first half
 without this file changing.
 */
import { execFileSync } from 'node:child_process';
import {
  mkdir,
  readdir,
  readFile,
  stat,
  writeFile,
} from 'node:fs/promises';
import { join } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  fixtureGit,
  fixtureGitProgram,
} from './fixture-git-that-is-never-the-wrapper-for-tests.ts';
import { createGateScratchDirectory } from './stand-in-gate-programs-for-tests.ts';

/**
 The identity every fixture commit of a control is made with, which a leaked identity variable must not replace.
 */
const fixtureIdentity = {
  name: 'Fixture Git',
  email: 'fixture-git@example.invalid',
} as const;

/**
 The `-c` arguments that give each commit that identity, because the fixture's Git configuration is empty.
 */
const identityArguments = [
  '-c',
  `user.name=${fixtureIdentity.name}`,
  '-c',
  `user.email=${fixtureIdentity.email}`,
] as const;

/**
 The variables the installed Git names when it is asked for its repository-local ones.

 `git rev-parse --local-env-vars` prints them, so the rows follow the release that runs them: a name this release adds
 becomes a row here without an edit, and a name it drops stops being one.
 */
const localEnvironmentVariables: readonly string[] = execFileSync(
  fixtureGitProgram,
  [
    'rev-parse',
    '--local-env-vars',
  ],
  { encoding: 'utf8', },
).split('\n')
  .filter(function hasText(line) {
    return line !== '';
  });

/**
 The further variables the installed Git's documentation names, which a fixture commit reads and which can move it,
 redirect it or make it fail.

 Each entry says what it selects, so a reader can tell why it is a row. The list is the documented half of the picture;
 the local half is printed by the Git that runs, above.
 */
const documentedEnvironmentVariables = [
  /**
   Who made the commit and when: a date Git cannot read fails the commit, and an identity replaces the fixture's own.
   */
  'GIT_AUTHOR_NAME',
  'GIT_AUTHOR_EMAIL',
  'GIT_AUTHOR_DATE',
  'GIT_COMMITTER_NAME',
  'GIT_COMMITTER_EMAIL',
  'GIT_COMMITTER_DATE',
  'EMAIL',
  /**
   Configuration from the environment or from a file: `commit.gpgsign` from here fails a commit that has no key.
   */
  'GIT_CONFIG',
  'GIT_CONFIG_PARAMETERS',
  'GIT_CONFIG_COUNT',
  'GIT_CONFIG_GLOBAL',
  'GIT_CONFIG_SYSTEM',
  'GIT_CONFIG_NOSYSTEM',
  'GIT_ATTR_SOURCE',
  /**
   What `git init` writes and where Git finds the programs it starts.
   */
  'GIT_DEFAULT_HASH',
  'GIT_DEFAULT_REF_FORMAT',
  'GIT_REFERENCE_BACKEND',
  'GIT_TEMPLATE_DIR',
  'GIT_EXEC_PATH',
  /**
   Programs a commit may start: an editor or an askpass program that waits is a commit that never ends.
   */
  'GIT_EDITOR',
  'GIT_SEQUENCE_EDITOR',
  'GIT_ASKPASS',
  'GIT_TERMINAL_PROMPT',
  /**
   Where a trace is written, which is a write outside the fixture when the value names one.
   */
  'GIT_TRACE',
  'GIT_TRACE2_EVENT',
  /**
   How far Git looks for a repository, which decides whether the named folder is the one it works in.
   */
  'GIT_CEILING_DIRECTORIES',
  'GIT_DISCOVERY_ACROSS_FILESYSTEM',
] as const;

/**
 Every variable Git reads that could move, redirect or fail a fixture command, from both halves above.
 */
const controlledVariables: readonly string[] = [
  ...new Set([
    ...localEnvironmentVariables,
    ...documentedEnvironmentVariables,
  ]),
].sort();

/**
 A value of one variable that would redirect the fixture into the decoy checkout or make its commit fail.

 @param name - the variable's name

 @param decoy - the throwaway checkout the control watches, which a redirect would write into

 @param hostileConfig - a configuration file that makes a commit fail, for the variables that select one

 @returns the value to give the variable
 */
function hostileValueOf({
  name,
  decoy,
  hostileConfig,
}: {
  readonly name: string;
  readonly decoy: string;
  readonly hostileConfig: string;
}): string {
  switch (name) {
    case 'GIT_AUTHOR_DATE':
    case 'GIT_COMMITTER_DATE':
      return 'not-a-date';
    case 'GIT_AUTHOR_NAME':
      return 'Hostile Author';
    case 'GIT_AUTHOR_EMAIL':
      return 'hostile-author@example.invalid';
    case 'GIT_COMMITTER_NAME':
      return 'Hostile Committer';
    case 'GIT_COMMITTER_EMAIL':
      return 'hostile-committer@example.invalid';
    case 'EMAIL':
      return 'hostile-fallback@example.invalid';
    case 'GIT_DIR':
      return join(
        decoy,
        '.git',
      );
    case 'GIT_WORK_TREE':
    case 'GIT_CEILING_DIRECTORIES':
      return decoy;
    case 'GIT_IMPLICIT_WORK_TREE':
    case 'GIT_NO_REPLACE_OBJECTS':
    case 'GIT_CONFIG_COUNT':
      return '1';
    case 'GIT_DISCOVERY_ACROSS_FILESYSTEM':
      return '1';
    case 'GIT_TERMINAL_PROMPT':
      return '1';
    case 'GIT_CONFIG_NOSYSTEM':
      return '0';
    case 'GIT_INDEX_FILE':
      return join(
        decoy,
        '.git',
        'index',
      );
    case 'GIT_COMMON_DIR':
      return join(
        decoy,
        'hostile-common',
      );
    case 'GIT_OBJECT_DIRECTORY':
      return join(
        decoy,
        'hostile-objects',
      );
    case 'GIT_ALTERNATE_OBJECT_DIRECTORIES':
      return join(
        decoy,
        'hostile-alternates',
      );
    case 'GIT_GRAFT_FILE':
      return join(
        decoy,
        'hostile-grafts',
      );
    case 'GIT_SHALLOW_FILE':
      return join(
        decoy,
        'hostile-shallow',
      );
    case 'GIT_REPLACE_REF_BASE':
      return 'refs/hostile/';
    case 'GIT_NAMESPACE':
      return 'hostile-namespace';
    case 'GIT_PREFIX':
      return 'hostile-prefix/';
    case 'GIT_CONFIG':
    case 'GIT_CONFIG_GLOBAL':
    case 'GIT_CONFIG_SYSTEM':
      return hostileConfig;
    case 'GIT_CONFIG_PARAMETERS':
      return "'commit.gpgsign=true'";
    case 'GIT_EXEC_PATH':
      return join(
        decoy,
        'hostile-exec',
      );
    case 'GIT_TEMPLATE_DIR':
      return join(
        decoy,
        'hostile-templates',
      );
    case 'GIT_ATTR_SOURCE':
      return join(
        decoy,
        'hostile-attributes',
      );
    case 'GIT_REFERENCE_BACKEND':
    case 'GIT_DEFAULT_HASH':
    case 'GIT_DEFAULT_REF_FORMAT':
      return 'hostile';
    case 'GIT_ASKPASS':
      return join(
        decoy,
        'hostile-askpass',
      );
    case 'GIT_EDITOR':
      return join(
        decoy,
        'hostile-editor',
      );
    case 'GIT_SEQUENCE_EDITOR':
      return join(
        decoy,
        'hostile-sequence-editor',
      );
    case 'GIT_TRACE':
      return join(
        decoy,
        'hostile-trace',
      );
    case 'GIT_TRACE2_EVENT':
      return join(
        decoy,
        'hostile-trace2',
      );
    default:
      return join(
        decoy,
        'hostile-value',
      );
  }
}

/**
 One control: the variables it sets in the test process, and the name it is reported under.

 The pair of variables Git's own documentation says select one repository is a row of its own, because either alone is
 enough to move a commit for one release and both together are what a Git hook leaves behind.
 */
type ControlRow = {
  /**
   The variables to set, in the test process's own environment, before the fixture is built.
   */
  readonly variables: readonly string[];
  /**
   The control's subject, naming those variables.
   */
  readonly subject: string;
};

/**
 The rows of the control, generated from the variable list so a name added there becomes a row.

 @param variables - the variables every row is generated from

 @returns one row per variable, and the pair that selects one repository

 @example
 ```ts
 const rows = controlRowsOf({ variables: ['GIT_DIR', 'GIT_INDEX_FILE'] });
 ```
 */
function controlRowsOf({ variables }: { readonly variables: readonly string[]; }): readonly ControlRow[] {
  return [
    ...variables.map(function rowOf(variable) {
      return {
        variables: [variable,],
        subject: `${variable} is set in the test process's environment`,
      };
    }),
    {
      variables: [
        'GIT_DIR',
        'GIT_WORK_TREE',
      ],
      subject: 'GIT_DIR and GIT_WORK_TREE are both set in the test process\'s environment',
    },
  ];
}

/**
 What one throwaway checkout holds, which a redirect into it would change in some way.

 The entry list catches a directory or a file a redirected command writes, the commit catches a moved `HEAD`, the status
 catches a changed index, and the text catches a changed work tree.
 */
type CheckoutState = {
  /**
   Every path under the checkout, relative to it, sorted.
   */
  readonly entries: readonly string[];
  /**
   The commit `HEAD` names, in the ten digits the controls below compare.
   */
  readonly head: string;
  /**
   What `status --short` prints, which is empty for a clean checkout.
   */
  readonly status: string;
  /**
   The text of the one tracked file.
   */
  readonly tracked: string;
};

/**
 Tells whether one path is there.

 @param path - the path to look at

 @returns whether it is there

 @example
 ```ts
 await pathExists({ path: join(fixture, '.git') });
 ```
 */
async function pathExists({ path }: { readonly path: string; }): Promise<boolean> {
  try {
    await stat(path,);
    return true;
  }
  catch {
    return false;
  }
}

/**
 Make a checkout with one commit and one tracked file, which a redirect would write into.

 @param directory - the folder to make it in

 @returns the checkout's path

 @example
 ```ts
 const decoy = await createDecoyCheckout({ directory: join(scratch.path, 'decoy') });
 ```
 */
async function createDecoyCheckout({ directory }: { readonly directory: string; }): Promise<string> {
  await mkdir(
    directory,
    { recursive: true, },
  );
  await writeFile(
    join(
      directory,
      'tracked.txt'
    ),
    'decoy\n'
  );
  await fixtureGit({
    directory,
    args: [
      'init',
      '--quiet',
      '--initial-branch=main',
      '.',
    ],
  });
  await fixtureGit({
    directory,
    args: [
      'add',
      'tracked.txt',
    ],
  });
  await fixtureGit({
    directory,
    args: [
      ...identityArguments,
      'commit',
      '--quiet',
      '-m',
      'decoy-first',
      'tracked.txt',
    ],
  });
  return directory;
}

/**
 Read what one checkout holds, which a redirect into it would change.

 @param directory - the checkout to read

 @returns its entry list, its `HEAD`, its status and its tracked text

 @example
 ```ts
 const before = await checkoutStateOf({ directory: decoy });
 ```
 */
async function checkoutStateOf({ directory }: { readonly directory: string; }): Promise<CheckoutState> {
  return {
    entries: (await readdir(
      directory,
      { recursive: true, },
    )).sort(),
    head: (await fixtureGit({
      directory,
      args: [
        'rev-parse',
        '--short=10',
        'HEAD',
      ],
    })).trim(),
    status: (await fixtureGit({
      directory,
      args: [
        'status',
        '--short',
      ],
    })).trim(),
    tracked: await readFile(
      join(
        directory,
        'tracked.txt'
      ),
      'utf8'
    ),
  };
}

/**
 Read what one commit holds, as the controls compare it with what they asked for.

 @param directory - the checkout to read

 @returns the commit's subject and its author and committer identity

 @example
 ```ts
 const commit = await commitStateOf({ directory: fixture });
 ```
 */
async function commitStateOf({ directory }: { readonly directory: string; }): Promise<{
  readonly subject: string;
  readonly identity: string;
}> {
  return {
    subject: (await fixtureGit({
      directory,
      args: [
        'log',
        '-1',
        '--format=%s',
      ],
    })).trim(),
    identity: (await fixtureGit({
      directory,
      args: [
        'log',
        '-1',
        '--format=%an|%ae|%cn|%ce',
      ],
    })).trim(),
  };
}

/**
 The environment one row sets, built from the decoy and the hostile configuration file.

 @param row - the row whose variables are set

 @param decoy - the throwaway checkout a redirect would write into

 @param hostileConfig - a configuration file that makes a commit fail

 @returns the values to give the row's variables

 @example
 ```ts
 const values = environmentOf({ row, decoy, hostileConfig });
 ```
 */
function environmentOf({
  row,
  decoy,
  hostileConfig,
}: {
  readonly row: ControlRow;
  readonly decoy: string;
  readonly hostileConfig: string;
}): Readonly<Record<string, string>> {
  return Object.fromEntries(row.variables.map(function valueOf(name) {
    return [name, hostileValueOf({
      name,
      decoy,
      hostileConfig,
    }),];
  },),);
}

/**
 Build one fixture with one of the caller's variables set, and answer where it is and what the decoy then holds.

 The commit is read by the caller, after it has asserted that the fixture holds a `.git` entry: reading a commit out
 of a fixture a redirect left empty fails with Git's "not a git repository", which would name that read instead of the
 redirect.

 @param row - the control's row

 @param scratchPath - the folder the row's throwaway checkouts belong under

 @returns the fixture's folder, whether it holds a `.git` entry, and the decoy's state before and after

 @example
 ```ts
 const built = await buildUnderHostileEnvironment({ row, scratchPath: scratch.path });
 ```
 */
async function buildUnderHostileEnvironment({
  row,
  scratchPath,
}: {
  readonly row: ControlRow;
  readonly scratchPath: string;
}): Promise<{
  readonly fixture: string;
  readonly fixtureHasGitEntry: boolean;
  readonly decoyBefore: CheckoutState;
  readonly decoyAfter: CheckoutState;
}> {
  /**
   The checkout a redirect writes into, which is this row's own so one row cannot spoil the next.
   */
  const decoy = await createDecoyCheckout({ directory: join(
    scratchPath,
    'decoy',
  ), });
  /**
   The configuration file a hostile `GIT_CONFIG*` names, whose one setting fails a commit that has no signing key.
   */
  const hostileConfig = join(
    scratchPath,
    'hostile.gitconfig',
  );
  await writeFile(
    hostileConfig,
    '[commit]\n\tgpgsign = true\n'
  );
  /**
   The decoy as it stands before the fixture is built.
   */
  const decoyBefore = await checkoutStateOf({ directory: decoy, });
  /**
   The fixture's own folder, which every command names with `-C`.
   */
  const fixture = join(
    scratchPath,
    'fixture',
  );
  await mkdir(
    fixture,
    { recursive: true, },
  );
  await writeFile(
    join(
      fixture,
      'tracked.txt'
    ),
    'fixture\n'
  );
  /**
   The values the test process's environment held, which the row puts back in a `finally`.
   */
  const saved = new Map(row.variables.map(function savedValue(name) {
    return [name, process.env[name],];
  },),);
  try {
    for (const [name, value] of Object.entries(environmentOf({
      row,
      decoy,
      hostileConfig,
    })))
      process.env[name] = value;
    await fixtureGit({
      directory: fixture,
      args: [
        'init',
        '--quiet',
        '--initial-branch=main',
        '.',
      ],
    });
    await fixtureGit({
      directory: fixture,
      args: [
        'add',
        'tracked.txt',
      ],
    });
    await fixtureGit({
      directory: fixture,
      args: [
        ...identityArguments,
        'commit',
        '--quiet',
        '-m',
        'control-commit',
        'tracked.txt',
      ],
    });
  }
  finally {
    for (const [name, value] of saved) {
      if (value === undefined)
        delete process.env[name];
      else
        process.env[name] = value;
    }
  }
  return {
    fixture,
    fixtureHasGitEntry: await pathExists({ path: join(
      fixture,
      '.git'
    ), }),
    decoyBefore,
    decoyAfter: await checkoutStateOf({ directory: decoy, }),
  };
}

await describe({
  name: 'fixtureGit, which runs the system Git for a fixture',
  // Each row sets one variable in this process's own environment, so the rows must not overlap: a row that ran beside
  // another would be built under the other's hostile value, which is a defect of the control rather than of the helper.
  concurrency: 1,
  children: controlRowsOf({ variables: controlledVariables, })
    .map(function testOf(row) {
      return it({
        name: `builds the fixture where it was named when ${row.subject}`,
        fn: async () => {
          await using scratch = await createGateScratchDirectory({ prefix: 'fixture-git-control-', });
          /**
           What the fixture and the decoy hold once the row's variable is set.
           */
          const built = await buildUnderHostileEnvironment({
            row,
            scratchPath: scratch.path,
          });
          expect(built.fixtureHasGitEntry,).toBe(true,);
          expect(await commitStateOf({ directory: built.fixture, }),).toEqual({
            subject: 'control-commit',
            identity: `${fixtureIdentity.name}|${fixtureIdentity.email}`
            + `|${fixtureIdentity.name}|${fixtureIdentity.email}`,
          },);
          expect(built.decoyAfter,).toEqual(built.decoyBefore,);
        },
      });
    }),
});
