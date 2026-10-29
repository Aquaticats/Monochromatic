import {
  MANIFEST_VERSION,
  type DestinationStamp,
  type FileStamp,
  type GlobStamp,
  type StalenessEntry,
  type StalenessEntryKind,
  type StalenessManifest,
} from './staleness-types.ts';

/**
 Returns true when a value is a non-array object.
 
 @param value - Unknown value from JSON parsing.
 
 @returns Whether value can be inspected as a record.
 
 @example
 ```ts
 const valid = isRecord(value);
 ```
 */
function isRecord(value: unknown,): value is Record<string, unknown> {
  if ((typeof value) !== 'object')
    return false;
  if (value === null)
    return false;

  return !Array.isArray(value,);
}

/**
 Returns true when a value is a persisted file stamp.
 
 @param value - Unknown value from JSON parsing.
 
 @returns Whether value matches {@link FileStamp}.
 
 @example
 ```ts
 const valid = isFileStamp(parsed);
 ```
 */
function isFileStamp(value: unknown,): value is FileStamp {
  if (!isRecord(value,))
    return false;
  if ((typeof value.path) !== 'string')
    return false;
  if ((typeof value.size) !== 'number')
    return false;

  return (typeof value.mtimeMs) === 'number';
}

/**
 Returns true when a value is a persisted destination stamp.
 
 @param value - Unknown value from JSON parsing.
 
 @returns Whether value matches {@link DestinationStamp}.
 
 @example
 ```ts
 const valid = isDestinationStamp(parsed);
 ```
 */
function isDestinationStamp(value: unknown,): value is DestinationStamp {
  if (!isRecord(value,))
    return false;

  /**
   Hash value captured while `value` is still a generic record.
   */
  const { hash, } = value;
  if (!isFileStamp(value,))
    return false;

  return (typeof hash) === 'string';
}

/**
 Returns true when a value is a persisted glob stamp.
 
 @param value - Unknown value from JSON parsing.
 
 @returns Whether value matches {@link GlobStamp}.
 
 @example
 ```ts
 const valid = isGlobStamp(parsed);
 ```
 */
function isGlobStamp(value: unknown,): value is GlobStamp {
  if (!isRecord(value,))
    return false;
  if ((typeof value.pattern) !== 'string')
    return false;
  if (!Array.isArray(value.paths,))
    return false;

  return value
    .paths
    .every(function isPath(path,): path is string {
      return (typeof path) === 'string';
    },);
}

/**
 Returns true when a value is a supported entry kind.
 
 @param value - Unknown value from JSON parsing.
 
 @returns Whether value is a staleness entry kind.
 
 @example
 ```ts
 const valid = isEntryKind(value);
 ```
 */
function isEntryKind(value: unknown,): value is StalenessEntryKind {
  if (value === 'single')
    return true;

  return value === 'each';
}

/**
 Returns true when a value is a persisted staleness entry, validating its
 kind with {@link isEntryKind} and its per-field stamp shapes with
 {@link isFileStamp}, {@link isGlobStamp}, and {@link isDestinationStamp}.
 
 @param value - Unknown value from JSON parsing.
 
 @returns Whether value matches {@link StalenessEntry}.
 
 @example
 ```ts
 const valid = isStalenessEntry(parsed);
 ```
 */
function isStalenessEntry(value: unknown,): value is StalenessEntry {
  if (!isRecord(value,))
    return false;
  if (!isEntryKind(value.kind,))
    return false;
  if (!Array.isArray(value.sourceFiles,))
    return false;
  if (!value
    .sourceFiles
    .every(function sourceFileIsStamp(sourceFile,): boolean {
      return isFileStamp(sourceFile,);
    },))
    return false;
  if (!Array.isArray(value.sourceGlobs,))
    return false;
  if (!value
    .sourceGlobs
    .every(function sourceGlobIsStamp(sourceGlob,): boolean {
      return isGlobStamp(sourceGlob,);
    },))
    return false;
  if (!Array.isArray(value.destinationFiles,))
    return false;
  if (!value
    .destinationFiles
    .every(function destinationFileIsStamp(destinationFile,): boolean {
      return isDestinationStamp(destinationFile,);
    },))
    return false;
  if ((typeof value.sourceSetHash) !== 'string')
    return false;

  return (typeof value.updatedAt) === 'string';
}

/**
 Returns true when a value is a persisted staleness manifest.
 
 @param value - Unknown value from JSON parsing.
 
 @returns Whether value matches {@link StalenessManifest}.
 
 @example
 ```ts
 const valid = isStalenessManifest(parsed);
 ```
 */
export function isStalenessManifest(value: unknown,): value is StalenessManifest {
  if (!isRecord(value,))
    return false;
  if (value.version !== MANIFEST_VERSION)
    return false;
  if (!isRecord(value.entries,))
    return false;

  return Object.values(value.entries,)
    .every(function entryIsStalenessEntry(entry,): boolean {
      return isStalenessEntry(entry,);
    },);
}
