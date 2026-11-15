/**
 Copies Pi's standard error to a file for as long as Pi writes it, masking it on the way.

 Pi's diagnostics are written by Pi and by the extensions it loads, and a line of them can hold a value a tool printed: this is the
 one place that text passes, so it is masked before the file holds it. The copy holds back the last characters a value's longest
 form could need, so a value split across two writes is still matched, and writes what it held back when Pi's error ends.

 @module
 */

import { createWriteStream, } from 'node:fs';
import type { Readable, } from 'node:stream';
import {
  MASKING_NOTHING,
  type TextMasking,
} from './secret-value-masking.ts';

/**
 Copies Pi's standard error to a file, reporting a failure rather than throwing it, so the copy can run beside the run itself.

 @param source - Pi's standard error

 @param file - file that receives it

 @param onProblem - where to report a failure

 @param append - whether the copy is added to what the file holds, which a run whose Pi is started again needs so no start's
 lines replace another's

 @param mask - the masking every write goes through

 @example
 ```ts
 const kept = keepStandardError({ source: child.stderr, file: 'stderr.log', onProblem: report, },);
 ```
 */
export async function keepStandardError(
  {
    source,
    file,
    onProblem,
    append = false,
    mask = MASKING_NOTHING,
  }: {
    readonly source: Readable;
    readonly file: string;
    readonly onProblem: (problem: string,) => void;
    readonly append?: boolean;
    readonly mask?: TextMasking;
  },
): Promise<void> {
  /**
   How many characters the copy holds back between two writes: one fewer than the longest form a value is printed in.
   */
  const holdBack = Math.max(
    0,
    mask.longestFormLength - 1,
  );
  /**
   The characters of the last write that are held back, which the next write completes.
   */
  const carry: { held: string } = { held: '', };
  /**
   Where the masked bytes go.
   */
  const sink = createWriteStream(
    file,
    { flags: append ? 'a' : 'w', },
  );
  try {
    source.setEncoding('utf8',);
    for await (const chunk of source as AsyncIterable<string>) {
      /**
       The text this write is about, which is what was held back and what arrived.
       */
      const text = carry.held + chunk;
      /**
       How much of it is held back for the next write.
       */
      const keep = Math.min(
        holdBack,
        text.length,
      );
      carry.held = text.slice(text.length - keep,);
      sink.write(
        mask.maskText(text.slice(
          0,
          text.length - keep,
        ),),
      );
    }
    if (carry.held !== '') {
      sink.write(mask.maskText(carry.held,),);
      carry.held = '';
    }
    // A write stream ends through a callback, which is the one shape Node gives for waiting on it.
    /* oxlint-disable-next-line promise/avoid-new -- ending a write stream is a callback API, and this promise is what waits on it. */
    await new Promise<void>(function endTheSink(resolve,) {
      sink.end(resolve,);
    },);
  }
  catch (error) {
    onProblem(`could not keep Pi's standard error: ${String(error,)}`,);
  }
}
