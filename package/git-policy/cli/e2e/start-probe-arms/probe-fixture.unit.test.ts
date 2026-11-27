import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  configuration,
  handler,
  openZfsPath,
  probeDataset,
  probeScript,
  raisedDeadlineHandler,
  record,
  recordLineAssertion,
} from './probe-fixture.ts';

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
  ],
},);
