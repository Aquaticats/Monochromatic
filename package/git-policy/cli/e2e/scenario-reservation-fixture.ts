/**
 Starvation reservation:
 a commit that keeps losing landing races reserves the next slot and lands within the bound,
 before a commit started while it held the reservation.

 The victim holds in its editor until the first winner lands,
 then pauses after each lost race through the test-only `race-lost` phase marker,
 so each further winner lands while it replays;
 after its `landing.reserveAfterLostRaces`-th lost race it holds the reservation.
 The scenario repository has no cli-git config,
 so that is the default of 1:
 the first winner alone makes the victim reserve.

 @module
 */

import {
  mkdir,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import {
  BARRIER_TIMEOUT_MS,
  reached,
  releaseAt,
  settleWithin,
} from './barrier-fixture.ts';
import { synthesizeText, } from './content-fixture.ts';
import { isAncestor, } from './git-read-fixture.ts';
import type { AttemptRecord, } from './ledger-fixture.ts';
import { waitForMarker, } from './process-fixture.ts';
import { realGit, } from './repository-fixture.ts';
import {
  repositoryOptions,
  seedTexts,
} from './scenario-helper-fixture.ts';
import {
  allSucceeded,
  type ScenarioDefinition,
} from './scenario-model-fixture.ts';
import {
  BLOCKED,
  BLOCKED_WINDOW_MS,
  commitWinner,
  eventCount,
  FIRST_WINNER,
  LATER_WINNERS,
  landedCommit,
  PATHS,
  RESERVE_AFTER_LOST_RACES,
} from './scenario-reservation-helper-fixture.ts';
import {
  startAttempt,
  writeWorktree,
} from './worker-fixture.ts';

//region Scenario

/**
 Commit that loses the default number of races, reserves, and lands before a later commit.
 */
const reservationAfterLostRaces: ScenarioDefinition = {
  name: 'reservation-after-lost-races',
  group: 'concurrency',
  summary: 'a commit loses the default number of landing races, reserves the next slot, and lands before a commit started while it held the reservation',
  repository(random,) {
    return repositoryOptions({ seedFiles: seedTexts({
      random,
      paths: PATHS,
    },), },);
  },
  async run(context,) {
    await Promise.all(PATHS.map(async function rewrite(path,) {
      await writeWorktree({
        ...context,
        path,
        bytes: synthesizeText({
          random: context.random
            .fork(path,),
          size: 400,
        },),
      },);
    },),);
    /**
     Directory of the victim's phase files.
     */
    const phases = join(
      context.repository
        .markerDir,
      'victim-phases',
    );
    await mkdir(
      phases,
      { recursive: true, },
    );
    /**
     Victim held in its editor, pausing after each lost race.
     */
    const victim = await startAttempt({
      ...context,
      label: 'victim',
      paths: ['victim.txt',],
      mode: 'explicit',
      extraArgs: ['-e',],
      env: {
        GIT_EDITOR: context.repository
          .editorProgram,
        CLI_GIT_TEST_ONLY_PHASE_SIGNAL: `race-lost:pause:${phases}`,
      },
    },);
    /**
     Whether the victim reached its editor.
     */
    const inEditor = await reached({
      repository: context.repository,
      token: victim.token,
      running: victim.running,
      event: 'editor',
    },);
    /**
     First winner, landing while the victim is in its editor.
     */
    const first = await commitWinner({
      context,
      label: FIRST_WINNER,
    },);
    await releaseAt({
      repository: context.repository,
      token: victim.token,
      event: 'editor',
    },);
    /**
     Each further winner lands while the victim pauses after its previous lost race,
     and each lost race's marker outcome is kept in order.
     */
    const {
      winners,
      lostMarkers,
    } = await LATER_WINNERS
      .reduce<Promise<Readonly<{
        winners: readonly AttemptRecord[];
        lostMarkers: readonly string[];
      }>>>(
      async function landWhileReplaying(
        previous,
        label,
        index,
      ) {
        /**
         Winners and marker outcomes so far.
         */
        const earlier = await previous;
        /**
         Lost race the victim pauses after.
         */
        const race = index + 1;
        /**
         Whether the victim lost that race.
         */
        const lost = await waitForMarker({
          path: join(
            phases,
            `race-lost-${String(race,)}.reached`,
          ),
          timeoutMs: BARRIER_TIMEOUT_MS,
          isSettled: victim.running
            .isSettled,
        },);
        /**
         Winner landing while the victim replays.
         */
        const winner = await commitWinner({
          context,
          label,
        },);
        await writeFile(
          join(
            phases,
            `race-lost-${String(race,)}.release`,
          ),
          '',
        );
        return {
          winners: [
            ...earlier.winners,
            winner,
          ],
          lostMarkers: [
            ...earlier.lostMarkers,
            lost,
          ],
        };
      },
      Promise.resolve({
        winners: [first,],
        lostMarkers: [],
      },),
      );
    /**
     Whether the victim lost its last race and holds the reservation.
     */
    const lostReserving = await waitForMarker({
      path: join(
        phases,
        `race-lost-${String(RESERVE_AFTER_LOST_RACES,)}.reached`,
      ),
      timeoutMs: BARRIER_TIMEOUT_MS,
      isSettled: victim.running
        .isSettled,
    },);
    /**
     Marker outcome of every lost race, in order.
     */
    const lostRaceMarkers = [
      ...lostMarkers,
      lostReserving,
    ];
    /**
     Branch while the victim holds the reservation.
     */
    const headHeld = await realGit({
      repository: context.repository,
      args: [
        'rev-parse',
        'HEAD',
      ],
    },);
    /**
     Commit started while the reservation is held.
     */
    const blocked = await startAttempt({
      ...context,
      label: BLOCKED,
      paths: [`${BLOCKED}.txt`,],
      mode: 'explicit',
    },);
    /**
     Whether it settled while the reservation was held.
     */
    const settledEarly = await settleWithin({
      running: blocked.running,
      windowMs: BLOCKED_WINDOW_MS,
    },);
    /**
     Branch after the window.
     */
    const headAfterWindow = await realGit({
      repository: context.repository,
      args: [
        'rev-parse',
        'HEAD',
      ],
    },);
    await writeFile(
      join(
        phases,
        `race-lost-${String(RESERVE_AFTER_LOST_RACES,)}.release`,
      ),
      '',
    );
    /**
     Settled victim and blocked commit.
     */
    const [victimRecord, blockedRecord,] = await Promise.all([
      victim.finished,
      blocked.finished,
    ],);
    /**
     Landed victim and blocked commits.
     */
    const [victimCommit, blockedCommit,] = await Promise.all([
      landedCommit({
        context,
        attempt: victimRecord,
      },),
      landedCommit({
        context,
        attempt: blockedRecord,
      },),
    ],);
    /**
     Lost races the victim reported.
     */
    const lostRaces = eventCount({
      attempt: victimRecord,
      type: 'landing-race-lost',
    },);
    return [
      allSucceeded({
        name: 'all-commits-succeed',
        attempts: [
          ...winners,
          victimRecord,
          blockedRecord,
        ],
      },),
      {
        name: 'victim-lost-races',
        holds: (inEditor === 'marker') && lostRaceMarkers.every(function reachedMarker(marker,) {
          return marker === 'marker';
        },),
        detail: `editor ${inEditor}, lost races ${lostRaceMarkers.join(', ',)}`,
      },
      {
        name: 'reserved-within-bound',
        holds: (eventCount({
          attempt: victimRecord,
          type: 'landing-reserved',
        },) === 1)
          && (lostRaces <= (RESERVE_AFTER_LOST_RACES
            + 1)),
        detail: `victim reported ${String(lostRaces,)} lost races and ${String(eventCount({
          attempt: victimRecord,
          type: 'landing-reserved',
        },),)} reservations`,
      },
      {
        name: 'reservation-blocks-landing',
        holds: (!settledEarly) && (headAfterWindow === headHeld),
        detail: settledEarly ? 'the commit started during the reservation settled before the holder landed' : `HEAD moved from ${headHeld} to ${headAfterWindow} while the reservation was held`,
      },
      {
        name: 'reserved-lands-first',
        holds: (victimCommit !== '') && (blockedCommit !== '')
          && (await isAncestor({
            repository: context.repository,
            oid: victimCommit,
            ref: blockedCommit,
          },)),
        detail: `victim ${victimCommit} is not an ancestor of the later commit ${blockedCommit}`,
      },
    ];
  },
};

/**
 Reservation scenarios in report order.
 */
export const RESERVATION_SCENARIOS: readonly ScenarioDefinition[] = [reservationAfterLostRaces,];

//endregion Scenario
