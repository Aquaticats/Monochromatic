/**
 Tests for the one way a spend ledger gains a line: the line lands on the end
 of the file as JSON, the directory is made on the way, and a second line
 follows the first rather than replacing it.

 Every case writes under a disposable directory and removes it after.

 @module
 */

import { readFile, } from 'node:fs/promises';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { appendLedgerLine, } from '../dist/final/node/index.mjs';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';

await describe({
  name: appendLedgerLine.name,
  children: [
    it({
      name: 'MAKES THE DIRECTORY ON THE WAY and writes the entry as one JSON line',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'whiskers-ledger-line-', },);
        const path = join(scratch.path, 'state', 'cats', 'spend.jsonl',);
        await appendLedgerLine({ path, entry: { at: '2026-10-20T12:00:00.000Z', name: 'Whiskers', }, },);
        expect(await readFile(path, 'utf8',),).toBe('{"at":"2026-10-20T12:00:00.000Z","name":"Whiskers"}\n',);
      },
    },),

    it({
      name: 'APPENDS A SECOND LINE after the first rather than replacing it',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'whiskers-ledger-line-', },);
        const path = join(scratch.path, 'spend.jsonl',);
        await appendLedgerLine({ path, entry: { n: 1, }, },);
        await appendLedgerLine({ path, entry: { n: 2, }, },);
        expect(await readFile(path, 'utf8',),).toBe('{"n":1}\n{"n":2}\n',);
      },
    },),
  ],
},);
