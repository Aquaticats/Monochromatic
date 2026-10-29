/**
 Setup steps shared by several scenarios:
 trace seeding,
 token-free amend bases,
 and partial staging.

 @module
 */

import {
  editText,
  synthesizeText,
} from './content-fixture.ts';
import {
  hookDisableArguments,
  type HookEvent,
} from './hook-program-fixture.ts';
import type { AttemptRecord, } from './ledger-fixture.ts';
import { realGit, } from './repository-fixture.ts';
import type {
  Expectations,
  ScenarioContext,
} from './scenario-model-fixture.ts';
import type { planTraceOperations, } from './trace-replay-fixture.ts';
import {
  runWrapper,
  writeWorktree,
} from './worker-fixture.ts';

//region Constants

/**
 Every commit hook event the hook scenarios install.
 */
export const ALL_HOOK_EVENTS: readonly HookEvent[] = [
  'pre-commit',
  'prepare-commit-msg',
  'commit-msg',
  'post-commit',
];

/**
 Per-commit change cap for trace replay.
 */
export const TRACE_MAX_CHANGES = 16;

/**
 Commit ID prefix length in diagnostics.
 */
const SHORT_OID = 12;

//endregion Constants

//region Setup steps

/**
 Commits trace seed files with real Git before the replay window.
 Seeding bypasses the wrapper and its hooks on purpose:
 it is fixture setup,
 not workload.

 @param context - scenario context

 @param seeds - seed paths

 @example
 ```ts
 await seedTracePaths({ context, seeds });
 ```
 */
export async function seedTracePaths({
  context,
  seeds,
}: Readonly<{
  context: ScenarioContext;
  seeds: ReturnType<typeof planTraceOperations>['seeds'];
}>,): Promise<void> {
  if (seeds.length === 0)
    return;
  await Promise.all(seeds.map(async function seed(entry,) {
    await writeWorktree({
      ...context,
      path: entry.path,
      bytes: synthesizeText({
        random: context.random
          .fork(`seed:${entry.path}`,),
        size: entry.size,
      },),
    },);
  },),);
  await realGit({
    repository: context.repository,
    args: [
      'add',
      '--',
      ...seeds.map(function seedPath(entry,) {
        return entry.path;
      },),
    ],
  },);
  await realGit({
    repository: context.repository,
    args: [
      ...hookDisableArguments(ALL_HOOK_EVENTS,),
      'commit',
      '--quiet',
      '--no-verify',
      '--message',
      'e2e trace seed',
    ],
  },);
  await realGit({
    repository: context.repository,
    args: [
      'push',
      '--quiet',
    ],
  },);
}

/**
 Commits one setup change with real Git so an amend has a token-free commit to replace.

 @param context - scenario context

 @param path - path the setup commit changes

 @returns parent of the setup commit

 @example
 ```ts
 await commitAmendBase({ context, path: 'amend.txt' });
 ```
 */
export async function commitAmendBase({
  context,
  path,
}: Readonly<{
  context: ScenarioContext;
  path: string;
}>,): Promise<Readonly<{ parent: string; }>> {
  await writeWorktree({
    ...context,
    path,
    bytes: synthesizeText({
      random: context.random
        .fork('amend-base',),
      size: 300,
    },),
  },);
  await realGit({
    repository: context.repository,
    args: [
      ...hookDisableArguments(ALL_HOOK_EVENTS,),
      'commit',
      '--quiet',
      '--no-verify',
      '--message',
      'e2e amend base',
      '--',
      path,
    ],
  },);
  await realGit({
    repository: context.repository,
    args: [
      'push',
      '--quiet',
    ],
  },);
  return { parent: (await realGit({
    repository: context.repository,
    args: [
      'rev-parse',
      'HEAD^',
    ],
  },)).trim(), };
}

/**
 Expectation that a successful amend replaced exactly the commit `HEAD` named at invocation.

 @param context - scenario context

 @param amend - finished amend attempt

 @param parent - parent of the commit being amended

 @returns expectation record

 @example
 ```ts
 await amendReplacedBase({ context, amend, parent });
 ```
 */
export async function amendReplacedBase({
  context,
  amend,
  parent,
}: Readonly<{
  context: ScenarioContext;
  amend: AttemptRecord;
  parent: string;
}>,): Promise<Expectations[number]> {
  if (amend.outcome
    .exitCode
    !== 0)
    return {
      name: 'amend-safe',
      holds: true,
      detail: '',
    };
  /**
   Parent of the commit carrying the amend token.
   */
  const landedParent = (await realGit({
    repository: context.repository,
    args: [
      'log',
      '--branches',
      '--format=%P',
      '--fixed-strings',
      `--grep=[${amend.token}]`,
    ],
  },)).trim();
  return {
    name: 'amend-safe',
    holds: landedParent === parent,
    detail: `amend landed on parent ${landedParent.slice(
      0,
      SHORT_OID,
    )}, expected ${parent.slice(
      0,
      SHORT_OID,
    )}`,
  };
}

/**
 Stages new bytes for a path,
 then leaves a different unstaged tail in the worktree.

 @param context - scenario context

 @param path - tracked path

 @example
 ```ts
 await stagePartially({ context, path: 'partial.txt' });
 ```
 */
export async function stagePartially({
  context,
  path,
}: Readonly<{
  context: ScenarioContext;
  path: string;
}>,): Promise<void> {
  /**
   Staged version.
   */
  const staged = synthesizeText({
    random: context.random
      .fork(`${path}:staged`,),
    size: 600,
  },);
  await writeWorktree({
    ...context,
    path,
    bytes: staged,
  },);
  await runWrapper({
    ...context,
    label: `add ${path}`,
    args: [
      'add',
      '--',
      path,
    ],
    mustSucceed: true,
  },);
  context.ledger
    .recordStaged({
      path,
      staged: true,
    },);
  await writeWorktree({
    ...context,
    path,
    bytes: editText({
      random: context.random
        .fork(`${path}:tail`,),
      current: staged,
      added: 2,
      deleted: 0,
    },),
  },);
}

//endregion Setup steps
