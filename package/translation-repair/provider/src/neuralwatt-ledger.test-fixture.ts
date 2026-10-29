import type { DisposableSandbox, } from '@monochromatic-dev/test-harness-core/ts';
import { isJsonRecord, } from '@monochromatic-dev/translation-repair-document-structure/ts';

//region NeuralWatt ledger test support
// A CLOCK A CASE MOVES BY HAND AND THE LEDGER LINES IT WRITES, for the cases
// about the NeuralWatt energy ledger and the client that spends against it.
//
// TEST SUPPORT, NOT PACKAGE SOURCE: the ledger reads time through an injected
// clock, and no case reads the real clock.

/**
 Builds a clock a case moves by hand.

 @param start - ISO 8601 instant the clock starts at

 @returns Clock and way to set it

 @example
 ```ts
 const { now, setTo, } = handClock({ start: '2026-10-20T12:00:00.000Z', },);
 ```
 */
export function handClock(
  { start, }: { readonly start: string; },
): {
  readonly now: () => Date;
  readonly setTo: (instant: string,) => void;
} {
  /**
   Instant the clock reads now.
   */
  const state = { at: new Date(start,), };
  return {
    now: function now(): Date {
      return new Date(state.at,);
    },
    setTo: function setTo(instant: string,): void {
      state.at = new Date(instant,);
    },
  };
}

/**
 A day that started at 00:00 UTC on 2026-10-20, for the cases that count ledger lines and do not care where the day
 starts; `neuralwatt-day.unit.test.ts` covers where it does.
 */
export const MIDNIGHT_DAY: {
  readonly kind: 'anchored';
  readonly start: Date
} = {
  kind: 'anchored',
  start: new Date('2026-10-20T00:00:00.000Z',),
};

/**
 One ledger line as the ledger writes it.

 @param entry - fields of the line

 @returns Line with its newline

 @example
 ```ts
 await appendFile(path, lineOf({ at: '2026-10-20T01:00:00.000Z', model: 'glm-5.3-flash', wh: 1, promptTokens: 10, },),);
 ```
 */
export function lineOf(entry: Readonly<Record<string, unknown>>,): string {
  return `${JSON.stringify(entry,)}\n`;
}

/**
 One JSON object read from text, for cases that look inside a ledger line or a
 request body.

 @param text - JSON text

 @returns Object

 @throws {@link Error} when the text is not a JSON object

 @example
 ```ts
 const entry = jsonRecordOf({ text: (await readFile(path, 'utf8',)).trim(), },);
 ```
 */
export function jsonRecordOf({ text, }: { readonly text: string; },): Readonly<Record<string, unknown>> {
  /**
   Parsed text, unknown until checked.
   */
  const parsed: unknown = JSON.parse(text,);
  if (!isJsonRecord(parsed,))
    throw new Error('the fixture text is not a JSON object',);
  return parsed;
}

/**
 Lines written to the console's info and warn channels while a body runs,
 through the case's own sandbox, which answers to that case alone and puts
 both channels back when the case ends.

 @param run - body to run

 @param sinon - sandbox of the case that calls this

 @returns Lines, in order

 @example
 ```ts
 const lines = await consoleLinesDuring({ run: async () => { await ask({ client, },); }, sinon: ctx.sinon, },);
 ```
 */
export async function consoleLinesDuring(
  {
    run,
    sinon,
  }: {
    readonly run: () => Promise<void>;
    readonly sinon: DisposableSandbox;
  },
): Promise<readonly string[]> {
  /**
   Lines received.
   */
  const lines: string[] = [];

  /**
   Keeps what a channel is handed first, which is the whole line the tagged
   logger writes.

   @param line - what was logged
   */
  function keep(line: unknown,): void {
    lines.push(String(line,),);
  }
  sinon
    .stub(
    console,
    'info',
    )
    .callsFake(keep,);
  sinon
    .stub(
    console,
    'warn',
    )
    .callsFake(keep,);
  await run();
  return lines;
}

//endregion NeuralWatt ledger test support
