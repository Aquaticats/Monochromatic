/**
 Unit tests for the backend-neutral ephemeral run against recording stand-ins for
 its backend operations: both numbers a caller asks for reach the create that
 makes the machine, a clone carries no number because it boots its source's size,
 and a call naming a source machine beside a floor is refused before any
 operation runs rather than quietly dropping the floors.

 No machine is made and no command runs: the four operations are stand-ins that
 record their arguments and answer as a backend would.

 @module
 */

import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  ephemeralRun,
  type EphemeralOps,
  type ExecResult,
} from '../../dist/final/node/index.mjs';

//region Stand-in operations

/**
 How many vCPUs the cases ask a machine to have.
 */
const ASKED_VCPUS = 4;

/**
 How much memory the cases ask a machine to have, in gibibytes.
 */
const ASKED_MEMORY_GIB = 8;

/**
 Command the cases hand the guest.
 */
const GUEST_COMMAND = 'uname -a';

/**
 Result the stand-in exec answers with, standing for a guest's own output.
 */
const EXEC_RESULT: ExecResult = {
  exitCode: 0,
  stderr: '',
  stdout: 'Linux baseline',
};

/**
 One operation's arguments, as the helper handed them over.
 */
type Recorded = {
  /**
   Arguments of every create call, in order.
   */
  readonly creates: { readonly name: string; readonly memoryGiB?: number; readonly vcpus?: number; }[];
  /**
   Arguments of every clone call, in order.
   */
  readonly clones: { readonly destination: string; readonly source: string; }[];
  /**
   Names every destroy call was given.
   */
  readonly destroys: string[];
  /**
   Commands every exec call was given, with the machine each named.
   */
  readonly execs: { readonly command: string; readonly name: string; }[];
};

/**
 Stand-in backend operations recording what the helper asks of them.
 
 @returns the operations and the record they write

 @example
 ```ts
 const { ops, recorded } = recordingOps();
 ```
 */
function recordingOps(): { readonly ops: EphemeralOps; readonly recorded: Recorded; } {
  /**
   What the operations were asked, growing as the helper calls them.
   */
  const recorded: Recorded = {
    creates: [],
    clones: [],
    destroys: [],
    execs: [],
  };
  return {
    ops: {
      create: async function create(args,) {
        recorded.creates.push(args,);
      },
      clone: async function clone(args,) {
        recorded.clones.push(args,);
      },
      destroy: async function destroy({ name, },) {
        recorded.destroys.push(name,);
      },
      exec: async function exec(args,) {
        recorded.execs.push(args,);
        return EXEC_RESULT;
      },
    },
    recorded,
  };
}

//endregion Stand-in operations

await describe({
  name: ephemeralRun.name,
  concurrency: 1,
  children: [
    it({
      name: 'hands both floors to the create that makes a fresh machine',
      fn: async () => {
        const { ops, recorded, } = recordingOps();
        const result = await ephemeralRun({
          command: GUEST_COMMAND,
          memoryGiB: ASKED_MEMORY_GIB,
          ops,
          vcpus: ASKED_VCPUS,
        },);
        expect(result,).toBe(EXEC_RESULT,);
        const [created,] = recorded.creates;
        expect(created?.memoryGiB,).toBe(ASKED_MEMORY_GIB,);
        expect(created?.vcpus,).toBe(ASKED_VCPUS,);
        // The command ran in the machine the create made, and that machine was destroyed.
        expect(recorded.execs,).toHaveLength(1,);
        expect(recorded.execs[0]?.name,).toBe(created?.name,);
        expect(recorded.destroys,).toEqual([created?.name,],);
      },
    },),
    it({
      name: 'hands a create no number when the caller asked for none',
      fn: async () => {
        const { ops, recorded, } = recordingOps();
        await ephemeralRun({
          command: GUEST_COMMAND,
          ops,
        },);
        const [created,] = recorded.creates;
        expect(created,).toBeDefined();
        // Absence travels as a missing key, which is what keeps a backend doing
        // exactly what it did before the floors existed.
        expect(Object.hasOwn(created ?? {}, 'memoryGiB',),).toBe(false,);
        expect(Object.hasOwn(created ?? {}, 'vcpus',),).toBe(false,);
      },
    },),
    it({
      name: 'clones when a source machine was named and no floor was asked',
      fn: async () => {
        const { ops, recorded, } = recordingOps();
        await ephemeralRun({
          command: GUEST_COMMAND,
          from: 'base-01',
          ops,
        },);
        expect(recorded.creates,).toHaveLength(0,);
        expect(recorded.clones,).toHaveLength(1,);
        expect(recorded.clones[0]?.source,).toBe('base-01',);
        expect(recorded.execs[0]?.name,).toBe(recorded.clones[0]?.destination,);
      },
    },),
    it({
      name: 'refuses a floor beside a source machine instead of dropping the floor',
      fn: async () => {
        const { ops, recorded, } = recordingOps();
        /**
         The refusal this call threw, when it threw one.
         */
        let caught: unknown;
        try {
          await ephemeralRun({
            command: GUEST_COMMAND,
            from: 'base-01',
            memoryGiB: ASKED_MEMORY_GIB,
            ops,
            vcpus: ASKED_VCPUS,
          },);
        }
        catch (thrown: unknown) {
          caught = thrown;
        }
        /**
         What the refusal says: the source machine it cannot resize and the floors
         it cannot honour.
         */
        const message = caughtValueText(caught,);
        expect(message,).toContain('at least 4 vCPUs',);
        expect(message,).toContain('at least 8 GiB of memory',);
        expect(message,).toContain('base-01',);
        // Nothing was created, cloned, run or destroyed: the refusal comes before the
        // first operation.
        expect(recorded.creates,).toHaveLength(0,);
        expect(recorded.clones,).toHaveLength(0,);
        expect(recorded.execs,).toHaveLength(0,);
        expect(recorded.destroys,).toHaveLength(0,);
      },
    },),
  ],
},);
