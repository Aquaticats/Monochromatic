/**
 One ffmpeg run whose chosen output stream is collected into a single buffer.

 Three call sites carried this skeleton by hand before it lived here: spawn the
 binary with the same stdio shape, collect one stream's chunks, await `close`,
 `Buffer.concat` them, and reject an empty result. They differed only in argv,
 which stream they read, and whether empty output is an error, so a fix to chunk
 collection, process-error handling, or the empty-output check had to be made
 three times. Each caller now passes those three things and owns nothing else.

 @module
 */

import { spawn as cpSpawn, } from 'node:child_process';
import { once, } from 'node:events';

//region Command

/**
 Path to the ffmpeg binary.
 */
const FFMPEG = '/usr/bin/ffmpeg';

/**
 Which of ffmpeg's two output streams a run collects.
 */
type FfmpegOutputStream = 'stdout' | 'stderr';

//endregion Command

//region Runner

/**
 Runs ffmpeg and returns one of its output streams, concatenated.

 The collected stream is the only one read: a caller that wants the JPEG ffmpeg
 writes on stdout leaves stderr inherited, so a decode failure still reaches the
 daemon's own stderr, and a caller that wants the `blackdetect` markers on stderr
 ignores stdout.

 @param args - argv handed to ffmpeg

 @param collect - stream whose bytes the caller wants

 @param stdin - bytes written to ffmpeg's stdin before it is closed; the stream
   is left unconnected when omitted

 @param emptyOutputMessage - message thrown when the collected output is empty;
   an empty result is returned as-is when omitted, because a caller reading a
   marker out of the stream treats absence as a verdict

 @param command - binary to run; ffmpeg by default, and another command only so a
   test can drive the collector with a program whose output it decides

 @returns concatenated bytes of the collected stream

 @throws when the run fails, or when `emptyOutputMessage` is given and the
   collected output is empty

 @example
 ```ts
 const jpeg = await runFfmpeg({
   args: ['-i', 'pipe:0', '-f', 'image2', 'pipe:1'],
   collect: 'stdout',
   emptyOutputMessage: 'Webcam capture produced empty output',
 });
 ```
 */
export async function runFfmpeg({
  args,
  collect,
  emptyOutputMessage,
  stdin,
  command = FFMPEG,
}: Readonly<{
  args: readonly string[];
  collect: FfmpegOutputStream;
  emptyOutputMessage?: string;
  stdin?: Buffer;
  command?: string;
}>): Promise<Buffer> {
  /**
   Child running ffmpeg; the stdio shape follows what the caller collects.
   */
  const proc = cpSpawn(
    command,
    [...args,],
    {
      stdio: [
        (stdin === undefined) ? 'ignore' : 'pipe',
        (collect === 'stdout') ? 'pipe' : 'ignore',
        (collect === 'stderr') ? 'pipe' : 'inherit',
      ],
    },
  );

  /**
   The one stream this run reads.
   */
  const collected = (collect === 'stdout') ? proc.stdout : proc.stderr;
  if (collected === null) {
    throw new Error(
      `ffmpeg was started without a ${collect} pipe, so nothing can be collected`,
    );
  }

  if (stdin !== undefined) {
    if (proc.stdin === null) {
      throw new Error(
        'ffmpeg was started without a stdin pipe, so the caller bytes have nowhere to go',
      );
    }
    proc.stdin
      .end(stdin,);
  }

  /**
   Accumulated byte chunks streamed from the collected stream.
   */
  const chunks: Buffer[] = [];
  collected.on(
    'data',
    function collectChunk(chunk: Buffer,) {
      chunks.push(chunk,);
    },
  );

  await once(
    proc,
    'close',
  );

  /**
   Concatenated output; rejected when the caller treats empty output as a failure.
   */
  const buf = Buffer.concat(chunks,);
  if ((emptyOutputMessage !== undefined) && (buf.length === 0))
    throw new Error(emptyOutputMessage,);
  return buf;
}

//endregion Runner

export type { FfmpegOutputStream, };
