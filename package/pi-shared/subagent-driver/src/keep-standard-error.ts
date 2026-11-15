/**
 Copies Pi's standard error to a file for as long as Pi writes it, masking it on the way.

 Pi's diagnostics are written by Pi and by the extensions it loads, and a line of them can hold a value a tool printed: this is the
 one place that text passes, so it is masked before the file holds it. The copy holds back the last characters a value's longest
 form could need, so a value split across two writes is still matched, and writes what it held back when Pi's error ends.

 @module
 */

import { createWriteStream, } from 'node:fs';
import type { Readable, } from 'node:stream';
import { createStreamMatcher, } from './masking-a-stream.ts';
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
   The one matcher every write of this stream goes through, which holds back whatever could still become a match.
   */
  const matcher = createStreamMatcher({ mask, },);
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
       What may be written of this chunk now, which is nothing while it is still the beginning of a possible match.
       */
      const writable = matcher.push(chunk,);
      if (writable !== '') {
        sink.write(writable,);
      }
    }
    sink.write(matcher.flush(),);
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
