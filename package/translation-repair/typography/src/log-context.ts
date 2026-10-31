import { AsyncLocalStorage, } from 'node:async_hooks';

import {
  type Logger,
  tagged,
} from '@monochromatic-dev/logger-core/ts';

import { runLogger, } from './run-log.ts';

//region Log context
// WHICH ENTRY, ATTEMPT, LANE AND SLICE A LINE BELONGS TO, carried by the async
// call rather than threaded through every stage (ledger A11, B960).
//
// The run client is built once per pass and shared by every entry, so its
// lines could never name one: 3279 of 5943 lines of TianqiChen66616.log, SPEND
// among them, carried no entry, and the lane lines carried no slice while
// slices ran side by side. A context set where an entry starts and where each
// lane takes up a slice reaches every call beneath it, the provider queues
// included (`p-limit` attaches its queued continuation in the caller's context),
// and every logger built here reads it when it writes.
//
// ABSENCE IS EMPTY, not missing: a line outside any entry, or a request that
// spans the page, carries no tag rather than a placeholder.

/**
 What a line belongs to, each part empty where it belongs to none.

 @example
 ```ts
 const context: LogContext = { entry: 'Tabby', generation: 'sha256-tree-v1:0', attempt: '2', lane: 'repair', slice: '3', };
 ```
 */
export type LogContext = {
  /**
   Entry being settled.
   */
  readonly entry: string;

  /**
   Built pipeline settling it.
   */
  readonly generation: string;

  /**
   Number of the entry's attempt as text, as the pass's attempts file holds it once the attempt is counted (ledger
   B960): the lines the logger writes for an entry name the attempt as the pass's own lines for it do.
   */
  readonly attempt: string;

  /**
   Lane working the slice: `repair`, `translate`, `contest` or `consolidation`.
   */
  readonly lane: string;

  /**
   Slice index as text.
   */
  readonly slice: string;
};

/**
 Context of work outside any entry.
 */
const NO_LOG_CONTEXT: LogContext = {
  entry: '',
  generation: '',
  attempt: '',
  lane: '',
  slice: '',
};

/**
 Context each async call carries.
 */
const store = new AsyncLocalStorage<LogContext>();

/**
 What the running call belongs to.

 @returns Context set by the nearest enclosing run, or the empty one

 @example
 ```ts
 const { entry, slice, } = currentLogContext();
 ```
 */
export function currentLogContext(): LogContext {
  return store.getStore() ?? NO_LOG_CONTEXT;
}

/**
 Runs one entry's work under its name.

 @param entry - entry being settled

 @param generation - built pipeline settling it

 @param attempt - number of the entry's attempt, whole and from 1, as the attempts file holds it once the attempt is
 counted; required so a caller cannot forget it (ledger B960)

 @param run - entry's work

 @returns What the work returned

 @example
 ```ts
 const outcome = await inEntryLogContext({ entry: 'Tabby', generation, attempt: 2, run: async () => settle(), },);
 ```
 */
export async function inEntryLogContext<ResultT,>(
  {
    entry,
    generation,
    attempt,
    run,
  }: {
    readonly entry: string;
    readonly generation: string;
    readonly attempt: number;
    readonly run: () => Promise<ResultT>;
  },
): Promise<ResultT> {
  return await store.run(
    {
      entry,
      generation,
      attempt: String(attempt,),
      lane: '',
      slice: '',
    },
    run,
  );
}

/**
 Runs one lane's work on one slice, inside the entry it belongs to.

 @param lane - lane working the slice

 @param sliceIndex - slice worked

 @param run - lane's work on it

 @returns What the work returned

 @example
 ```ts
 const settled = await inSliceLogContext({ lane: 'repair', sliceIndex: 3, run: async () => settleOne(), },);
 ```
 */
export async function inSliceLogContext<ResultT,>(
  {
    lane,
    sliceIndex,
    run,
  }: {
    readonly lane: string;
    readonly sliceIndex: number;
    readonly run: () => Promise<ResultT>;
  },
): Promise<ResultT> {
  /**
   Entry the slice belongs to.
   */
  const {
    entry,
    generation,
    attempt,
  } = currentLogContext();
  return await store.run(
    {
      entry,
      generation,
      attempt,
      lane,
      slice: String(sliceIndex,),
    },
    run,
  );
}

/**
 The attempt as one tag's text.

 @param context - context to render

 @returns `attempt 2`, or empty outside an attempt

 @example
 ```ts
 attemptText({ context: currentLogContext(), },); // 'attempt 2'
 ```
 */
function attemptText({ context, }: { readonly context: LogContext; },): string {
  return (context.attempt === '') ? '' : `attempt ${context.attempt}`;
}

/**
 Lane and slice as one tag's text.

 @param context - context to render

 @returns `repair slice 3`, `repair`, `slice 3`, or empty

 @example
 ```ts
 laneSliceText({ context: currentLogContext(), },); // 'repair slice 3'
 ```
 */
function laneSliceText({ context, }: { readonly context: LogContext; },): string {
  /**
   Slice part, empty outside a slice.
   */
  const slicePart = (context.slice === '') ? '' : `slice ${context.slice}`;
  return [
    context.lane,
    slicePart,
  ]
    .filter(function isNamed(part,): boolean {
      return part !== '';
    },)
    .join(' ',);
}

/**
 Tags as they are written before a message.

 @param texts - tag texts, empty ones skipped

 @returns `[a] [b] `, or empty

 @example
 ```ts
 bracketed({ texts: ['Tabby', 'repair slice 3',], },); // '[Tabby] [repair slice 3] '
 ```
 */
function bracketed({ texts, }: { readonly texts: readonly string[]; },): string {
  return texts
    .filter(function isNamed(text,): boolean {
      return text !== '';
    },)
    .map(function toTag(text,): string {
      return `[${text}] `;
    },)
    .join('',);
}

/**
 How a logger writes one line: what it does with the message it is given and the function that hands the line on.
 */
type LineWriter = (
  line: {
    readonly message: string;
    readonly send: (text: string,) => void;
  },
) => void;

/**
 The level methods of a logger that carry a line.
 */
type LogLevel = 'debug' | 'error' | 'fatal' | 'info' | 'trace' | 'warn';

/**
 Function that hands a line to one level of a logger.

 @param l - logger the line goes to

 @param level - level the line is written at

 @returns Function writing the text it is given at that level

 @example
 ```ts
 sendingTo({ l, level: 'info', },)('purrs',);
 ```
 */
function sendingTo(
  {
    l,
    level,
  }: {
    readonly l: Logger;
    readonly level: LogLevel;
  },
): (text: string,) => void {
  return function send(text: string,): void {
    l[level](text,);
  };
}

/**
 Logger that writes every line through one writer, at every level.

 @param l - logger the lines go to

 @param write - what the logger does with a message before it hands it on

 @returns Logger whose levels all write through `write`, and whose flush is the wrapped logger's

 @example
 ```ts
 const logger = writing({ l, write: writeSliceTagsOnce, },);
 ```
 */
function writing(
  {
    l,
    write,
  }: {
    readonly l: Logger;
    readonly write: LineWriter;
  },
): Logger {
  return {
    debug: function debug(message: string,): void {
      write({
        message,
        send: sendingTo({
          l,
          level: 'debug',
        },),
      },);
    },
    error: function error(message: string,): void {
      write({
        message,
        send: sendingTo({
          l,
          level: 'error',
        },),
      },);
    },
    fatal: function fatal(message: string,): void {
      write({
        message,
        send: sendingTo({
          l,
          level: 'fatal',
        },),
      },);
    },
    flush: async function flush(): Promise<void> {
      await l.flush();
    },
    info: function info(message: string,): void {
      write({
        message,
        send: sendingTo({
          l,
          level: 'info',
        },),
      },);
    },
    trace: function trace(message: string,): void {
      write({
        message,
        send: sendingTo({
          l,
          level: 'trace',
        },),
      },);
    },
    warn: function warn(message: string,): void {
      write({
        message,
        send: sendingTo({
          l,
          level: 'warn',
        },),
      },);
    },
  };
}

/**
 Logger writing a prefix read afresh on every line.

 @param l - logger the lines go to

 @param prefixOf - prefix for the line being written

 @returns Logger prepending that prefix

 @example
 ```ts
 const logger = prefixing({ l, prefixOf: () => '[Tabby] ', },);
 ```
 */
function prefixing(
  {
    l,
    prefixOf,
  }: {
    readonly l: Logger;
    readonly prefixOf: () => string;
  },
): Logger {
  return writing({
    l,
    write: function prefixed(
      {
        message,
        send,
      },
    ): void {
      send(`${prefixOf()}${message}`,);
    },
  },);
}

/**
 Holds `true` while a slice-tagged logger hands on a line it has put its attempt, lane and slice tags on.

 A line passes through every logger wrapped around the one that is called, so an entry logger that names the attempt
 and a lane that slice-tags the logger it is handed would each add the tags. The logger called first adds them and runs
 the rest of the line's journey under this store; each slice-tagged logger beneath it writes the line as it is.
 */
const sliceTagsWritten = new AsyncLocalStorage<true>();

/**
 Entry tag for the line being written.

 @returns `[entry] `, or empty outside an entry

 @example
 ```ts
 const prefix = entryPrefix(); // '[Tabby] '
 ```
 */
function entryPrefix(): string {
  /**
   Context the line is written in.
   */
  const { entry, } = currentLogContext();
  return bracketed({ texts: [entry,], },);
}

/**
 Attempt, lane and slice tags for the line being written.

 @returns `[attempt n] [lane slice n] `, or empty outside an entry

 @example
 ```ts
 const prefix = slicePrefix(); // '[attempt 2] [repair slice 3] '
 ```
 */
function slicePrefix(): string {
  /**
   Context the line is written in.
   */
  const context = currentLogContext();
  return bracketed({
    texts: [
      attemptText({ context, },),
      laneSliceText({ context, },),
    ],
  },);
}

/**
 Hands a line on with the attempt, lane and slice tags, unless a slice-tagged logger that was called first has put them
 on it.

 @param message - line as the logger was given it

 @param send - hands the line to the logger beneath

 @example
 ```ts
 const logger = writing({ l, write: writeSliceTagsOnce, },);
 ```
 */
function writeSliceTagsOnce(
  {
    message,
    send,
  }: {
    readonly message: string;
    readonly send: (text: string,) => void;
  },
): void {
  if (sliceTagsWritten.getStore() === true) {
    send(message,);
    return;
  }
  sliceTagsWritten.run(
    true,
    function sendTagged(): void {
      send(`${slicePrefix()}${message}`,);
    },
  );
}

/**
 An entry's logger, naming the attempt, lane and slice of every line written
 inside one; the entry is already its tag.

 A line passes through a slice-tagged logger once: where one wraps a logger that is itself built from a slice-tagged
 one (a lane tags the logger an entry hands it with its function and slice-tags that), the tags are written by the
 logger called first and the order the lanes have always written is kept (ledger B963).
 The guard holds between every slice-tagged logger and `contextRoot`, which is one.

 @param l - logger tagged with the entry

 @returns Logger writing `[entry] [attempt n] [lane slice n] message`

 @example
 ```ts
 const el = sliceTagged({ l: tagged({ tag: entry.id, },), },);
 ```
 */
export function sliceTagged({ l, }: { readonly l: Logger; },): Logger {
  return writing({
    l,
    write: writeSliceTagsOnce,
  },);
}

/**
 A module's root logger, naming the entry, attempt, lane and slice of every
 line written inside one.

 THE ROOT IS SLICE-TAGGED ITSELF. It writes the attempt, lane and slice through the same once-per-line guard as an
 entry logger and the entry in front of them, so a lane that slice-tags a logger built over a root (the default parent
 of `repairPreparedDocument`) leaves the root to write the entry alone and the line names the attempt once (ledger
 B963). Called first, the root writes all of it.

 @param tag - root tag

 @returns Logger writing `[tag] [entry] [attempt n] [lane slice n] message`

 @example
 ```ts
 const l = contextRoot({ tag: 'translation-repair', },);
 ```
 */
export function contextRoot({ tag, }: { readonly tag: string; },): Logger {
  return sliceTagged({
    l: prefixing({
      // WRITES THROUGH THE RUN LOG A PASS INSTALLED (`run-log.ts`), chosen at each call, and through the default logger
      // while none is.
      l: tagged({
        tag,
        l: runLogger,
      },),
      prefixOf: entryPrefix,
    },),
  },);
}

//endregion Log context
