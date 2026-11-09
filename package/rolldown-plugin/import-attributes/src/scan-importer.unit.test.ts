/**
 Behavior tests for {@link scanImporterForAttribute}.
 
 The scanner decides whether a dynamic import carried an attribute clause, and
 it must never fail a build: an importer it cannot read reports no attribute
 type and surfaces the cause through the module logger.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { join, } from 'node:path';
import { setTimeout as delay, } from 'node:timers/promises';

import {
  NO_ATTR_TYPE,
  scanImporterForAttribute,
} from '../dist/final/node/index.mjs';

/** Interval between logger observations, short enough to keep the case fast. */
const LOG_POLL_INTERVAL_MS = 10;

/** Deadline for one observation, generous enough for the logger's sink verification. */
const LOG_WAIT_TIMEOUT_MS = 2_000;

/**
 Polls `isObserved` until it holds or the deadline passes, so a logger that
 verifies sinks and flushes on a microtask is still observed without a fixed
 sleep.
 
 @param isObserved - predicate over the state the case waits for
 
 @returns whether the observation held before the deadline
 */
async function waitForObservation({
  isObserved,
}: {
  readonly isObserved: () => boolean;
},): Promise<boolean> {
  for (let elapsed = 0; elapsed < LOG_WAIT_TIMEOUT_MS; elapsed += LOG_POLL_INTERVAL_MS) {
    if (isObserved())
      return true;
    await delay(LOG_POLL_INTERVAL_MS,);
  }
  return isObserved();
}

await describe({
  name: '',
  children: [
    describe({
      name: scanImporterForAttribute.name,
      children: [
        it({
          name: 'reports no attribute type for an importer it cannot read',
          fn: async (ctx,) => {
            /**
             Console spy standing in for the logger's console sink.
             */
            const warn = ctx.sinon.spy(console, 'warn',);
            /**
             Path that does not exist, so the read fails with ENOENT.
             */
            const importerPath = join(import.meta.dirname, 'fixture', 'absent-importer.ts',);

            const attrType = await scanImporterForAttribute({
              specifier: './sample.sql',
              importerPath,
              importerSourceCache: new Map(),
            },);

            expect(attrType,).toBe(NO_ATTR_TYPE,);
            /**
             The warning is the observable proof that the read failure was
             reported rather than swallowed.
             */
            expect(await waitForObservation({ isObserved: () => warn.called, },),).toBe(true,);
            expect(warn.calledOnce,).toBe(true,);
            /**
             Emitted record; the logger formats it as
             `[warn] [<iso>] [<module tag>] [<function tag>] <message>`.
             */
            const logged = String(warn.firstCall.args[0],);

            expect(logged,).toContain('[import-attributes]',);
            expect(logged,).toContain(`[${scanImporterForAttribute.name}]`,);
            expect(logged,).toContain(importerPath,);
            /**
             The tag lives on the logger, never inside the message.
             */
            expect(logged,).not.toContain('import-attributes: ',);
          },
        },),
      ],
    },),
  ],
},);
