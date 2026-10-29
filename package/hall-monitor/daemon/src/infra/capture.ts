import { spawn as cpSpawn, } from 'node:child_process';
import { once, } from 'node:events';
import { unlink, } from 'node:fs/promises';

import spawn from 'nano-spawn';

/**
 Path to the ffmpeg binary.
 */
const FFMPEG = '/usr/bin/ffmpeg';

/**
 Maximum long-edge resolution for downscaled screenshots.
 */
const SCREENSHOT_LONG_EDGE = 1_440;

/**
 Maximum long-edge resolution for downscaled webcam frames.
 */
const WEBCAM_LONG_EDGE = 720;

/**
 Builds an ffmpeg scale filter that constrains the longer edge to
 {@link longEdge} pixels while preserving aspect ratio.
 
 @param longEdge - target pixel count for the longer dimension
 
 @returns ffmpeg `-vf` filter string
 
 @example
 ```ts
 scaleFilter(1440); // "scale='if(gt(iw,ih),1440,-2)':'if(gt(iw,ih),-2,1440)'"
 ```
 */
function scaleFilter(longEdge: number,): string {
  return `scale='if(gt(iw,ih),${longEdge},-2)':'if(gt(iw,ih),-2,${longEdge})'`;
}

/**
 Captures a full-screen screenshot via Spectacle, downscales it with ffmpeg,
 and returns the result as a JPEG buffer.
 
 @returns JPEG-encoded screenshot buffer
 
 @throws when Spectacle or ffmpeg produces empty output
 
 @example
 ```ts
 const screenshot = await captureScreenshot();
 log.debug(`Screenshot: ${(screenshot.length / 1024).toFixed(0)}KB`);
 ```
 */
export async function captureScreenshot(): Promise<Buffer> {
  /**
   Temp PNG path used as a handoff file between spectacle and ffmpeg; cleaned up by the disposable below.
   */
  const tmp = `/tmp/hall-monitor-screen-${Date.now()}.png`;
  /**
   Disposable wrapper for temp file cleanup.
   */
  await using _cleanup = {
    [Symbol.asyncDispose]: async function cleanupTempFile(): Promise<void> {
      try {
        await unlink(tmp,);
      }
      catch (error) {
        if (!(Error.isError(error,)))
          throw error;

        /* temp file cleanup is best-effort */
      }
    },
  } as AsyncDisposable;

  await spawn(
    'spectacle',
    [
      '-f',
      '-b',
      '-n',
      '-o',
      tmp,
    ],
    {
      stdout: 'ignore',
      stderr: 'ignore',
    },
  );
  /**
   ffmpeg child process that downscales the PNG handoff file and writes JPEG bytes to stdout.
   */
  const proc = cpSpawn(
    FFMPEG,
    [
      '-y',
      '-i',
      tmp,
      '-vf',
      scaleFilter(SCREENSHOT_LONG_EDGE,),
      '-q:v',
      '2',
      '-f',
      'image2',
      '-vcodec',
      'mjpeg',
      'pipe:1',
    ],
    { stdio: [
      'ignore',
      'pipe',
      'inherit',
    ], },
  );
  /**
   Accumulated JPEG byte chunks streamed from ffmpeg's stdout.
   */
  const chunks: Buffer[] = [];
  proc.stdout
    .on(
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
   Concatenated screenshot buffer; rejected when ffmpeg produced no output.
   */
  const buf = Buffer.concat(chunks,);
  if (buf.length
    === 0)
    throw new Error('Screenshot resize produced empty output',);
  return buf;
}

/**
 Captures a single webcam frame from `/dev/video0` via ffmpeg v4l2,
 downscales it, and returns the result as a JPEG buffer.
 
 @returns JPEG-encoded webcam frame buffer
 
 @throws when ffmpeg produces empty output
 
 @example
 ```ts
 const webcam = await captureWebcam();
 log.debug(`Webcam: ${(webcam.length / 1024).toFixed(0)}KB`);
 ```
 */
export async function captureWebcam(): Promise<Buffer> {
  /**
   ffmpeg child process that grabs a single v4l2 frame and emits JPEG bytes on stdout.
   */
  const proc = cpSpawn(
    FFMPEG,
    [
      '-f',
      'v4l2',
      '-i',
      '/dev/video0',
      '-frames:v',
      '1',
      '-vf',
      scaleFilter(WEBCAM_LONG_EDGE,),
      '-q:v',
      '2',
      '-f',
      'image2',
      '-vcodec',
      'mjpeg',
      'pipe:1',
    ],
    { stdio: [
      'ignore',
      'pipe',
      'inherit',
    ], },
  );
  /**
   Accumulated JPEG byte chunks streamed from ffmpeg's stdout.
   */
  const chunks: Buffer[] = [];
  proc.stdout
    .on(
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
   Concatenated webcam frame buffer; rejected when ffmpeg produced no output.
   */
  const buf = Buffer.concat(chunks,);
  if (buf.length
    === 0)
    throw new Error('Webcam capture produced empty output',);
  return buf;
}
