/**
 Library surface of one driver run: the starts Pi makes before one of them takes the prompt, the watch that follows the prompt,
 the answer to a model error, the ask of an agent whose last answer said nothing, the work the run left running, how the run is
 put away, and the line the driver prints as a run that did not end on its own terms ends.

 Kept in a file of its own, as the other surfaces are, so no one list outgrows the line budget a file is allowed.

 @module
 */

export {
  type AnotherStart,
  anotherStartOf,
  type StartThatTookNoPrompt,
  startThatTookNoPromptOf,
  waitForAnotherStart,
} from './another-start-after-a-pi-exited-before-its-prompt.ts';
export {
  answerModelError,
  type RetryWaiting,
} from './answer-a-model-error.ts';
export {
  COMMAND_ANSWER_MILLISECONDS,
  FEED_PREVIEW_CHARACTERS,
  MILLISECONDS_PER_SECOND,
  NESTED_RUN_VARIABLE,
  START_ATTEMPTS,
  START_PAUSE_MILLISECONDS,
  START_SEQUENCE_MILLISECONDS,
  START_UP_ANSWER_MILLISECONDS,
} from './driver-constants.ts';
export type {
  NoFurtherStart,
  PiStartFailure,
} from './driver-types.ts';
export {
  type RunPutAway,
  putTheRunAway,
} from './put-the-run-away.ts';
export {
  type StartsOfARun,
  startsOfARun,
} from './starts-of-a-run.ts';
export {
  NestedDriverError,
  PiRunEndedEarlyError,
  supervisePiRun,
} from './supervise-pi-run.ts';
export {
  type Ending,
  endReasonOfRun,
} from './take-the-first-prompt.ts';
export { endingLineOf, } from './the-line-a-driver-ends-with.ts';
export { watchUntilEnded, } from './watch-pi-run.ts';
export type { WatchRequest, } from './watch-request.ts';
export {
  type StartWaitingRequest,
  startWaitingForAnAnswer,
  type WaitingForAnAnswer,
} from './wait-for-an-answer.ts';
export {
  askAnAgentWhoseAnswerWasEmpty,
  type EmptyAnswerAnswer,
  goOnMessage,
} from './ask-an-agent-whose-answer-was-empty.ts';
export {
  messageToPiCameTo,
  type MessageToPi,
} from './what-pi-did-with-a-message.ts';
export {
  cgroupOf,
  unitNamedBy,
  wakeMessageOf,
  workOutlivingTheTurnIn,
  workTheToolReportedIn,
} from './work-the-run-left-running.ts';
