/**
 The plays a scenario makes of a prompt around a compaction: one Pi finishes before it answers, one it never finishes, and one
 that fails before Pi says anything more. Every play is started without waiting for it, because a real Pi goes on answering
 other commands while its compaction runs and the stand-in has to keep reading them.

 @module
 */

import { setTimeout as wait, } from 'node:timers/promises';
import {
  answer,
  emit,
  finishRun,
} from './stand-in-pi-records-fixture.ts';

/**
 Milliseconds a compaction takes in the scenarios that finish one, far above the silence bound a test asks for and far above the
 time the stand-in takes to report a compaction, so a wait that counted the compaction as silence would end mid-way through it.
 */
export const COMPACTION_MILLISECONDS = 1_000;

/**
 Why Pi begins these compactions: the session is over the part of its model's window Pi keeps for a reply, which is the reason
 `compaction_start` reports as `threshold`.
 */
const THRESHOLD_REASON = 'threshold';

/**
 What Pi reports when its summarisation request is larger than its model will take, shaped as a failed `compaction_end` writes
 it.
 */
export const COMPACTION_FAILURE_TEXT = 'Auto-compaction failed: the request is larger than the model window';

/**
 Writes the `compaction_end` record of a compaction that finished.
 */
function emitFinishedCompaction(): void {
  emit({
    type: 'compaction_end',
    reason: THRESHOLD_REASON,
    result: {
      summary: 'a summary of the session',
      firstKeptEntryId: 'kept-1',
      tokensBefore: 900_000,
      estimatedTokensAfter: 30_000,
    },
    aborted: false,
    willRetry: false,
  },);
}

/**
 Finishes the compaction and answers the prompt it held, once the compaction has run its course.

 @param id - command's identifier
 */
async function finishCompactionAndAnswer(id: unknown,): Promise<void> {
  await wait(COMPACTION_MILLISECONDS,);
  emitFinishedCompaction();
  answer({ id, },);
  finishRun('done',);
}

/**
 Compacts, leaving the prompt to be answered once the compaction ends.

 @param id - command's identifier

 @example
 ```ts
 compactThenAnswer('c1',);
 ```
 */
export function compactThenAnswer(id: unknown,): void {
  emit({
    type: 'compaction_start',
    reason: THRESHOLD_REASON,
  },);
  void finishCompactionAndAnswer(id,);
}

/**
 Compacts and never finishes, then answers nothing: a compaction whose request no bound of Pi's own ever ends.

 @example
 ```ts
 void startCompactionNeverEnding();
 ```
 */
export function startCompactionNeverEnding(): void {
  emit({
    type: 'compaction_start',
    reason: THRESHOLD_REASON,
  },);
}

/**
 Reports the failed compaction, once it has run its course.

 @param id - command's identifier, which is never answered
 */
async function failAfterWaiting(id: unknown,): Promise<void> {
  await wait(COMPACTION_MILLISECONDS,);
  emit({
    type: 'compaction_end',
    reason: THRESHOLD_REASON,
    aborted: false,
    willRetry: false,
    errorMessage: COMPACTION_FAILURE_TEXT,
  },);
  // Pi says nothing more: the session cannot be summarised, so its prompt is never taken.
  void id;
}

/**
 Compacts, fails, and answers nothing more: the session cannot be summarised, so the model window is still overfull when the
 prompt would reach it.

 @param id - command's identifier, which is never answered

 @example
 ```ts
 failCompactionThenStaySilent('c1',);
 ```
 */
export function failCompactionThenStaySilent(id: unknown,): void {
  emit({
    type: 'compaction_start',
    reason: THRESHOLD_REASON,
  },);
  void failAfterWaiting(id,);
}

/**
 What each compaction scenario does with a prompt, by scenario name.
 */
export const compactionPlays: ReadonlyMap<string, (id: unknown,) => void> = new Map<
  string,
  (id: unknown,) => void
  >([
    [
      'compaction-then-answer',
      compactThenAnswer,
    ],
    [
      'compaction-never-ends',
      startCompactionNeverEnding,
    ],
    [
      'compaction-fails',
      failCompactionThenStaySilent,
    ],
  ],);
