/**
 Unit tests pinning what the stdio transport writes to stderr.

 Issue 1754 reported that every JSON-RPC frame reached stderr with no verbosity gate,
 which would copy tool arguments and results into whatever captured the host's stderr.
 Both per-frame traces now log at `debug`, and the console sink drops `debug` records
 unless verbosity is asked for, so these cases hold that line: the default arm proves a
 request and its response leave no frame text on stderr while the transport still sends
 both frames, and the verbose arm proves `MONOCHROMATIC_VERBOSE=true` is what switches
 the echo on.

 Each arm runs in its own child process because verbosity detection is memoized per
 console sink on its first record, so one process cannot be observed both ways.

 @module
 */

import {
  spawnSync,
} from 'node:child_process';
import process from 'node:process';
import {
  fileURLToPath,
} from 'node:url';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { serve, } from '@monochromatic-dev/mcp-stdio';

//region Fixtures and helpers

/**
 Fixture program both arms run: it feeds one request and one unterminated frame through
 the built `serve()` and reports what crossed the wire.
 */
const PROGRAM_PATH = fileURLToPath(
  new URL('transport-frame-echo-child-program.ts', import.meta.url,),
);

/**
 Deadline for one fixture run.

 Generous against a loaded host, still finite: a child that never exits would otherwise
 hold the suite open with no diagnostic.
 */
const PROGRAM_TIMEOUT_MS = 30_000;

/**
 Responses the fixture's two input frames must produce: one reply to the request and one
 parse-error reply for the unterminated text, so the frames are derived from the input
 rather than assumed.
 */
const EXPECTED_FRAME_COUNT = 2;

/**
 What the fixture program reports on its stdout.
 */
type ProgramReport = {
  /**
   Request frame the transport read, as sent.
   */
  readonly requestFrame: string;
  /**
   Text only the dispatch result carries.
   */
  readonly resultPayloadMarker: string;
  /**
   Unterminated trailing text the transport rejected, as sent.
   */
  readonly unterminatedFrame: string;
  /**
   Frames the transport handed to its output writer, in write order.
   */
  readonly written: readonly string[];
};

/**
 Runs the fixture program once and reports what crossed the wire and what reached stderr.

 @param environment - Complete environment the child sees; leaving `MONOCHROMATIC_VERBOSE` out is the default-settings arm.

 @returns Parsed stdout report plus the stderr text the run left behind.

 @example
 ```ts
 const { report, stderr } = runProgram({ environment: {} });
 ```
 */
function runProgram(
  { environment, }: { readonly environment: Readonly<Record<string, string>>; },
): { readonly report: ProgramReport; readonly stderr: string; } {
  /**
   Finished child: both streams drained, exit status checked separately because a child
   that failed would otherwise surface as an unreadable JSON parse.
   */
  const run = spawnSync(
    process.execPath,
    [PROGRAM_PATH,],
    {
      encoding: 'utf8',
      env: { ...environment, },
      timeout: PROGRAM_TIMEOUT_MS,
    },
  );
  if (run.status !== 0)
    throw new Error(`fixture program exited with ${String(run.status,)}: ${run.stderr}`,);
  return {
    report: JSON.parse(run.stdout,) as ProgramReport,
    stderr: run.stderr,
  };
}

//endregion

await describe({
  name: serve.name,
  children: [
    //region Default settings

    it({
      name: 'leaves no JSON-RPC frame or tool payload on stderr while still answering',
      fn: async function testNoFrameEchoByDefault(): Promise<void> {
        /**
         Run with a child environment holding nothing, so no verbosity flag is set.
         */
        const { report, stderr, } = runProgram({ environment: {}, },);

        // Positive controls first: the request was answered and the response payload
        // crossed the wire, so its absence from stderr below is silence, not a run that
        // never produced it. The rejected trailing text then proves stderr was captured.
        expect(report.written.length,).toBe(EXPECTED_FRAME_COUNT,);
        expect(report.written.join('',),).toContain(report.resultPayloadMarker,);
        expect(stderr,).toContain('stdin frame is not JSON',);

        expect(stderr,).not.toContain(report.requestFrame,);
        expect(stderr,).not.toContain(report.unterminatedFrame,);
        expect(stderr,).not.toContain(report.resultPayloadMarker,);
        expect(stderr,).not.toContain('reading frame:',);
        expect(stderr,).not.toContain('writing frame:',);
        expect(stderr,).not.toContain('flushing trailing buffer',);
      },
    },),

    //endregion Default settings

    //region Verbose opt-in

    it({
      name: 'echoes both per-frame traces on stderr when MONOCHROMATIC_VERBOSE is true',
      fn: async function testFrameEchoWhenVerbose(): Promise<void> {
        /**
         Run with the one variable the console sink reads for verbosity.
         */
        const { report, stderr, } = runProgram({
          environment: { MONOCHROMATIC_VERBOSE: 'true', },
        },);

        expect(report.written.length,).toBe(EXPECTED_FRAME_COUNT,);
        // The echo the default arm forbids: inbound frame, outbound frame and the
        // line reader's end-of-stream flush, each carrying its payload.
        expect(stderr,).toContain('reading frame:',);
        expect(stderr,).toContain(report.requestFrame,);
        expect(stderr,).toContain('writing frame:',);
        expect(stderr,).toContain(report.resultPayloadMarker,);
        expect(stderr,).toContain('flushing trailing buffer',);
        expect(stderr,).toContain(report.unterminatedFrame,);
      },
    },),

    //endregion Verbose opt-in
  ],
},);
