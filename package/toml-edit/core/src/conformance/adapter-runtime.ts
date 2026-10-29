/**
 Shared process plumbing for the conformance decode and encode adapters.
 
 Both adapters read all of stdin as raw bytes (the decoder validates UTF-8
 itself; the encoder feeds JSON) and signal rejection through a non-zero exit
 code without calling `process.exit`, so buffered stdout still flushes.
 
 @module
 */

import { buffer, } from 'node:stream/consumers';

/**
 Read the whole of stdin as a byte buffer.
 
 @returns Buffered stdin bytes.
 
 @example
 ```ts
 const bytes = await readStdin();
 ```
 */
export function readStdin(): Promise<Buffer> {
  return buffer(process.stdin,);
}

/**
 Report adapter failure: write `message` to stderr and set a failing exit code.
 
 Uses `process.exitCode` rather than `process.exit` so any already-written
 stdout is not truncated, matching the runner's expectation that rejection is
 signalled purely by exit status.
 
 @param message - Diagnostic written to stderr.
 
 @example
 ```ts
 failAdapter({ message: 'invalid input', });
 ```
 */
export function failAdapter({ message, }: { readonly message: string; },): void {
  process.stderr
    .write(`${message}\n`,);
  process.exitCode = 1;
}
