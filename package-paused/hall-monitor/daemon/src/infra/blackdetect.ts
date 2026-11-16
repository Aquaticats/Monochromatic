import { runFfmpeg, } from './ffmpeg-runner.ts';

/**
 Detects whether a JPEG frame is essentially all-black (e.g. webcam privacy
 cover is down) using ffmpeg's blackdetect filter.
 Thresholds: 98% of pixels must be below 10% luminance to count as black.
 
 @param jpegBuf - raw JPEG image bytes to analyze
 
 @returns `true` when the frame is considered black
 
 @mutates jpegBuf through the child process's writable stdin stream
 
 @example
 ```ts
 if (await isBlackFrame(webcamBuffer)) {
   log.debug("Webcam cover detected, skipping");
 }
 ```
 */
export async function isBlackFrame(jpegBuf: Buffer,): Promise<boolean> {
  /**
   Decoded ffmpeg stderr; blackdetect emits its `black_start` marker there, not on stdout.
   */
  const stderr = (await runFfmpeg({
    args: [
      '-f',
      'image2pipe',
      '-i',
      'pipe:0',
      '-vf',
      'blackdetect=d=0:pix_th=0.10:pic_th=0.98',
      '-f',
      'null',
      '-',
    ],
    collect: 'stderr',
    stdin: jpegBuf,
  },)).toString(
    'utf8',
  );
  // blackdetect emits "black_start:..." lines on stderr when a black frame is found
  return stderr.includes('black_start',);
}
