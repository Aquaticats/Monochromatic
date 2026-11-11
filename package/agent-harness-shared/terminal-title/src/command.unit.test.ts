/**
 Tests for shell command terminal title summaries.
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { terminalTitleCommand, } from '@monochromatic-dev/agent-harness-shared-terminal-title';

await describe({
  name: terminalTitleCommand.name,
  children: [
    it({
      name: 'leaves plain command details visible',
      fn: async () => {
        expect(terminalTitleCommand('npm test',),).toBe('npm test',);
      },
    },),
    it({
      name: 'strips shell environment assignment prefix',
      fn: async () => {
        expect(terminalTitleCommand('NODE_ENV=production npm test',),).toBe('npm test',);
      },
    },),
    it({
      name: 'unwraps env and timeout wrappers together',
      fn: async () => {
        expect(terminalTitleCommand('env timeout 10 npm test',),).toBe('npm test',);
      },
    },),
    it({
      name: 'unwraps nice priority wrapper',
      fn: async () => {
        expect(terminalTitleCommand('nice -n 5 npm test',),).toBe('npm test',);
      },
    },),
    it({
      name: 'unwraps timeout options and duration',
      fn: async () => {
        expect(terminalTitleCommand('timeout --signal=TERM 10 npm test',),).toBe('npm test',);
      },
    },),
    it({
      name: 'unwraps env assignments before the command',
      fn: async () => {
        expect(terminalTitleCommand('env FOO=1 BAR=2 npm test',),).toBe('npm test',);
      },
    },),
    it({
      name: 'unwraps env options that consume the next token',
      fn: async () => {
        expect(terminalTitleCommand('env --unset FOO npm test',),).toBe('npm test',);
      },
    },),
    it({
      name: 'unwraps timeout options that consume the next token before the duration',
      fn: async () => {
        expect(terminalTitleCommand('timeout -k 5 10 npm test',),).toBe('npm test',);
      },
    },),
    it({
      name: 'unwraps nohup without walking its arguments',
      fn: async () => {
        expect(terminalTitleCommand('nohup npm test',),).toBe('npm test',);
      },
    },),
    it({
      name: 'keeps a wrapper whose own arguments are the whole command',
      fn: async () => {
        expect(terminalTitleCommand('env FOO=1',),).toBe('env FOO=1',);
      },
    },),
    it({
      name: 'falls back to raw command when parsing fails',
      fn: async () => {
        /**
         Shell source with unterminated quote.
         */
        const rawCommand = 'echo "unterminated';
        expect(terminalTitleCommand(rawCommand,),).toBe(rawCommand,);
      },
    },),
  ],
},);
