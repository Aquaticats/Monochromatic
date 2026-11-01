/**
 What a child process of production code can see of the environment its
 parent held, read from the child's own side and never from a variable
 handed to it by name.

 A case calls `inventedEnvironment` to give THIS process an invented
 credential (`WHISKER_API_KEY`), an invented setting of the package, a plain
 variable that must reach the child as the positive control, and the trace
 variables that make a real git child write the values of the names it holds
 (`GIT_TRACE2_EVENT` with `GIT_TRACE2_ENV_VARS`), then runs the site. The
 values are invented; no real key is read or written, and the trace lists
 only the names this module watches.

 @module
 */

import {
  readFile,
} from 'node:fs/promises';

/**
 Invented credential, named as every credential of the package is.
 */
const INVENTED_KEY = 'WHISKER_API_KEY';

/**
 Invented setting of the package.
 */
const INVENTED_SETTING = 'TRANSLATION_REPAIR_WHISKER_SETTING';

/**
 Invented plain variable, the positive control: a child that holds it can see.
 */
export const INVENTED_PLAIN = 'WHISKER_PLAIN_NOTE';

/**
 Names the git trace is asked to report, the repository routing variable a
 corpus child must also lose among them.
 */
const WATCHED: readonly string[] = [
  INVENTED_KEY,
  INVENTED_SETTING,
  INVENTED_PLAIN,
  'GIT_DIR',
];

/**
 Writes one variable of this process from a source, deleting it where the
 source holds none.

 @param name - variable to write

 @param source - object whose value for the name is written, absent meaning delete

 @example
 ```ts
 writeFrom({ name: 'WHISKER_PLAIN_NOTE', source: { WHISKER_PLAIN_NOTE: 'a cat naps', }, },);
 ```
 */
function writeFrom(
  {
    name,
    source,
  }: {
    readonly name: string;
    readonly source: Readonly<NodeJS.ProcessEnv>;
  },
): void {
  /**
   Value to write, absent to delete.
   */
  const value = source[name];
  if (value === undefined)
    Reflect.deleteProperty(
      process.env,
      name,
    );
  else
    process.env[name] = value;
}

/**
 Sets variables in this process and gives back a disposer that restores
 each to what stood before.

 @param values - variables to set, by name; a name set to `undefined` is
 deleted

 @returns Disposable that restores every variable it touched

 @example
 ```ts
 using held = variablesHeld({ values: { WHISKER_PLAIN_NOTE: 'a cat naps', }, },);
 ```
 */
function variablesHeld({ values, }: { readonly values: Readonly<NodeJS.ProcessEnv>; },): Disposable {
  /**
   What stood before, by name.
   */
  const before: NodeJS.ProcessEnv = {};
  for (const name of Object.keys(values,))
    before[name] = process.env[name];
  for (const name of Object.keys(values,)) {
    writeFrom({
      name,
      source: values,
    },);
  }
  return {
    [Symbol.dispose]: function restoreVariables(): void {
      for (const name of Object.keys(before,)) {
        writeFrom({
          name,
          source: before,
        },);
      }
    },
  };
}

/**
 Gives this process the invented variables a site's child is read against.

 @param tracePath - file a git child appends its trace to

 @param unset - names removed from this process for the case, such as the
 runs directory setting a function reads before it starts git

 @param extra - further variables the case sets, such as `GIT_DIR`

 @returns Disposable that restores this process's own variables

 @example
 ```ts
 using held = inventedEnvironment({ tracePath, },);
 ```
 */
export function inventedEnvironment(
  {
    tracePath,
    unset = [],
    extra = {},
  }: {
    readonly tracePath: string;
    readonly unset?: readonly string[];
    readonly extra?: Readonly<Record<string, string>>;
  },
): Disposable {
  /**
   Variables to write, the unset ones as absent.
   */
  const values: NodeJS.ProcessEnv = {};
  for (const name of unset)
    values[name] = undefined;
  return variablesHeld({
    values: {
      ...values,
      [INVENTED_KEY]: 'a cat naps',
      [INVENTED_SETTING]: 'a kitten naps',
      [INVENTED_PLAIN]: 'a tabby naps',
      GIT_TRACE2_EVENT: tracePath,
      GIT_TRACE2_ENV_VARS: WATCHED.join(',',),
      ...extra,
    },
  },);
}

/**
 The variable a trace event reports as held, empty for any other event.

 @param event - one parsed line of the trace

 @returns The variable's name, empty where the event is no report of one

 @example
 ```ts
 const name = reportedParam({ event: { event: 'def_param', param: 'WHISKER_PLAIN_NOTE', }, },);
 ```
 */
function reportedParam({ event, }: { readonly event: unknown; },): string {
  if ((typeof event) !== 'object')
    return '';
  if (event === null)
    return '';
  if (!('event' in event))
    return '';
  if (event.event !== 'def_param')
    return '';
  if (!('param' in event))
    return '';
  if ((typeof event.param) !== 'string')
    return '';
  return event.param;
}

/**
 Which watched names the git children of a case held, read from the
 children's own trace.

 @param tracePath - file the children appended their trace to

 @returns The watched names any child held, sorted and without repeats

 @throws Error when a line of the trace is not JSON, or no child wrote one

 @example
 ```ts
 const names = await namesGitHeld({ tracePath, },); // ['WHISKER_PLAIN_NOTE']
 ```
 */
export async function namesGitHeld({ tracePath, }: { readonly tracePath: string; },): Promise<readonly string[]> {
  /**
   Whole trace, one JSON event per line.
   */
  const text = await readFile(
    tracePath,
    'utf8',
  );
  /**
   Events the children wrote.
   */
  const events = text
    .split('\n',)
    .filter(function written(line,): boolean {
      return line !== '';
    },)
    .map(function parsed(line,): unknown {
      return JSON.parse(line,);
    },);
  if (events.length === 0)
    throw new Error(`unreachable: no git child wrote a trace to ${tracePath}, so the case read no child`,);
  /**
   Watched names the events report.
   */
  const names = new Set<string>();
  for (const event of events) {
    /**
     The variable the event reports, empty where it reports none.
     */
    const param = reportedParam({ event, },);
    if (WATCHED.includes(param,))
      names.add(param,);
  }
  return [...names,].toSorted();
}
