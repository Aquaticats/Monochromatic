import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  createOmissionLedger,
  EXTERNAL_OMISSIONS_INVALID,
  type ExternalOmissionRecord,
  mergeExternalOmissions,
  parseExternalOmissionRecords,
} from '../dist/final/node/index.mjs';

/** Reason category a direct-summary construction failure is persisted under. */
const CONSTRUCTION_REASON = 'direct-summary-construction-failed';

/**
 Builds a valid record for one installed-package source.

 @param sourceFileName - Source the record names.

 @returns record with one omitted callable and the construction-failure reason.
 */
function record(sourceFileName: string,): ExternalOmissionRecord {
  return {
    sourceFileName,
    omittedCount: 1,
    reasons: [CONSTRUCTION_REASON,],
  };
}

/**
 Parses a candidate list as the entry of a consumer source would.

 @param value - Parsed list candidate.

 @returns parser outcome.
 */
function parseForConsumer(value: unknown,): ReturnType<typeof parseExternalOmissionRecords> {
  return parseExternalOmissionRecords({
    value,
    fileName: 'consumer.ts',
  },);
}

await describe({
  name: parseExternalOmissionRecords.name,
  concurrency: 1,
  children: [
    it({
      name: 'keeps a valid list as parsed',
      fn: async () => {
        /** Valid list naming two other sources. */
        const valid = [
          record('a.js',),
          {
            sourceFileName: 'b.js',
            omittedCount: 65_534,
            reasons: [CONSTRUCTION_REASON,],
          },
        ];
        expect(parseForConsumer(valid,),).toEqual(valid,);
        expect(parseForConsumer([],),).toEqual([],);
      },
    },),
    it({
      name: 'rejects a value that is not a list or whose list is longer than the callable bound',
      fn: async () => {
        expect(parseForConsumer(undefined,),).toBe(EXTERNAL_OMISSIONS_INVALID,);
        expect(parseForConsumer({ length: 0, },),).toBe(EXTERNAL_OMISSIONS_INVALID,);
        /** One more record than the bound admits, each naming its own source. */
        const tooMany = Array.from({ length: 65_536, }, function numberedRecord(_, index,): ExternalOmissionRecord {
          return record(`source-${String(index,)}.js`,);
        },);
        expect(parseForConsumer(tooMany,),).toBe(EXTERNAL_OMISSIONS_INVALID,);
      },
    },),
    it({
      name: 'rejects a record that is not an object or names no usable source',
      fn: async () => {
        expect(parseForConsumer(['a.js',],),).toBe(EXTERNAL_OMISSIONS_INVALID,);
        expect(parseForConsumer([null,],),).toBe(EXTERNAL_OMISSIONS_INVALID,);
        expect(parseForConsumer([{ ...record('a.js',), sourceFileName: 3, },],),).toBe(EXTERNAL_OMISSIONS_INVALID,);
        expect(parseForConsumer([{ ...record('a.js',), sourceFileName: 'x'.repeat(65_536,), },],),).toBe(
          EXTERNAL_OMISSIONS_INVALID,
        );
      },
    },),
    it({
      name: 'rejects a count that is not a positive bounded integer',
      fn: async () => {
        expect(parseForConsumer([{ ...record('a.js',), omittedCount: '1', },],),).toBe(EXTERNAL_OMISSIONS_INVALID,);
        expect(parseForConsumer([{ ...record('a.js',), omittedCount: 0, },],),).toBe(EXTERNAL_OMISSIONS_INVALID,);
        expect(parseForConsumer([{ ...record('a.js',), omittedCount: 1.5, },],),).toBe(EXTERNAL_OMISSIONS_INVALID,);
        expect(parseForConsumer([{ ...record('a.js',), omittedCount: -1, },],),).toBe(EXTERNAL_OMISSIONS_INVALID,);
        expect(parseForConsumer([{ ...record('a.js',), omittedCount: 65_535, },],),).toBe(EXTERNAL_OMISSIONS_INVALID,);
      },
    },),
    it({
      name: 'rejects reasons that are absent, empty, repeated or outside the bounded categories',
      fn: async () => {
        expect(parseForConsumer([{ ...record('a.js',), reasons: CONSTRUCTION_REASON, },],),).toBe(EXTERNAL_OMISSIONS_INVALID,);
        expect(parseForConsumer([{ ...record('a.js',), reasons: [], },],),).toBe(EXTERNAL_OMISSIONS_INVALID,);
        expect(parseForConsumer([{ ...record('a.js',), reasons: [CONSTRUCTION_REASON, CONSTRUCTION_REASON,], },],),).toBe(
          EXTERNAL_OMISSIONS_INVALID,
        );
        expect(parseForConsumer([{ ...record('a.js',), reasons: ['made-up-reason',], },],),).toBe(
          EXTERNAL_OMISSIONS_INVALID,
        );
      },
    },),
    it({
      name: 'rejects a repeated source and the entry\'s own source',
      fn: async () => {
        expect(parseForConsumer([record('a.js',), record('a.js',),],),).toBe(EXTERNAL_OMISSIONS_INVALID,);
        expect(parseForConsumer([record('consumer.ts',),],),).toBe(EXTERNAL_OMISSIONS_INVALID,);
      },
    },),
  ],
},);

await describe({
  name: mergeExternalOmissions.name,
  concurrency: 1,
  children: [
    it({
      name: 'keeps one record per source with the larger count and the union of reasons, sorted by source',
      fn: async () => {
        expect(mergeExternalOmissions([
          [
            record('b.js',),
            {
              sourceFileName: 'a.js',
              omittedCount: 2,
              reasons: [CONSTRUCTION_REASON,],
            },
          ],
          [
            {
              sourceFileName: 'a.js',
              omittedCount: 1,
              reasons: [CONSTRUCTION_REASON,],
            },
          ],
        ],),).toEqual([
          {
            sourceFileName: 'a.js',
            omittedCount: 2,
            reasons: [CONSTRUCTION_REASON,],
          },
          record('b.js',),
        ],);
        expect(mergeExternalOmissions([],),).toEqual([],);
      },
    },),
  ],
},);

await describe({
  name: createOmissionLedger.name,
  concurrency: 1,
  children: [
    it({
      name: 'names the omissions of a source and every source reached from it, and nothing else',
      fn: async () => {
        /** Ledger over a chain, one unrelated source, and a source with its own external record. */
        const ledger = createOmissionLedger();
        /** Source that lost nothing. */
        const nothingLost = {
          omittedCallableKeys: [],
          omissionReasons: [],
          externalOmissions: [],
        } as const;
        ledger.record({
          fileName: 'entry.js',
          source: nothingLost,
          dependencies: [
            'middle.js',
            'never-loaded.js',
          ],
        },);
        ledger.record({
          fileName: 'middle.js',
          source: nothingLost,
          dependencies: ['leaf.js',],
        },);
        ledger.record({
          fileName: 'leaf.js',
          source: {
            omittedCallableKeys: ['leaf.js:1:2:3',],
            omissionReasons: [CONSTRUCTION_REASON,],
            externalOmissions: [record('deeper.js',),],
          },
          dependencies: ['entry.js',],
        },);
        ledger.record({
          fileName: 'unrelated.js',
          source: {
            omittedCallableKeys: ['unrelated.js:1:2:3',],
            omissionReasons: [CONSTRUCTION_REASON,],
            externalOmissions: [],
          },
          dependencies: [],
        },);
        /** Expected records for everything the chain reaches, cycle included. */
        const chainRecords = [
          record('deeper.js',),
          record('leaf.js',),
        ];
        expect(ledger.reachedFrom('entry.js',),).toEqual(chainRecords,);
        expect(ledger.reachedFrom('middle.js',),).toEqual(chainRecords,);
        expect(ledger.reachedFrom('leaf.js',),).toEqual(chainRecords,);
        expect(ledger.reachedFrom('unrelated.js',),).toEqual([
          {
            sourceFileName: 'unrelated.js',
            omittedCount: 1,
            reasons: [CONSTRUCTION_REASON,],
          },
        ],);
        expect(ledger.reachedFrom('never-loaded.js',),).toEqual([],);
      },
    },),
  ],
},);
