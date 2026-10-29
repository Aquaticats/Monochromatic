/**
 What the package's own root logger warned, or logged at the info level,
 while one piece of work ran, for cases whose reader takes no logger and
 writes through its module's root.

 TEST SUPPORT, NOT PACKAGE SOURCE. The built package carries its own logger,
 whose console sink looks `console.warn` and `console.info` up by name at
 each write, so the divert here is what it calls. That logger is flushed
 through a root the build itself hands out before the diverted method is put
 back: a line still waiting on a sink when the divert ended would be printed
 instead of kept.

 THE LOGGER STARTS BEFORE THE DIVERT (ledger B140). It is built on a
 process's first line and verifies its sinks then, and it reports a sink it
 could not verify on `console.warn` itself, untagged. Started inside the
 divert, that report was read as one of the work's lines and refused: a file
 sink slow to verify under load put it there five seconds into a case. Flushed
 first, the logger finishes verifying before `console.warn` is replaced, and
 its report reaches the console it was written for.

 SO DOES THE TEST FRAMEWORK'S LOGGER. The build bundles its own copy of the
 logger, so a test process holds a second one: module-logger's default
 logger, which the test framework writes through and which reports its sinks
 the same way. It starts on the framework's first line, before any case, and
 a verify that ran past its time limit would report into whatever divert was
 open by then. That timing was not reproduced; flushing it here waits its
 verification out before the divert opens, whenever it started.

 A CASE CALLING THIS RUNS ONE AT A TIME: the divert is process-wide and held
 across an await, so every suite around the case sets `concurrency: 1`
 (`global-writes-sequenced.unit.test.ts`).

 @module
 */

import { logger as frameworkLogger, } from '@monochromatic-dev/logger-core/ts';

import { contextRoot, } from '../dist/final/node/index.mjs';

/**
 What the console sink writes after its level tag and after its timestamp
 tag, before the message: `[warn] [2026-10-05T00:00:00.000Z] message`.
 */
const TAG_END = '] ';

/**
 Message of one line the console sink wrote, without the level and the
 timestamp it writes first, since a case cannot know the timestamp.

 @param line - one line as the sink printed it

 @returns Everything after the timestamp tag

 @throws Error when the line carries no level and timestamp tags, which the
 console sink writes on every line: such a line is no line the package
 logged, but the logger reporting a sink that failed during the work, or a
 direct call of the diverted method

 @example
 ```ts
 messageOf({ line: '[warn] [2026-10-05T00:00:00.000Z] [whiskers] purr', },);
 // => '[whiskers] purr'
 ```
 */
function messageOf({ line, }: { readonly line: string; },): string {
  /**
   Where the level tag ends.
   */
  const levelEnd = line.indexOf(TAG_END,);
  /**
   Where the timestamp tag behind it ends.
   */
  const stampEnd = line.indexOf(
    TAG_END,
    levelEnd + TAG_END.length,
  );
  if ((levelEnd === (-1)) || (stampEnd === (-1)))
    throw new Error(
      'a diverted console method received a line without the console sink\'s level and timestamp tags, so no '
        + `line the package logged: ${line}`,
    );
  return line.slice(stampEnd + TAG_END.length,);
}

/**
 Waits out both loggers' pending lines and sink checks, so what either says
 about a sink it could not verify reaches the console it was written for.

 @example
 ```ts
 await flushBothLoggers();
 ```
 */
async function flushBothLoggers(): Promise<void> {
  await contextRoot({ tag: 'console-warn-lines', },)
    .flush();
  await frameworkLogger.flush();
}

/**
 Every line one diverted console method was handed, each from its first tag
 on.

 @param calls - each text the method was called with; the sink joins the
 lines of one turn into one call and passes that one text

 @returns Lines in the order written

 @example
 ```ts
 const lines = messagesOf({ calls, },);
 ```
 */
function messagesOf({ calls, }: { readonly calls: readonly string[]; },): readonly string[] {
  return calls.flatMap(function linesOf(call,): readonly string[] {
    return call
      .split('\n',)
      .map(function toMessage(line,): string {
        return messageOf({ line, },);
      },);
  },);
}

/**
 Runs one piece of work with `console.warn` diverted, and hands back what
 the work returned beside every warning the package's root logger wrote
 meanwhile.

 @param run - work that may warn

 @returns What the work returned, and the warnings in the order written,
 each from its first tag on

 @throws whatever the work throws, after `console.warn` is put back

 @example
 ```ts
 const { result, warned, } = await warnLinesDuring({ run: async () => readCachedLookup({ dir, query, },), },);
 ```
 */
export async function warnLinesDuring<ResultT,>(
  { run, }: { readonly run: () => Promise<ResultT>; },
): Promise<{
  readonly result: ResultT;
  readonly warned: readonly string[];
}> {
  // Both loggers verify their sinks before `console.warn` is replaced, so what
  // either says about one it could not verify is not taken for the work's.
  await flushBothLoggers();
  /**
   Each text `console.warn` was called with.
   */
  const calls: string[] = [];
  /**
   `console.warn` as it was, put back when this returns or throws.
   */
  const real = console.warn;
  console.warn = function keep(text: unknown,): void {
    calls.push(String(text,),);
  };
  /**
   Puts `console.warn` back when this scope ends, however it ends.
   */
  using _restore = {
    [Symbol.dispose]: function putBack(): void {
      console.warn = real;
    },
  };
  /**
   What the work returned.
   */
  const result = await run();
  await contextRoot({ tag: 'console-warn-lines', },)
    .flush();
  return {
    result,
    warned: messagesOf({ calls, },),
  };
}

/**
 Runs one piece of work with `console.info` diverted, and hands back what
 the work returned beside every line the package's root logger wrote at the
 info level meanwhile, for a case on what a reader logs and in which order.

 @param run - work that may log

 @returns What the work returned, and the info lines in the order written,
 each from its first tag on

 @throws whatever the work throws, after `console.info` is put back

 @example
 ```ts
 const { result, logged, } = await infoLinesDuring({ run: async () => withCitedReferences({ subjects, reader, },), },);
 ```
 */
export async function infoLinesDuring<ResultT,>(
  { run, }: { readonly run: () => Promise<ResultT>; },
): Promise<{
  readonly result: ResultT;
  readonly logged: readonly string[];
}> {
  // Both loggers verify their sinks before `console.info` is replaced, so a
  // line either had pending is printed rather than taken for the work's.
  await flushBothLoggers();
  /**
   Each text `console.info` was called with.
   */
  const calls: string[] = [];
  /**
   `console.info` as it was, put back when this returns or throws.
   */
  const real = console.info;
  console.info = function keep(text: unknown,): void {
    calls.push(String(text,),);
  };
  /**
   Puts `console.info` back when this scope ends, however it ends.
   */
  using _restore = {
    [Symbol.dispose]: function putBack(): void {
      console.info = real;
    },
  };
  /**
   What the work returned.
   */
  const result = await run();
  await contextRoot({ tag: 'console-warn-lines', },)
    .flush();
  return {
    result,
    logged: messagesOf({ calls, },),
  };
}
