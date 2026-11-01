/**
 Single-commit baselines for hooks,
 signing,
 fault injection after the last lock,
 and the checker's own positive control.
 Each must pass against today's build.

 @module
 */

import { writeFile, } from 'node:fs/promises';
import { join, } from 'node:path';

import {
  holdAt,
  reached,
} from './barrier-fixture.ts';
import { synthesizeText, } from './content-fixture.ts';
import { realGit, } from './repository-fixture.ts';
import {
  repositoryOptions,
  runSequentialWorkers,
  seedTexts,
} from './scenario-helper-fixture.ts';
import {
  allSucceeded,
  type ScenarioContext,
  type ScenarioDefinition,
} from './scenario-model-fixture.ts';
import { ALL_HOOK_EVENTS, } from './scenario-setup-fixture.ts';
import {
  runWrapper,
  startAttempt,
  writeWorktree,
} from './worker-fixture.ts';

//region Constants

/**
 Text size of files the baselines write.
 */
const FILE_BYTES = 400;

//endregion Constants

//region Scenarios

/**
 Two sequential commits in a hooked or signed repository.

 @param context - scenario context

 @returns expectations

 @example
 ```ts
 await runHookedSequence(context);
 ```
 */
async function runHookedSequence(context: ScenarioContext,): ReturnType<ScenarioDefinition['run']> {
  /**
   Finished attempts.
   */
  const attempts = await runSequentialWorkers({
    context,
    plans: [
      {
        label: 'hooked-a',
        paths: ['a.txt',],
      },
      {
        label: 'hooked-b',
        paths: ['b.txt',],
      },
    ],
  },);
  return [allSucceeded({
    name: 'all-commits-succeed',
    attempts,
  },),];
}

/**
 Config-based hooks on sequential commits.
 The former `baseline-hooks-hookdir` twin (traditional hooks in the hooks directory) is removed:
 traditional hooks are unsupported by the wrapper,
 and this scenario ran the identical workload and assertions.
 */
const configHooks: ScenarioDefinition = {
  name: 'baseline-hooks-config',
  group: 'baseline',
  // Sequential: no commit can lose a landing race, so this also runs on Git without replay plumbing.
  replayPlumbing: 'unused',
  summary: 'the same hooks registered through hook.<name>.command config',
  minimumGit: '2.54.0',
  repository(random,) {
    return repositoryOptions({
      seedFiles: seedTexts({
        random,
        paths: [
          'a.txt',
          'b.txt',
        ],
      },),
      hooks: 'config',
      hookEvents: ALL_HOOK_EVENTS,
    },);
  },
  run: runHookedSequence,
};

/**
 SSH-signed sequential commits.
 */
const sshSigning: ScenarioDefinition = {
  name: 'baseline-ssh-signing',
  group: 'baseline',
  // Sequential: no commit can lose a landing race, so this also runs on Git without replay plumbing.
  replayPlumbing: 'unused',
  summary: 'two sequential SSH-signed commits',
  repository(random,) {
    return repositoryOptions({
      seedFiles: seedTexts({
        random,
        paths: [
          'a.txt',
          'b.txt',
        ],
      },),
      signing: true,
    },);
  },
  run: runHookedSequence,
};

/**
 Kill inside `post-commit`,
 after native Git released every lock,
 then recover and commit again.
 This proves the kill machinery and leftover checks against a phase today's build already survives;
 kills in earlier phases are design scenarios (`scenario-sigkill-fixture.ts`).
 */
const sigkillPostCommit: ScenarioDefinition = {
  name: 'baseline-sigkill-post-commit',
  group: 'baseline',
  // Sequential: no commit can lose a landing race, so this also runs on Git without replay plumbing.
  replayPlumbing: 'unused',
  summary: 'SIGKILL one commit in post-commit, run git status to recover, commit again',
  minimumGit: '2.54.0',
  repository(random,) {
    return repositoryOptions({
      seedFiles: seedTexts({
        random,
        paths: [
          'a.txt',
          'b.txt',
        ],
      },),
      hooks: 'config',
      hookEvents: ['post-commit',],
    },);
  },
  async run(context,) {
    await writeWorktree({
      ...context,
      path: 'a.txt',
      bytes: synthesizeText({
        random: context.random
          .fork('a',),
        size: FILE_BYTES,
      },),
    },);
    /**
     Attempt killed in its hook.
     */
    const victim = await startAttempt({
      ...context,
      label: 'killed',
      paths: ['a.txt',],
      mode: 'explicit',
    },);
    await holdAt({
      repository: context.repository,
      token: victim.token,
      event: 'post-commit',
    },);
    await reached({
      repository: context.repository,
      token: victim.token,
      running: victim.running,
      event: 'post-commit',
    },);
    victim.kill();
    await victim.finished;
    await runWrapper({
      ...context,
      label: 'recovery status',
      args: [
        'status',
        '--short',
      ],
      mustSucceed: true,
    },);
    await writeWorktree({
      ...context,
      path: 'b.txt',
      bytes: synthesizeText({
        random: context.random
          .fork('b',),
        size: FILE_BYTES,
      },),
    },);
    /**
     Follow-up attempt.
     */
    const followUp = await (await startAttempt({
      ...context,
      label: 'follow-up',
      paths: ['b.txt',],
      mode: 'explicit',
    },)).finished;
    return [allSucceeded({
      name: 'follow-up-commit-succeeds',
      attempts: [followUp,],
    },),];
  },
};

/**
 Positive control for the observer and checker:
 one commit lands through the wrapper,
 then real Git tampers with it so that five invariants must be reported.
 */
const checkerPositiveControl: ScenarioDefinition = {
  name: 'baseline-checker-positive-control',
  group: 'baseline',
  // Sequential: no commit can lose a landing race, so this also runs on Git without replay plumbing.
  replayPlumbing: 'unused',
  summary: 'land one commit, then tamper with real Git; the checker must report exactly the planted violations',
  expectedViolations: [
    'index-no-revert',
    'landed-bytes',
    'no-leftovers',
    'remote-contains',
    'worktree-preserved',
  ],
  repository(random,) {
    return repositoryOptions({ seedFiles: seedTexts({
      random,
      paths: ['a.txt',],
    },), },);
  },
  async run(context,) {
    /**
     Seed blob of the path, which the landed commit replaces.
     */
    const seedBlob = (await realGit({
      repository: context.repository,
      args: [
        'rev-parse',
        'HEAD:a.txt',
      ],
    },)).trim();
    await writeWorktree({
      ...context,
      path: 'a.txt',
      bytes: synthesizeText({
        random: context.random
          .fork('landed',),
        size: FILE_BYTES,
      },),
    },);
    /**
     Landed attempt.
     */
    const landed = await (await startAttempt({
      ...context,
      label: 'landed',
      paths: ['a.txt',],
      mode: 'explicit',
    },)).finished;
    // Tampering bypasses the ledger on purpose, so the worktree no longer holds what the harness wrote.
    await writeFile(
      join(
        context.repository
          .worktree,
        'a.txt',
      ),
      synthesizeText({
        random: context.random
          .fork('tampered',),
        size: FILE_BYTES,
      },),
    );
    await realGit({
      repository: context.repository,
      args: [
        'commit',
        '--quiet',
        '--amend',
        '--no-edit',
        '--no-verify',
        '--',
        'a.txt',
      ],
    },);
    await realGit({
      repository: context.repository,
      args: [
        'update-index',
        '--cacheinfo',
        `100644,${seedBlob},a.txt`,
      ],
    },);
    await writeFile(
      join(
        context.repository
          .commonDir,
        'refs',
        'heads',
        'stale.lock',
      ),
      '',
    );
    return [allSucceeded({
      name: 'all-commits-succeed',
      attempts: [landed,],
    },),];
  },
};

/**
 Hook,
 signing,
 fault,
 and checker baselines in report order.
 */
export const BASELINE_HOOK_SCENARIOS: readonly ScenarioDefinition[] = [
  configHooks,
  sshSigning,
  sigkillPostCommit,
  checkerPositiveControl,
];

//endregion Scenarios
