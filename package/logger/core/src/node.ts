/**
 Node-only entry (`\@monochromatic-dev/logger-core/node`).

 Ships the file sink, whose static `node:fs/promises` and `node:path`
 imports must never reach the platform-neutral root entry. Built only by
 `rolldown.node.config.ts`, so the neutral artifact carries no `node:`
 specifier at all.

 @module
 */

export { createFileSink, } from './sink/file.ts';

//region Retention policy surface of the file sink
export {
  DEFAULT_LOG_MAX_AGE_DAYS,
  DEFAULT_LOG_MAX_BYTES,
  DEFAULT_LOG_MAX_FILES,
  NO_LOG_RETENTION_SWEEP,
  type LogRetentionPolicy,
} from './sink/file-retention-policy.ts';
//endregion Retention policy surface of the file sink

//region Internal seams
export { userLogDirectory as _userLogDirectory, } from './sink/file-state.ts';
// Underscore-prefixed re-exports let `file.unit.test.ts` exercise the ancestor
// search through the built artifact (the `require-eventual-artifact` rule)
// without widening the documented API; they are not part of the public
// contract.
export {
  findNodeModulesUp as _findNodeModulesUp,
  NO_NODE_MODULES_FOUND as _NO_NODE_MODULES_FOUND,
} from './sink/file-project.ts';
export {
  cutoffDay as _cutoffDay,
  sweepLogDirectory as _sweepLogDirectory,
} from './sink/file-retention-sweep.ts';
export {
  loggerOwnedDay as _loggerOwnedDay,
  NOT_LOGGER_OWNED as _NOT_LOGGER_OWNED,
} from './sink/file-retention-name.ts';
export {
  logRetentionPolicyFromEnv as _logRetentionPolicyFromEnv,
} from './sink/file-retention-policy.ts';
export {
  openLogFile as _openLogFile,
  probeLogDirectory as _probeLogDirectory,
} from './sink/file-open.ts';
//endregion Internal seams
