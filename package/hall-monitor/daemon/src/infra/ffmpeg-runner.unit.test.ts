/**
 Collector tests for the one ffmpeg run shape the capture path uses.

 Three call sites shared the spawn / collect / await-close / concat /
 empty-check skeleton by hand, so a defect in any of those steps had to be fixed
 three times. `runFfmpeg` now owns it, and these cases prove each step against a
 command this test controls: the collected stream is the requested one, stdin
 reaches the child, and empty output is an error only when the caller names one.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { runFfmpeg, } from './ffmpeg-runner.ts';

//region Fixtures

/**
 Interpreter used as the run's command, so the collector is exercised against a
 program whose output this test decides. It is the only command any case needs.
 */
const COMMAND = process.execPath;

/**
 Bytes written to the collected stream by the multi-chunk cases, split so the
 case proves the collector concatenates rather than keeping the last chunk.
 */
const FIRST_CHUNK = 'first ';
const SECOND_CHUNK = 'second';

/**
 Bytes the stdin case feeds the child, echoed back by the child itself.
 */
const STDIN_PAYLOAD = 'through stdin';

/**
 Message the empty-output case expects the collector to throw.
 */
const EMPTY_OUTPUT_MESSAGE = 'the command produced nothing';

//endregion Fixtures

await describe({
  name: '',
  children: [
    describe({
      name: runFfmpeg.name,
      children: [
        it({
          name: 'concatenates every chunk a command writes to the collected stream',
          fn: async () => {
            /**
             Collected bytes; both writes must be present, in order.
             */
            const collected = await runFfmpeg({
              args: [
                '-e',
                `process.stdout.write('${FIRST_CHUNK}'); process.stdout.write('${SECOND_CHUNK}');`,
              ],
              collect: 'stdout',
              command: COMMAND,
            },);

            expect(collected.toString('utf8',),).toBe(`${FIRST_CHUNK}${SECOND_CHUNK}`,);
          },
        },),

        it({
          name: 'collects stderr when stderr is the stream the caller wants',
          fn: async () => {
            /**
             Collected bytes; the stdout write must not appear.
             */
            const collected = await runFfmpeg({
              args: [
                '-e',
                `process.stdout.write('on stdout'); process.stderr.write('${SECOND_CHUNK}');`,
              ],
              collect: 'stderr',
              command: COMMAND,
            },);

            expect(collected.toString('utf8',),).toBe(SECOND_CHUNK,);
          },
        },),

        it({
          name: 'writes the caller stdin bytes to the command',
          fn: async () => {
            /**
             Collected bytes; the child echoes its stdin back on stdout.
             */
            const collected = await runFfmpeg({
              args: [
                '-e',
                'process.stdin.pipe(process.stdout);',
              ],
              collect: 'stdout',
              command: COMMAND,
              stdin: Buffer.from(STDIN_PAYLOAD,),
            },);

            expect(collected.toString('utf8',),).toBe(STDIN_PAYLOAD,);
          },
        },),

        it({
          name: 'throws the caller message when empty output is an error',
          fn: async () => {
            await expect(runFfmpeg({
              args: [
                '-e',
                '',
              ],
              collect: 'stdout',
              command: COMMAND,
              emptyOutputMessage: EMPTY_OUTPUT_MESSAGE,
            },),).rejects.toThrow(EMPTY_OUTPUT_MESSAGE,);
          },
        },),

        it({
          name: 'returns empty output when the caller has no message for it',
          fn: async () => {
            /**
             Collected bytes; an empty result is a verdict, not a failure, for a
             caller that reads a marker out of the stream.
             */
            const collected = await runFfmpeg({
              args: [
                '-e',
                '',
              ],
              collect: 'stderr',
              command: COMMAND,
            },);

            expect(collected.length,).toBe(0,);
          },
        },),
      ],
    },),
  ],
},);
