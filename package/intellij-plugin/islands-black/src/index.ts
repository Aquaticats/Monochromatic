/**
 Archive listing helpers the plugin JAR build uses.

 Exported so the package's unit tests reach the built bundle rather than a
 sibling source file; the build script itself is the only consumer inside this
 package.

 @module
 */

export {
  type ArchiveEntry,
  compareDirentNames,
  listArchiveEntries,
  toArchivePath,
} from './archive-entries.ts';
