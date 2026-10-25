import { AsyncLocalStorage, } from 'node:async_hooks';

import {
  type Logger,
  tagged,
} from '@monochromatic-dev/module-logger/ts';

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

 @param run - the entry's work

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

 @param run - the lane's work on it

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
  return {
    debug: function debug(message: string,): void {
      l.debug(`${prefixOf()}${message}`,);
    },
    error: function error(message: string,): void {
      l.error(`${prefixOf()}${message}`,);
    },
    fatal: function fatal(message: string,): void {
      l.fatal(`${prefixOf()}${message}`,);
    },
    flush: async function flush(): Promise<void> {
      await l.flush();
    },
    info: function info(message: string,): void {
      l.info(`${prefixOf()}${message}`,);
    },
    trace: function trace(message: string,): void {
      l.trace(`${prefixOf()}${message}`,);
    },
    warn: function warn(message: string,): void {
      l.warn(`${prefixOf()}${message}`,);
    },
  };
}

/**
 Entry, attempt, lane and slice tags for the line being written.

 @returns `[entry] [attempt n] [lane slice n] `, or empty outside an entry

 @example
 ```ts
 const prefix = contextPrefix(); // '[Tabby] [attempt 2] [repair slice 3] '
 ```
 */
function contextPrefix(): string {
  /**
   Context the line is written in.
   */
  const context = currentLogContext();
  return bracketed({
    texts: [
      context.entry,
      attemptText({ context, },),
      laneSliceText({ context, },),
    ],
  },);
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
 A module's root logger, naming the entry, attempt, lane and slice of every
 line written inside one.

 @param tag - root tag

 @returns Logger writing `[tag] [entry] [attempt n] [lane slice n] message`

 @example
 ```ts
 const l = contextRoot({ tag: 'translation-repair', },);
 ```
 */
export function contextRoot({ tag, }: { readonly tag: string; },): Logger {
  return prefixing({
    l: tagged({ tag, },),
    prefixOf: contextPrefix,
  },);
}

/**
 An entry's logger, naming the attempt, lane and slice of every line written
 inside one; the entry is already its tag.

 @param l - logger tagged with the entry

 @returns Logger writing `[entry] [attempt n] [lane slice n] message`

 @example
 ```ts
 const el = sliceTagged({ l: tagged({ tag: entry.id, },), },);
 ```
 */
export function sliceTagged({ l, }: { readonly l: Logger; },): Logger {
  return prefixing({
    l,
    prefixOf: slicePrefix,
  },);
}

//endregion Log context
