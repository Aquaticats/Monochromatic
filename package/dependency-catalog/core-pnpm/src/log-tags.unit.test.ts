/**
 Guards that each rejection warning names the helper that emitted it.

 `AGENTS.md` rule LG1 requires a logger at every module and function boundary,
 tagged with the emitting function's name, so a reader of aggregated logs can
 tell `warnRejectedEntry` from `warnMalformedBlock` without parsing the message
 text. Both helpers are reachable only through `parseCatalogFromYaml`, so each
 test drives the path that emits the line and reads the text the console sink
 writes.

 The console sink is the destination these tests read, so a host that sets
 `MONOCHROMATIC_LOG_CONSOLE=off` or `MONOCHROMATIC_WARN=false` makes them time
 out; the repository's own build and test tasks set neither.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { parseCatalogFromYaml, } from '../dist/final/node/index.mjs';

//region Expected tags

/**
 Tag the module logger prefixes every record with, proving the record still
 passes through the module boundary after the helper adds its own.
 */
const MODULE_TAG = '[pnpm-workspace-catalog]';

/**
 Tag `warnRejectedEntry` must add to the chain when it skips one entry.
 */
const REJECTED_ENTRY_TAG = '[warnRejectedEntry]';

/**
 Tag `warnMalformedBlock` must add to the chain when it ignores one block.
 */
const MALFORMED_BLOCK_TAG = '[warnMalformedBlock]';

//endregion Expected tags

//region Console capture

/**
 Text `warnRejectedEntry` writes around the reason, present with or without the
 helper's tag, so a failed assertion still reports the line the logger wrote.
 */
const REJECTED_ENTRY_MESSAGE = 'Rejected catalog entry';

/**
 Counterpart for `warnMalformedBlock`, present with or without its tag.
 */
const MALFORMED_BLOCK_MESSAGE = 'Ignoring catalog:';

/**
 Milliseconds a test waits for its record to reach the console before the
 harness ends it.

 Measured on 2026-11-10 on this workstation: a record reaches `console.warn` 9
 to 10 milliseconds after the parse call that emits it, because the default
 logger verifies its sinks once and the console sink then drains one microtask
 queue over a scratch log directory. The deadline keeps two orders of magnitude
 of headroom for a host whose log scratch sits on a slow disk.
 */
const CONSOLE_WRITE_TIMEOUT_MS = 3_000;

//endregion Console capture

await describe({
  name: 'rejection warning tags',
  // One console stub at a time: the sink reads `console.warn` when it drains.
  concurrency: 1,
  children: [
    //region Rejected entries

    it({
      name: `names ${REJECTED_ENTRY_TAG} on each rejection path`,
      timeout: CONSOLE_WRITE_TIMEOUT_MS,
      fn: async (ctx,) => {
        /**
         Line the first rejection resolves with, so the test ends as soon as the
         console sink writes instead of waiting out a fixed delay.
         */
        const written = Promise.withResolvers<string>();
        /**
         Every line the stub saw, so a failed assertion reports what the logger
         actually wrote.
         */
        const lines: string[] = [];
        ctx.sinon
          .stub(console, 'warn',)
          .callsFake(function captureWarning(text: unknown,): void {
            /**
             Text the console sink formatted for one record.
             */
            const line = String(text,);
            lines.push(line,);
            if (line.includes(REJECTED_ENTRY_MESSAGE,))
              written.resolve(line,);
          },);

        /**
         Workspace text whose entries are rejected for both reasons the helper
         reports: a key no npm package may use, and a value that is not a string.
         */
        const content = [
          'catalog:',
          "  '__proto__': '>=9.9.9'",
          '  numeric: 1',
        ].join('\n',);

        parseCatalogFromYaml(content,);
        await written.promise;
        /**
         Captured lines joined, including the records these rejections emitted.
         */
        const captured = lines.join('\n',);

        expect(captured,).toContain(MODULE_TAG,);
        expect(captured,).toContain(REJECTED_ENTRY_TAG,);
        expect(captured,).toContain(REJECTED_ENTRY_MESSAGE,);
        expect(captured,).toContain('catalog values must be strings',);
      },
    },),

    //endregion Rejected entries

    //region Malformed blocks

    it({
      name: `names ${MALFORMED_BLOCK_TAG} when it ignores an unreadable block`,
      timeout: CONSOLE_WRITE_TIMEOUT_MS,
      fn: async (ctx,) => {
        /**
         Line the ignored block resolves with, so the test ends when the sink
         writes rather than after a fixed delay.
         */
        const written = Promise.withResolvers<string>();
        /**
         Every line the stub saw, so a failed assertion reports what the logger
         actually wrote.
         */
        const lines: string[] = [];
        ctx.sinon
          .stub(console, 'warn',)
          .callsFake(function captureWarning(text: unknown,): void {
            /**
             Text the console sink formatted for one record.
             */
            const line = String(text,);
            lines.push(line,);
            if (line.includes(MALFORMED_BLOCK_MESSAGE,))
              written.resolve(line,);
          },);

        /**
         Workspace text declaring the default catalog as a sequence, which no
         catalog block reader can treat as a mapping.
         */
        const content = 'catalog: []';

        parseCatalogFromYaml(content,);
        await written.promise;
        /**
         Captured lines joined, including the record this rejection emitted.
         */
        const captured = lines.join('\n',);

        expect(captured,).toContain(MODULE_TAG,);
        expect(captured,).toContain(MALFORMED_BLOCK_TAG,);
        expect(captured,).toContain(MALFORMED_BLOCK_MESSAGE,);
      },
    },),

    //endregion Malformed blocks
  ],
},);
