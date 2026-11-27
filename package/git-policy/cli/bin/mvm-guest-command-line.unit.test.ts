import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  detachedLaunchText,
  detachedStepText,
  endedLineHolds,
  endedStatusText,
  flagGiven,
  flagValue,
  guestCommandLine,
  MILLISECONDS_PER_SECOND,
  secondsOf,
  shellQuote,
} from './mvm-guest-command-line.ts';

await describe({
  name: '',
  children: [
    describe({
      name: secondsOf.name,
      children: [
        it({
          name: 'turns a whole number of seconds into itself',
          fn: async () => {
            expect(secondsOf({ milliseconds: 2_400_000, },),).toBe(2_400,);
            expect(MILLISECONDS_PER_SECOND,).toBe(1_000,);
          },
        },),
        it({
          name: 'rounds a part of a second down, as a shell timeout cannot take it',
          fn: async () => {
            expect(secondsOf({ milliseconds: 1_500, },),).toBe(1,);
            expect(secondsOf({ milliseconds: 999, },),).toBe(0,);
          },
        },),
      ],
    },),
    describe({
      name: shellQuote.name,
      children: [
        it({
          name: 'wraps a script in one pair of single quotes',
          fn: async () => {
            expect(shellQuote({ script: 'zpool list', },),).toBe(`'zpool list'`,);
          },
        },),
        it({
          name: 'closes, escapes and reopens each quote the script holds',
          fn: async () => {
            expect(shellQuote({ script: `echo 'hi'`, },),).toBe(`'echo '\\''hi'\\'''`,);
          },
        },),
        it({
          name: 'leaves a script with no quote as one word',
          fn: async () => {
            expect(shellQuote({ script: '', },),).toBe(`''`,);
          },
        },),
      ],
    },),
    describe({
      name: guestCommandLine.name,
      children: [
        it({
          name: 'checks the guest hostname first, then runs the script under the guest timeout',
          fn: async () => {
            expect(guestCommandLine({
              name: 'git-policy-lock',
              guestTimeoutSeconds: 1_740,
              script: 'zpool list',
            },),).toBe(`test "$(hostname)" = git-policy-lock && timeout 1740 sh -ec 'zpool list'`,);
          },
        },),
        it({
          name: 'quotes the script so a second shell reads it as one word',
          fn: async () => {
            expect(guestCommandLine({
              name: 'guest-a',
              guestTimeoutSeconds: 60,
              script: `echo 'a b'`,
            },),).toBe(`test "$(hostname)" = guest-a && timeout 60 sh -ec 'echo '\\''a b'\\'''`,);
          },
        },),
      ],
    },),
    describe({
      name: detachedStepText.name,
      children: [
        it({
          name: 'bounds the script, sends both streams to the shared log and appends its status',
          fn: async () => {
            expect(detachedStepText({
              guestTimeoutSeconds: 10,
              script: 'true',
              log: 'build.log',
            },),).toBe(
              `timeout 10 sh -ec 'true' > /mnt/shared/build.log 2>&1\n`
                + 'status=$?\n'
                + 'echo "DONE=$status" >> /mnt/shared/build.log\n',
            );
          },
        },),
      ],
    },),
    describe({
      name: detachedLaunchText.name,
      children: [
        it({
          name: 'starts the step file in its own session holding nothing, and returns at once',
          fn: async () => {
            expect(detachedLaunchText({ log: 'build.log', },),).toBe(
              'setsid nohup sh /mnt/shared/build.log.step.sh < /dev/null > /dev/null 2>&1 &\n'
                + 'exit 0\n',
            );
          },
        },),
      ],
    },),
    describe({
      name: endedLineHolds.name,
      children: [
        it({
          name: 'reads a status line whatever status it carries',
          fn: async () => {
            expect(endedLineHolds({ text: 'DONE=0\n', },),).toBe(true,);
            expect(endedLineHolds({ text: 'DONE=1\n', },),).toBe(true,);
          },
        },),
        it({
          name: 'reads a line that is not a status line as a script still working',
          fn: async () => {
            expect(endedLineHolds({ text: 'Compiling zbus\n', },),).toBe(false,);
            expect(endedLineHolds({ text: '', },),).toBe(false,);
          },
        },),
      ],
    },),
    describe({
      name: endedStatusText.name,
      children: [
        it({
          name: 'reads the digits of the first status line',
          fn: async () => {
            expect(endedStatusText({ text: 'building\nDONE=1\nmore\n', },),).toBe('1',);
            expect(endedStatusText({ text: 'DONE=0\nDONE=1\n', },),).toBe('0',);
          },
        },),
        it({
          name: 'answers empty text for a log with no status line',
          fn: async () => {
            expect(endedStatusText({ text: 'building\n', },),).toBe('',);
            expect(endedStatusText({ text: '', },),).toBe('',);
          },
        },),
        it({
          name: 'refuses a status word that is not only decimal digits',
          fn: async () => {
            expect(endedStatusText({ text: 'DONE=x\n', },),).toBe('',);
            expect(endedStatusText({ text: 'DONE=1x\n', },),).toBe('',);
            expect(endedStatusText({ text: 'xDONE=1\n', },),).toBe('',);
            expect(endedStatusText({ text: 'DONE=\n', },),).toBe('',);
          },
        },),
      ],
    },),
    describe({
      name: flagValue.name,
      children: [
        it({
          name: 'reads the word after the flag',
          fn: async () => {
            expect(flagValue({
              argumentWords: ['run', '--name', 'guest-a',],
              flag: '--name',
              fallback: 'default',
            },),).toBe('guest-a',);
          },
        },),
        it({
          name: 'answers the fallback for a flag that is absent',
          fn: async () => {
            expect(flagValue({
              argumentWords: ['run',],
              flag: '--name',
              fallback: 'default',
            },),).toBe('default',);
          },
        },),
        it({
          name: 'answers the fallback for a flag written last with nothing after it',
          fn: async () => {
            expect(flagValue({
              argumentWords: ['run', '--name',],
              flag: '--name',
              fallback: 'default',
            },),).toBe('default',);
          },
        },),
      ],
    },),
    describe({
      name: flagGiven.name,
      children: [
        it({
          name: 'reads a flag written on the command line',
          fn: async () => {
            expect(flagGiven({
              argumentWords: ['run', '--keep-guest',],
              flag: '--keep-guest',
            },),).toBe(true,);
          },
        },),
        it({
          name: 'reads a flag that is absent',
          fn: async () => {
            expect(flagGiven({
              argumentWords: ['run',],
              flag: '--keep-guest',
            },),).toBe(false,);
          },
        },),
      ],
    },),
  ],
},);
