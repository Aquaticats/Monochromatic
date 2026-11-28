import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  configuration,
  guestBuildEndOf,
  handler,
  logTailOf,
  openZfsPath,
  probeDataset,
  probeScript,
  raisedDeadlineHandler,
  record,
  recordLineAssertion,
  tailLineCount,
} from './probe-fixture.ts';

/**
 The last lines one of a guest build's two cargo builds writes with `tail --lines=3`,
 as the build's own log holds them.

 A stand-in for the guest's output and not a capture: what these cases are about is
 which lines a refusal carries, and a compiler's words are not the subject. The
 status line is the shape `bin/mvm-guest-command-line.ts` writes, whose own cases
 hold it against a real log.
 */
const buildTail: readonly string[] = [
  '   Compiling monochromatic-git-policy-cli v0.6.1-alpha.49',
  '    Finished `release` profile [optimized] target(s) in 3m 42s',
];

/**
 One guest build's log, as the detached script writes it.

 @param lines - the lines the build printed

 @param status - the status line's digits

 @returns the whole log
 */
function logOf({
  lines,
  status,
}: {
  readonly lines: readonly string[];
  readonly status: string;
},): string {
  return `${[...lines, `DONE=${status}`,].join('\n',)}\n`;
}

/**
 The header every arm script starts with.

 @returns the header's own lines, in the order the arms read them
 */
function headerLines(): readonly string[] {
  return [
    'set -eu',
    `export PATH=${openZfsPath}`,
  ];
}

await describe({
  name: '',
  children: [
    describe({
      name: probeScript.name,
      children: [
        it({
          name: 'starts every arm with a shell that stops at a failing command',
          fn: async () => {
            const script = probeScript({ lines: ['echo one',], },);
            expect(script.startsWith(`${headerLines().join('\n',)}\n`,),).toBe(true);
          },
        },),
        it({
          name: 'names the handler, the raised-deadline build and the record each arm reads',
          fn: async () => {
            const script = probeScript({ lines: [], },);
            expect(script,).toContain(`H='${handler}'`);
            expect(script,).toContain(`R='${raisedDeadlineHandler}'`);
            expect(script,).toContain(`REC=${record}`);
            expect(script,).toContain(`CONF=${configuration}`);
            expect(script,).toContain(`SRC='${probeDataset}@ready'`);
          },
        },),
        it({
          name: 'quotes both handler assignments, because each holds a path and then the words that select a mode',
          fn: async () => {
            const script = probeScript({ lines: [], },);
            expect(script,).not.toContain(`H=${handler}`);
            expect(script,).not.toContain(`R=${raisedDeadlineHandler}`);
          },
        },),
        it({
          name: 'ends with the arm own lines, in the order the arm gave them',
          fn: async () => {
            const script = probeScript({ lines: [
              'echo one',
              'echo two',
            ], },);
            expect(script.endsWith('\necho one\necho two',),).toBe(true);
          },
        },),
        it({
          name: 'gives an arm with no lines the header alone',
          fn: async () => {
            const script = probeScript({ lines: [], },);
            expect(script,).toContain('leaks() {',);
            expect(script.endsWith('echo "leaks=$n"; }',),).toBe(true);
          },
        },),
      ],
    },),
    describe({
      name: 'recordLineAssertion',
      children: [
        it({
          name: 'fails an arm whose record line is missing, and prints the record it read',
          fn: async () => {
            expect(recordLineAssertion.join('\n',),).toContain('echo "MISSING $1 in $2"');
            expect(recordLineAssertion.join('\n',),).toContain('cat "$2"');
          },
        },),
        it({
          name: 'exits nonzero when a claim is missing, so the host sees a failed arm',
          fn: async () => {
            expect(recordLineAssertion.join('\n',),).toContain('exit 1');
          },
        },),
      ],
    },),
    describe({
      name: guestBuildEndOf.name,
      children: [
        it({
          name: 'reads a build that ended zero as one that did not fail',
          fn: async () => {
            expect(guestBuildEndOf({ text: logOf({
              lines: buildTail,
              status: '0',
            },), },),).toEqual({ failed: false, },);
          },
        },),
        it({
          name: 'reports the status a failing build ended with, and the lines above it',
          fn: async () => {
            /**
             What a build's log says when a compiler refused it.
             */
            const ended = guestBuildEndOf({ text: logOf({
              lines: [
                'error[E0308]: mismatched types',
                'error: could not compile `monochromatic-git-policy-cli` (bin "monochromatic-git-policy-cli") due to 1 previous error',
              ],
              status: '101',
            },), },);
            expect(ended.failed,).toBe(true,);
            if (!ended.failed)
              throw new Error('a build whose log names 101 was read as one that ended zero',);
            expect(ended.status,).toBe('101',);
            expect(ended.tail,).toEqual([
              'error[E0308]: mismatched types',
              'error: could not compile `monochromatic-git-policy-cli` (bin "monochromatic-git-policy-cli") due to 1 previous error',
            ],);
          },
        },),
        it({
          name: 'reports a build whose log names no status, as the guest writes it when its timeout ends the script',
          fn: async () => {
            /**
             A log with no status line at all.
             */
            const ended = guestBuildEndOf({ text: `${[...buildTail,].join('\n',)}\n`, },);
            expect(ended.failed,).toBe(true,);
            if (!ended.failed)
              throw new Error('a build whose log names no status was read as one that ended zero',);
            expect(ended.status,).toBe('without a status',);
            expect(ended.tail,).toEqual(buildTail,);
          },
        },),
        it({
          name: 'reads the guest own 124 as a failure, so a status is not read by its first digit',
          fn: async () => {
            /**
             What a build's log says when the guest's timeout ended it, which is the shell's own 124.
             */
            const ended = guestBuildEndOf({ text: logOf({
              lines: buildTail,
              status: '124',
            },), },);
            expect(ended.failed,).toBe(true,);
            if (!ended.failed)
              throw new Error('a build whose log names 124 was read as one that ended zero',);
            expect(ended.status,).toBe('124',);
          },
        },),
      ],
    },),
    describe({
      name: logTailOf.name,
      children: [
        it({
          name: 'carries the last lines the guest wrote, up to the count a refusal reads',
          fn: async () => {
            /**
             More lines than a refusal carries.
             */
            const lines: readonly string[] = Array.from(
              { length: tailLineCount + 3, },
              function numbered(_, index: number,): string {
                return `rustc line ${String(index,)}`;
              },
            );
            expect(logTailOf({ text: `${[...lines,].join('\n',)}\n`, },),).toEqual(
              lines.slice(-tailLineCount,),
            );
          },
        },),
        it({
          name: 'leaves the status line out of the tail it reports',
          fn: async () => {
            expect(logTailOf({ text: 'warning: unused import\nDONE=1\n', },),).toEqual(['warning: unused import',],);
          },
        },),
        it({
          name: 'answers no line for a log whose only line is its status',
          fn: async () => {
            expect(logTailOf({ text: 'DONE=0\n', },),).toEqual([],);
          },
        },),
      ],
    },),
  ],
},);
