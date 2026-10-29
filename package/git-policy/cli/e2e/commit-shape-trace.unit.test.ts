import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  CommitShapeLogError,
  parseCommitShapeLog,
} from './commit-shape-log-fixture.ts';
import {
  anonymizeCommitShapes,
  assertCommitShapeTrace,
  collectBlobOids,
  CommitShapeTraceError,
  parseBlobSizes,
  serializeCommitShapeTrace,
  shapeMode,
} from './commit-shape-trace-fixture.ts';

//region Fixtures

/**
 All-zero object ID for absent sides.
 */
const ZERO = '0'.repeat(40,);

/**
 Builds a fake full object ID from one character.

 @param character - repeated hexadecimal digit

 @returns 40-character object ID

 @example
 ```ts
 oid('a');
 ```
 */
function oid(character: string,): string {
  return character.repeat(40,);
}

/**
 Git log text with an add, a binary modify, a rename, a delete, and an empty commit.
 Layout matches `git log -z --format=%x1ecommit --raw --numstat --no-abbrev -M`.
 */
const LOG_TEXT = [
  '\u001Ecommit\0',
  `\n:000000 100644 ${ZERO} ${oid('a',)} A\0src/one.ts\0`,
  `:100644 100755 ${oid('b',)} ${oid('c',)} M\0bin/tool\0`,
  '3\t0\tsrc/one.ts\0',
  '-\t-\tbin/tool\0',
  '\u001Ecommit\0',
  `\n:100644 100644 ${oid('a',)} ${oid('d',)} R087\0src/one.ts\0src/two.ts\0`,
  `:100755 000000 ${oid('c',)} ${ZERO} D\0bin/tool\0`,
  '1\t1\t\0src/one.ts\0src/two.ts\0',
  '0\t9\tbin/tool\0',
  '\u001Ecommit\0',
].join('',);

/**
 Blob sizes matching {@link LOG_TEXT}.
 */
const SIZES = new Map([
  [oid('a',), 10,],
  [oid('c',), 2_048,],
  [oid('d',), 12,],
],);

//endregion Fixtures

await describe({
  name: '',
  children: [
    describe({
      name: parseCommitShapeLog.name,
      children: [
        it({
          name: 'pairs raw and numstat records per commit, including rename and binary forms',
          fn: async () => {
            const commits = parseCommitShapeLog(LOG_TEXT,);
            expect(commits,).toHaveLength(3,);
            expect(commits[0]?.changes.map(function summary(change,) {
              return [change.status, change.path, change.added, change.deleted,];
            },),).toEqual([
              ['A', 'src/one.ts', 3, 0,],
              ['M', 'bin/tool', 'binary', 0,],
            ],);
            expect(commits[1]?.changes[0],).toMatchObject({
              status: 'R',
              fromPath: 'src/one.ts',
              path: 'src/two.ts',
              added: 1,
              deleted: 1,
            },);
            expect(commits[2]?.changes,).toEqual([],);
          },
        },),
        it({
          name: 'rejects unknown statuses',
          fn: async () => {
            expect(function parseUnknown() {
              return parseCommitShapeLog(`\u001Ecommit\0:100644 100644 ${ZERO} ${ZERO} X\0a\u00000\t0\ta\0`,);
            },).toThrow(CommitShapeLogError,);
          },
        },),
        it({
          name: 'rejects numstat records that do not pair with raw records',
          fn: async () => {
            expect(function parseUnpaired() {
              return parseCommitShapeLog(`\u001Ecommit\0:100644 100644 ${ZERO} ${ZERO} M\0a\u00001\t0\tb\0`,);
            },).toThrow(CommitShapeLogError,);
          },
        },),
        it({
          name: 'rejects records without the commit marker',
          fn: async () => {
            expect(function parseMarkerless() {
              return parseCommitShapeLog('\u001Enot-commit\0',);
            },).toThrow(CommitShapeLogError,);
          },
        },),
      ],
    },),
    describe({
      name: anonymizeCommitShapes.name,
      children: [
        it({
          name: 'assigns stable path IDs so overlap survives and paths disappear',
          fn: async () => {
            const trace = anonymizeCommitShapes({
              commits: parseCommitShapeLog(LOG_TEXT,),
              blobSizes: SIZES,
              source: 'fixture',
            },);
            expect(trace.pathCount,).toBe(3,);
            expect(trace.commits[0]?.changes,).toEqual([
              { kind: 'add', path: 0, mode: 'file', size: 10, binary: false, added: 3, deleted: 0, },
              { kind: 'modify', path: 1, mode: 'executable', size: 2_048, binary: true, added: 0, deleted: 0, },
            ],);
            expect(trace.commits[1]?.changes,).toEqual([
              { kind: 'rename', path: 2, from: 0, mode: 'file', size: 12, binary: false, added: 1, deleted: 1, },
              { kind: 'delete', path: 1, mode: 'executable', size: 2_048, binary: false, added: 0, deleted: 9, },
            ],);
            expect(JSON.stringify(trace,),).not.toContain('src/',);
            expect(JSON.stringify(trace,),).not.toContain(oid('a',),);
          },
        },),
        it({
          name: 'fails when a blob size is missing',
          fn: async () => {
            expect(function anonymizeWithoutSizes() {
              return anonymizeCommitShapes({
                commits: parseCommitShapeLog(LOG_TEXT,),
                blobSizes: new Map(),
                source: 'fixture',
              },);
            },).toThrow(CommitShapeTraceError,);
          },
        },),
        it({
          name: 'records gitlinks without a blob size',
          fn: async () => {
            const trace = anonymizeCommitShapes({
              commits: parseCommitShapeLog(`\u001Ecommit\0:000000 160000 ${ZERO} ${oid('e',)} A\0sub\u00000\t0\tsub\0`,),
              blobSizes: new Map(),
              source: 'fixture',
            },);
            expect(trace.commits[0]?.changes[0],).toMatchObject({ mode: 'gitlink', size: 0, },);
          },
        },),
      ],
    },),
    describe({
      name: collectBlobOids.name,
      children: [
        it({
          name: 'lists each recorded-side blob once and skips absent sides',
          fn: async () => {
            expect(
              collectBlobOids(parseCommitShapeLog(LOG_TEXT,),),
            ).toEqual([oid('a',), oid('c',), oid('d',),],);
          },
        },),
      ],
    },),
    describe({
      name: parseBlobSizes.name,
      children: [
        it({
          name: 'parses batch-check lines',
          fn: async () => {
            expect(parseBlobSizes('abc 12\ndef 0\n',),).toEqual(new Map([['abc', 12,], ['def', 0,],],),);
          },
        },),
        it({
          name: 'rejects missing-object lines',
          fn: async () => {
            expect(function parseMissing() {
              return parseBlobSizes('abc missing\n',);
            },).toThrow(CommitShapeTraceError,);
          },
        },),
      ],
    },),
    describe({
      name: shapeMode.name,
      children: [
        it({
          name: 'maps symlinks and rejects unknown modes',
          fn: async () => {
            expect(shapeMode('120000',),).toBe('symlink',);
            expect(function unknownMode() {
              return shapeMode('040000',);
            },).toThrow(CommitShapeTraceError,);
          },
        },),
      ],
    },),
    describe({
      name: serializeCommitShapeTrace.name,
      children: [
        it({
          name: 'round-trips through assertCommitShapeTrace',
          fn: async () => {
            const trace = anonymizeCommitShapes({
              commits: parseCommitShapeLog(LOG_TEXT,),
              blobSizes: SIZES,
              source: 'fixture',
            },);
            const text = serializeCommitShapeTrace(trace,);
            expect(
              assertCommitShapeTrace(JSON.parse(text,),),
            ).toEqual(trace,);
            expect(text.split('\n',).filter(function commitLine(line,) {
              return line.startsWith('    {',);
            },),).toHaveLength(3,);
          },
        },),
      ],
    },),
    describe({
      name: assertCommitShapeTrace.name,
      children: [
        it({
          name: 'rejects count mismatches and out-of-range path IDs',
          fn: async () => {
            expect(function wrongCount() {
              return assertCommitShapeTrace({ schemaVersion: 1, source: '', commitCount: 2, pathCount: 0, commits: [], },);
            },).toThrow(CommitShapeTraceError,);
            expect(function outOfRange() {
              return assertCommitShapeTrace({
                schemaVersion: 1,
                source: '',
                commitCount: 1,
                pathCount: 1,
                commits: [{ changes: [{ kind: 'add', path: 4, mode: 'file', size: 1, binary: false, added: 1, deleted: 0, },], },],
              },);
            },).toThrow(CommitShapeTraceError,);
            expect(function notObject() {
              return assertCommitShapeTrace(null,);
            },).toThrow(CommitShapeTraceError,);
          },
        },),
      ],
    },),
  ],
},);
