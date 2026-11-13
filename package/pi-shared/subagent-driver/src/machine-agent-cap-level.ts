/**
 The one place the machine's level of agents is kept: a small file of this package holding the number and the time it was set,
 read at every start, so a level raised while the drives hold reaches every session without a code change and without a restart.

 The level is a number the search for what this machine's drives carry stands at, and it moves while the search runs: four, then
 eight at 11:57 on 2026-11-13, then sixteen at 12:58:50. The record of that search,
 `doc/handover/agents-at-once-on-this-machine.md`, names this file as where the number lives and says in "The cap now" which number
 it holds; a change of the level is an edit of the number here and of that record, and nothing else has to learn the new number.

 The file sits beside the gate's own module, and the build copies it beside the built module, so a gate run from source and one run
 from `dist` read the same file. `PI_RUNS_MACHINE_CAP` still sets the cap for one command, which is how a reading is taken at a
 level before the record is changed.

 @module
 */

import { errorCodeOf, } from './error-code-of.ts';

/**
 The level file's name, beside the module that reads it.
 */
export const MACHINE_CAP_LEVEL_FILE_NAME = 'machine-agent-cap.json';

/**
 What the level file holds: how many agents the machine may carry at once, and when that number was set.
 */
export type MachineCapLevel = {
  /**
   How many agents may be going on the machine at once, this start included.
   */
  readonly cap: number;

  /**
   When the number was set, as a time `Date.parse` reads, so a reader of the counts line knows which reading the level is.
   */
  readonly setAt: string;
};

/**
 The level file could not be read, or holds no level. The message names the file and what was found there, because a missing or
 unreadable file must never become a number nobody chose.
 */
export class MachineCapLevelError extends Error {
  /**
   The name printed with the message.
   */
  override readonly name = 'MachineCapLevelError';
}

/**
 Reads one file's text, which is how the gate reads its level file and how a test gives it a level of its own.
 */
export type ReadText = (path: string) => Promise<string>;

/**
 Says what a level file holds, as the refusals of this module say it.
 */
const LEVEL_FILE_SHAPE = '{"cap": 16, "setAt": "<when it was set>"}';

/**
 Reads the text of the level file, or refuses with the file's path and the code the read gave.

 @param path - the level file's path, which a refusal names

 @param readText - how a file's text is read, which a test replaces

 @returns the text the file holds

 @throws MachineCapLevelError when the file cannot be read

 @example
 ```ts
 const listed = await textOfLevelFile({ path: '/pkg/src/machine-agent-cap.json', readText: readFile, },);
 ```
 */
async function textOfLevelFile({
  path,
  readText,
}: {
  readonly path: string;
  readonly readText: ReadText;
},): Promise<string> {
  try {
    return await readText(path,);
  }
  catch (error) {
    /**
     The code the operating system gave the read, which is empty when it gave none.
     */
    const code = errorCodeOf(error,);
    throw new MachineCapLevelError(
      `${path} could not be read${(code === '') ? '' : ` (${code})`}, so the number of agents this machine may carry is unknown and no run starts on a guess. Write it there as ${LEVEL_FILE_SHAPE} and read the level from "The cap now" in doc/handover/agents-at-once-on-this-machine.md.`,
    );
  }
}

/**
 Reads the text of the level file as JSON, or refuses with the file's path.

 @param listed - the file's text

 @param path - the level file's path, which a refusal names

 @returns what the text parses as

 @throws MachineCapLevelError when the text is no JSON

 @example
 ```ts
 const parsed = parsedLevelJson({ listed: '{"cap": 16, "setAt": "2026-11-13T12:58:50-05:00"}', path: '/pkg/src/machine-agent-cap.json', },);
 ```
 */
function parsedLevelJson({
  listed,
  path,
}: {
  readonly listed: string;
  readonly path: string;
},): unknown {
  try {
    return JSON.parse(listed,);
  }
  catch (error) {
    /**
     The code the read of a text that is no JSON gives, which says the text was not JSON and nothing more.
     */
    const code = errorCodeOf(error,);
    throw new MachineCapLevelError(
      `${path} holds no JSON${(code === '') ? '' : ` (${code})`}, so the number of agents this machine may carry is unknown and no run starts on a guess. Write it there as ${LEVEL_FILE_SHAPE}.`,
    );
  }
}

/**
 Says whether a parsed value is an object whose fields can be read by name.

 @param value - what the level file's text parsed as

 @returns whether it is such an object

 @example
 ```ts
 const fields = isLevelObject({ cap: 16, setAt: '2026-11-13T12:58:50-05:00', },);
 ```
 */
function isLevelObject(value: unknown,): value is Readonly<Record<string, unknown>> {
  return (value !== null) && ((typeof value) === 'object');
}

/**
 Reads the fields the level file holds, or refuses with the file's path and the shape a level is written in.

 @param parsed - what the level file's text parsed as

 @param path - the level file's path, which a refusal names

 @returns the object the text holds, whose fields are read one by one

 @throws MachineCapLevelError when the text is no JSON object

 @example
 ```ts
 const fields = levelFieldsOf({ parsed: JSON.parse(text,), path: '/pkg/src/machine-agent-cap.json', },);
 ```
 */
function levelFieldsOf({
  parsed,
  path,
}: {
  readonly parsed: unknown;
  readonly path: string;
},): Readonly<Record<string, unknown>> {
  if (!isLevelObject(parsed,)) {
    throw new MachineCapLevelError(`${path} holds no level object: write it there as ${LEVEL_FILE_SHAPE}.`,);
  }
  return parsed;
}

/**
 Reads one field of the level file as a whole number of agents.

 @param value - what the file holds under the field's name

 @param path - the file's path, which the refusal names

 @returns the number the field holds

 @throws MachineCapLevelError when the field is no whole number of agents a number holds exactly

 @example
 ```ts
 const cap = capOfLevelField({ value: 16, path: '/a/machine-agent-cap.json', },);
 ```
 */
function capOfLevelField({
  value,
  path,
}: {
  readonly value: unknown;
  readonly path: string;
},): number {
  if ((typeof value) !== 'number') {
    throw new MachineCapLevelError(`${path} holds no "cap": a level names how many agents the machine may carry at once, as 16.`,);
  }
  if ((!Number.isSafeInteger(value,)) || (value < 0)) {
    throw new MachineCapLevelError(`${path} holds a "cap" of ${String(value,)}, which is no whole number of agents from 0 up that a number holds exactly.`,);
  }
  return value;
}

/**
 Reads the "setAt" field of the level file: the time the number was set, which the counts line prints.

 @param value - what the file holds under the field's name

 @param path - the file's path, which a refusal names

 @returns the time, as the file wrote it

 @throws MachineCapLevelError when the field is no time `Date.parse` reads

 @example
 ```ts
 const setAt = setAtOfLevelField({ value: '2026-11-13T12:58:50-05:00', path: '/a/machine-agent-cap.json', },);
 ```
 */
function setAtOfLevelField({
  value,
  path,
}: {
  readonly value: unknown;
  readonly path: string;
},): string {
  if ((typeof value) !== 'string') {
    throw new MachineCapLevelError(`${path} holds no "setAt": a level names when the number was set, as "2026-11-13T12:58:50-05:00".`,);
  }
  if (Number.isNaN(Date.parse(value,),)) {
    throw new MachineCapLevelError(`${path} holds a "setAt" of "${value}", which is no time; write it as "2026-11-13T12:58:50-05:00".`,);
  }
  return value;
}

/**
 Reads the machine's level of agents from its own file, at every start.

 @param path - the level file's path, which a refusal names

 @param readText - how a file's text is read, which a test replaces

 @returns the level the file holds

 @throws MachineCapLevelError when the file cannot be read, holds no JSON object, or holds no level a start can count against

 @example
 ```ts
 const level = await machineCapLevelIn({ path: '/pkg/src/machine-agent-cap.json', readText: readFile, },);
 ```
 */
export async function machineCapLevelIn({
  path,
  readText,
}: {
  readonly path: string;
  readonly readText: ReadText;
},): Promise<MachineCapLevel> {
  /**
   The fields the file's text holds, each read below.
   */
  const fields = levelFieldsOf({
    parsed: parsedLevelJson({
      listed: await textOfLevelFile({
        path,
        readText,
      },),
      path,
    },),
    path,
  },);
  return {
    cap: capOfLevelField({
      value: fields.cap,
      path,
    },),
    setAt: setAtOfLevelField({
      value: fields.setAt,
      path,
    },),
  };
}
