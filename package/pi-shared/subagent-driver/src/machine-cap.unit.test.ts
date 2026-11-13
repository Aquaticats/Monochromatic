/**
 How many agents may run on this machine at once, apart from every provider pool: the cap a caller sets, the agents the gate
 cannot see, the count that adds them, and the words a counts line and a refusal name them with. Every run here is one a test
 hands over, so no test reads a process table or a unit of this machine.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  LARGEST_EXACTLY_HELD_WHOLE_NUMBER,
  MACHINE_CAP_DEFAULT,
  MACHINE_CAP_VARIABLE,
  type MachineCap,
  MachineCapError,
  NEURALWATT_DEEPSEEK_FLEX,
  OTHER_AGENTS_GOING_VARIABLE,
  type GoingRun,
  agentsGoingOf,
  machineCapCountsTextOf,
  machineCapIn,
  machineCapRefusalTextOf,
  machineTakesNoMoreAgents,
  otherAgentsGoingIn,
} from '../dist/final/node/index.mjs';

/**
 One run going now, in the pool a test names.

 @param name - the run's name

 @returns the run, holding a slot in one pool of the table

 @example
 ```ts
 const run = aRun('copy-route');
 ```
 */
function aRun(name: string,): GoingRun {
  return {
    name,
    agents: 1,
    membership: {
      holds: 'a-pool',
      pool: NEURALWATT_DEEPSEEK_FLEX,
    },
  };
}

/**
 One run going now that no pool holds, such as an audit on another account.

 @param name - the run's name

 @returns the run, holding no slot anywhere

 @example
 ```ts
 const run = aRunInNoPool('sol-audit-1');
 ```
 */
function aRunInNoPool(name: string,): GoingRun {
  return {
    name,
    agents: 1,
    membership: { holds: 'no-pool', },
  };
}

/**
 The machine's cap as the shipped default counts it, with no agent the gate cannot see going.

 @returns the cap

 @example
 ```ts
 const machineCap = theShippedCap();
 ```
 */
function theShippedCap(): MachineCap {
  return {
    cap: MACHINE_CAP_DEFAULT,
    otherAgentsGoing: 0,
  };
}

await describe({
  name: '',
  children: [
    describe({
      name: machineCapIn.name,
      children: [
        it({
          name: 'answers the cap the owner set while the caller sets none, and reads a whole number a caller sets instead',
          fn: async () => {
            expect(machineCapIn({ listed: '', },),).toBe(MACHINE_CAP_DEFAULT,);
            /**
             A cap of the test's own, above the shipped one, as the owner raises it step by step.
             */
            const raised = MACHINE_CAP_DEFAULT * 2;
            expect(machineCapIn({ listed: String(raised,), },),).toBe(raised,);
            expect(machineCapIn({ listed: ` ${String(raised,)} `, },),).toBe(raised,);
            expect(machineCapIn({ listed: '0', },),).toBe(0,);
          },
        },),
        it({
          name: 'reads the largest whole number a number holds exactly, and refuses the first one it does not and a value of hundreds of digits',
          fn: async () => {
            /**
             The largest whole number a number holds exactly, and the first one above it.
             */
            const largest = LARGEST_EXACTLY_HELD_WHOLE_NUMBER;
            const firstNotHeld = String(BigInt(largest,) + 1n,);
            expect(machineCapIn({ listed: String(largest,), },),).toBe(largest,);
            for (const listed of [firstNotHeld, '9'.repeat(String(largest,).length * 20,),]) {
              try {
                machineCapIn({ listed, },);
              }
              catch (error) {
                expect(error,).toBeInstanceOf(MachineCapError,);
                expect(String(error,),).toContain(MACHINE_CAP_VARIABLE,);
                expect(String(error,),).toContain('too large',);
                expect(String(error,),).toContain(listed,);
                continue;
              }
              throw new Error(`the cap read "${listed.slice(0, 30,)}" instead of refusing it`,);
            }
          },
        },),
        it({
          name: 'refuses a setting that is no whole number, naming the variable and the value, rather than counting it as some other cap',
          fn: async () => {
            for (const listed of ['half', '4.5', '-1', `${String(MACHINE_CAP_DEFAULT,)} agents`,]) {
              try {
                machineCapIn({ listed, },);
              }
              catch (error) {
                expect(error,).toBeInstanceOf(MachineCapError,);
                expect(String(error,),).toContain(MACHINE_CAP_VARIABLE,);
                expect(String(error,),).toContain(listed,);
                continue;
              }
              throw new Error(`the cap read "${listed}" instead of refusing it`,);
            }
          },
        },),
      ],
    },),
    describe({
      name: otherAgentsGoingIn.name,
      children: [
        it({
          name: 'answers no agent of its own while the caller says nothing, and the count a caller gives otherwise',
          fn: async () => {
            expect(otherAgentsGoingIn({ listed: '', },),).toBe(0,);
            expect(otherAgentsGoingIn({ listed: '2', },),).toBe(2,);
          },
        },),
        it({
          name: 'reads the largest whole number a number holds exactly, and refuses the first one it does not and a value of hundreds of digits',
          fn: async () => {
            /**
             The largest whole number a number holds exactly, and the first one above it.
             */
            const largest = LARGEST_EXACTLY_HELD_WHOLE_NUMBER;
            const firstNotHeld = String(BigInt(largest,) + 1n,);
            expect(otherAgentsGoingIn({ listed: String(largest,), },),).toBe(largest,);
            for (const listed of [firstNotHeld, '9'.repeat(String(largest,).length * 20,),]) {
              try {
                otherAgentsGoingIn({ listed, },);
              }
              catch (error) {
                expect(error,).toBeInstanceOf(MachineCapError,);
                expect(String(error,),).toContain(OTHER_AGENTS_GOING_VARIABLE,);
                expect(String(error,),).toContain('too large',);
                expect(String(error,),).toContain(listed,);
                continue;
              }
              throw new Error(`the count read "${listed.slice(0, 30,)}" instead of refusing it`,);
            }
          },
        },),
        it({
          name: 'refuses a setting that is no whole number, naming the variable and the value, so a mistyped count never reads as none',
          fn: async () => {
            try {
              otherAgentsGoingIn({ listed: 'two', },);
            }
            catch (error) {
              expect(error,).toBeInstanceOf(MachineCapError,);
              expect(String(error,),).toContain(OTHER_AGENTS_GOING_VARIABLE,);
              expect(String(error,),).toContain('two',);
              return;
            }
            throw new Error('a count that is no whole number was read instead of refused',);
          },
        },),
      ],
    },),
    describe({
      name: agentsGoingOf.name,
      children: [
        it({
          name: 'counts every agent the runs stand for, including one no pool holds, so two runs that share a name are two agents',
          fn: async () => {
            /**
             A run in a pool, an audit on another account, and two runs going under one name, which is how an audit name may
             go twice and what a race can leave for a moment.
             */
            const runs = [
              aRun('copy-route',),
              aRunInNoPool('sol-audit-1',),
              {
                ...aRunInNoPool('sol-audit-2',),
                agents: 2,
              },
            ];
            /**
             How many agents of its own the caller says it has going.
             */
            const otherAgentsGoing = 2;
            /**
             One agent for the run in a pool and one for the audit, two for the name that stands for two agents, and the
             caller's own agents besides.
             */
            const agentsTheRunsStandFor = 4;
            expect(agentsGoingOf({
              runs,
              otherAgentsGoing,
            },),).toBe(agentsTheRunsStandFor + otherAgentsGoing,);
            expect(agentsGoingOf({
              runs,
              otherAgentsGoing: 0,
            },),).toBe(agentsTheRunsStandFor,);
          },
        },),
      ],
    },),
    describe({
      name: machineTakesNoMoreAgents.name,
      children: [
        it({
          name: 'takes another agent under the cap, and none at the cap or over it',
          fn: async () => {
            expect(machineTakesNoMoreAgents({
              agentsGoing: MACHINE_CAP_DEFAULT - 1,
              cap: MACHINE_CAP_DEFAULT,
            },),).toBe(false,);
            expect(machineTakesNoMoreAgents({
              agentsGoing: MACHINE_CAP_DEFAULT,
              cap: MACHINE_CAP_DEFAULT,
            },),).toBe(true,);
            expect(machineTakesNoMoreAgents({
              agentsGoing: MACHINE_CAP_DEFAULT + 1,
              cap: MACHINE_CAP_DEFAULT,
            },),).toBe(true,);
          },
        },),
      ],
    },),
    describe({
      name: machineCapCountsTextOf.name,
      children: [
        it({
          name: 'says every agent of the machine with the count against the cap, in the shape every pool prints',
          fn: async () => {
            expect(machineCapCountsTextOf({
              agentsGoing: theShippedCap().otherAgentsGoing,
              cap: MACHINE_CAP_DEFAULT,
            },),).toBe(`every agent on the machine 0/${String(MACHINE_CAP_DEFAULT,)}`,);
          },
        },),
      ],
    },),
    describe({
      name: machineCapRefusalTextOf.name,
      children: [
        it({
          name: 'names the count and the cap it is counted against, saying agent for one and agents for more',
          fn: async () => {
            expect(machineCapRefusalTextOf({
              agentsGoing: 1,
              cap: MACHINE_CAP_DEFAULT,
            },),).toBe(`1 agent going on the machine, of the ${String(MACHINE_CAP_DEFAULT,)} its cap allows`,);
            /**
             A count over the cap, which is what a caller's own agents alone can make it.
             */
            const overTheCap = MACHINE_CAP_DEFAULT + 2;
            expect(machineCapRefusalTextOf({
              agentsGoing: overTheCap,
              cap: MACHINE_CAP_DEFAULT,
            },),).toBe(`${String(overTheCap,)} agents going on the machine, of the ${String(MACHINE_CAP_DEFAULT,)} its cap allows`,);
          },
        },),
      ],
    },),
  ],
},);
