/**
 The daily update's runner: runs each group's steps, restores a failed group's files, commits what changed and tells the
 owner what failed.

 Every effect (commands, files, git, notifications) is passed in, so the sequencing rules are tested without a network.
 Rationale: `doc/planning/toolchain-always-latest.md` ("The daily update").

 @module
 */

import {
  type PathSelector,
  selects,
  type UpdateCommand,
  type UpdateGroup,
  type UpdateStep,
} from './daily-update-plan.ts';

/**
 What a finished command reports.
 */
export type CommandOutcome = {
  /**
   Exit status; any nonzero value, including the one of a command that was stopped, is a failure.
   */
  readonly exitCode: number;
  /**
   Whether the command was stopped for running past its time bound.
   */
  readonly timedOut: boolean;
  /**
   Last lines of the command's output, for the notification.
   */
  readonly outputTail: string;
};

/**
 The outside world the runner acts on.
 */
export type DailyUpdateEffects = {
  /**
   Runs one command in the repository and waits for it, within its own time bound.
   */
  readonly runCommand: (command: UpdateCommand,) => Promise<CommandOutcome>;
  /**
   Repository-relative paths of tracked files whose contents differ from `HEAD`, plus untracked files.
   */
  readonly dirtyPaths: () => Promise<readonly string[]>;
  /**
   Repository-relative paths of every tracked file.
   */
  readonly trackedPaths: () => Promise<readonly string[]>;
  /**
   Reads a file's text; the argument is an absolute path.
   */
  readonly readText: (absolutePath: string,) => Promise<string>;
  /**
   Replaces a file's text.
   */
  readonly writeText: (request: {
    readonly absolutePath: string;
    readonly text: string
  },) => Promise<void>;
  /**
   Commits exactly the given repository-relative paths; rejects when the commit fails.
   */
  readonly commit: (request: {
    readonly message: string;
    readonly paths: readonly string[]
  },) => Promise<void>;
  /**
   Tells the owner something, on the desktop and in the journal.
   */
  readonly notify: (request: {
    readonly title: string;
    readonly body: string
  },) => Promise<void>;
  /**
   Writes one line to the journal.
   */
  readonly log: (line: string,) => void;
  /**
   Absolute path of the repository.
   */
  readonly repository: string;
};

/**
 What one group did.
 */
export type GroupReport = {
  /**
   Group name.
   */
  readonly name: string;
  /**
   What came of the group: `landed` (committed), `unchanged` (nothing moved), `failed` or `skipped` (files were dirty).
   */
  readonly outcome: 'failed' | 'landed' | 'skipped' | 'unchanged';
  /**
   Repository-relative paths committed, empty unless the outcome is `landed`.
   */
  readonly committed: readonly string[];
  /**
   Plain-language problems the owner is told about, empty when nothing went wrong.
   */
  readonly problems: readonly string[];
};

/**
 Joins a repository and a repository-relative path.

 @param repository - absolute repository path

 @param repositoryPath - repository-relative path

 @returns the absolute path

 @example
 ```ts
 absoluteOf({ repository: '/r', repositoryPath: 'a/b', },); // '/r/a/b'
 ```
 */
export function absoluteOf({
  repository,
  repositoryPath,
}: {
  readonly repository: string;
  readonly repositoryPath: string;
},): string {
  return `${repository}/${repositoryPath}`;
}

/**
 The repository-relative part of an absolute path inside the repository.

 @param repository - absolute repository path

 @param absolutePath - absolute path

 @returns the relative path, or the empty string for a path outside the repository

 @example
 ```ts
 relativeOf({ repository: '/r', absolutePath: '/r/a/b', },); // 'a/b'
 ```
 */
export function relativeOf({
  repository,
  absolutePath,
}: {
  readonly repository: string;
  readonly absolutePath: string;
},): string {
  return absolutePath.startsWith(`${repository}/`,) ? absolutePath.slice(repository.length + 1,) : '';
}

/**
 Puts files back to the text they had before the group began.

 @param effects - the outside world

 @param snapshot - absolute path to original text

 @param selector - which files to put back; absent means every file in the snapshot, external ones included

 @returns the absolute paths whose text had to be written back

 @example
 ```ts
 await restoreFiles({ effects, snapshot, },);
 ```
 */
export async function restoreFiles({
  effects,
  snapshot,
  selector,
}: {
  readonly effects: DailyUpdateEffects;
  readonly snapshot: ReadonlyMap<string, string>;
  readonly selector?: PathSelector;
},): Promise<readonly string[]> {
  /**
   Paths written back.
   */
  const restored: string[] = [];
  for (const [absolutePath, text,] of snapshot) {
    /**
     Whether this file is among those to put back.
     */
    const wanted = (selector === undefined)
      || selects({
        selector,
        repositoryPath: relativeOf({
          repository: effects.repository,
          absolutePath,
        },),
      },);
    // Reading before writing keeps untouched files untouched, so their modification times stay.
    if (wanted && (await effects.readText(absolutePath,) !== text)) {
      await effects.writeText({
        absolutePath,
        text,
      },);
      restored.push(absolutePath,);
    }
  }
  return restored;
}

/**
 Runs one step's commands in order.

 @param effects - the outside world

 @param step - the step

 @returns a problem text when a command failed, empty when every command succeeded

 @example
 ```ts
 const problem = await runStep({ effects, step, },);
 ```
 */
export async function runStep({
  effects,
  step,
}: {
  readonly effects: DailyUpdateEffects;
  readonly step: UpdateStep;
},): Promise<string> {
  for (const command of step.commands) {
    effects.log(`running: ${command.label}`,);
    // Commands of one step depend on each other, so they run one after another.
    /**
     How the command ended.
     */
    const outcome = await effects.runCommand(command,);
    if (outcome.exitCode !== 0) {
      /**
       Why the command ended, in words the owner can act on.
       */
      const reason = outcome.timedOut
        ? 'ran past its time bound and was stopped'
        : `exited with status ${outcome.exitCode}`;
      return `\`${command.label}\` ${reason}. Last output:\n${outcome.outputTail}`;
    }
  }
  return '';
}

/**
 Reads the text of every file a group may change, before its first command runs.

 @param effects - the outside world

 @param group - the group

 @returns absolute path to text

 @example
 ```ts
 const snapshot = await snapshotGroup({ effects, group, },);
 ```
 */
export async function snapshotGroup({
  effects,
  group,
}: {
  readonly effects: DailyUpdateEffects;
  readonly group: UpdateGroup;
},): Promise<ReadonlyMap<string, string>> {
  /**
   Tracked files the group owns, then the files outside the repository.
   */
  const absolutePaths: string[] = [];
  for (const repositoryPath of await effects.trackedPaths()) {
    if (selects({
      selector: group.owns,
      repositoryPath,
    },)) {
      absolutePaths.push(absoluteOf({
        repository: effects.repository,
        repositoryPath,
      },),);
    }
  }
  absolutePaths.push(...group.externalFiles,);

  /**
   Text by absolute path.
   */
  const snapshot = new Map<string, string>();
  for (const absolutePath of absolutePaths) {
    snapshot.set(
      absolutePath,
      await effects.readText(absolutePath,),
    );
  }
  return snapshot;
}

/**
 Lists the owned files whose text now differs from the snapshot.

 @param effects - the outside world

 @param snapshot - texts taken before the group began

 @returns repository-relative paths

 @example
 ```ts
 const changed = await changedOwnedPaths({ effects, snapshot, },);
 ```
 */
export async function changedOwnedPaths({
  effects,
  snapshot,
}: {
  readonly effects: DailyUpdateEffects;
  readonly snapshot: ReadonlyMap<string, string>;
},): Promise<readonly string[]> {
  /**
   Changed paths.
   */
  const changed: string[] = [];
  for (const [absolutePath, text,] of snapshot) {
    /**
     Repository-relative path, empty for an external file.
     */
    const repositoryPath = relativeOf({
      repository: effects.repository,
      absolutePath,
    },);
    if ((repositoryPath !== '') && (await effects.readText(absolutePath,) !== text)) {
      changed.push(repositoryPath,);
    }
  }
  return changed;
}

/**
 Runs the steps of a group in order, putting files back as each failure requires.

 @param effects - the outside world

 @param group - the group

 @param snapshot - texts taken before the group began

 @returns the problems found and whether a hard step ended the group

 @example
 ```ts
 const { problems, ended, } = await runSteps({ effects, group, snapshot, },);
 ```
 */
export async function runSteps({
  effects,
  group,
  snapshot,
}: {
  readonly effects: DailyUpdateEffects;
  readonly group: UpdateGroup;
  readonly snapshot: ReadonlyMap<string, string>;
},): Promise<{
  readonly problems: readonly string[];
  readonly ended: boolean
}> {
  /**
   Problems found so far.
   */
  const problems: string[] = [];
  for (const step of group.steps) {
    /**
     Problem text of the step, empty when it succeeded.
     */
    const problem = await runStep({
      effects,
      step,
    },);
    if (problem === '') {
      continue;
    }
    if (step.soft) {
      await restoreFiles({
        effects,
        snapshot,
        selector: step.touches,
      },);
      problems.push(
        `${group.name}: "${step.name}" failed and its files were put back; the rest of the group went on. ${problem}`,
      );
      continue;
    }
    await restoreFiles({
      effects,
      snapshot,
    },);
    problems.push(
      `${group.name}: "${step.name}" failed, so nothing from this group was landed and its files were put back. ${problem}`,
    );
    return {
      problems,
      ended: true,
    };
  }
  return {
    problems,
    ended: false,
  };
}

/**
 Runs one group: checks that its files are clean, runs the steps, restores on a hard failure, commits what changed.

 @param effects - the outside world

 @param group - the group

 @returns what happened, including every problem the owner should read

 @example
 ```ts
 const report = await runGroup({ effects, group, },);
 ```
 */
export async function runGroup({
  effects,
  group,
}: {
  readonly effects: DailyUpdateEffects;
  readonly group: UpdateGroup;
},): Promise<GroupReport> {
  /**
   Files with uncommitted changes that this group would change and commit.
   */
  const dirtyBefore = await effects.dirtyPaths();
  /**
   Owned files that were already dirty before the run.
   */
  const dirtyOwned = dirtyBefore.filter(function isOwned(repositoryPath,) {
    return selects({
      selector: group.owns,
      repositoryPath,
    },);
  },);
  if (dirtyOwned.length > 0) {
    return {
      name: group.name,
      outcome: 'skipped',
      committed: [],
      problems: [
        `${group.name}: skipped because these files have uncommitted changes that the update would mix with its own: ${
          dirtyOwned.join(', ',)
        }. Commit or discard them and the next run picks the update up.`,
      ],
    };
  }

  /**
   Tracked files the group may change, with the text each has now.
   */
  const snapshot = await snapshotGroup({
    effects,
    group,
  },);
  /**
   Problems found by the steps, and whether a hard step ended the group.
   */
  const stepResult = await runSteps({
    effects,
    group,
    snapshot,
  },);
  /**
   Problems so far, to which later findings are added.
   */
  const problems: string[] = [...stepResult.problems,];
  if (stepResult.ended) {
    return {
      name: group.name,
      outcome: 'failed',
      committed: [],
      problems,
    };
  }

  /**
   Owned files whose text changed during the run.
   */
  const changed = await changedOwnedPaths({
    effects,
    snapshot,
  },);

  /**
   Files changed outside the group's ownership, which the update does not commit.
   */
  const dirtyAfter = await effects.dirtyPaths();
  /**
   Files that changed during the run but are not part of the group.
   */
  const unexpected = dirtyAfter.filter(function isNewAndForeign(repositoryPath,) {
    return (!dirtyBefore.includes(repositoryPath,)) && (!selects({
      selector: group.owns,
      repositoryPath,
    },));
  },);
  if (unexpected.length > 0) {
    problems.push(
      `${group.name}: these files changed but are not part of the group, so they were left uncommitted: ${
        unexpected.join(', ',)
      }`,
    );
  }

  if ((changed.length === 0) || (group.commitSubject === '')) {
    return {
      name: group.name,
      outcome: 'unchanged',
      committed: [],
      problems,
    };
  }

  /**
   Commit message: the subject, then what moved and what went wrong.
   */
  const message = [
    group.commitSubject,
    '',
    `Changed files: ${changed.length}.`,
    ...problems,
  ].join('\n',);
  try {
    await effects.commit({
      message,
      paths: changed,
    },);
  }
  catch (error) {
    problems.push(
      `${group.name}: the commit failed (${String(error,)}); the changed files stay in the working tree: ${
        changed.join(', ',)
      }`,
    );
    return {
      name: group.name,
      outcome: 'failed',
      committed: [],
      problems,
    };
  }
  return {
    name: group.name,
    outcome: 'landed',
    committed: changed,
    problems,
  };
}

/**
 Runs every group in order and tells the owner about anything that went wrong.
 A failed or skipped group never stops the groups after it.

 @param effects - the outside world

 @param groups - groups in the order they run

 @returns one report per group

 @example
 ```ts
 const reports = await runDailyUpdate({ effects, groups: dailyUpdateGroups({ globalLockPath, },), },);
 ```
 */
export async function runDailyUpdate({
  effects,
  groups,
}: {
  readonly effects: DailyUpdateEffects;
  readonly groups: readonly UpdateGroup[];
},): Promise<readonly GroupReport[]> {
  /**
   Reports so far.
   */
  const reports: GroupReport[] = [];
  for (const group of groups) {
    effects.log(`group: ${group.name}`,);
    // Groups share the working tree and the toolchain, so they run one after another.
    /**
     What the group did.
     */
    const report = await runGroup({
      effects,
      group,
    },);
    effects.log(`group ${group.name}: ${report.outcome} (${report.committed
      .length} files committed)`,);
    reports.push(report,);
  }

  /**
   Every problem of every group.
   */
  const problems = reports.flatMap(function problemsOf(report,) {
    return report.problems;
  },);
  if (problems.length > 0) {
    await effects.notify({
      title: 'Daily toolchain update needs a look',
      body: `${problems.join('\n\n',)}\n\nThe journal has the full output: journalctl --user --unit toolchain-daily-update`,
    },);
  }
  return reports;
}
