/**
 Unit tests for the file mutation queue port.

 @module
 */

import { setTimeout as sleep, } from 'node:timers/promises';
import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  withFileMutationQueue,
} from '../dist/final/node/index.mjs';

//region Fixtures

/**
 Path fixture unique to the serialization test.
 */
const SERIALIZE_PATH = '/tmp/search-fetch-queue-serialize.json';

/**
 Path fixture unique to the concurrency test.
 */
const CONCURRENT_BLOCKED_PATH = '/tmp/search-fetch-queue-concurrent-blocked.json';

/**
 Second path fixture unique to the concurrency test.
 */
const CONCURRENT_OTHER_PATH = '/tmp/search-fetch-queue-concurrent-other.json';

/**
 Path fixture unique to the failure test.
 */
const FAILURE_PATH = '/tmp/search-fetch-queue-failure.json';

//endregion Fixtures

await describe({
  name: '',
  children: [
    describe({
      name: withFileMutationQueue.name,
      children: [
        it({
          name: 'serializes same-path mutations in registration order',
          fn: async () => {
            /**
             Mutation lifecycle events in observed order.
             */
            const order: string[] = [];
            /**
             First mutation holding the queue across a delay.
             */
            const first = withFileMutationQueue({
              filePath: SERIALIZE_PATH,
              mutate: async function slowFirstMutation() {
                order.push('start-1',);
                await sleep(20,);
                order.push('end-1',);
                return 'first';
              },
            },);
            /**
             Second mutation that may only start after the first ended.
             */
            const second = withFileMutationQueue({
              filePath: SERIALIZE_PATH,
              mutate: async function secondMutation() {
                order.push('start-2',);
                await sleep(1,);
                order.push('end-2',);
                return 'second';
              },
            },);

            expect(await first,).toBe('first',);
            expect(await second,).toBe('second',);
            expect(order,).toEqual([
              'start-1',
              'end-1',
              'start-2',
              'end-2',
            ],);
          },
        },),
        it({
          name: 'runs different-path mutations concurrently',
          fn: async () => {
            /**
             Gate released by the second mutation to prove overlap.
             */
            const gate = Promise.withResolvers<void>();
            /**
             Mutation lifecycle events in observed order.
             */
            const order: string[] = [];
            /**
             First mutation blocked on the gate.
             */
            const blocked = withFileMutationQueue({
              filePath: CONCURRENT_BLOCKED_PATH,
              mutate: async function blockedMutation() {
                order.push('start-blocked',);
                await Promise.race([
                  gate.promise,
                  sleep(1_000,),
                ]);
                order.push('end-blocked',);
              },
            },);
            /**
             Mutation on another path that releases the gate.
             */
            const other = withFileMutationQueue({
              filePath: CONCURRENT_OTHER_PATH,
              mutate: async function gateReleaseMutation() {
                order.push('start-other',);
                gate.resolve();
              },
            },);

            await blocked;
            await other;
            expect(order,).toEqual([
              'start-blocked',
              'start-other',
              'end-blocked',
            ],);
          },
        },),
        it({
          name: 'propagates mutation failures and keeps the queue usable',
          fn: async () => {
            /**
             Mutation failure captured instead of thrown.
             */
            const failureText = (async function runFailingMutation(): Promise<string> {
              try {
                await withFileMutationQueue({
                  filePath: FAILURE_PATH,
                  mutate: async function failingMutation() {
                    throw new Error('mutation boom',);
                  },
                },);
                return 'no failure';
              }
              catch (error: unknown) {
                return caughtValueText(error,);
              }
            })();

            expect(await failureText,).toBe('mutation boom',);
            /**
             Follow-up mutation proving the queue tail was released.
             */
            const followUp = await withFileMutationQueue({
              filePath: FAILURE_PATH,
              mutate: async function followUpMutation() {
                return 'ran after failure';
              },
            },);
            expect(followUp,).toBe('ran after failure',);
          },
        },),
      ],
    },),
  ],
},);
