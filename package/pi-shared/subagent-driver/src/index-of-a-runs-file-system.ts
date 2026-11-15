/**
 What a run's unit is given to write, and what the launcher reads about it: the folders a unit may and may not write, the one path
 written as systemd reads it, and what a unit left in its run's folder. Re-exported from `index-of-starting-a-run.ts`, so the
 package's surface is unchanged.

 @module
 */

export {
  type FileSystemOfARun,
  FolderAToolMakesCannotBeMadeError,
  WorktreeGoneError,
  fileSystemOfARun,
  fileSystemPropertiesOf,
  foldersAToolMakesIn,
  makeTheFoldersAToolMakesIn,
  refuseUnlessTheWorktreeIsThere,
  worktreeGoneWordsOf,
} from './file-system-of-a-run.ts';
export {
  PathThatIsNotAbsoluteError,
  pathInAUnitProperty,
} from './path-in-a-unit-property.ts';
export {
  type TheRunAUnitLeft,
  theRunAUnitLeftIn,
  theRunThatOutlivedItsWorktreeLineOf,
  theWorktreeThatWentLineIn,
} from './the-run-a-unit-left.ts';

export {
  type OpenAFile,
  NotAnOrdinaryFileError,
  ORDINARY_FILE_READ_MILLISECONDS,
  ReadThatTookTooLongError,
  readOrdinaryFile,
  readOrdinaryFileWithin,
} from './read-ordinary-file.ts';
