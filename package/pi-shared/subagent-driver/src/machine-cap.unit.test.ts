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
  MACHINE_CAP_VARIABLE,
  type MachineCap,
  MachineCapError,
  type MachineCapLevel,
  MachineCapLevelError,
  NEURALWATT_DEEPSEEK_FLEX,
  OTHER_AGENTS_GOING_VARIABLE,
  type GoingRun,
  agentsGoingOf,
  machineCapCountsTextOf,
  machineCapFromLevel,
  machineCapFromVariable,
  machineCapIn,
  machineCapLevelIn,
  machineCapRefusalTextOf,
  machineCapSourceTextOf,
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
 A cap of a test's own, with no agent the gate cannot see going and the one variable as its source, so no test here depends on the
 level file this machine holds.

 @returns the cap

 @example
 ```ts
 const machineCap = theCapOfATest();
 ```
 */
function theCapOfATest(): MachineCap {
  return {
    cap: 4,
    otherAgentsGoing: 0,
    source: { from: 'the-variable', },
  };
}

await describe({
  name: '',
  children: [
    describe({
      name: machineCapIn.name,
      children: [
        it({
          name: 'reads a whole number a caller sets, and refuses nothing at all, because the caller takes the machine own level instead',
          fn: async () => {
            /**
             A cap of the test's own, as the level is raised step by step.
             */
            const raised = theCapOfATest().cap * 2;
            expect(machineCapIn({ listed: String(raised,), },),).toBe(raised,);
            expect(machineCapIn({ listed: ` ${String(raised,)} `, },),).toBe(raised,);
            expect(machineCapIn({ listed: '0', },),).toBe(0,);
            try {
              machineCapIn({ listed: '', },);
            }
            catch (error) {
              expect(error,).toBeInstanceOf(MachineCapError,);
              expect(String(error,),).toContain(MACHINE_CAP_VARIABLE,);
              expect(String(error,),).toContain('empty',);
              return;
            }
            throw new Error('an empty cap was read instead of refused',);
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
            for (const listed of ['half', '4.5', '-1', `${String(theCapOfATest().cap,)} agents`,]) {
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
      name: machineCapFromVariable.name,
      children: [
        it({
          name: 'names the one variable as the source of a cap set for one command',
          fn: async () => {
            expect(machineCapFromVariable({ listed: '8', },),).toEqual({
              cap: 8,
              source: { from: 'the-variable', },
            },);
          },
        },),
      ],
    },),
    describe({
      name: machineCapFromLevel.name,
      children: [
        it({
          name: 'names the level file and the time it sets as the source of the machine own number',
          fn: async () => {
            /**
             The level a test's file holds.
             */
            const level: MachineCapLevel = {
              cap: 16,
              setAt: '2026-11-13T12:58:50-05:00',
            };
            expect(machineCapFromLevel({
              level,
              path: '/pkg/src/machine-agent-cap.json',
            },),).toEqual({
              cap: 16,
              source: {
                from: 'the-level-file',
                path: '/pkg/src/machine-agent-cap.json',
                setAt: '2026-11-13T12:58:50-05:00',
              },
            },);
          },
        },),
      ],
    },),
    describe({
      name: machineCapSourceTextOf.name,
      children: [
        it({
          name: 'says the variable where a command set the cap, and the file with its time where the machine own level did',
          fn: async () => {
            expect(machineCapSourceTextOf({ source: { from: 'the-variable', }, },),).toBe(MACHINE_CAP_VARIABLE,);
            expect(machineCapSourceTextOf({
              source: {
                from: 'the-level-file',
                path: '/pkg/src/machine-agent-cap.json',
                setAt: '2026-11-13T12:58:50-05:00',
              },
            },),).toBe('/pkg/src/machine-agent-cap.json set 2026-11-13T12:58:50-05:00',);
          },
        },),
      ],
    },),
    describe({
      name: machineCapLevelIn.name,
      children: [
        it({
          name: 'reads the number and the time a level file holds, from the text a test gives it',
          fn: async () => {
            /**
             The level file's path, which a refusal would name.
             */
            const path = '/pkg/src/machine-agent-cap.json';
            expect(await machineCapLevelIn({
              path,
              readText: async function held() {
                return '{"cap": 16, "setAt": "2026-11-13T12:58:50-05:00"}';
              },
            },),).toEqual({
              cap: 16,
              setAt: '2026-11-13T12:58:50-05:00',
            },);
          },
        },),
        it({
          name: 'refuses a level file that cannot be read, naming the path and what the read said, so no run starts on a guess',
          fn: async () => {
            /**
             The level file's path the refusal must name.
             */
            const path = '/pkg/src/machine-agent-cap.json';
            try {
              await machineCapLevelIn({
                path,
                readText: async function missing() {
                  throw Object.assign(
                    new Error('the file is not there',),
                    { code: 'ENOENT', },
                  );
                },
              },);
            }
            catch (error) {
              expect(error,).toBeInstanceOf(MachineCapLevelError,);
              expect(String(error,),).toContain(path,);
              expect(String(error,),).toContain('could not be read',);
              expect(String(error,),).toContain('ENOENT',);
              expect(String(error,),).toContain('no run starts on a guess',);
              return;
            }
            throw new Error('a level file that could not be read was counted instead of refused',);
          },
        },),
        it({
          name: 'refuses text that is no JSON, naming the path and what the parser said',
          fn: async () => {
            /**
             The level file's path the refusal must name.
             */
            const path = '/pkg/src/machine-agent-cap.json';
            try {
              await machineCapLevelIn({
                path,
                readText: async function held() {
                  return 'cap: sixteen';
                },
              },);
            }
            catch (error) {
              expect(error,).toBeInstanceOf(MachineCapLevelError,);
              expect(String(error,),).toContain(path,);
              expect(String(error,),).toContain('no JSON',);
              return;
            }
            throw new Error('text that is no JSON was read as a level instead of refused',);
          },
        },),
        it({
          name: 'refuses text that is no JSON object, naming the shape a level file has',
          fn: async () => {
            for (const held of ['null', '16',]) {
              try {
                // oxlint-disable-next-line eslint/no-await-in-loop -- one shape after another, each a text of its own; see doc/lint/sequential-awaits.md
                await machineCapLevelIn({
                  path: '/pkg/src/machine-agent-cap.json',
                  readText: async function heldText() {
                    return held;
                  },
                },);
              }
              catch (error) {
                expect(error,).toBeInstanceOf(MachineCapLevelError,);
                expect(String(error,),).toContain('no level object',);
                continue;
              }
              throw new Error(`"${held}" was read as a level instead of refused`,);
            }
          },
        },),
        it({
          name: 'refuses a cap that is no whole number of agents, naming the value and the field',
          fn: async () => {
            for (const cap of ['4.5', '-1', '"16"', 'true', '1e400',]) {
              try {
                // oxlint-disable-next-line eslint/no-await-in-loop -- one cap value after another, each a file text of its own; see doc/lint/sequential-awaits.md
                await machineCapLevelIn({
                  path: '/pkg/src/machine-agent-cap.json',
                  readText: async function heldCap() {
                    return `{"cap": ${cap}, "setAt": "2026-11-13T12:58:50-05:00"}`;
                  },
                },);
              }
              catch (error) {
                expect(error,).toBeInstanceOf(MachineCapLevelError,);
                expect(String(error,),).toContain('"cap"',);
                continue;
              }
              throw new Error(`a cap of ${cap} was read instead of refused`,);
            }
          },
        },),
        it({
          name: 'refuses a file with no time the level was set, or one no time reads, naming the field',
          fn: async () => {
            for (const setAt of ['', '"not a time"', '17',]) {
              try {
                // oxlint-disable-next-line eslint/no-await-in-loop -- one time after another, each a file text of its own; see doc/lint/sequential-awaits.md
                await machineCapLevelIn({
                  path: '/pkg/src/machine-agent-cap.json',
                  readText: async function heldAt() {
                    return `{"cap": 16, "setAt": ${((setAt === '') ? '""' : setAt)}}`;
                  },
                },);
              }
              catch (error) {
                expect(error,).toBeInstanceOf(MachineCapLevelError,);
                expect(String(error,),).toContain('"setAt"',);
                continue;
              }
              throw new Error(`a "setAt" of ${setAt} was read instead of refused`,);
            }
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
              agentsGoing: theCapOfATest().cap - 1,
              cap: theCapOfATest().cap,
            },),).toBe(false,);
            expect(machineTakesNoMoreAgents({
              agentsGoing: theCapOfATest().cap,
              cap: theCapOfATest().cap,
            },),).toBe(true,);
            expect(machineTakesNoMoreAgents({
              agentsGoing: theCapOfATest().cap + 1,
              cap: theCapOfATest().cap,
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
              agentsGoing: theCapOfATest().otherAgentsGoing,
              cap: theCapOfATest().cap,
              source: theCapOfATest().source,
            },),).toBe(`every agent on the machine 0/${String(theCapOfATest().cap,)} from ${MACHINE_CAP_VARIABLE}`,);
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
              cap: theCapOfATest().cap,
            },),).toBe(`1 agent going on the machine, of the ${String(theCapOfATest().cap,)} its cap allows`,);
            /**
             A count over the cap, which is what a caller's own agents alone can make it.
             */
            const overTheCap = theCapOfATest().cap + 2;
            expect(machineCapRefusalTextOf({
              agentsGoing: overTheCap,
              cap: theCapOfATest().cap,
            },),).toBe(`${String(overTheCap,)} agents going on the machine, of the ${String(theCapOfATest().cap,)} its cap allows`,);
          },
        },),
      ],
    },),
  ],
},);
