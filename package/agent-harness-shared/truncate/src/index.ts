/**
 Public API for \@monochromatic-dev/agent-harness-shared-truncate.
 */
export {
  BYTES_PER_KIBIBYTE,
  BYTES_PER_MEBIBYTE,
  DEFAULT_MAX_BYTES,
  DEFAULT_MAX_LINES,
  formatSize,
  truncateHead,
} from './truncate.ts';
export type {
  TruncationOptions,
  TruncationResult,
} from './truncate.ts';
