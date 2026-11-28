import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { GUEST_BACKENDS, } from './mvm-guest-backend.ts';
import {
  ensureTheGuestSharedFolder,
  forgetPreparedGuests,
} from './mvm-guest-shared-folder.ts';

/**
 One guest's facts, as every step of a run names them.
 */
const CHOSEN = {
  name: 'guest-a',
  logDir: '/logs',
};

/**
 One command the step asked for, as the stand-in run recorded it.
 */
type AskedCommand = readonly string[];

/**
 A stand-in for the guest command, which records what was asked and answers what the case says.

 @param status - the status the stand-in answers with

 @returns the stand-in, and the commands it was asked to run

 @example
 ```ts
 const asked = aStandInRun(0);
 await ensureTheGuestSharedFolder({ chosen: CHOSEN, run: asked.run });
 ```
 */
function aStandInRun(status: number,): {
  readonly run: (argumentsRead: {
    readonly chosen: { readonly name: string; readonly logDir: string };
    readonly argumentWords: readonly string[];
  },) => Promise<number>;
  readonly asked: AskedCommand[];
} {
  /** The commands the step asked for, in the order it asked. */
  const asked: AskedCommand[] = [];
  return {
    asked,
    run: async function record(argsRead) {
      asked.push(argsRead.argumentWords,);
      return status;
    },
  };
}

await describe({
  name: ensureTheGuestSharedFolder.name,
  children: [
    it({
      name: 'runs nothing on a backend that mounts the folder itself',
      fn: async () => {
        forgetPreparedGuests();
        /** What the stand-in was asked, which must stay empty. */
        const standIn = aStandInRun(0,);
        await ensureTheGuestSharedFolder({
          chosen: CHOSEN,
          backend: GUEST_BACKENDS.libvirt,
          run: standIn.run,
        },);
        expect(standIn.asked,).toEqual([],);
      },
    },),
    it({
      name: 'makes the folder once on a backend without a share, owned by the account pushes run as',
      fn: async () => {
        forgetPreparedGuests();
        /** What the stand-in was asked. */
        const standIn = aStandInRun(0,);
        await ensureTheGuestSharedFolder({
          chosen: CHOSEN,
          backend: GUEST_BACKENDS.ovh,
          run: standIn.run,
        },);
        expect(standIn.asked,).toEqual([
          [
            'install',
            '--directory',
            '--owner',
            'ubuntu',
            '--group',
            'ubuntu',
            '--mode',
            '0755',
            '/mnt/shared',
          ],
        ],);
      },
    },),
    it({
      name: 'asks once for a guest, however many pushes a run makes',
      fn: async () => {
        forgetPreparedGuests();
        /** What the stand-in was asked. */
        const standIn = aStandInRun(0,);
        for (const _ of [1, 2, 3,]) {
          await ensureTheGuestSharedFolder({
            chosen: CHOSEN,
            backend: GUEST_BACKENDS.ovh,
            run: standIn.run,
          },);
        }
        expect(standIn.asked.length,).toBe(1,);
      },
    },),
    it({
      name: 'asks again for another guest',
      fn: async () => {
        forgetPreparedGuests();
        /** What the stand-in was asked. */
        const standIn = aStandInRun(0,);
        for (const name of ['guest-a', 'guest-b',]) {
          await ensureTheGuestSharedFolder({
            chosen: {
              ...CHOSEN,
              name,
            },
            backend: GUEST_BACKENDS.ovh,
            run: standIn.run,
          },);
        }
        expect(standIn.asked.length,).toBe(2,);
      },
    },),
    it({
      name: 'refuses in words naming the command and its status when the folder could not be made',
      fn: async () => {
        forgetPreparedGuests();
        /** What the stand-in was asked. */
        const standIn = aStandInRun(2,);
        /** What the refusal said. */
        let said = '';
        try {
          await ensureTheGuestSharedFolder({
            chosen: CHOSEN,
            backend: GUEST_BACKENDS.ovh,
            run: standIn.run,
          },);
        }
        catch (caught) {
          said = String(caught,);
        }
        expect(said,).toContain('exited 2',);
        expect(said,).toContain('/mnt/shared',);
        expect(said,).toContain('no pushed file could reach it',);
      },
    },),
  ],
},);
