import {
  mkdtemp,
  rm,
} from 'node:fs/promises';
import { join, } from 'node:path';

import { tagged, } from '@monochromatic-dev/logger-core/ts';
import spawn from 'nano-spawn';

import { runFfmpeg, } from './ffmpeg-runner.ts';
import {
  log,
  reportExpectedFailure,
} from './syslog.ts';

/**
 Module logger: the journal identity this package files every record under, then this module's
 subsystem tag.
 */
const l = tagged({
  l: log,
  tag: 'capture',
},);

//region Capture configuration
// The ffmpeg long-edge bounds and the private directory the screenshot handoff lives in.

/**
 Maximum long-edge resolution for downscaled screenshots.
 */
const SCREENSHOT_LONG_EDGE = 1_440;

/**
 Name of the environment variable naming the per-user runtime directory.

 `XDG_RUNTIME_DIR` is the graphical session's own directory: this account alone
 can reach it, it is cleared when the session ends, and no other local account
 can pre-create anything inside it. The system temporary directory cannot make
 that claim, which is why this path never falls back to it.
 */
const RUNTIME_DIRECTORY_VARIABLE = 'XDG_RUNTIME_DIR';

/**
 Prefix of the private directory one screenshot handoff file lives in.
 */
const HANDOFF_DIRECTORY_PREFIX = 'hall-monitor-';

/**
 Name of the PNG handoff file inside that directory.
 */
const HANDOFF_FILE_NAME = 'screen.png';

/**
 Maximum long-edge resolution for downscaled webcam frames.
 */
const WEBCAM_LONG_EDGE = 720;

//endregion Capture configuration

//region Scale filter
// One ffmpeg filter string, so both captures constrain the same dimension the same way.

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

//endregion Scale filter

//region Screenshot handoff
// The private directory Spectacle writes into and ffmpeg reads back from.

/**
 A private directory holding one screenshot handoff file, plus the release that
 removes it.
 */
export type ScreenshotHandoff = {
  /**
   Absolute path the PNG is written to and read back from.
   */
  readonly pngPath: string;
  /**
   Removes the directory and the file inside it.
   */
  readonly release: () => Promise<void>;
};

/**
 Resolves the directory a screenshot handoff directory is created in.

 @param environment - environment the per-user runtime directory is read from;
   this process's own by default

 @returns directory only this account can write to

 @throws when the variable names no directory, because the only alternative
   left is one another local account can write to as well

 @example
 ```ts
 handoffParentDirectory({ environment: { XDG_RUNTIME_DIR: '/run/user/1000' } }); // '/run/user/1000'
 ```
 */
export function handoffParentDirectory({
  environment = process.env,
}: Readonly<{ environment?: Readonly<NodeJS.ProcessEnv>; }> = {},): string {
  /**
   Value the graphical session exported; absent when the daemon runs outside one.
   */
  const runtimeDirectory = environment[RUNTIME_DIRECTORY_VARIABLE];
  if ((runtimeDirectory === undefined) || (runtimeDirectory === '')) {
    throw new Error(
      `${RUNTIME_DIRECTORY_VARIABLE} is not set, and a screenshot handoff must not be written where another local account can reach it`,
    );
  }
  return runtimeDirectory;
}

/**
 Creates a private directory for the PNG one screenshot hands from Spectacle to
 ffmpeg.

 The handoff used to be a timestamp-named file in the shared temporary
 directory, so any other local account could pre-create that name as a symlink
 and have Spectacle write PNG data through it into a file this user owns
 (CWE-377). The directory is now created inside
 {@link handoffParentDirectory} by `mkdtemp`, which gives it a name no caller can
 predict and a mode no other account can write to, so the file inside it starts
 absent and stays this account's.

 @param parentDirectory - directory the handoff directory is created in; the
   per-user runtime directory by default

 @returns the handoff path and the release that removes the directory

 @example
 ```ts
 await using handoff = releasingHandoff(await createScreenshotHandoff());
 await spectacleTo({ pngPath: handoff.pngPath });
 ```
 */
export async function createScreenshotHandoff({
  parentDirectory = handoffParentDirectory(),
}: Readonly<{ parentDirectory?: string; }> = {},): Promise<ScreenshotHandoff> {
  /**
   Fresh private directory; `mkdtemp` creates it with mode 0700.
   */
  const directory = await mkdtemp(join(
    parentDirectory,
    HANDOFF_DIRECTORY_PREFIX,
  ),);
  return {
    pngPath: join(
      directory,
      HANDOFF_FILE_NAME,
    ),
    release: async function releaseHandoffDirectory(): Promise<void> {
      await rm(
        directory,
        {
          force: true,
          recursive: true,
        },
      );
    },
  };
}

//endregion Screenshot handoff

//region Captures
// The two capture entry points: a Spectacle screenshot and a v4l2 webcam frame.

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
   Private handoff between Spectacle and ffmpeg; released by the disposable below.
   */
  const handoff = await createScreenshotHandoff();
  /**
   Disposable wrapper for handoff cleanup.
   */
  await using _cleanup = {
    [Symbol.asyncDispose]: async function cleanupHandoff(): Promise<void> {
      try {
        await handoff.release();
      }
      catch (error: unknown) {
        reportExpectedFailure({
          error,
          l,
          operation: `removal of the screenshot handoff ${handoff.pngPath}`,
        },);
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
      handoff.pngPath,
    ],
    {
      stdout: 'ignore',
      stderr: 'ignore',
    },
  );
  return runFfmpeg({
    args: [
      '-y',
      '-i',
      handoff.pngPath,
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
    collect: 'stdout',
    emptyOutputMessage: 'Screenshot resize produced empty output',
  },);
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
export function captureWebcam(): Promise<Buffer> {
  return runFfmpeg({
    args: [
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
    collect: 'stdout',
    emptyOutputMessage: 'Webcam capture produced empty output',
  },);
}

//endregion Captures
