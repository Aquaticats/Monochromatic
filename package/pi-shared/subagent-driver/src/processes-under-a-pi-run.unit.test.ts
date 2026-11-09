/**
 The processes under one Pi run, read from the text `ps` would have printed and from a stand-in for the command that asks it.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  ProcessListingUnreadableError,
  type RunCommand,
  parseProcessListing,
  processesUnder,
  startProcessTable,
} from '../dist/final/node/index.mjs';

/**
 A listing shaped as `ps -e --no-headers -o pid=,ppid=,etimes=,stat=,wchan:18=,args=` prints it: the kernel wait is padded to
 its own column, and the command keeps the spaces inside it.
 */
const LISTING = [
  '      1       0  300000 Ss   -                  /usr/lib/systemd/systemd --switched-root --system',
  '   4242    5697    2919 Ssl  ep_poll            /node /work/package/pi-shared/subagent-driver/src/drive-pi-subagent.ts',
  '   4300    4242     120 S    do_wait            bash -c mise run //package/x:test',
  '   4301    4300      60 R    -                  cargo test --all',
  '   4400    4300      30 S    futex_wait_queue   node --input-type=module-typescript -e "void 0"',
  '   5000       1      10 S    ep_poll            /usr/bin/other',
  '  1 0 30',
].join(
  '\n',
  );

await describe({
  name: parseProcessListing.name,
  children: [
    it({
      name: 'reads every process, keeping the command whole and the padded kernel wait on its own',
      fn: async () => {
        /**
         What the listing describes.
         */
        const table = parseProcessListing({ listing: LISTING, },);
        expect(table,).toHaveLength(6,);
        expect(table[2],).toEqual({
          pid: 4300,
          parentPid: 4242,
          elapsedSeconds: 120,
          state: 'S',
          waitingIn: 'do_wait',
          command: 'bash -c mise run //package/x:test',
        },);
      },
    },),
    it({
      name: 'leaves out a line with too few columns to describe a process',
      fn: async () => {
        /**
         A listing holding one real line and one cut one.
         */
        const table = parseProcessListing({
          listing: [
            '   4242    5697    2919 Ssl  ep_poll            /node /work/driver.ts',
            '  5000       1      10',
          ].join(
            '\n',
            ),
        },);
        expect(table,).toHaveLength(1,);
      },
    },),
  ],
},);

await describe({
  name: processesUnder.name,
  children: [
    it({
      name: 'finds every descendant of a process and not the process itself nor its neighbours',
      fn: async () => {
        /**
         The processes under the run.
         */
        const under = processesUnder({
          table: parseProcessListing({ listing: LISTING, },),
          rootPid: 4242,
        },);
        expect(under.map(function pidOf(entry,) {
          return entry.pid;
        },),).toEqual([4300, 4301, 4400,],);
      },
    },),
    it({
      name: 'finds nothing under a process that started nothing',
      fn: async () => {
        expect(processesUnder({
          table: parseProcessListing({ listing: LISTING, },),
          rootPid: 5000,
        },),).toEqual([],);
      },
    },),
  ],
},);

await describe({
  name: startProcessTable.name,
  children: [
    it({
      name: 'asks ps for the table without the environment, and reads it',
      fn: async () => {
        /**
         What the command was asked.
         */
        const asked: { command: string; words: readonly string[] }[] = [];
        /**
         A stand-in for `ps`.
         */
        const run: RunCommand = async function standInPs(request,) {
          asked.push(request,);
          return {
            status: 0,
            stdout: LISTING,
            stderr: '',
          };
        };
        /**
         Reads the table.
         */
        const table = await startProcessTable({ run, },)();
        expect(table,).toHaveLength(6,);
        expect(asked[0]?.command,).toBe('ps',);
        /**
         The columns asked of `ps`, whose format is where the environment would be printed if it were asked for.
         */
        const columns = asked[0]?.words[asked[0].words.indexOf('-o',) + 1] ?? '';
        expect(columns,).toContain('wchan',);
        expect(columns.split(',',),).not.toContain('e',);
      },
    },),
    it({
      name: 'refuses when ps could not be run, naming what it said',
      fn: async () => {
        /**
         A stand-in for a `ps` that is not there.
         */
        const run: RunCommand = async function refuse() {
          return {
            status: -1,
            stdout: '',
            stderr: '',
            problem: 'spawn ps ENOENT',
          };
        };
        try {
          await startProcessTable({ run, },)();
        }
        catch (error) {
          expect(error,).toBeInstanceOf(ProcessListingUnreadableError,);
          expect(String(error,),).toContain('spawn ps ENOENT',);
          return;
        }
        throw new Error('a missing ps was read as an empty table',);
      },
    },),
  ],
},);
