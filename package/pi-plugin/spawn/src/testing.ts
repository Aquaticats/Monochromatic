/**
 Built entry the unit tests import.

 The package entry registers this pi extension's session handlers when it loads, so a
 test cannot import it. This module re-exports the spawn state, path, session-finder,
 CLI-core, setup, and message-extraction behavior the suites drive, so they load what
 this package builds rather than sibling source, as
 `test-import/require-eventual-artifact` requires.

 `rolldown.node.config.ts` builds this file to `dist/final/node/testing.mjs`, beside
 the extension bundle. Nothing in this package imports it, and pi never loads it.

 @module
 */

//region Paths

/**
 Where spawn state, pid mappings, and the pi agent directory live.

 @internal
 */
export {
  byPidDir,
  piAgentDir,
  reportedStatePath,
  spawnResultsDir,
  spawnsDir,
  spawnStatePath,
  type PidMapping,
  type SpawnState,
} from './paths.ts';

//endregion Paths

//region Spawn state

/**
 The spawn records: claims, completions, the results a parent reports, and the pid
 mappings the session finder reads.

 @internal
 */
export {
  checkCompletedChildren,
  claimSpawn,
  completeSpawn,
  formatSpawnResult,
  isSpawnJsonFilename,
  NOTHING_TO_REPORT,
  spawnIdFromJsonFilename,
  writeInitialSpawnState,
  writePidMapping,
} from './state.ts';

//endregion Spawn state

//region Session finding

/**
 The calling session: process-tree walk, pid mappings, and the newest-mapping fallback.

 @internal
 */
export {
  findByMostRecent,
  findCallingSession,
  readByPidDir,
  readParentPid,
  readPidMapping,
  SESSION_NOT_FOUND,
  walkProcessTreeFrom,
} from './session-finder.ts';

//endregion Session finding

//region CLI core

/**
 Argument vectors for the spawned pi process and the state it starts from.

 @internal
 */
export {
  extensionArguments,
  initialSpawnState,
  piCommandArguments,
  SESSION_NOT_FOUND_WARNING,
  SPAWN_ID_TITLE_LENGTH,
  terminalInvocation,
  UNLINKED_SPAWN_TITLE,
} from './cli-core.ts';

//endregion CLI core

//region CLI setup

/**
 Whether the spawn CLI is on `PATH`, and what to warn when it is not.

 @internal
 */
export {
  autoSetupCli,
  cliIsOnPath,
  cliPathFromExtensionPath,
  isBuiltExtensionPath,
  NO_CLI_SETUP_WARNING,
  packageRootFromExtensionPath,
} from './setup-cli.ts';

//endregion CLI setup

//region Message extraction

/**
 Reading the last assistant text out of a session's messages.

 @internal
 */
export {
  assistantContentText,
  extractLastAssistantText,
  isAssistantMessage,
  isTextBlock,
} from './message-extract.ts';

//endregion Message extraction

//region Constants

/**
 Environment variables and the custom message type this extension publishes.

 @internal
 */
export {
  SPAWN_EXTENSION_PATH_ENV,
  SPAWN_ID_ENV,
  SPAWN_PI_CUSTOM_TYPE,
} from './constants.ts';

//endregion Constants
