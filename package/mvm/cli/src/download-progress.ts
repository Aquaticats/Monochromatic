/**
 Download progress display for image downloads.
 
 Polls the destination file size and prints progress to stderr.
 
 @module
 */

import { createWriteStream, } from 'node:fs';
import { stat, } from 'node:fs/promises';
import { Readable, } from 'node:stream';
import { pipeline, } from 'node:stream/promises';

import { formatBytes, } from '@monochromatic-dev/module-numeric-format/ts';

//region Display constants

/**
 Multiplier for converting a ratio to a percentage.
 */
const PERCENT = 100;

/**
 Number of trailing spaces to overwrite stale progress line characters.
 */
const PROGRESS_LINE_PAD = 20;

//endregion Display constants

/**
 Polls the destination file size and prints download progress to stderr.
 Runs until the `signal` is aborted by the caller after the download completes.
 
 @param destPath - Destination file path being written to
 
 @param contentLength - Expected total bytes (0 when unknown)
 
 @param totalStr - Pre-formatted total size string for display
 
 @param signal - AbortSignal that stops the polling loop
 */
async function pollProgress({
  contentLength,
  destPath,
  signal,
  totalStr,
}: {
  readonly contentLength: number;
  readonly destPath: string;
  readonly signal: AbortSignal;
  readonly totalStr: string;
},): Promise<void> {
  /**
   Milliseconds between file size polls for progress display.
   */
  const POLL_INTERVAL_MS = 500;

  while (!signal.aborted) {
    // oxlint-disable-next-line no-await-in-loop, promise/avoid-new -- deliberate serial polling loop
    await new Promise(function pollDelay(resolve,) {
      setTimeout(
        resolve,
        POLL_INTERVAL_MS,
      );
    },);
    if (signal.aborted)
      break;
    try {
      /**
       Current on-disk size of the destination file; polled each tick to drive progress output.
       */
      // oxlint-disable-next-line no-await-in-loop -- deliberate serial polling loop
      const { size, } = await stat(destPath,);
      /**
       Human-readable form of `size` (e.g. "12.3 MB"); cached so it appears in both branches.
       */
      const downloadedStr = formatBytes(size,);
      if (contentLength > 0) {
        /**
         Integer percentage of the download completed; only meaningful when content-length is known.
         */
        const pct = Math.round((size / contentLength) * PERCENT,);
        process.stderr
          .write(
          `\r  downloading: ${downloadedStr} / ${totalStr} (${String(pct,)}%)`,
          );
      }
      else {
        process.stderr
          .write(`\r  downloading: ${downloadedStr}`,);
      }
    }
    catch (error) {
      if (!(Error.isError(error,)))
        throw error;

      // File may not exist yet during initial write setup
    }
  }
}

/**
 Streams a fetch response body to disk while printing download progress to stderr
 via {@link pollProgress}.
 Uses AbortController to coordinate between the progress poller and the stream pipeline.
 
 @param destPath - Destination file path to write to
 
 @param response - Fetch response with a body to stream
 
 @param rl - Logger for status messages
 
 @mutates response through `Readable.from` asynchronous body iteration
 
 @example
 ```ts
 await writeWithProgress({ response, destPath: '/tmp/image.img', rl: logger });
 ```
 */
export async function writeWithProgress({
  destPath,
  response,
  rl,
}: {
  readonly destPath: string;
  readonly response: Response;
  readonly rl: { readonly info: (msg: string,) => void; };
},): Promise<void> {
  /**
   Expected total bytes from the `content-length` header; 0 when the server omits it.
   */
  const contentLength = Number(response.headers
    .get('content-length',)
    ?? 0,);
  /**
   Pre-formatted display string for the total size; computed once because progress prints it every tick.
   */
  const totalStr = contentLength > 0 ? formatBytes(contentLength,) : 'unknown';

  /**
   Coordinates between the polling loop and the stream pipeline; abort stops the poller cleanly.
   */
  const controller = new AbortController();

  /**
   Background progress poller; awaited at the end to ensure the final tick flushes before returning.
   */
  // Start progress polling in the background
  const progressDone = pollProgress({
    contentLength,
    destPath,
    signal: controller.signal,
    totalStr,
  },);

  /**
   Response body stream destructured for null-check; null bodies trigger an explicit error.
   */
  // Stream response body to disk via AsyncIterable protocol (runtime-neutral)
  const { body, } = response;
  if (body === null) {
    controller.abort();
    await progressDone;
    throw new Error(`response body is null for ${destPath}`,);
  }
  await pipeline(
    Readable.from(body,),
    createWriteStream(destPath,),
  );

  // Stop progress polling
  controller.abort();
  await progressDone;

  /**
   Final on-disk size after the pipeline completes; printed as the "downloaded: ..." line.
   */
  const { size, } = await stat(destPath,);
  process.stderr
    .write(
    `\r  downloaded: ${formatBytes(size,)} total${' '.repeat(PROGRESS_LINE_PAD,)}\n`,
    );
  rl.info(`saved to ${destPath}`,);
}
