/**
 The library surface of what one run does after its prompt: the watch, the answer to a model error, the ask of an agent whose
 last answer said nothing, and the work the run left running. `index.ts` re-exports it.

 @module
 */

/**
 One run after its prompt: the watch that decides when it is over, the attempt after a model error, the ask of an agent whose
 last answer said nothing, and the work that run left running. The command is `drive-pi-subagent.ts`, and the launcher's own
 wake reads the same work.
 */
export { watchUntilEnded, } from './watch-pi-run.ts';
export type { WatchRequest, } from './watch-request.ts';
export {
  answerModelError,
  type RetryWaiting,
} from './answer-a-model-error.ts';
export {
  askAnAgentWhoseAnswerWasEmpty,
  type EmptyAnswerAnswer,
  goOnMessage,
} from './ask-an-agent-whose-answer-was-empty.ts';
export {
  cgroupOf,
  unitNamedBy,
  wakeMessageOf,
  workOutlivingTheTurnIn,
  workTheToolReportedIn,
} from './work-the-run-left-running.ts';
