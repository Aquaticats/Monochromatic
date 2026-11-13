/**
 The library surface of what a start needs before it makes anything: the room of each volume a run cannot work without, the floor
 it needs on each, how a folder's volume is found, and how many agents the machine may carry at once. `index-of-starting-a-run.ts`
 re-exports it.

 @module
 */

export {
  BUILD_VOLUME,
  BUILD_VOLUME_FLOOR_BYTES,
  BYTES_IN_A_GIBIBYTE,
  BYTES_IN_A_KIBIBYTE,
  type FreeBlocksOfAVolume,
  type FreeSpace,
  type FreeSpaceAgainstItsFloor,
  RUNS_FOLDER_FLOOR_BYTES,
  SESSION_FILES_FLOOR_BYTES,
  type VolumeAgainstItsFloor,
  freeSpaceFloorTextOf,
  freeSpaceIsUnderTheFloor,
  freeSpaceOnVolume,
  freeSpaceTextOf,
  freeSpaceUnderTheFloorTextOf,
  strictestFloorPerVolume,
  wholeGibibytesOf,
} from './free-space-floor.ts';
export {
  MACHINE_CAP_VARIABLE,
  type MachineCap,
  MachineCapError,
  type MachineCapPlace,
  type MachineCapSource,
  OTHER_AGENTS_GOING_VARIABLE,
  agentsGoingOf,
  machineCapCountsTextOf,
  machineCapFromLevel,
  machineCapFromVariable,
  machineCapIn,
  machineCapRefusalTextOf,
  machineCapSourceTextOf,
  machineTakesNoMoreAgents,
  otherAgentsGoingIn,
} from './machine-cap.ts';
export {
  MACHINE_CAP_LEVEL_FILE_NAME,
  type MachineCapLevel,
  MachineCapLevelError,
  type ReadText,
  machineCapLevelIn,
} from './machine-agent-cap-level.ts';
export {
  LARGEST_EXACTLY_HELD_WHOLE_NUMBER,
  type WholeNumberProblem,
  type WholeNumberReading,
  wholeNumberIn,
} from './whole-number-setting.ts';
export {
  SESSION_FOLDER_VARIABLE,
  sessionFolderOf,
} from './pi-session-folder.ts';
export {
  type AnswerAboutAFolder,
  freeSpaceOfTheFolder,
  volumeOfFolder,
} from './volume-of-a-folder.ts';
export {
  FOLDER_VOLUMES_VARIABLE,
  VOLUME_FREE_SPACE_BYTES_VARIABLE,
  VolumeSettingsError,
  givenFolderVolumes,
  givenVolumeFreeSpaceBytes,
} from './volume-settings.ts';
