/**
 One command's wait for its answer: the silence it allows, the compaction Pi reports while it waits, and the deadline that ends
 every wait whatever Pi is doing. No process starts here, so what is judged is the wait alone.

 @module
 */

import { setTimeout as sleep, } from 'node:timers/promises';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  type CommandAnswer,
  type CompactionOfPi,
  NO_RUN_FAILURE,
  startWaitingForAnAnswer,
  type WaitingForAnAnswer,
} from '../dist/final/node/index.mjs';

/**
 Milliseconds of silence a wait here allows, short so the tests are quick and long enough that a loaded host cannot spend them
 before the wait is armed.
 */
const SILENCE_BOUND_MILLISECONDS = 100;

/**
 Milliseconds ahead that a deadline a test is not about is placed, far past every wait here.
 */
const FAR_DEADLINE_MILLISECONDS = 60_000;

/**
 Milliseconds of compaction a test spends, above the silence bound so a wait that did not notice the compaction would end during
 it.
 */
const COMPACTION_MILLISECONDS = 300;

/**
 Milliseconds a wait is given to settle before a test concludes it is still waiting, above the silence bound so a wait that was
 about to end has ended.
 */
const SETTLED_BY_MILLISECONDS = 250;

/**
 Milliseconds between looks at whether a wait settled.
 */
const LOOK_MILLISECONDS = 10;

/**
 A wait and the test's view of its answer.
 */
type Watched = {
  readonly waiting: WaitingForAnAnswer;

  /**
   Whether the wait has settled.
   */
  readonly settled: () => boolean;

  /**
   What the wait settled with, which a test reads only once `settled` says there is one.
   */
  readonly answer: () => CommandAnswer;
  readonly done: Promise<void>;
};

/**
 Starts a wait and keeps its answer without awaiting it, so a test can say whether it settled.

 @param request - the wait's bounds and the compaction Pi reports

 @returns the wait, and what the test can ask of its answer
 */
function watchedWait(request: {
  readonly answerMilliseconds: number;
  readonly deadline: number;
  readonly compactionOfPi: () => CompactionOfPi;
},): Watched {
  /**
   The wait itself.
   */
  const waiting = startWaitingForAnAnswer(request,);
  /**
   What came back, once it did.
   */
  const answers: { read?: CommandAnswer } = {};
  const done = (async function took(): Promise<void> {
    const answer = await waiting.answer;
    answers.read = answer;
  })();
  return {
    waiting,
    settled: () => answers.read !== undefined,
    answer: () => {
      const { read } = answers;
      if (read === undefined) {
        throw new Error('the wait had not settled',);
      }
      return read;
    },
    done,
  };
}

/**
 Waits until either the answer settled or the bound passed.

 @param watched - what the test watches

 @param milliseconds - longest wait

 @returns whether the answer settled
 */
async function settledWithin(watched: Watched, milliseconds: number,): Promise<boolean> {
  /**
   When to stop looking.
   */
  const deadline = Date.now() + milliseconds;
  for (;;) {
    if (watched.settled()) {
      return true;
    }
    if (Date.now() >= deadline) {
      return false;
    }
    // oxlint-disable-next-line eslint/no-await-in-loop -- the answer arrives on its own schedule, so the test looks again after waiting; see doc/lint/sequential-asyncs.md
    await sleep(LOOK_MILLISECONDS,);
  }
}

/**
 Pi's compaction as a test reports it.

 @returns the state a wait reads, and the two ways a test moves it
 */
function compactionUnderTest(): {
  readonly of: () => CompactionOfPi;
  readonly begin: () => void;
  readonly end: () => void
} {
  /**
   What Pi reports, which the wait reads each time a bound is due.
   */
  const state: { compaction: CompactionOfPi } = { compaction: { compacting: false, }, };
  return {
    of: () => state.compaction,
    begin: function begin() {
      state.compaction = {
        compacting: true,
        reason: 'threshold',
        since: Date.now(),
      };
    },
    end: function end() {
      state.compaction = { compacting: false, };
    },
  };
}

/**
 What a settled answer says of the bound or the compaction it ended on.

 @param answer - how the wait ended

 @returns the driver's words
 */
function problemOf(answer: CommandAnswer,): string {
  return ((answer.ending === 'silence') || (answer.ending === 'deadline'))
    ? answer.problem
    : '';
}

/**
 What a settled answer says of the compaction it saw fail.

 @param answer - how the wait ended

 @returns the compaction's words, or nothing
 */
function compactionProblemOf(answer: CommandAnswer,): string {
  return ((answer.ending === 'silence') || (answer.ending === 'deadline'))
    ? String(answer.compactionProblem,)
    : '';
}

await describe({
  name: startWaitingForAnAnswer.name,
  children: [
    it({
      name: 'ends on the silence bound when Pi reports no compaction, naming that bound',
      fn: async () => {
        /**
         The compaction Pi reports, which stays absent here.
         */
        const compaction = compactionUnderTest();
        /**
         What the test watches.
         */
        const watched = watchedWait({
          answerMilliseconds: SILENCE_BOUND_MILLISECONDS,
          deadline: Date.now() + FAR_DEADLINE_MILLISECONDS,
          compactionOfPi: compaction.of,
        },);
        await watched.done;
        expect(watched.answer().ending,).toBe('silence',);
        expect(
          problemOf(watched.answer(),),
        ).toBe(`no answer in ${String(SILENCE_BOUND_MILLISECONDS,)} ms`,);
      },
    },),
    it({
      name: 'a compaction under way is not silence, so the wait outlives the bound, and the answer is still taken',
      fn: async () => {
        /**
         The compaction Pi reports.
         */
        const compaction = compactionUnderTest();
        compaction.begin();
        /**
         What the test watches.
         */
        const watched = watchedWait({
          answerMilliseconds: SILENCE_BOUND_MILLISECONDS,
          deadline: Date.now() + FAR_DEADLINE_MILLISECONDS,
          compactionOfPi: compaction.of,
        },);
        expect(await settledWithin(watched, COMPACTION_MILLISECONDS,),).toBe(false,);
        // Pi finishes and answers in the same breath, which is what a real Pi does after compacting.
        compaction.end();
        watched.waiting
          .settle({
            ending: 'answered',
            record: { success: true, },
          },);
        await watched.done;
        expect(watched.answer().ending,).toBe('answered',);
      },
    },),
    it({
      name: 'the silence starts afresh when a compaction ends, so a wait that was nearly out gets the whole bound again',
      fn: async () => {
        /**
         The compaction Pi reports.
         */
        const compaction = compactionUnderTest();
        /**
         What the test watches.
         */
        const watched = watchedWait({
          answerMilliseconds: SILENCE_BOUND_MILLISECONDS,
          deadline: Date.now() + FAR_DEADLINE_MILLISECONDS,
          compactionOfPi: compaction.of,
        },);
        // Almost all of the bound is spent before Pi begins compacting.
        await sleep(SILENCE_BOUND_MILLISECONDS - LOOK_MILLISECONDS,);
        compaction.begin();
        watched.waiting
          .pauseSilence();
        expect(await settledWithin(watched, COMPACTION_MILLISECONDS,),).toBe(false,);
        compaction.end();
        watched.waiting
          .resumeSilence(NO_RUN_FAILURE,);
        /**
         When the silence began counting again, so the test can tell a fresh bound from what was left of the old one.
         */
        const resumedAt = Date.now();
        await watched.done;
        expect(watched.answer().ending,).toBe('silence',);
        // A wait that kept the silence it served before would have ended on the little the bound had left, near 10 ms.
        expect(Date.now() - resumedAt,).toBeGreaterThanOrEqual(SILENCE_BOUND_MILLISECONDS - LOOK_MILLISECONDS,);
      },
    },),
    it({
      name: 'a compaction that never ends is bounded by the deadline, which names the compaction it waited on',
      fn: async () => {
        /**
         The compaction Pi reports, which never ends.
         */
        const compaction = compactionUnderTest();
        compaction.begin();
        /**
         What the test watches.
         */
        const watched = watchedWait({
          answerMilliseconds: FAR_DEADLINE_MILLISECONDS,
          deadline: Date.now() + COMPACTION_MILLISECONDS,
          compactionOfPi: compaction.of,
        },);
        await watched.done;
        expect(watched.answer().ending,).toBe('deadline',);
        expect(
          compactionProblemOf(watched.answer(),),
        ).toContain('compacting',);
      },
    },),
    it({
      name: 'a compaction that fails is named when the silence after it runs out',
      fn: async () => {
        /**
         The compaction Pi reports, which fails.
         */
        const compaction = compactionUnderTest();
        compaction.begin();
        /**
         What the test watches.
         */
        const watched = watchedWait({
          answerMilliseconds: SILENCE_BOUND_MILLISECONDS,
          deadline: Date.now() + FAR_DEADLINE_MILLISECONDS,
          compactionOfPi: compaction.of,
        },);
        await sleep(LOOK_MILLISECONDS,);
        compaction.end();
        watched.waiting
          .resumeSilence({
            failed: true,
            problem: 'Auto-compaction failed: the request is larger than the model window',
          },);
        await watched.done;
        expect(watched.answer().ending,).toBe('silence',);
        expect(
          compactionProblemOf(watched.answer(),),
        ).toContain('larger than the model window',);
      },
    },),
    it({
      name: 'a settled wait arms nothing, so a compaction reported afterwards cannot keep the driver alive',
      fn: async () => {
        /**
         The compaction Pi reports.
         */
        const compaction = compactionUnderTest();
        /**
         What the test watches, with a deadline far enough away that an armed timer would hold the process open.
         */
        const watched = watchedWait({
          answerMilliseconds: SILENCE_BOUND_MILLISECONDS,
          deadline: Date.now() + FAR_DEADLINE_MILLISECONDS,
          compactionOfPi: compaction.of,
        },);
        watched.waiting
          .settle({
            ending: 'answered',
            record: { success: true, },
          },);
        /**
         How many timers the process holds once the wait is settled, which is the harness's own: the wait's bound is gone.
         */
        const timersSettled = timersHeld();
        compaction.begin();
        watched.waiting
          .pauseSilence();
        compaction.end();
        watched.waiting
          .resumeSilence(NO_RUN_FAILURE,);
        // A bound armed after the answer arrived would raise this count, and would hold the process until its deadline.
        expect(timersHeld(),).toBe(timersSettled,);
      },
    },),
  ],
},);

/**
 How many timers this process holds, which an armed bound would raise by one.

 @returns the count
 */
function timersHeld(): number {
  return process.getActiveResourcesInfo()
    .filter((resource,) => resource === 'Timeout').length;
}
