/**
 Single-commit baselines for the message editor and for a kill after the landing.
 The wrapper prepares a commit that asks for an editor by running the editor itself,
 so the editor is the suite's message-time instrument;
 the push record the wrapper leaves after a landing is its post-landing instrument.
 Each must pass against today's build.

 @module
 */

import { editorOptions, } from './editor-attempt-fixture.ts';
import { synthesizeText, } from './content-fixture.ts';
import type { AttemptRecord, } from './ledger-fixture.ts';
import {
  repositoryOptions,
  rewritePaths,
  seedTexts,
} from './scenario-helper-fixture.ts';
import {
  allSucceeded,
  type ScenarioDefinition,
} from './scenario-model-fixture.ts';
import { landedCommit, } from './landed-commit-fixture.ts';
import { settleServiceWork, } from './service-settle-fixture.ts';
import { waitForPushRecord, } from './work-record-fixture.ts';
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
 Two sequential commits whose message editor rewrites the message and appends a trailer.
 The wrapper still runs the editor while it prepares a commit,
 so the editor is the message-time instrument that the `prepare-commit-msg` and `commit-msg` hooks were:
 what the editor leaves is what lands.
 */
const editorTrailer: ScenarioDefinition = {
  name: 'baseline-editor-trailer',
  group: 'baseline',
  // Sequential: no commit can lose a landing race, so this also runs on Git without replay plumbing.
  replayPlumbing: 'unused',
  summary: 'two sequential commits whose message editor appends a trailer; the landed messages carry it and settle one push record each',
  repository(random,) {
    return repositoryOptions({
      seedFiles: seedTexts({
        random,
        paths: [
          'a.txt',
          'b.txt',
        ],
      },),
      editorTrailer: true,
      afterCommitRecords: true,
    },);
  },
  async run(context,) {
    /**
     Plans of the two commits.
     */
    const plans = [
      {
        label: 'edited-a',
        paths: ['a.txt',],
      },
      {
        label: 'edited-b',
        paths: ['b.txt',],
      },
    ] as const;
    await rewritePaths({
      context,
      plans,
    },);
    /**
     Finished attempts.
     */
    const attempts = await plans.reduce<Promise<readonly AttemptRecord[]>>(
      async function commitNext(
        previous,
        plan,
      ) {
        /**
         Records so far.
         */
        const done = await previous;
        return [
          ...done,
          await (await startAttempt({
            ...context,
            label: plan.label,
            paths: plan.paths,
            mode: 'explicit',
            ...editorOptions({
              repository: context.repository,
              trailer: true,
            },),
          },)).finished,
        ];
      },
      Promise.resolve([],),
    );
    return [allSucceeded({
      name: 'all-commits-succeed',
      attempts,
    },),];
  },
};

/**
 Kill once the commit landed and its push record exists, as best the harness can time it,
 the point the retired `post-commit` hook marked,
 then recover and commit again.
 The record is written after the landing and before the service is told about it,
 so the kill leaves a landed commit whose push waits for the next wake.
 This proves the kill machinery and leftover checks against a point today's build already survives;
 kills before the landing are design scenarios (`scenario-sigkill-fixture.ts`).
 */
const sigkillAfterLanding: ScenarioDefinition = {
  name: 'baseline-sigkill-after-landing',
  group: 'baseline',
  // Sequential: no commit can lose a landing race, so this also runs on Git without replay plumbing.
  replayPlumbing: 'unused',
  summary: 'SIGKILL one commit once it landed and its push record exists, run git status to recover, commit again',
  repository(random,) {
    return repositoryOptions({
      seedFiles: seedTexts({
        random,
        paths: [
          'a.txt',
          'b.txt',
        ],
      },),
      afterCommitRecords: true,
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
     Attempt killed after its landing.
     */
    const victim = await startAttempt({
      ...context,
      label: 'killed',
      paths: ['a.txt',],
      mode: 'explicit',
    },);
    /**
     Whether the push record appeared before the attempt settled.
     */
    const arrival = await waitForPushRecord({
      repository: context.repository,
      running: victim.running,
    },);
    // Best effort: the wrapper exits within milliseconds of writing the record, so it may already have exited,
    // which is the same state with nothing to kill. The frozen-service kill in `sigkill-after-landing` is exact.
    victim.kill();
    /**
     Settled victim.
     */
    const victimRecord = await victim.finished;
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
    await settleServiceWork(context.repository
      .gitDir,);
    /**
     The killed commit, which the landing put on the branch.
     */
    const landed = (await landedCommit({
      context,
      attempt: victimRecord,
    },)).trim();
    return [
      {
        name: 'victim-reached-point',
        holds: arrival !== 'timeout',
        detail: `victim ${arrival} before its push record appeared`,
      },
      {
        name: 'killed-commit-landed-once',
        holds: (landed !== '') && (!landed.includes('\n',)),
        detail: `the killed commit's token names ${landed === '' ? 'no commit' : 'several commits'}`,
      },
      allSucceeded({
        name: 'follow-up-commit-succeeds',
        attempts: [followUp,],
      },),
    ];
  },
};

//endregion Scenarios

/**
 Editor and post-landing kill baselines in report order.
 */
export const BASELINE_EDITOR_SCENARIOS: readonly ScenarioDefinition[] = [
  editorTrailer,
  sigkillAfterLanding,
];
