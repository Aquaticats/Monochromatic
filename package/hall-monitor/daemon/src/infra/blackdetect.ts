import { spawn as cpSpawn, } from 'node:child_process';
import { once, } from 'node:events';

/**
 Detects whether a JPEG frame is essentially all-black (e.g. webcam privacy
 cover is down) using ffmpeg's blackdetect filter.
 Thresholds: 98% of pixels must be below 10% luminance to count as black.
 
 @param jpegBuf - raw JPEG image bytes to analyze
 
 @returns `true` when the frame is considered black
 
 @mutates jpegBuf through proc.stdin.end child-process writable-stream access
 
 @example
 ```ts
 if (await isBlackFrame(webcamBuffer)) {
   log.debug("Webcam cover detected, skipping");
 }
 ```
 */
export async function isBlackFrame(jpegBuf: Buffer,): Promise<boolean> {
  /**
   ffmpeg child process running blackdetect; the JPEG is piped in via stdin.
   */
  const proc = cpSpawn(
    '/usr/bin/ffmpeg',
    [
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
    { stdio: [
      'pipe',
      'pipe',
      'pipe',
    ], },
  );
  proc.stdin
    .end(jpegBuf,);
  /**
   Stderr byte chunks; blackdetect emits its `black_start` marker here, not on stdout.
   */
  const stderrChunks: Buffer[] = [];
  proc.stderr
    .on(
    'data',
    function collectChunk(chunk: Buffer,) {
      stderrChunks.push(chunk,);
    },
    );
  await once(
    proc,
    'close',
  );
  /**
   Decoded ffmpeg stderr; searched for the blackdetect marker to determine the verdict.
   */
  const stderr = Buffer.concat(stderrChunks,)
    .toString('utf8',);
  // blackdetect emits "black_start:..." lines on stderr when a black frame is found
  return stderr.includes('black_start',);
}
