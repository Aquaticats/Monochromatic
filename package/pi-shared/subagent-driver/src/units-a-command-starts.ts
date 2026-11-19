/**
 What a recorded command says about the units it names: the unit a command starts of its own, the unit a command merely names, and
 what the manager's answer about that unit says.

 A command started with `systemd-run --user --unit=<name>` runs in a unit of its own, which outlives the client that asked for it, so
 the number the `process` tool holds stops naming the work while the unit still runs it. A command started with anything else may
 name a unit too, as `journalctl --user --unit=<name>` does, and that unit is somebody else's work: only a program of `systemd-run`
 starts a unit this reader may keep a run for.

 The manager's answer says whether the unit's work is going and when the unit left the inactive state, and the second is what tells
 this run's unit from one another session owns under the same name.

 @module
 */

import type { RunCommand, } from './run-command.ts';

/**
 The program that starts a unit of its own, whose `--unit=` names a unit a run may be kept for.
 */
const UNIT_STARTING_PROGRAM = 'systemd-run';

/**
 The states a unit's work goes on in, as the manager answers `ActiveState`.
 */
const STATES_WHOSE_WORK_RUNS: ReadonlySet<string> = new Set<string>([
  'active',
  'activating',
  'reloading',
]);

/**
 The substate a unit carries once its work has ended, which the manager keeps the unit loaded through: a timer, or a service that
 stays after its work.
 */
const SUBSTATE_AFTER_THE_WORK = 'exited';

/**
 How many parts the timestamp form `--timestamp=us+utc` prints has: the day's name, the date, the time and the zone.
 */
const PARTS_OF_A_TIMESTAMP = 4;

/**
 A timestamp that names no moment, as a symbol so no moment can be mistaken for it: the manager writes `n/a` for a unit that never
 left the inactive state on this boot.
 */
export const NO_MOMENT: unique symbol = Symbol('no InactiveExitTimestamp value names a start',);

/**
 The manager words that ask one unit, in a single question, what state it is in and when it left the inactive state.

 `--timestamp=us+utc` is what makes the answer's timestamp a form this reader can turn into a moment: the manager's own default is a
 day and a time in the machine's zone, without the microseconds.
 */
export const QUESTION_ABOUT_A_UNIT: readonly string[] = [
  '--user',
  'show',
  '--timestamp=us+utc',
  '--property=ActiveState,SubState,InactiveExitTimestamp',
];

/**
 Says whether a character separates one word of a command from the next.

 @param character - the character to read

 @returns whether it is whitespace

 @example
 ```ts
 const separates = separatesWords('\t',);
 ```
 */
function separatesWords(character: string,): boolean {
  return (character === ' ')
    || (character === '\t')
    || (character === '\n')
    || (character === '\r')
    || (character === '\f')
    || (character === '\v');
}

/**
 Reads the words a command was written with, which whitespace of any width separates.

 One pass over the command, so a command of any length costs the time it takes to read once.

 @param command - the command the tool recorded

 @returns its words, in order, with no empty word

 @example
 ```ts
 const words = wordsOf({ command: 'systemd-run\t--user  --unit=pi-build', },);
 ```
 */
function wordsOf({ command, }: { readonly command: string },): readonly string[] {
  return (function collect(): readonly string[] {
    /**
     The words read so far, appended once each.
     */
    const words: string[] = [];
    /**
     Where the reader is, which moves forward only.
     */
    let at = 0;
    while (at < command.length) {
      if (separatesWords(command.charAt(at,),)) {
        at += 1;
        continue;
      }
      /**
       Where this word starts.
       */
      const start = at;
      while ((at < command.length) && (!separatesWords(command.charAt(at,),))) {
        at += 1;
      }
      words.push(command.slice(
        start,
        at,
      ),);
    }
    return words;
  })();
}

/**
 Says whether a character may sit inside a variable's name.

 @param character - the character to read

 @returns whether a name may hold it

 @example
 ```ts
 const allowed = isNameCharacter('_',);
 ```
 */
function isNameCharacter(character: string,): boolean {
  return (character === '_')
    || ((character >= 'a') && (character <= 'z'))
    || ((character >= 'A') && (character <= 'Z'))
    || ((character >= '0') && (character <= '9'));
}

/**
 Says whether a character may start a variable's name, which no digit may.

 @param character - the character to read

 @returns whether a name may start with it

 @example
 ```ts
 const allowed = startsAName('A',);
 ```
 */
function startsAName(character: string,): boolean {
  return (character === '_')
    || ((character >= 'a') && (character <= 'z'))
    || ((character >= 'A') && (character <= 'Z'));
}

/**
 Says whether a word assigns a variable, which a shell accepts before the program it runs, so the program word is the first that
 assigns nothing.

 @param word - one word of a command

 @returns whether it is `NAME=value`

 @example
 ```ts
 const assigns = assignsAVariable({ word: 'PI_TIMEOUT=600', },);
 ```
 */
function assignsAVariable({ word, }: { readonly word: string },): boolean {
  /**
   Where the assignment's equals sign sits, or minus one when it holds none.
   */
  const equalsAt = word.indexOf('=',);
  if ((equalsAt <= 0) || word.startsWith('-',)) {
    return false;
  }
  /**
   The name the word declares.
   */
  const name = word.slice(
    0,
    equalsAt,
  );
  if (!startsAName(name.charAt(0,),)) {
    return false;
  }
  for (const character of name) {
    if (!isNameCharacter(character,)) {
      return false;
    }
  }
  return true;
}

/**
 Reads a unit's name as the shell handed it on, with one pair of matching quotes around it taken off.

 A name is written `--unit="pi-build"` when it sits beside other words, and the process tool records the command as it was given,
 so the quotes reach this reader.

 @param word - the word that names the unit

 @returns the name itself

 @example
 ```ts
 const unit = unquoted({ word: '"pi-build"', },);
 ```
 */
function unquoted({ word, }: { readonly word: string },): string {
  /**
   The word's first character, which opens a quote when the shell wrote one.
   */
  const first = word.charAt(0,);
  /**
   Whether the word is wrapped in one pair of the same quote.
   */
  const quoted = ((first === '"') || (first === "'"))
    && (word.length >= 2)
    && word.endsWith(first,);
  return quoted ? word.slice(
    1,
    -1,
  ) : word;
}

/**
 Reads the unit a command names, when it names one.

 A command started with `systemd-run --user --unit=<name>` runs in a unit of its own, which outlives the command that asked for
 it, so the number the `process` tool holds stops naming the work while the unit still runs it.

 @param command - the command the tool started

 @returns the unit's name, or nothing when the command names none

 @example
 ```ts
 const unit = unitNamedBy({ command: 'systemd-run --user --unit=pi-build true', },);
 ```
 */
export function unitNamedBy({ command, }: { readonly command: string },): { readonly unit?: string } {
  /**
   The words the command was written with.
   */
  const words = wordsOf({ command, },);
  for (const [at, word,] of words.entries()) {
    if (word.startsWith('--unit=',)) {
      return { unit: unquoted({ word: word.slice('--unit='.length,), },), };
    }
    /**
     The word after `--unit`, which names the unit when it stands apart.
     */
    const named = words[at + 1];
    if ((word === '--unit') && (named !== undefined)) {
      return { unit: unquoted({ word: named, },), };
    }
  }
  return {};
}

/**
 Reads the unit a command starts, which is the only unit a run may be kept for.

 A command whose program is `systemd-run` starts a unit of its own; a command whose program is anything else may name a unit it
 only reads, such as `journalctl --user --unit=<name>`, and that unit is no work of the run's.

 @param command - the command the tool started

 @returns the unit the command starts, or nothing when it starts no unit

 @example
 ```ts
 const started = unitStartedBy({ command: 'cd /repo && systemd-run --user --unit=pi-build -- mise run build', },);
 ```
 */
export function unitStartedBy({ command, }: { readonly command: string },): { readonly unit?: string } {
  /**
   The words the command was written with.
   */
  const words = wordsOf({ command, },);
  /**
   Where the program word sits: the first word that assigns no variable, which a shell may put after assignments.
   */
  const programAt = words.findIndex(function isTheProgram(word,) {
    return !assignsAVariable({ word, },);
  },);
  /**
   The program word, or nothing when every word assigns a variable.
   */
  const program = words[programAt];
  if (program === undefined) {
    return {};
  }
  /**
   The program's own name, without the folder it was found in.
   */
  const programName = program.slice(program.lastIndexOf('/') + 1,);
  if (programName !== UNIT_STARTING_PROGRAM) {
    return {};
  }
  return unitNamedBy({ command, },);
}

/**
 Says whether the unit a command starts is this run's own work that still runs, asking the manager about a unit it has not settled.

 A command that starts no unit, a unit already answered about, and a unit that runs nothing or began before the command did all keep
 no run. A unit answered so is settled: the caller keeps the settled units for the run, so every later look reads that answer
 instead of asking the manager again.

 @param run - how a command is run, which a test replaces

 @param program - the program that answers for the manager

 @param command - the command the tool recorded

 @param startedAt - when the command started, in milliseconds since the epoch

 @param settled - units already answered about, which this adds to

 @returns whether the run is kept for the unit that command started

 @example
 ```ts
 const keeps = await unitKeepsTheRun({ run, program: 'systemctl', command: 'systemd-run --user --unit=pi-build -- mise run build', startedAt: Date.now(), settled: new Set(), },);
 ```
 */
export async function unitKeepsTheRun({
  run,
  program,
  command,
  startedAt,
  settled,
}: {
  readonly run: RunCommand;
  readonly program: string;
  readonly command: string;
  readonly startedAt: number;
  readonly settled: Set<string>;
},): Promise<boolean> {
  /**
   The unit the command starts, when it starts one.
   */
  const { unit, } = unitStartedBy({ command, },);
  if ((unit === undefined) || settled.has(unit,)) {
    return false;
  }
  /**
   What the manager says about that unit, in one question.
   */
  const state = unitStateIn({ shown: (await run({
    command: program,
    words: [
      ...QUESTION_ABOUT_A_UNIT,
      unit,
    ],
  },)).stdout, },);
  /**
   Whether the unit began after the command did, which is the only way a unit of that name is this run's own.
   */
  const beganAfterTheCommand = (state.startedAt !== undefined) && (state.startedAt >= startedAt);
  /**
   Whether the run is kept for it.
   */
  const keeps = state.runs && beganAfterTheCommand;
  if (!keeps) {
    settled.add(unit,);
  }
  return keeps;
}

/**
 What the manager's answer about one unit says: whether its work is going, and when the unit left the inactive state.
 */
export type UnitState = {
  /**
   Whether the unit's work is going.
   */
  readonly runs: boolean;

  /**
   When the unit last left the inactive state, as milliseconds since the epoch, or nothing when the manager named no time.
   */
  readonly startedAt?: number;
};

/**
 Reads when a timestamp the manager printed stands for, in the form `--timestamp=us+utc` prints: a day's name, the date, the time
 with microseconds, and `UTC`.

 @param shown - the value the manager printed for the timestamp

 @returns the moment, as milliseconds since the epoch, or {@link NO_MOMENT} when the value names none

 @example
 ```ts
 const at = momentIn({ shown: 'Thu 2026-11-19 02:29:45.948889 UTC', },);
 ```
 */
function momentIn({ shown, }: { readonly shown: string },): number | typeof NO_MOMENT {
  /**
   The parts of the printed timestamp.
   */
  const parts = shown.split(' ',)
    .filter(function isAPart(part,) {
      return part !== '';
    },);
  // A value that names no moment is `n/a`, empty, or a plain number: none of them has the parts the form above has.
  if (parts.length < PARTS_OF_A_TIMESTAMP) {
    return NO_MOMENT;
  }
  /**
   The parts that hold the date, the time and the zone.
   */
  const [, date = '', time = '', zone = '',] = parts;
  if (zone !== 'UTC') {
    return NO_MOMENT;
  }
  /**
   The moment the answer names.
   */
  const moment = Date.parse(`${date}T${time}Z`,);
  return Number.isFinite(moment,) ? moment : NO_MOMENT;
}

/**
 Reads what the manager's answer about one unit says.

 The three fields are the ones the question asks for: `ActiveState`, `SubState`, and `InactiveExitTimestamp`, which the manager
 writes as a realtime timestamp with microseconds of when the unit last left the inactive state, and as no time when it never did
 on this boot.

 @param shown - what the manager printed

 @returns whether the unit's work runs, and when the unit started

 @example
 ```ts
 const state = unitStateIn({ shown: 'ActiveState=active\nSubState=running\nInactiveExitTimestamp=Thu 2026-11-19 02:29:45.948889 UTC', },);
 ```
 */
export function unitStateIn({ shown, }: { readonly shown: string },): UnitState {
  /**
   Each field the answer names, by its name.
   */
  const fields = new Map<string, string>();
  for (const line of shown.split('\n',)) {
    /**
     Where this line's equals sign sits, or minus one when the line names no field.
     */
    const equalsAt = line.indexOf('=',);
    if (equalsAt <= 0) {
      continue;
    }
    fields.set(
      line.slice(
        0,
        equalsAt,
      )
        .trim(),
      line.slice(equalsAt + 1,)
        .trim(),
    );
  }
  /**
   The state the unit is in.
   */
  const active = fields.get('ActiveState') ?? '';
  /**
   The substate it is in.
   */
  const sub = fields.get('SubState') ?? '';
  /**
   Whether the manager holds the unit after its work ended, which it calls active all the same.
   */
  const heldAfterTheWork = (active === 'active') && (sub === SUBSTATE_AFTER_THE_WORK);
  /**
   When it started, which the manager leaves out when it never left the inactive state on this boot.
   */
  const startedAt = momentIn({ shown: fields.get('InactiveExitTimestamp') ?? '', },);
  return {
    runs: STATES_WHOSE_WORK_RUNS.has(active,) && (!heldAfterTheWork),
    ...((typeof startedAt) === 'symbol' ? {} : { startedAt, }),
  };
}
