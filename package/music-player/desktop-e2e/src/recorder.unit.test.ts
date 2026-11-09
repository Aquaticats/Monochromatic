/**
 Tests for the recorder: outcomes of passing, failing and skipped checks, the filter, and the safety stop.
 
 @module
 */

import {
  mkdtempDisposable,
  readdir,
  readFile,
} from 'node:fs/promises';
import { join, } from 'node:path';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';

import {
  describe,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  CheckFailureError,
  SafetyStopError,
} from './errors.ts';
import type { EvidenceState, } from './failure-evidence.ts';
import {
  createRecorder,
  type Recorder,
} from './recorder.ts';

/**
 Builds the states a session would have read, numbered from one.

 @param count - how many states
 @returns states whose fields name their own number, so a test can tell which ones a file kept
 */
function statesOf(count: number,): readonly EvidenceState[] {
  return Array.from({ length: count, }, (_unused, index,): EvidenceState => {
    const number = index + 1;
    return {
      seq: number,
      reason: `reason-${number}`,
      zone: `zone-${number}`,
      focus: `focus-${number}`,
      route: `route-${number}`,
      unfolded: (number % 2) === 0,
    };
  },);
}

/**
 Attaches a session that keeps no files of its own, for a test about its states or the failure text alone.

 @param recorder - the recorder

 @param label - session label

 @param states - what the session's log holds now
 */
function attachStates(recorder: Recorder, label: string, states: readonly EvidenceState[],): void {
  recorder.attach({
    label,
    readStates: async () => states,
    readSessionFiles: async () => [],
  },);
}

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

await describe({
  name: `${createRecorder.name} failure evidence`,
  concurrency: 1,
  children: [
    it({
      name: 'keeps the failure text and the last states, zone focus route and unfolded, in a file of the output folder',
      fn: async ({ expect, }) => {
        await using scratchDirectory = await mkdtempDisposable(join(scratchRoot(), 'e2e-evidence-',),);
        const evidenceDir = scratchDirectory.path;
        const recorder = createRecorder({ only: '', evidenceDir, },);
        attachStates(recorder, 'wide', statesOf(30,),);
        await recorder.verify({
          id: 'settings/licenses-row',
          title: 'the licenses row opens',
          body: async () => {
            throw new CheckFailureError('timed out waiting for the licenses page',);
          },
        },);
        const [outcome,] = recorder.outcomes();
        const path = outcome?.evidence ?? '';
        const text = await readFile(path, 'utf8',);
        expect(path.startsWith(evidenceDir,),).toBe(true,);
        expect(path,).toContain('settings--licenses-row',);
        expect(text,).toContain('settings/licenses-row',);
        expect(text,).toContain('timed out waiting for the licenses page',);
        expect(text,).toContain('session: wide',);
        expect(text,).toContain('seq=30 reason=reason-30 zone=zone-30 focus=focus-30 route=route-30 unfolded=true',);
        expect(text,).toContain('seq=29 reason=reason-29 zone=zone-29 focus=focus-29 route=route-29 unfolded=false',);
        expect(text,).not.toContain('zone-1 ',);
        expect(text,).not.toContain('seq=1 ',);
      },
    },),
    it({
      name: 'skips the position ticks of a playing track, which would fill the file and hide what changed, and counts them',
      fn: async ({ expect, }) => {
        await using scratchDirectory = await mkdtempDisposable(join(scratchRoot(), 'e2e-evidence-',),);
        const evidenceDir = scratchDirectory.path;
        const recorder = createRecorder({ only: '', evidenceDir, },);
        const changes = statesOf(14,).map((state,) => ({ ...state, reason: `key-${state.seq}`, }),);
        const ticks = Array.from({ length: 40, }, (_unused, index,): EvidenceState => ({
          seq: 100 + index,
          reason: 'position',
          zone: 'surface',
          focus: '',
          route: 'Player',
          unfolded: true,
        }),);
        attachStates(recorder, 'wide', [...changes, ...ticks,],);
        await recorder.verify({
          id: 'a/ticks',
          title: 'fails while a track plays',
          body: async () => {
            throw new CheckFailureError('broken',);
          },
        },);
        const text = await readFile(recorder.outcomes()[0]?.evidence ?? '', 'utf8',);
        expect(text,).toContain('seq=14 reason=key-14',);
        expect(text,).toContain('seq=5 reason=key-5',);
        expect(text,).not.toContain('seq=4 reason=key-4',);
        expect(text,).toContain('40 position ticks skipped',);
        expect(text,).toContain('the final state, seq=139',);
        expect(text,).not.toContain('seq=120 reason=position',);
      },
    },),
    it({
      name: 'keeps the session\'s own files beside the failure file and names each where it came from',
      fn: async ({ expect, }) => {
        await using scratchDirectory = await mkdtempDisposable(join(scratchRoot(), 'e2e-evidence-',),);
        const evidenceDir = scratchDirectory.path;
        const recorder = createRecorder({ only: '', evidenceDir, },);
        recorder.attach({
          label: 'wide',
          readStates: async () => statesOf(2,),
          readSessionFiles: async () => [
            { kind: 'kept', name: 'unit.log', source: '/work/run/wide.log', text: 'the app said nothing new\n', },
            { kind: 'kept', name: 'cpu.stat', source: '/sys/fs/cgroup/x.service/cpu.stat', text: 'usage_usec 4815161\n', },
          ],
        },);
        await recorder.verify({
          id: 'play/enter-plays-row',
          title: 'Enter plays the row',
          body: async () => {
            throw new CheckFailureError('timed out waiting for now-playing',);
          },
        },);
        const [outcome,] = recorder.outcomes();
        const failure = await readFile(outcome?.evidence ?? '', 'utf8',);
        const log = await readFile(join(evidenceDir, 'play--enter-plays-row.unit.log',), 'utf8',);
        const cpuStat = await readFile(join(evidenceDir, 'play--enter-plays-row.cpu.stat',), 'utf8',);
        expect(log,).toBe('the app said nothing new\n',);
        expect(cpuStat,).toBe('usage_usec 4815161\n',);
        expect(failure,).toContain('the session\'s own files, beside this one:',);
        expect(failure,).toContain('play--enter-plays-row.unit.log <- /work/run/wide.log',);
        expect(failure,).toContain('play--enter-plays-row.cpu.stat <- /sys/fs/cgroup/x.service/cpu.stat',);
      },
    },),
    it({
      name: 'says in the failure file why a file the session names was not kept',
      fn: async ({ expect, }) => {
        await using scratchDirectory = await mkdtempDisposable(join(scratchRoot(), 'e2e-evidence-',),);
        const evidenceDir = scratchDirectory.path;
        const recorder = createRecorder({ only: '', evidenceDir, },);
        recorder.attach({
          label: 'wide',
          readStates: async () => statesOf(1,),
          readSessionFiles: async () => [
            {
              kind: 'missing',
              name: 'io.pressure',
              source: 'music-player-e2e-4242-wide.service',
              reason: 'systemd names no control group for the unit',
            },
          ],
        },);
        await recorder.fail({
          id: 'session/wide',
          title: 'the wide session ran to its end',
          detail: 'the unit had already stopped',
        },);
        const failure = await readFile(recorder.outcomes()[0]?.evidence ?? '', 'utf8',);
        const files = await readdir(evidenceDir,);
        expect(files,).toEqual([
          'session--wide.failure.txt',
        ],);
        expect(failure,).toContain('session--wide.io.pressure not kept: systemd names no control group for the unit',);
      },
    },),
    it({
      name: 'keeps the states when the session cannot hand over its files',
      fn: async ({ expect, }) => {
        await using scratchDirectory = await mkdtempDisposable(join(scratchRoot(), 'e2e-evidence-',),);
        const evidenceDir = scratchDirectory.path;
        const recorder = createRecorder({ only: '', evidenceDir, },);
        recorder.attach({
          label: 'wide',
          readStates: async () => statesOf(3,),
          readSessionFiles: async () => {
            throw new Error('systemctl is not running',);
          },
        },);
        await recorder.verify({
          id: 'a/unreadable',
          title: 'fails while the files cannot be read',
          body: async () => {
            throw new CheckFailureError('the check failed',);
          },
        },);
        const failure = await readFile(recorder.outcomes()[0]?.evidence ?? '', 'utf8',);
        expect(failure,).toContain('the check failed',);
        expect(failure,).toContain('seq=3 reason=reason-3',);
        expect(failure,).not.toContain('the session\'s own files',);
      },
    },),
    it({
      name: 'records a failure outside any check, whatever the filter selects, with its evidence',
      fn: async ({ expect, }) => {
        await using scratchDirectory = await mkdtempDisposable(join(scratchRoot(), 'e2e-evidence-',),);
        const evidenceDir = scratchDirectory.path;
        const recorder = createRecorder({ only: 'settings', evidenceDir, },);
        attachStates(recorder, 'wide', statesOf(3,),);
        await recorder.fail({
          id: 'session/wide',
          title: 'the wide session ran to its end',
          detail: 'timed out waiting for the player page',
        },);
        const [outcome,] = recorder.outcomes();
        const text = await readFile(outcome?.evidence ?? '', 'utf8',);
        expect(outcome?.ok,).toBe(false,);
        expect(outcome?.id,).toBe('session/wide',);
        expect(outcome?.detail,).toBe('timed out waiting for the player page',);
        expect(text,).toContain('timed out waiting for the player page',);
        expect(text,).toContain('seq=3 reason=reason-3',);
      },
    },),
    it({
      name: 'keeps the failure text when no session was attached, and says so',
      fn: async ({ expect, }) => {
        await using scratchDirectory = await mkdtempDisposable(join(scratchRoot(), 'e2e-evidence-',),);
        const evidenceDir = scratchDirectory.path;
        const recorder = createRecorder({ only: '', evidenceDir, },);
        await recorder.verify({
          id: 'a/early',
          title: 'fails before any session',
          body: async () => {
            throw new CheckFailureError('too early',);
          },
        },);
        const text = await readFile(recorder.outcomes()[0]?.evidence ?? '', 'utf8',);
        expect(text,).toContain('too early',);
        expect(text,).toContain('no session',);
      },
    },),
    it({
      name: 'keeps the failure text when the states cannot be read, and says why',
      fn: async ({ expect, }) => {
        await using scratchDirectory = await mkdtempDisposable(join(scratchRoot(), 'e2e-evidence-',),);
        const evidenceDir = scratchDirectory.path;
        const recorder = createRecorder({ only: '', evidenceDir, },);
        recorder.attach({
          label: 'wide',
          readStates: async () => {
            throw new Error('the log file is gone',);
          },
          readSessionFiles: async () => [],
        },);
        await recorder.verify({
          id: 'a/gone',
          title: 'fails with the log gone',
          body: async () => {
            throw new CheckFailureError('the check failed',);
          },
        },);
        const text = await readFile(recorder.outcomes()[0]?.evidence ?? '', 'utf8',);
        expect(text,).toContain('the check failed',);
        expect(text,).toContain('the log file is gone',);
      },
    },),
    it({
      name: 'writes nothing for a pass, a skip or a check the filter leaves out',
      fn: async ({ expect, }) => {
        await using scratchDirectory = await mkdtempDisposable(join(scratchRoot(), 'e2e-evidence-',),);
        const evidenceDir = scratchDirectory.path;
        const recorder = createRecorder({ only: 'a/', evidenceDir, },);
        attachStates(recorder, 'wide', statesOf(3,),);
        await recorder.verify({ id: 'a/ok', title: 'passes', body: async () => undefined, },);
        recorder.skip({ id: 'a/skipped', title: 'cannot run', reason: 'no way', },);
        await recorder.verify({
          id: 'b/filtered',
          title: 'filtered out',
          body: async () => {
            throw new CheckFailureError('never runs',);
          },
        },);
        const files = await readdir(evidenceDir,);
        expect(files,).toEqual([],);
        expect(recorder.outcomes().map((outcome,) => outcome.evidence,),).toEqual([undefined, undefined,],);
      },
    },),
    it({
      name: 'reads the states of the session attached last',
      fn: async ({ expect, }) => {
        await using scratchDirectory = await mkdtempDisposable(join(scratchRoot(), 'e2e-evidence-',),);
        const evidenceDir = scratchDirectory.path;
        const recorder = createRecorder({ only: '', evidenceDir, },);
        attachStates(recorder, 'first', statesOf(2,),);
        attachStates(recorder, 'second', statesOf(1,),);
        await recorder.verify({
          id: 'a/second',
          title: 'fails in the second session',
          body: async () => {
            throw new CheckFailureError('broken',);
          },
        },);
        const text = await readFile(recorder.outcomes()[0]?.evidence ?? '', 'utf8',);
        expect(text,).toContain('session: second',);
        expect(text,).not.toContain('session: first',);
        expect(text,).not.toContain('seq=2 ',);
      },
    },),
  ],
},);
