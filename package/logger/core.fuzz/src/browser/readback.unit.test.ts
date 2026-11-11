/**
 Tests for the JSONL readback the browser property bundle uses to reparse what
 the web-storage sinks persisted.

 The readback is fuzz evidence: a batch line it cannot turn into a record means
 the store holds something the sink would not have written, and a property that
 compared a shortened list against the written records would report the wrong
 thing, so each unusable line is reported instead of dropped.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { recordsInBatch, } from './readback.ts';

//region Fixtures

/**
 Two well-formed records, as the sink serializes them.
 */
const TWO_RECORDS = [
  '{"level":"info","message":"first","timestamp":1}',
  '{"level":"warn","message":"second","timestamp":2}',
].join('\n',);

/**
 One line that parses as JSON but carries no record field.
 */
const NON_RECORD_LINE = '{"level":"info"}';

/**
 One line that is not JSON at all.
 */
const UNPARSABLE_LINE = '{"level":"info","message":';

//endregion Fixtures

await describe({
  name: '',
  children: [
    describe({
      name: recordsInBatch.name,
      children: [
        //region Well-formed batches

        it({
          name: 'answers one record per non-empty line, in line order',
          fn: async () => {
            const records = recordsInBatch({ text: `${TWO_RECORDS}\n`, },);
            expect(records,).toHaveLength(2,);
            expect(records.map(function messageOf(record,) {
              return record.message;
            },),).toEqual(['first', 'second',],);
          },
        },),

        //endregion

        //region Unusable lines are reported, never dropped

        it({
          name: 'reports a line that parses as JSON but is not a log record',
          fn: async () => {
            expect(function readBatch() {
              return recordsInBatch({ text: `${TWO_RECORDS}\n${NON_RECORD_LINE}`, },);
            },).toThrow(NON_RECORD_LINE,);
          },
        },),

        it({
          name: 'reports a line that is not JSON',
          fn: async () => {
            expect(function readBatch() {
              return recordsInBatch({ text: `${TWO_RECORDS}\n${UNPARSABLE_LINE}`, },);
            },).toThrow('JSON',);
          },
        },),

        //endregion
      ],
    },),
  ],
},);
