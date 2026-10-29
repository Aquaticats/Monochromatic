export {
  createLogger,
  DEFAULT_FLUSH_DEADLINE_MS,
  DEFAULT_VERIFY_TIMEOUT_MS,
  STARTUP_BUFFER_CAP,
} from './create-logger.ts';
export { logger, } from './logger.ts';
export * as sinks from './sink/index.ts';
export { tagged, } from './tagged.ts';
export {
  observeLoggerCallbacks,
  type LoggerCallbackName,
  type LoggerCallbackObservation,
} from './observe-callbacks.ts';

//region Internal seams
// Underscore-prefixed re-exports let unit tests exercise internal modules
// through the built artifact (the `require-eventual-artifact` rule) without
// widening the documented API; they are not part of the public contract.
export { neutralizeControlCharacters as _neutralizeControlCharacters, } from './sink/console-control-chars.ts';
export {
  buildLogKey as _buildLogKey,
  compareLogKeys as _compareLogKeys,
  parseLogKey as _parseLogKey,
} from './sink/local-storage-key.ts';
export { detectLocalStorageQuotaChars as _detectLocalStorageQuotaChars, } from './sink/local-storage-quota.ts';
export { createLocalStorageStore as _createLocalStorageStore, } from './sink/local-storage-store.ts';
export { createRecordBuffer as _createRecordBuffer, } from './sink/record-buffer.ts';
export { detectSessionStorageQuotaChars as _detectSessionStorageQuotaChars, } from './sink/session-storage-quota.ts';
export { isQuotaExceededError as _isQuotaExceededError, } from './sink/web-storage-quota-error.ts';
//endregion Internal seams
export type {
  Level,
  Logger,
  LogRecord,
  Sink,
  SinkFlush,
  Verify,
} from './types.ts';
