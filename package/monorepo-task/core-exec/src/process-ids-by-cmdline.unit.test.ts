/**
 Process ids selected from a `/proc` listing whose command-line read races process exit.

 Every case injects the listing and the per-entry read, so the two operating-system errors a
 process that ends mid-scan produces, `ENOENT` and `ESRCH`, are reproduced without loading the
 host or starting a process.

 @module
 */

import {
  caught,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  processIdsWithCmdline,
} from '../dist/final/node/testing.mjs';

//region Fixture values

/** NUL byte `/proc/<pid>/cmdline` places after each argument. */
const ARGUMENT_SEPARATOR = '\u0000';

/** Command line the scan looks for: a process running `sleep 0.5`. */
const EXPECTED_CMDLINE = `sleep${ARGUMENT_SEPARATOR}0.5${ARGUMENT_SEPARATOR}`;

/** Command line of a listed process running something else. */
const OTHER_CMDLINE = `cat${ARGUMENT_SEPARATOR}${ARGUMENT_SEPARATOR}`;

/** Process id of the listed process whose command line matches. */
const MATCHING_PID = 4_101;

/** Process id of a listed process running another command. */
const OTHER_PID = 4_102;

/** Process id of a listed process that ends before its command line is read. */
const VANISHING_PID = 4_103;

//endregion Fixture values

//region Helpers

/**
 Builds an error shaped like the one Node raises for a failed system call.

 @param code - `errno` name the operating system reported, such as `ESRCH`

 @param syscall - System call name the message names

 @returns Error carrying the code as its own property, as Node's does
 */
function systemCallError({
  code,
  syscall,
}: { readonly code: string; readonly syscall: string; },): NodeJS.ErrnoException {
  const error: NodeJS.ErrnoException = new Error(`${code}: no such process, ${syscall}`,);
  error.code = code;
  error.syscall = syscall;
  return error;
}

//endregion Helpers

await describe({
  name: '',
  children: [
    describe({
      name: processIdsWithCmdline.name,
      children: [
        it({
          name: 'selects only the process whose command line matches exactly',
          fn: async () => {
            const matchingEntry = String(MATCHING_PID,);
            const scanned = processIdsWithCmdline({
              entries: [String(OTHER_PID,), matchingEntry,],
              expectedCmdline: EXPECTED_CMDLINE,
              readCmdline: function readListedCmdline(entry,): string {
                return (entry === matchingEntry) ? EXPECTED_CMDLINE : OTHER_CMDLINE;
              },
            },);

            expect(scanned,).toEqual([MATCHING_PID,],);
          },
        },),
        it({
          name: 'skips table names that are not process ids without reading them',
          fn: async () => {
            const scanned = processIdsWithCmdline({
              entries: ['self', 'sys', 'uptime',],
              expectedCmdline: EXPECTED_CMDLINE,
              readCmdline: function refuseTableNameRead(entry,): string {
                throw new Error(`read ${entry}, although a table name is not a process id`,);
              },
            },);

            expect(scanned,).toEqual([],);
          },
        },),
        it({
          name: 'selects nothing from an empty table',
          fn: async () => {
            const scanned = processIdsWithCmdline({
              entries: [],
              expectedCmdline: EXPECTED_CMDLINE,
              readCmdline: function refuseEmptyTableRead(entry,): string {
                throw new Error(`read ${entry}, although the table lists nothing`,);
              },
            },);

            expect(scanned,).toEqual([],);
          },
        },),
        it({
          name: 'skips a process that ended before its read answered ENOENT',
          fn: async () => {
            const refusal = systemCallError({ code: 'ENOENT', syscall: 'open', },);
            const scanned = processIdsWithCmdline({
              entries: [String(VANISHING_PID,),],
              expectedCmdline: EXPECTED_CMDLINE,
              readCmdline: function readVanishedDirectory() {
                throw refusal;
              },
            },);

            expect(scanned,).toEqual([],);
          },
        },),
        it({
          name: 'skips a process that ended before its read answered ESRCH',
          fn: async () => {
            const refusal = systemCallError({ code: 'ESRCH', syscall: 'read', },);
            const scanned = processIdsWithCmdline({
              entries: [String(VANISHING_PID,),],
              expectedCmdline: EXPECTED_CMDLINE,
              readCmdline: function readVanishedTask() {
                throw refusal;
              },
            },);

            expect(scanned,).toEqual([],);
          },
        },),
        it({
          name: 'hands back a failure that does not name a vanished process',
          fn: async () => {
            const refusal = systemCallError({ code: 'EACCES', syscall: 'open', },);
            const thrown = caught(function scanForbiddenEntry() {
              processIdsWithCmdline({
                entries: [String(VANISHING_PID,),],
                expectedCmdline: EXPECTED_CMDLINE,
                readCmdline: function readForbiddenEntry() {
                  throw refusal;
                },
              },);
            },);

            expect(thrown,).toBe(refusal,);
          },
        },),
        it({
          name: 'hands back an error that carries no code',
          fn: async () => {
            const refusal = new Error('the command-line read is broken',);
            const thrown = caught(function scanUncodedError() {
              processIdsWithCmdline({
                entries: [String(VANISHING_PID,),],
                expectedCmdline: EXPECTED_CMDLINE,
                readCmdline: function readUncodedError() {
                  throw refusal;
                },
              },);
            },);

            expect(thrown,).toBe(refusal,);
          },
        },),
        it({
          name: 'hands back an error whose code is not text',
          fn: async () => {
            const refusal: Error & { code?: unknown; } = new Error('the command-line read failed oddly',);
            refusal.code = 3;
            const thrown = caught(function scanUnlitCode() {
              processIdsWithCmdline({
                entries: [String(VANISHING_PID,),],
                expectedCmdline: EXPECTED_CMDLINE,
                readCmdline: function readNumericCode() {
                  throw refusal;
                },
              },);
            },);

            expect(thrown,).toBe(refusal,);
          },
        },),
        it({
          name: 'hands back a thrown value that is not an error, even with a vanished-process code',
          fn: async () => {
            const refusal = { code: 'ESRCH', };
            const thrown = caught(function scanPlainObject() {
              processIdsWithCmdline({
                entries: [String(VANISHING_PID,),],
                expectedCmdline: EXPECTED_CMDLINE,
                readCmdline: function readPlainObject() {
                  // oxlint-disable-next-line typescript/only-throw-error -- the fixture IS a thrown non-Error, which is the whole case: the scan must not trust a code on a value it cannot read as an error
                  throw refusal;
                },
              },);
            },);

            expect(thrown,).toBe(refusal,);
          },
        },),
      ],
    },),
  ],
},);
