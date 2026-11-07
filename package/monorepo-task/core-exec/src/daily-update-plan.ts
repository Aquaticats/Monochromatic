/**
 The ordered steps of the daily update and the files each group may change.

 Rationale: `doc/planning/toolchain-always-latest.md` ("The daily update") explains why the order is what it is and why
 each group lands or fails as a unit.

 @module
 */

/**
 Milliseconds in one second.
 */
const MILLISECONDS_PER_SECOND = 1_000;

/**
 Seconds in one minute.
 */
const SECONDS_PER_MINUTE = 60;

/**
 Milliseconds in one minute.
 */
const MILLISECONDS_PER_MINUTE: number = MILLISECONDS_PER_SECOND * SECONDS_PER_MINUTE;

/**
 Minutes a lock re-resolution may run.
 Measured at under one minute (`doc/planning/toolchain-always-latest.md`, "Measured run"); the bound only stops a hung registry.
 */
const RESOLVE_MINUTES = 15;

/**
 Minutes an install, dependency update, catalog tightening or file sync may run.
 The measured longest was under one minute on warm caches; a cold toolchain download is the case this bound covers.
 */
const INSTALL_MINUTES = 45;

/**
 Longest a lock re-resolution may run, in milliseconds.
 */
export const RESOLVE_TIMEOUT_MS: number = RESOLVE_MINUTES * MILLISECONDS_PER_MINUTE;

/**
 Longest an install, a dependency update, a catalog tightening or a file sync may run, in milliseconds.
 */
export const INSTALL_TIMEOUT_MS: number = INSTALL_MINUTES * MILLISECONDS_PER_MINUTE;

/**
 Which repository-relative paths a group or step covers: whole paths, and paths that end in a suffix.
 */
export type PathSelector = {
  /**
   Paths covered exactly, for example `pnpm-lock.yaml`.
   */
  readonly exact: readonly string[];
  /**
   Suffixes whose paths are covered at any depth, for example `/Cargo.lock`.
   */
  readonly suffixes: readonly string[];
};

/**
 One command the update runs, always through `mise run` or `mise exec` so the repository's own tasks and shadows apply.
 */
export type UpdateCommand = {
  /**
   Words of the command, kept for the log and for the failure message.
   */
  readonly label: string;
  /**
   Program and arguments, spawned without a shell.
   */
  readonly argv: readonly string[];
  /**
   Longest the command may run before it is stopped, in milliseconds.
   */
  readonly timeoutMs: number;
};

/**
 One step: commands that succeed or fail together.
 */
export type UpdateStep = {
  /**
   Name used in the log, the notification and the commit body.
   */
  readonly name: string;
  /**
   Commands in order; the first failure ends the step.
   */
  readonly commands: readonly UpdateCommand[];
  /**
   Whether a failure of this step still lets the group's later steps run.
   A soft step restores its own files and is reported; a hard step restores the whole group and ends it.
   */
  readonly soft: boolean;
  /**
   Files this step may change, restored after a failure.
   */
  readonly touches: PathSelector;
};

/**
 A group: the steps whose files land in one commit, or fail to land together.
 */
export type UpdateGroup = {
  /**
   Name used in the log and the notification.
   */
  readonly name: string;
  /**
   Subject of the commit that lands the group; empty for a group whose files are outside the repository.
   */
  readonly commitSubject: string;
  /**
   Files the group may change and commit.
   */
  readonly owns: PathSelector;
  /**
   Absolute paths outside the repository that the group changes and restores on failure but never commits.
   */
  readonly externalFiles: readonly string[];
  /**
   Steps in the order they run.
   */
  readonly steps: readonly UpdateStep[];
};

/**
 Whether a selector covers a repository-relative path.

 @param selector - the selector

 @param repositoryPath - repository-relative path

 @returns true when the path is listed or ends with a listed suffix

 @example
 ```ts
 selects({ selector: { exact: ['a',], suffixes: ['/b',], }, repositoryPath: 'x/b', },); // true
 ```
 */
export function selects({
  selector,
  repositoryPath,
}: {
  readonly selector: PathSelector;
  readonly repositoryPath: string;
},): boolean {
  return selector.exact
    .includes(repositoryPath,)
    || selector.suffixes
      .some(function endsWithSuffix(suffix,) {
        return repositoryPath.endsWith(suffix,);
      },);
}

/**
 Builds a mise task command.

 @param task - task name, for example `toolchain:update`

 @param timeoutMs - longest the task may run

 @returns the command

 @example
 ```ts
 miseTask({ task: 'toolchain:update', timeoutMs: RESOLVE_TIMEOUT_MS, },);
 ```
 */
export function miseTask({
  task,
  timeoutMs,
}: {
  readonly task: string;
  readonly timeoutMs: number;
},): UpdateCommand {
  return {
    label: `mise run ${task}`,
    argv: [
      'mise',
      'run',
      task,
    ],
    timeoutMs,
  };
}

/**
 Builds a command that runs inside the mise environment, so the repository's `.cache` shadows are on `PATH`.

 @param words - program and arguments

 @param timeoutMs - longest the command may run

 @returns the command

 @example
 ```ts
 miseExec({ words: ['cargo-shadow', 'update',], timeoutMs: INSTALL_TIMEOUT_MS, },);
 ```
 */
export function miseExec({
  words,
  timeoutMs,
}: {
  readonly words: readonly string[];
  readonly timeoutMs: number;
},): UpdateCommand {
  return {
    label: `mise exec -- ${words.join(' ',)}`,
    argv: [
      'mise',
      'exec',
      '--',
      ...words,
    ],
    timeoutMs,
  };
}

/**
 The repository toolchain lock, the one tracked file the toolchain group changes.
 */
const TOOLCHAIN_LOCK: PathSelector = {
  exact: ['toolchain.lock.json',],
  suffixes: [],
};

/**
 The pnpm files the dependency group changes.
 */
const PNPM_FILES: PathSelector = {
  exact: [
    'pnpm-lock.yaml',
    'pnpm-workspace.yaml',
  ],
  suffixes: [],
};

/**
 The Cargo lock files at any depth.
 */
const CARGO_LOCKS: PathSelector = {
  exact: ['Cargo.lock',],
  suffixes: ['/Cargo.lock',],
};

/**
 The Rust catalog, the manifests it renders into and the lock files.
 */
const RUST_DEPENDENCY_FILES: PathSelector = {
  exact: [
    'rust-catalog.toml',
    'Cargo.lock',
    'Cargo.toml',
  ],
  suffixes: [
    '/Cargo.lock',
    '/Cargo.toml',
  ],
};

/**
 Everything the dependency group may change.
 */
const DEPENDENCY_FILES: PathSelector = {
  exact: [
    ...PNPM_FILES.exact,
    ...RUST_DEPENDENCY_FILES.exact,
  ],
  suffixes: RUST_DEPENDENCY_FILES.suffixes,
};

/**
 Files that no step of a group changes.
 */
const NO_FILES: PathSelector = {
  exact: [],
  suffixes: [],
};

/**
 Builds the groups for a machine whose global toolchain lock sits at a path.

 @param globalLockPath - absolute path of the machine-global `toolchain.lock.json`

 @returns the groups in the order they run

 @example
 ```ts
 const groups = dailyUpdateGroups({ globalLockPath: '/home/user/.config/monochromatic-dev/toolchain/toolchain.lock.json', },);
 ```
 */
export function dailyUpdateGroups({
  globalLockPath,
}: {
  readonly globalLockPath: string;
},): readonly UpdateGroup[] {
  return [
    {
      name: 'repository toolchain',
      commitSubject: 'chore(toolchain): daily update of the repository toolchain lock',
      owns: {
        exact: [
          ...TOOLCHAIN_LOCK.exact,
          'toolchain.plan.jsonc',
        ],
        suffixes: [],
      },
      externalFiles: [],
      steps: [
        {
          name: 'update and install the repository toolchain',
          commands: [
            miseTask({
              task: 'toolchain:update',
              timeoutMs: RESOLVE_TIMEOUT_MS,
            },),
            miseTask({
              task: 'toolchain:install',
              timeoutMs: INSTALL_TIMEOUT_MS,
            },),
          ],
          soft: false,
          touches: TOOLCHAIN_LOCK,
        },
      ],
    },
    {
      name: 'machine-global toolchain',
      commitSubject: '',
      owns: NO_FILES,
      externalFiles: [globalLockPath,],
      steps: [
        {
          name: 'update and install the machine-global toolchain',
          commands: [
            miseTask({
              task: 'toolchain:update:global',
              timeoutMs: RESOLVE_TIMEOUT_MS,
            },),
            miseTask({
              task: 'toolchain:install:global',
              timeoutMs: INSTALL_TIMEOUT_MS,
            },),
          ],
          soft: false,
          touches: NO_FILES,
        },
      ],
    },
    {
      name: 'dependency locks',
      commitSubject: 'chore(*): daily update of the pnpm and Cargo dependency locks',
      owns: DEPENDENCY_FILES,
      externalFiles: [],
      steps: [
        {
          name: 'update the pnpm lock',
          commands: [miseTask({
            task: 'deps:update',
            timeoutMs: INSTALL_TIMEOUT_MS,
          },),],
          soft: true,
          touches: PNPM_FILES,
        },
        {
          name: 'update the Cargo locks',
          commands: [miseExec({
            words: [
              'cargo-shadow',
              'update',
            ],
            timeoutMs: INSTALL_TIMEOUT_MS,
          },),],
          soft: true,
          touches: CARGO_LOCKS,
        },
        {
          name: 'tighten the catalogs, render the manifests, make every lock satisfy them',
          commands: [
            miseTask({
              task: 'catalog:tighten',
              timeoutMs: INSTALL_TIMEOUT_MS,
            },),
            miseTask({
              task: 'sync:files',
              timeoutMs: INSTALL_TIMEOUT_MS,
            },),
            miseExec({
              words: [
                'pnpm',
                'install',
              ],
              timeoutMs: INSTALL_TIMEOUT_MS,
            },),
            miseExec({
              words: [
                'cargo-shadow',
                'install',
              ],
              timeoutMs: INSTALL_TIMEOUT_MS,
            },),
            miseTask({
              task: 'check:files',
              timeoutMs: INSTALL_TIMEOUT_MS,
            },),
          ],
          soft: false,
          touches: DEPENDENCY_FILES,
        },
      ],
    },
  ];
}
