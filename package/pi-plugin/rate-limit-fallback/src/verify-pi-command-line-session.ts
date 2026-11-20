/**
 One real Pi command line against the package's stand-in members, with the package installed from a
 `packages` list the way `~/.pi/agent/settings.json` lists it.
 @module
 */

import { spawn, } from 'node:child_process';
import { once, } from 'node:events';
import {
  mkdir,
  writeFile,
} from 'node:fs/promises';
import {
  dirname,
  join,
} from 'node:path';
import { text, } from 'node:stream/consumers';
import { fileURLToPath, } from 'node:url';

//region The install this harness reproduces

/**
 Package directory Pi loads, whose `package.json` `pi.extensions` names the built entry.
 */
export const PACKAGE_DIR: string = fileURLToPath(new URL(
  '..',
  import.meta.url,
),);

/**
 Stand-in member extension the session loads from the same `packages` list.
 */
export const STAND_IN_EXTENSION: string = fileURLToPath(new URL(
  'verify-stand-in-member-extension.ts',
  import.meta.url,
),);

/**
 Model id of the writer pool's physical entry, the routing target inside the package.
 */
export const WRITER_POOL_MODEL = 'writer-pool';

/**
 Pi's own command-line entry, resolved from the installed package rather than from `PATH`.
 */
export const PI_ENTRY: string = join(
  dirname(
    fileURLToPath(import.meta.resolve('@earendil-works/pi-coding-agent',),),
  ),
  'bundle',
  'cli.js',
);

//endregion The install this harness reproduces

//region Records Pi wrote

/**
 One JSON-mode record Pi wrote.
 */
export type PiRecord = {
  readonly type: string;
  readonly message?: PiMessage;
  readonly reason?: string;
  readonly willRetry?: boolean
};

/**
 Message fields this harness reads.
 */
export type PiMessage = {
  readonly role: string;
  readonly provider?: string;
  readonly model?: string;
  readonly stopReason?: string;
  readonly errorMessage?: string
};

/**
 One complete record, with the message the predicate proved it carries.
 */
type AnsweredRecord = PiRecord & { readonly message: PiMessage; };

/**
 Characters of a malformed line a failure message quotes, enough to identify it without dumping the
 stream.

 Made up: no Pi release has been seen writing a non-record line; the quote is for the failure path.
 */
const QUOTED_LINE_CHARS = 120;

/**
 Whether one parsed line has the minimum shape every record shares.

 @param value - value a JSON line parsed to

 @returns whether it carries a record type

 @example
 ```ts
 isRecord({ type: 'agent_start' });
 ```
 */
function isRecord(value: unknown,): value is PiRecord {
  if ((typeof value) !== 'object')
    return false;
  if (value === null)
    return false;
  return Reflect.has(
    value,
    'type',
  ) && ((typeof Reflect.get(
    value,
    'type',
  )) === 'string');
}

/**
 Whether one record closes an assistant message.

 @param record - record Pi wrote

 @returns whether the record is an assistant `message_end`

 @example
 ```ts
 isAssistantEnd({ type: 'message_end', message: { role: 'assistant' } });
 ```
 */
function isAssistantEnd(record: PiRecord,): record is AnsweredRecord {
  return (record.type === 'message_end') && (record.message
    ?.role
    === 'assistant');
}

/**
 Parse the strict JSONL stream Pi writes on stdout.

 @param output - stdout bytes Pi wrote

 @returns one record per line, in order

 @example
 ```ts
 parseRecords('{"type":"agent_start"}\n');
 ```
 */
export function parseRecords(output: string,): PiRecord[] {
  /**
   Every non-empty line of the stream.
   */
  const lines = output.split('\n',)
    .filter(function nonEmpty(line,) {
      return line.trim()
        .length
        > 0;
    },);
  return lines.map(function parseLine(line,) {
    /**
     Parsed record, before its shape is trusted.
     */
    const parsed: unknown = JSON.parse(line,);
    if (!isRecord(parsed,))
      throw new Error(`Pi wrote a JSON line that is not a record: ${line.slice(
        0,
        QUOTED_LINE_CHARS,
      )}`,);
    return parsed;
  },);
}

/**
 Assistant messages one record stream reported, in order.

 @param records - Pi's JSON-mode records

 @returns every final assistant message

 @example
 ```ts
 assistantMessages(records);
 ```
 */
export function assistantMessages(records: readonly PiRecord[],): PiMessage[] {
  return records
    .filter(isAssistantEnd,)
    .map(function message(record,) {
      return record.message;
    },);
}

/**
 Record types one run emitted, for the assertions that name absent events.

 @param records - Pi's JSON-mode records

 @returns every distinct record type in first-seen order

 @example
 ```ts
 recordTypes(records);
 ```
 */
export function recordTypes(records: readonly PiRecord[],): string[] {
  /**
   Types in the order they first appeared.
   */
  const seen = new Set(records.map(function type(record,) {
    return record.type;
  },),);
  return [...seen,];
}

//endregion Records Pi wrote

//region Running one session

/**
 Write the disposable agent directory whose `packages` list is the install under test.

 @param agentDir - agent directory the session reads

 @example
 ```ts
 await installPackageList(agentDir);
 ```
 */
export async function installPackageList(agentDir: string,): Promise<void> {
  await mkdir(
    agentDir,
    { recursive: true, },
  );
  await writeFile(
    join(
      agentDir,
      'settings.json',
    ),
    // The way the owner's `~/.pi/agent/settings.json` lists this package: a directory whose
    // package.json names the built entry under `pi.extensions`.
    `${JSON.stringify(
      {
      packages: [
        PACKAGE_DIR,
        STAND_IN_EXTENSION,
      ],
      defaultProvider: 'rate-limit-fallback',
      defaultModel: 'writer',
    },
      null,
      2,
    )}\n`,
  );
}

/**
 Environment one session gets: the disposable home, offline mode, and the behaviour its members play.

 @param home - disposable home the run reads and writes

 @param agentDir - agent directory holding the install under test

 @param behaviour - behaviour every stand-in member plays

 @returns environment for the child process

 @example
 ```ts
 sessionEnvironment({ home, agentDir, behaviour: 'answer' });
 ```
 */
function sessionEnvironment({
  home,
  agentDir,
  behaviour,
}: {
  readonly home: string;
  readonly agentDir: string;
  readonly behaviour: string;
},): Record<string, string> {
  return {
    PATH: process.env
      .PATH
      ?? '',
    HOME: home,
    XDG_CONFIG_HOME: home,
    XDG_CACHE_HOME: home,
    XDG_DATA_HOME: home,
    XDG_RUNTIME_DIR: home,
    TMPDIR: home,
    PI_OFFLINE: '1',
    PI_CODING_AGENT_DIR: agentDir,
    PI_STAND_IN_BEHAVIOUR: behaviour,
  };
}

/**
 What one session recorded.
 */
export type PiRun = Readonly<{
  records: readonly PiRecord[];
  stderr: string;
  exitCode: number
}>;

/**
 Run one real Pi session and collect what it recorded.

 @param home - disposable home the session runs in

 @param behaviour - behaviour every stand-in member plays

 @param args - Pi arguments after the working directory

 @returns records, standard error and the exit code

 @example
 ```ts
 await runPiSession({ home, behaviour: 'answer', args: ['--mode', 'json'] });
 ```
 */
export async function runPiSession({
  home,
  behaviour,
  args,
}: {
  readonly home: string;
  readonly behaviour: string;
  readonly args: readonly string[];
},): Promise<PiRun> {
  /**
   Directory the session runs in, inside the disposable home.
   */
  const workDir = join(
    home,
    'work',
  );
  await mkdir(
    workDir,
    { recursive: true, },
  );
  /**
   Pi process running one prompt against the stand-in members.
   */
  const child = spawn(
    process.execPath,
    [
      PI_ENTRY,
      ...args,
    ],
    {
      cwd: workDir,
      env: sessionEnvironment({
        home,
        agentDir: join(
          home,
          'agent',
        ),
        behaviour,
      },),
      stdio: [
        'ignore',
        'pipe',
        'pipe',
      ],
    },
  );
  /**
   Exit status and both pipes, kept together so the process is drained before it is read.
   */
  const [
    ,
    stdout,
    stderr,
  ] = await Promise.all([
    once(
      child,
      'close',
    ),
    text(child.stdout,),
    text(child.stderr,),
  ],);
  return {
    records: parseRecords(stdout,),
    stderr,
    exitCode: child.exitCode ?? 1,
  };
}

//endregion Running one session
