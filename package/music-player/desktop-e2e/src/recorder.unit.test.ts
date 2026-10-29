/**
 Tests for the recorder: outcomes of passing, failing and skipped checks, the filter, and the safety stop.
 
 @module
 */

import {
  describe,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  CheckFailureError,
  SafetyStopError,
} from './errors.ts';
import { createRecorder, } from './recorder.ts';

await describe({
  name: createRecorder.name,
  concurrency: 1,
  children: [
    it({
      name: 'records a passing check with its detail',
      fn: async ({ expect, }) => {
        const recorder = createRecorder({ only: '', },);
        await recorder.check({ id: 'a/b', title: 'works', body: async () => 'detail text', },);
        expect(recorder.outcomes().map((outcome,) => [outcome.id, outcome.ok, outcome.skipped, outcome.detail,],),).toEqual([
          ['a/b', true, false, 'detail text',],
        ],);
      },
    },),
    it({
      name: 'records a check without detail through verify',
      fn: async ({ expect, }) => {
        const recorder = createRecorder({ only: '', },);
        await recorder.verify({ id: 'a/c', title: 'works quietly', body: async () => undefined, },);
        expect(recorder.outcomes()[0]?.detail,).toBe('',);
        expect(recorder.outcomes()[0]?.ok,).toBe(true,);
      },
    },),
    it({
      name: 'records a failing check with its message and goes on',
      fn: async ({ expect, }) => {
        const recorder = createRecorder({ only: '', },);
        await recorder.verify({
          id: 'a/fail',
          title: 'fails',
          body: async () => {
            throw new CheckFailureError('the ring is off',);
          },
        },);
        await recorder.verify({ id: 'a/after', title: 'still runs', body: async () => undefined, },);
        expect(recorder.outcomes().map((outcome,) => [outcome.id, outcome.ok,],),).toEqual([['a/fail', false,], ['a/after', true,],],);
        expect(recorder.outcomes()[0]?.detail,).toBe('the ring is off',);
      },
    },),
    it({
      name: 'records a thrown value that is not an error by its text',
      fn: async ({ expect, }) => {
        const recorder = createRecorder({ only: '', },);
        await recorder.verify({
          id: 'a/odd',
          title: 'throws text',
          body: async () => {
            const notAnError: unknown = 'plain text';
            throw notAnError;
          },
        },);
        expect(recorder.outcomes()[0]?.detail,).toBe('plain text',);
      },
    },),
    it({
      name: 'lets a safety stop end the run and records nothing for it',
      fn: async ({ expect, }) => {
        const recorder = createRecorder({ only: '', },);
        let caught: unknown;
        try {
          await recorder.verify({
            id: 'a/stop',
            title: 'plays elsewhere',
            body: async () => {
              throw new SafetyStopError('SAFETY STOP: elsewhere',);
            },
          },);
        }
        catch (error: unknown) {
          caught = error;
        }
        expect(caught,).toBeInstanceOf(SafetyStopError,);
        expect(recorder.outcomes(),).toHaveLength(0,);
      },
    },),
    it({
      name: 'records a skip as neither passed nor failed',
      fn: async ({ expect, }) => {
        const recorder = createRecorder({ only: '', },);
        recorder.skip({ id: 'k/f5', title: 'F5 rescans', reason: 'the compositor cannot inject F5', },);
        expect(recorder.outcomes(),).toEqual([
          { id: 'k/f5', title: 'F5 rescans', ok: true, skipped: true, detail: 'the compositor cannot inject F5', ms: 0, },
        ],);
      },
    },),
    it({
      name: 'applies the filter to checks and skips by id and by group',
      fn: async ({ expect, }) => {
        const recorder = createRecorder({ only: 'seek', },);
        recorder.setGroup('playback',);
        await recorder.verify({ id: 'play/space', title: 'not selected', body: async () => undefined, },);
        recorder.setGroup('seek',);
        await recorder.verify({ id: 'play/space-in-seek-group', title: 'selected by group', body: async () => undefined, },);
        recorder.setGroup('playback',);
        await recorder.verify({ id: 'seek/alt-left', title: 'selected by id', body: async () => undefined, },);
        recorder.skip({ id: 'row/menu-key', title: 'not selected skip', reason: 'x', },);
        recorder.skip({ id: 'seek/skip', title: 'selected skip', reason: 'y', },);
        expect(recorder.outcomes().map((outcome,) => outcome.id,),).toEqual([
          'play/space-in-seek-group',
          'seek/alt-left',
          'seek/skip',
        ],);
      },
    },),
  ],
},);
