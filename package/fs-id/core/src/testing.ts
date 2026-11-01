/**
 Explicit testing seam for filesystem identity adapters and platform fixtures.
 
 This subpath is not required by ordinary consumers.
 
 @module
 */

export {
  createFsId,
  isFsId,
  normalizeIdentityPayload,
  parseDfDevice,
  parseDiskutilVolumeUuid,
  parseFindmntUuid,
  parseVolumeSerial,
  windowsDriveRoot,
} from './parsers.ts';
export {
  resolveDarwinFsId,
  resolveLinuxFsId,
  resolveWindowsFsId,
} from './platform-resolvers.ts';
export { createFsIdResolver, } from './resolve-fs-id.ts';
export type {
  FsIdCommand,
  FsIdResolver,
  FsIdResolverAdapters,
  SupportedFsIdPlatform,
} from './types.ts';
