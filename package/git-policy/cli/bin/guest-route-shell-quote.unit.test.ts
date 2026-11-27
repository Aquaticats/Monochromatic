/**
 Guards for the quoting every guest script is wrapped in: the word the guest's `sh -ec` reads has to be the script byte
 for byte, whatever quotes, dollars and newlines the script holds.
 */
import { spawnSync, } from 'node:child_process';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { shellQuote, } from './guest-route-shell-quote.ts';

await describe({
  name: '',
  children: [
    describe({
      name: shellQuote.name,
      children: [
        it({
          name: 'wraps a script with no quote of its own in one pair of single quotes',
          fn: async () => {
            expect(shellQuote('zpool list -H -o name',),).toBe(`'zpool list -H -o name'`,);
          },
        },),
        it({
          name: 'closes, escapes and reopens around each single quote inside the script',
          fn: async () => {
            expect(shellQuote(`echo 'forcedact'`,),).toBe(`'echo '\\''forcedact'\\'''`,);
          },
        },),
        it({
          name: 'escapes a quote at the start and at the end of the script',
          fn: async () => {
            expect(shellQuote(`'`,),).toBe(`''\\'''`,);
            expect(shellQuote(`''`,),).toBe(`''\\'''\\'''`,);
          },
        },),
        it({
          name: 'leaves dollars, backticks and backslashes for the guest shell to read',
          fn: async () => {
            expect(shellQuote('echo $HOME `id` a\\b',),).toBe(`'echo $HOME \`id\` a\\b'`,);
          },
        },),
        it({
          name: 'keeps a newline inside the one word it hands the guest',
          fn: async () => {
            expect(shellQuote('set -eu\ntrue',),).toBe(`'set -eu\ntrue'`,);
          },
        },),
        it({
          name: 'quotes the empty script as one empty word',
          fn: async () => {
            expect(shellQuote('',),).toBe(`''`,);
          },
        },),
        //region The word a real shell reads back
        it({
          name: 'hands a shell a script holding a quote, a dollar and a newline, and the shell reads it back whole',
          fn: async () => {
            /**
             A script the shell would read as something else if the quoting around it broke: a quote that would end the
             word early, a dollar and a backtick the shell would expand, and a newline between two commands.
             */
            const script = [
              "printf '%s' \"'\"",
              "printf '%s' '$HOME `id`'",
              "printf '%s' 'two words'",
            ]
              .join('\n',);
            /**
             The line a guest driver's own shell reads: the script quoted whole stands where the guest runs it, so this
             checks the encoding the drivers emit rather than a word handed to `-c` directly.
             */
            const line = `sh -ec ${shellQuote(script,)}`;
            /** The shell's answer to that line. */
            const answered = spawnSync(
              'sh',
              [
                '-c',
                line,
              ],
              { encoding: 'utf8', },
            );
            /**
             What the shell must print: the quote as data, the dollar and the backtick unexpanded, and both words, which
             a word broken anywhere would change.
             */
            const expected = "'$HOME `id`two words";
            expect(answered.status,).toBe(0,);
            expect(answered.stdout,).toBe(expected,);
          },
        },),
        //endregion The word a real shell reads back
      ],
    },),
  ],
},);
