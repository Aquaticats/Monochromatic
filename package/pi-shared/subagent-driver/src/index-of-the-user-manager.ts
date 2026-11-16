/**
 What this machine's user manager is to a start: the question a run's unit depends on, the words of the refusal when it does not
 answer, and the folder its socket lives in. Re-exported from `index-of-starting-a-run.ts`.

 @module
 */

export {
  MANAGER_ANSWER_MILLISECONDS,
  type ManagerAnswer,
  managerAnswerOf,
  managerProblemIn,
  runtimeDirectoryTheManagerKeepsItsSocketIn,
} from './the-user-manager-this-machine-has.ts';
