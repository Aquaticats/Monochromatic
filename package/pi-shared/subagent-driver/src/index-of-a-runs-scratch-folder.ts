/**
 The library surface of a run's one scratch folder: the name it carries, the folder the launcher makes with its `keep/` subfolder,
 the note the run's first prompt carries, the emptying of it when the run ends, and the step a launcher takes once its unit ended.
 `index-of-starting-a-run.ts` re-exports it.

 @module
 */

//region The one scratch folder a run gets, and the emptying of it

/**
 The one scratch folder a run gets, the emptying of it when the run ends, and the step a launcher takes once its unit ended. The
 commands are `start-pi-run-in-a-worktree.ts` and `empty-the-scratch-folder-of-a-run.ts`.
 */
export {
  KEEP_FOLDER,
  type MadeScratchFolder,
  SCRATCH_FOLDER_END,
  SCRATCH_FOLDER_MODE,
  SCRATCH_FOLDER_VARIABLE,
  SCRATCH_ROOT_IN_HOME,
  ScratchFolderError,
  defaultScratchRoot,
  makeScratchFolder,
  nameProblemOf,
  scratchFolderNameOf,
  scratchFolderObstacleIn,
  scratchFolderOf,
  scratchFolderProblemIn,
  scratchNoteOf,
} from './scratch-folder-of-a-run.ts';
export {
  type OwnedScratchEmptyingRequest,
  type ScratchEmptyingRequest,
  type ScratchFolderEmptiness,
  emptyTheFolderIn,
  emptyTheFolderItsRunStillOwns,
  emptyTheFolderOfRun,
} from './emptying-a-scratch-folder.ts';
export {
  ownershipProblemNow,
  ownershipProblemOf,
} from './owner-of-a-scratch-folder.ts';
export {
  type ScratchRemovalTally,
  type UnremovedEntry,
  emptyTally,
  entriesLeftIn,
  leftWhereItIsIn,
  removeInto,
} from './deleting-what-a-run-left-in-its-scratch-folder.ts';
export {
  entryIn,
  namesIn,
  reasonOfFailure,
} from './what-a-scratch-folder-holds.ts';
export {
  mountPathsIn,
  mountPointsNow,
} from './mounts-of-this-machine.ts';
export {
  ENTRIES_NAMED_IN_A_LINE,
  bytesTextOf,
  scratchEmptyingLinesOf,
} from './the-line-an-emptied-scratch-folder-answers-with.ts';
export {
  SCRATCH_EMPTYING_BOUND_SECONDS,
  emptyTheScratchFolderAtTheLowestDiskPriority,
} from './scratch-folder-emptied-at-the-lowest-disk-priority.ts';
export { scratchFolderLinesOf, } from './scratch-folders-of-finished-runs.ts';
export {
  type FinishedRun,
  type FinishRequest,
  finishTheRun,
} from './finish-a-run.ts';

//endregion The one scratch folder a run gets, and the emptying of it
