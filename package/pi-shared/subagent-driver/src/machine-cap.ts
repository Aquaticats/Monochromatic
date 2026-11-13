/**
 How many agents may run on this machine at once, apart from every provider pool: the cap, the agents the gate cannot see, and
 the words a count and a refusal name them with.

 A provider pool is how many runs one account answers at once, which is not the work a run does, and the pools added up allow far
 more runs than this machine's drives carry. On 2026-11-12 the home drive filled to 1 MiB free and twenty Pi runs ended within four
 minutes with `stop=error`, while every pool still read room. So a start is refused when the machine is already carrying its cap,
 whatever the pools say.

 The cap is the level a search for what this machine's drives carry stands at, and it says one thing only: not to start one more
 yet. It never stops a run that is going. A count above the cap, because another session started a run or the cap was lowered, is a
 reading at that higher count, said on the counts line as it is; the next start is refused and nothing else happens. Owner,
 2026-11-13, when a session stopped a run of its own to bring the machine back to the number: "\"Forced you stop yours\"? Didn't I
 say bisect up until there's a problem?" The canonical record of the level is `doc/handover/agents-at-once-on-this-machine.md`.

 The cap counts every agent going on the machine, never a slot of a pool: every Pi run, in a pool or in none, whoever started it,
 a run whose name begins `sol-` as much as one on a provider no pool holds, and every launcher that has not made its unit yet. Two
 runs that share a name are two agents, while a launcher and the unit it made are one, because they are one run. A caller also says
 how many agents of its own it has going which are no Pi run, and those are added, because a subagent of a session fills this
 machine's drives as a Pi run does.

 Owner, 2026-11-13: "you set your max concurrent pi+sonnet subagents starting from 4, then bisecting up until we hit this disk's
 ceiling. We'll re-find the ceiling once the new disk arrives." Asked whose runs count: "Every agent on the machine".

 @module
 */

import type { GoingRun, } from './going-runs.ts';
import type { MachineCapLevel, } from './machine-agent-cap-level.ts';
import {
  LARGEST_EXACTLY_HELD_WHOLE_NUMBER,
  wholeNumberIn,
} from './whole-number-setting.ts';

/**
 The variable that sets the cap for one command, as `<whole number of agents>`, which is how a reading is taken at a level before
 the level file and the record are changed.

 Unset, the cap is the machine's level, read from the level file at every start. Zero is a cap of its own: it starts nothing at
 all, which is how a caller pauses every start on the machine without touching a pool.
 */
export const MACHINE_CAP_VARIABLE = 'PI_RUNS_MACHINE_CAP';

/**
 The variable a caller sets to say how many agents it has going that this gate cannot see: subagents of its own session, which are
 no Pi run and so have neither a launcher nor a unit the count could find.

 Unset, the caller has none. The gate adds what it is given here to the runs it counted itself, so a coordinator that starts four
 subagents of its own and sets four here makes the next Pi run wait behind them.
 */
export const OTHER_AGENTS_GOING_VARIABLE = 'PI_RUNS_OTHER_AGENTS_GOING';

/**
 Where the cap's number came from: the machine's level file, or the one variable that sets it for one command.
 */
export type MachineCapSource =
  | {
    /**
     Whether the number came from the one variable, which is true here.
     */
    readonly from: 'the-variable';
  }
    | {
      /**
       Whether the number came from the level file, which is true here.
       */
      readonly from: 'the-level-file';

      /**
       The level file the number was read from.
       */
      readonly path: string;

      /**
       When that number was set, as the file names the time.
       */
      readonly setAt: string;
    };

/**
 A cap and where it came from, before the agents the gate cannot see are added.
 */
export type MachineCapPlace = {
  /**
   How many agents may be going on the machine at once, this run included.
   */
  readonly cap: number;

  /**
   Where that number came from.
   */
  readonly source: MachineCapSource;
};

/**
 How many agents the machine may carry at once, where that number came from, and how many of them the gate cannot see.

 @example
 ```ts
 const machineCap: MachineCap = { cap: 16, source: { from: 'the-level-file', path: '/pkg/src/machine-agent-cap.json', setAt: '2026-11-13T12:58:50-05:00', }, otherAgentsGoing: 0, };
 ```
 */
export type MachineCap = MachineCapPlace & {
  /**
   How many agents the caller says it has going which are no Pi run, so the count of runs the gate read is short by that many.
   */
  readonly otherAgentsGoing: number;
};

/**
 A cap, or a count of other agents, that this command cannot use: a value that is not a whole number.
 */
export class MachineCapError extends Error {
  /**
   The name printed with the message.
   */
  override readonly name = 'MachineCapError';
}

/**
 Reads the cap a caller set for one command, from the value of {@link MACHINE_CAP_VARIABLE}.

 A value that is no whole number is refused rather than counted, because a mistyped cap would otherwise start more agents than the
 machine carries, or stop every start, with no line saying which. An empty value is refused too: the caller reads the machine's
 level instead of passing nothing.

 @param listed - the variable's value, which must hold a whole number

 @returns the cap to count against

 @throws MachineCapError when the value is no whole number, empty included

 @example
 ```ts
 const cap = machineCapIn({ listed: '8', },);
 ```
 */
export function machineCapIn({ listed, }: { readonly listed: string },): number {
  /**
   The value as the caller wrote it, without the spaces around it.
   */
  const written = listed.trim();
  if (written === '') {
    throw new MachineCapError(
      `${MACHINE_CAP_VARIABLE} is empty, and a cap for one command is a whole number of agents. Leave it unset to take the machine's level, or set it to a number.`,
    );
  }
  /**
   What this value was read as, or why it could not be read.
   */
  const reading = wholeNumberIn(written,);
  if (!reading.read) {
    if (reading.problem === 'too-large-to-hold-exactly') {
      throw new MachineCapError(
        `${MACHINE_CAP_VARIABLE} holds "${listed}", which is a whole number too large for a number to hold exactly, so it would count as no cap at all. Set it to at most ${String(LARGEST_EXACTLY_HELD_WHOLE_NUMBER,)} agents.`,
      );
    }
    throw new MachineCapError(
      `${MACHINE_CAP_VARIABLE} holds "${listed}", which is no whole number of agents. Set it to the number of agents this machine may carry at once.`,
    );
  }
  return reading.value;
}

/**
 Reads the cap a caller set for one command, with where it came from.

 @param listed - the variable's value, which must hold a whole number

 @returns the cap, and that the one variable named it

 @throws MachineCapError when the value is no whole number of agents

 @example
 ```ts
 const place = machineCapFromVariable({ listed: '8', },);
 ```
 */
export function machineCapFromVariable({ listed, }: { readonly listed: string },): MachineCapPlace {
  return {
    cap: machineCapIn({ listed, },),
    source: { from: 'the-variable', },
  };
}

/**
 Reads the machine's level from the file that holds it, with where it came from.

 @param level - the level the file holds

 @param path - the file's path, which a counts line names

 @returns the cap, and that the level file named it

 @example
 ```ts
 const place = machineCapFromLevel({ level: { cap: 16, setAt: '2026-11-13T12:58:50-05:00', }, path: '/pkg/src/machine-agent-cap.json', },);
 ```
 */
export function machineCapFromLevel({
  level,
  path,
}: {
  readonly level: MachineCapLevel;
  readonly path: string;
},): MachineCapPlace {
  return {
    cap: level.cap,
    source: {
      from: 'the-level-file',
      path,
      setAt: level.setAt,
    },
  };
}

/**
 Says where the cap came from, as the counts line names it: the one variable, or the level file and when its number was set.

 @param source - where the cap came from

 @returns the variable's name, or the file's path and the time the file names

 @example
 ```ts
 const text = machineCapSourceTextOf({ source: { from: 'the-variable', }, },);
 ```
 */
export function machineCapSourceTextOf({ source }: { readonly source: MachineCapSource },): string {
  return (source.from === 'the-variable')
    ? MACHINE_CAP_VARIABLE
    : `${source.path} set ${source.setAt}`;
}

/**
 Reads how many other agents a caller says it has going, from the value of {@link OTHER_AGENTS_GOING_VARIABLE}.

 @param listed - the variable's value, empty when the caller set nothing

 @returns how many agents to add to the runs the gate counted, which is none when the caller said nothing

 @throws MachineCapError when the value is not a whole number

 @example
 ```ts
 const otherAgentsGoing = otherAgentsGoingIn({ listed: '0', },);
 ```
 */
export function otherAgentsGoingIn({ listed, }: { readonly listed: string },): number {
  /**
   The value as the caller wrote it, without the spaces around it.
   */
  const written = listed.trim();
  if (written === '') {
    return 0;
  }
  /**
   What this value was read as, or why it could not be read.
   */
  const reading = wholeNumberIn(written,);
  if (!reading.read) {
    if (reading.problem === 'too-large-to-hold-exactly') {
      throw new MachineCapError(
        `${OTHER_AGENTS_GOING_VARIABLE} holds "${listed}", which is a whole number too large for a number to hold exactly, so it would count as every agent there is. Set it to at most ${String(LARGEST_EXACTLY_HELD_WHOLE_NUMBER,)} agents.`,
      );
    }
    throw new MachineCapError(
      `${OTHER_AGENTS_GOING_VARIABLE} holds "${listed}", which is no whole number of agents. Set it to how many agents you have going that this gate cannot see, and leave it unset when you have none.`,
    );
  }
  return reading.value;
}

/**
 Counts the agents going on the machine now: every Pi run the gate found, and the agents its caller says it has going besides.

 @param runs - every Pi run going now, in a pool or in none, each saying how many agents it stands for

 @param otherAgentsGoing - how many agents the caller has going which are no Pi run

 @returns how many agents the machine is carrying, this run not yet among them

 @example
 ```ts
 const agentsGoing = agentsGoingOf({ runs, otherAgentsGoing: 2, },);
 ```
 */
export function agentsGoingOf({
  runs,
  otherAgentsGoing,
}: {
  readonly runs: readonly GoingRun[];
  readonly otherAgentsGoing: number;
},): number {
  /**
   Every agent the runs stand for, which is more than their number when two of them share one name.
   */
  let agents = otherAgentsGoing;
  for (const run of runs) {
    agents += run.agents;
  }
  return agents;
}

/**
 Says whether the machine is already carrying its cap, so one more agent would go over it.

 @param agentsGoing - how many agents are going on the machine now

 @param cap - how many the machine may carry at once

 @returns whether a new agent would be one too many

 @example
 ```ts
 const full = machineTakesNoMoreAgents({ agentsGoing: 16, cap: 16, },);
 ```
 */
export function machineTakesNoMoreAgents({
  agentsGoing,
  cap,
}: {
  readonly agentsGoing: number;
  readonly cap: number;
},): boolean {
  return agentsGoing >= cap;
}

/**
 Says the count against the cap and where the cap came from, as the counts line prints it beside the pools, so a coordinator reads
 how full the machine is and which level it is counting against.

 @param agentsGoing - how many agents are going on the machine now

 @param cap - how many the machine may carry at once

 @param source - where that number came from

 @returns the count and the cap in the same `<count>/<limit>` shape every pool prints, and the source of the number

 @example
 ```ts
 const text = machineCapCountsTextOf({ agentsGoing: 1, cap: 16, source: { from: 'the-variable', }, },);
 ```
 */
export function machineCapCountsTextOf({
  agentsGoing,
  cap,
  source,
}: {
  readonly agentsGoing: number;
  readonly cap: number;
  readonly source: MachineCapSource;
},): string {
  return `every agent on the machine ${String(agentsGoing,)}/${String(cap,)} from ${machineCapSourceTextOf({ source, },)}`;
}

/**
 Says why the cap refuses a start, naming the count and the cap, as the refusal names it.

 @param agentsGoing - how many agents are going on the machine now

 @param cap - how many the machine may carry at once

 @returns what the refusal says beside the run it did not start

 @example
 ```ts
 const text = machineCapRefusalTextOf({ agentsGoing: 5, cap: 16, },);
 ```
 */
export function machineCapRefusalTextOf({
  agentsGoing,
  cap,
}: {
  readonly agentsGoing: number;
  readonly cap: number;
},): string {
  /**
   The word for what is going, so a machine carrying one agent does not read as carrying "1 agents".
   */
  const agents = (agentsGoing === 1) ? 'agent' : 'agents';
  return `${String(agentsGoing,)} ${agents} going on the machine, of the ${String(cap,)} its cap allows`;
}
