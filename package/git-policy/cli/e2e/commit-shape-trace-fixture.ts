/**
 Commit-shape trace model:
 anonymized per-commit change shapes mined from repository history.

 A trace keeps only shapes
 (files per commit,
 path overlap through stable numeric path IDs,
 sizes,
 line counts,
 change kinds,
 modes,
 and binary flags).
 It never holds file contents,
 paths,
 object IDs,
 messages,
 or author data.

 @module
 */

import type {
  ParsedChange,
  ParsedCommit,
} from './commit-shape-log-fixture.ts';

//region Types

/**
 Change kind replayed by the container workload.
 Copies replay as additions.
 */
export type ShapeChangeKind = 'add' | 'delete' | 'modify' | 'rename';

/**
 Tree entry kind after the change (before it for deletions).
 */
export type ShapeMode = 'executable' | 'file' | 'gitlink' | 'symlink';

/**
 One anonymized path change.
 */
export type ShapeChange = Readonly<{
  /**
   Replayed change kind.
   */
  kind: ShapeChangeKind;
  /**
   Stable anonymous path ID; equal IDs mean the same path.
   */
  path: number;
  /**
   Source path ID of a rename.
   */
  from?: number;
  /**
   Entry kind.
   */
  mode: ShapeMode;
  /**
   Blob byte size after the change, or before it for deletions.
   */
  size: number;
  /**
   Whether Git's numstat treated the change as binary.
   */
  binary: boolean;
  /**
   Added lines, `0` for binary changes.
   */
  added: number;
  /**
   Deleted lines, `0` for binary changes.
   */
  deleted: number;
}>;

/**
 One anonymized commit.
 */
export type CommitShape = Readonly<{
  /**
   Path changes in diff order.
   */
  changes: readonly ShapeChange[];
}>;

/**
 Complete committed trace document.
 */
export type CommitShapeTrace = Readonly<{
  /**
   Trace schema revision.
   */
  schemaVersion: 1;
  /**
   Human-readable provenance without identifying data.
   */
  source: string;
  /**
   Number of commits in `commits`.
   */
  commitCount: number;
  /**
   Number of distinct path IDs.
   */
  pathCount: number;
  /**
   Commits oldest first.
   */
  commits: readonly CommitShape[];
}>;

//endregion Types

//region Errors

/**
 Trace document or blob-size data is inconsistent.
 */
export class CommitShapeTraceError extends Error {
  /**
   Creates a trace error.

   @param message - reason

   @example
   ```ts
   throw new CommitShapeTraceError('missing blob size');
   ```
   */
  constructor(message: string,) {
    super(message,);
    this.name = 'CommitShapeTraceError';
  }
}

//endregion Errors

//region Anonymization

/**
 Object ID Git prints for an absent side of a change.
 */
const ZERO_OID_CHARACTER = '0';

/**
 Reports whether an object ID names no object.

 @param oid - full hexadecimal object ID

 @returns whether every character is zero

 @example
 ```ts
 isZeroOid('0000'); // => true
 ```
 */
function isZeroOid(oid: string,): boolean {
  return oid.replaceAll(
    ZERO_OID_CHARACTER,
    '',
  ) === '';
}

/**
 Maps an octal Git mode to a shape mode.

 @param mode - six-digit octal mode

 @returns entry kind

 @throws {@link CommitShapeTraceError} on modes Git does not store in trees

 @example
 ```ts
 shapeMode('100755'); // => 'executable'
 ```
 */
export function shapeMode(mode: string,): ShapeMode {
  /**
   Mode lookup for the four tree entry kinds Git records.
   */
  const modes: Readonly<Record<string, ShapeMode>> = {
    '100644': 'file',
    '100755': 'executable',
    '120000': 'symlink',
    '160000': 'gitlink',
  };
  /**
   Matched entry kind.
   */
  const matched = modes[mode];
  if (matched === undefined)
    throw new CommitShapeTraceError(`unsupported tree mode ${mode}`,);
  return matched;
}

/**
 Lists blob IDs whose sizes the trace needs.

 @param commits - parsed commits

 @returns unique non-zero blob IDs, excluding gitlink commits

 @example
 ```ts
 collectBlobOids(parseCommitShapeLog(text));
 ```
 */
export function collectBlobOids(commits: readonly ParsedCommit[],): readonly string[] {
  return [
    ...new Set(commits.flatMap(function commitOids(commit,) {
      return commit.changes
        .flatMap(function changeOid(change,) {
        /**
         Side whose size the shape records.
         */
          const oid = change.status === 'D' ? change.srcOid : change.dstOid;
        /**
         Mode of that side.
         */
          const mode = change.status === 'D' ? change.srcMode : change.dstMode;
          return isZeroOid(oid,) || (mode === '160000') ? [] : [oid,];
        },);
    },),),
  ];
}

/**
 Parses `git cat-file --batch-check='%(objectname) %(objectsize)'` output.

 @param text - exact batch-check output

 @returns size per object ID

 @throws {@link CommitShapeTraceError} on missing objects or malformed lines

 @example
 ```ts
 parseBlobSizes('abc 12\n').get('abc'); // => 12
 ```
 */
export function parseBlobSizes(text: string,): ReadonlyMap<string, number> {
  return new Map(text
    .split('\n',)
    .filter(function nonEmptyLine(line,) {
      return line !== '';
    },)
    .map(function sizeEntry(line,): [
      string,
      number
    ] {
      /**
       Object ID and size fields.
       */
      const [oid = '', sizeField = '',] = line.split(' ',);
      /**
       Parsed byte size.
       */
      const size = Number(sizeField,);
      if (!Number.isSafeInteger(size,))
        throw new CommitShapeTraceError(`batch-check line has no size: ${line}`,);
      return [
        oid,
        size,
      ];
    },),);
}

/**
 Maps a raw status to a replayed kind.

 @param status - raw status letter

 @returns replayed change kind

 @example
 ```ts
 shapeKind('C'); // => 'add'
 ```
 */
function shapeKind(status: ParsedChange['status'],): ShapeChangeKind {
  /**
   Status-to-kind lookup.
   */
  const kinds: Readonly<Record<ParsedChange['status'], ShapeChangeKind>> = {
    A: 'add',
    C: 'add',
    D: 'delete',
    M: 'modify',
    R: 'rename',
    T: 'modify',
  };
  return kinds[status];
}

/**
 Anonymizes parsed commits into a trace.
 Path IDs are assigned in first-appearance order,
 so equal IDs across commits preserve path overlap.

 @param commits - parsed commits oldest first

 @param blobSizes - byte size per blob ID

 @param source - provenance text without identifying data

 @returns trace document

 @throws {@link CommitShapeTraceError} when a needed blob size is missing

 @example
 ```ts
 anonymizeCommitShapes({ commits, blobSizes, source: 'newest 10 commits' });
 ```
 */
export function anonymizeCommitShapes({
  commits,
  blobSizes,
  source,
}: Readonly<{
  commits: readonly ParsedCommit[];
  blobSizes: ReadonlyMap<string, number>;
  source: string;
}>,): CommitShapeTrace {
  /**
   Path ID per real path, filled in first-appearance order.
   */
  const pathIds = new Map<string, number>();
  /**
   Returns the stable ID of a path, assigning the next one on first sight.

   @param path - real repository path

   @returns anonymous path ID

   @example
   ```ts
   pathId('README.md'); // => 0
   ```
   */
  function pathId(path: string,): number {
    /**
     Previously assigned ID.
     */
    const known = pathIds.get(path,);
    if (known !== undefined)
      return known;
    pathIds.set(
      path,
      pathIds.size,
    );
    return pathIds.size - 1;
  }
  /**
   Anonymized commits.
   */
  const shapes = commits.map(function anonymizeCommit(commit,): CommitShape {
    return {
      changes: commit.changes
        .map(function anonymizeChange(change,): ShapeChange {
        /**
         Whether the recorded side is the pre-image.
         */
          const deleted = change.status === 'D';
        /**
         Recorded side's mode.
         */
          const mode = shapeMode(deleted ? change.srcMode : change.dstMode,);
        /**
         Recorded side's blob.
         */
          const oid = deleted ? change.srcOid : change.dstOid;
        /**
         Recorded side's size; gitlinks have no blob.
         */
          const size = mode === 'gitlink' ? 0 : blobSizes.get(oid,);
          if (size === undefined)
            throw new CommitShapeTraceError(`missing blob size for a ${change.status} change`,);
        /**
         Rename source ID assigned before the destination.
         */
          const from = (change.status === 'R') && (change.fromPath !== undefined) ? pathId(change.fromPath,) : undefined;
        /**
         Whether numstat reported a binary change.
         */
          const binary = change.added === 'binary';
          return {
            kind: shapeKind(change.status,),
            path: pathId(change.path,),
            ...(from === undefined ? {} : { from, }),
            mode,
            size,
            binary,
            added: change.added === 'binary' ? 0 : change.added,
            deleted: binary ? 0 : change.deleted,
          };
        },),
    };
  },);
  return {
    schemaVersion: 1,
    source,
    commitCount: shapes.length,
    pathCount: pathIds.size,
    commits: shapes,
  };
}

//endregion Anonymization

//region Serialization

/**
 Serializes a trace with one commit per line so regenerated diffs stay reviewable.

 @param trace - trace document

 @returns LF-terminated JSON text

 @example
 ```ts
 serializeCommitShapeTrace(trace);
 ```
 */
export function serializeCommitShapeTrace(trace: CommitShapeTrace,): string {
  /**
   One JSON line per commit.
   */
  const commitLines = trace.commits
    .map(function commitLine(commit,) {
      return `    ${JSON.stringify(commit,)}`;
    },);
  return [
    '{',
    `  "schemaVersion": ${JSON.stringify(trace.schemaVersion,)},`,
    `  "source": ${JSON.stringify(trace.source,)},`,
    `  "commitCount": ${JSON.stringify(trace.commitCount,)},`,
    `  "pathCount": ${JSON.stringify(trace.pathCount,)},`,
    '  "commits": [',
    commitLines.join(',\n',),
    '  ]',
    '}',
    '',
  ].join('\n',);
}

/**
 Reports whether a value is a plain object whose fields can be read.

 @param value - parsed JSON

 @returns whether fields can be inspected

 @example
 ```ts
 isRecord({}); // => true
 ```
 */
function isRecord(value: unknown,): value is Readonly<Record<string, unknown>> {
  return ((typeof value) === 'object') && (value !== null)
    && (!Array.isArray(value,));
}

/**
 Reports whether a value is an array without widening its elements to `any`.

 @param value - parsed JSON

 @returns whether the value is an array

 @example
 ```ts
 isUnknownArray([]); // => true
 ```
 */
function isUnknownArray(value: unknown,): value is readonly unknown[] {
  return Array.isArray(value,);
}

/**
 Reports whether a value is a non-negative safe integer.

 @param value - parsed JSON

 @returns whether the value counts something

 @example
 ```ts
 isCount(3); // => true
 ```
 */
function isCount(value: unknown,): value is number {
  return ((typeof value) === 'number') && Number.isSafeInteger(value,)
    && (value >= 0);
}

/**
 Change kinds a trace may contain.
 */
const SHAPE_KINDS: ReadonlySet<unknown> = new Set<ShapeChangeKind>([
  'add',
  'delete',
  'modify',
  'rename',
],);

/**
 Entry modes a trace may contain.
 */
const SHAPE_MODES: ReadonlySet<unknown> = new Set<ShapeMode>([
  'executable',
  'file',
  'gitlink',
  'symlink',
],);

/**
 Reports whether a parsed value is a valid change shape.

 @param value - parsed JSON

 @returns whether every field has its declared type

 @example
 ```ts
 isShapeChange({ kind: 'add', path: 0, mode: 'file', size: 1, binary: false, added: 1, deleted: 0 });
 ```
 */
function isShapeChange(value: unknown,): value is ShapeChange {
  return isRecord(value,)
    && SHAPE_KINDS.has(value.kind,)
    && SHAPE_MODES.has(value.mode,)
    && isCount(value.path,)
    && ((value.from === undefined) || isCount(value.from,))
    && isCount(value.size,)
    && ((typeof value.binary) === 'boolean')
    && isCount(value.added,)
    && isCount(value.deleted,);
}

/**
 Reports whether a parsed value is a valid commit shape.

 @param value - parsed JSON

 @returns whether every change is valid

 @example
 ```ts
 isCommitShape({ changes: [] }); // => true
 ```
 */
function isCommitShape(value: unknown,): value is CommitShape {
  return isRecord(value,)
    && isUnknownArray(value.changes,)
    && value.changes
      .every(isShapeChange,);
}

/**
 Validates a parsed trace document.

 @param value - parsed JSON

 @returns validated trace

 @throws {@link CommitShapeTraceError} on schema,
 count,
 or path-ID mismatches

 @example
 ```ts
 assertCommitShapeTrace(JSON.parse(text));
 ```
 */
export function assertCommitShapeTrace(value: unknown,): CommitShapeTrace {
  if ((!isRecord(value,)) || (value.schemaVersion !== 1))
    throw new CommitShapeTraceError('trace schema mismatch',);
  /**
   Top-level trace fields.
   */
  const {
    source,
    commitCount,
    pathCount,
    commits: commitValues,
  } = value;
  if (((typeof source) !== 'string') || (!isCount(commitCount,))
    || (!isCount(pathCount,))
    || (!isUnknownArray(commitValues,)))
    throw new CommitShapeTraceError('trace header fields malformed',);
  /**
   Commits that passed shape validation.
   */
  const commits = commitValues.filter(isCommitShape,);
  if ((commits.length !== commitValues.length) || (commits.length !== commitCount))
    throw new CommitShapeTraceError('trace commit count mismatch or malformed commit',);
  /**
   Every path ID referenced by any change.
   */
  const ids = commits.flatMap(function commitIds(commit,) {
    return commit.changes
      .flatMap(function changeIds(change,) {
        return change.from === undefined ? [change.path,] : [
          change.path,
          change.from,
        ];
      },);
  },);
  if (ids.some(function outOfRange(id,) {
    return id >= pathCount;
  },))
    throw new CommitShapeTraceError('trace path ID out of range',);
  return {
    schemaVersion: 1,
    source,
    commitCount,
    pathCount,
    commits,
  };
}

//endregion Serialization
