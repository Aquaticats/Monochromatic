/**
 Cross-platform stable and degraded filesystem identity resolution.
 
 @module
 */

export {
  FsIdResolutionError,
  UnsupportedFsIdPlatformError,
} from './errors.ts';
export {
  assertFsId,
  isFsId,
} from './parsers.ts';
export { resolveFsId, } from './resolve-fs-id.ts';
export type {
  FsId,
  FsIdResolution,
  FsIdSource,
} from './types.ts';
