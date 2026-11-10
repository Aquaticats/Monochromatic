/**
 Guards that each rejection warning names the helper that emitted it.

 `AGENTS.md` rule LG1 requires a logger at every module and function boundary,
 tagged with the emitting function's name, so a reader of aggregated logs can
 tell `warnRejectedEntry` from `warnMalformedBlock` without parsing the message
 text. Both helpers are reachable only through `parseCatalogFromYaml`, so each
 test drives the path that emits the line and reads the text the console sink
 writes.

 @module
 */

import { setTimeout as delay, } from 'node:timers/promises';

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
 Milliseconds between reads of the console capture while waiting for a record.
 */
const CONSOLE_POLL_INTERVAL_MS = 10;

/**
 Milliseconds the console capture is watched before an assertion reads it.

 Measured on 2026-11-10 on this workstation: a record reaches `console.warn`
 9 to 10 milliseconds after the parse call that emits it, because the default
 logger verifies its sinks once and the console sink then drains its own
 microtask queue over a scratch log directory. The deadline keeps two orders of
 magnitude of headroom for a host whose log scratch sits on a slow disk.
 */
const CONSOLE_WRITE_DEADLINE_MS = 3_000;

/**
 Reads the console capture until it mentions `marker`, or the deadline passes.

 @param readText - returns the capture joined so far, read once per poll

 @param marker - text whose arrival ends the wait early

 @returns capture as last read, whether or not the marker arrived

 @example
 ```ts
 const text = await capturedText({ readText: () => lines.join('\n'), marker: '[warn]' });
 ```
 */
async function capturedText(
  {
    readText,
    marker,
  }: {
    readonly readText: () => string;
    readonly marker: string;
  },
): Promise<string> {
  /**
   Attempts that fit in the deadline, so a capture that never arrives ends the
   wait instead of leaving the test blocked.
   */
  const attempts = Math.ceil(CONSOLE_WRITE_DEADLINE_MS / CONSOLE_POLL_INTERVAL_MS,);
  for (let attempt = 0; attempt < attempts; attempt += 1) {
    /**
     Capture read before this poll decides whether to wait again.
     */
    const text = readText();
    if (text.includes(marker,))
      return text;
    await delay(CONSOLE_POLL_INTERVAL_MS,);
  }
  return readText();
}

//endregion Console capture

await describe({
  name: 'rejection warning tags',
  // The console sink reads `console.warn` when it drains, so one spy at a time.
  concurrency: 1,
  children: [
    //region Rejected entries

    it({
      name: `names ${REJECTED_ENTRY_TAG} when it skips an invalid package name`,
      fn: async (ctx,) => {
        /**
         Console-warn capture standing in for the console sink the parser writes to.
         */
        const consoleWarn = ctx.sinon.spy(console, 'warn',);
        /**
         Workspace text whose only entry carries a key no npm package may use.
         */
        const content = [
          'catalog:',
          "  '__proto__': '>=9.9.9'",
        ].join('\n',);
        parseCatalogFromYaml(content,);

        /**
         Captured lines, including the record this rejection emitted.
         */
        const text = await capturedText({
          marker: REJECTED_ENTRY_TAG,
          readText: () => consoleWarn.getCalls()
            .map(function firstArgument(call,) {
              return String(call.args[0],);
            },)
            .join('\n',),
        },);

        expect(text,).toContain(MODULE_TAG,);
        expect(text,).toContain(REJECTED_ENTRY_TAG,);
        expect(text,).toContain('not a valid npm package name',);
      },
    },),

    it({
      name: `names ${REJECTED_ENTRY_TAG} when it skips a non-string value`,
      fn: async (ctx,) => {
        /**
         Console-warn capture standing in for the console sink the parser writes to.
         */
        const consoleWarn = ctx.sinon.spy(console, 'warn',);
        /**
         Workspace text whose only entry carries a YAML number, not a string.
         */
        const content = [
          'catalog:',
          '  numeric: 1',
        ].join('\n',);
        parseCatalogFromYaml(content,);

        /**
         Captured lines, including the record this rejection emitted.
         */
        const text = await capturedText({
          marker: REJECTED_ENTRY_TAG,
          readText: () => consoleWarn.getCalls()
            .map(function firstArgument(call,) {
              return String(call.args[0],);
            },)
            .join('\n',),
        },);

        expect(text,).toContain(MODULE_TAG,);
        expect(text,).toContain(REJECTED_ENTRY_TAG,);
        expect(text,).toContain('catalog values must be strings',);
      },
    },),

    //endregion Rejected entries

    //region Malformed blocks

    it({
      name: `names ${MALFORMED_BLOCK_TAG} when it ignores an unreadable default block`,
      fn: async (ctx,) => {
        /**
         Console-warn capture standing in for the console sink the parser writes to.
         */
        const consoleWarn = ctx.sinon.spy(console, 'warn',);
        /**
         Workspace text declaring the default catalog as a sequence.
         */
        const content = 'catalog: []';
        parseCatalogFromYaml(content,);

        /**
         Captured lines, including the record this rejection emitted.
         */
        const text = await capturedText({
          marker: MALFORMED_BLOCK_TAG,
          readText: () => consoleWarn.getCalls()
            .map(function firstArgument(call,) {
              return String(call.args[0],);
            },)
            .join('\n',),
        },);

        expect(text,).toContain(MODULE_TAG,);
        expect(text,).toContain(MALFORMED_BLOCK_TAG,);
        expect(text,).toContain('Ignoring catalog:',);
      },
    },),

    it({
      name: `names ${MALFORMED_BLOCK_TAG} when it ignores a non-mapping workspace document`,
      fn: async (ctx,) => {
        /**
         Console-warn capture standing in for the console sink the parser writes to.
         */
        const consoleWarn = ctx.sinon.spy(console, 'warn',);
        /**
         Workspace text whose root is a scalar, so no block can be read from it.
         */
        const content = 'just-a-scalar';
        parseCatalogFromYaml(content,);

        /**
         Captured lines, including the record this rejection emitted.
         */
        const text = await capturedText({
          marker: MALFORMED_BLOCK_TAG,
          readText: () => consoleWarn.getCalls()
            .map(function firstArgument(call,) {
              return String(call.args[0],);
            },)
            .join('\n',),
        },);

        expect(text,).toContain(MODULE_TAG,);
        expect(text,).toContain(MALFORMED_BLOCK_TAG,);
        expect(text,).toContain('Ignoring workspace document:',);
      },
    },),

    //endregion Malformed blocks
  ],
},);
