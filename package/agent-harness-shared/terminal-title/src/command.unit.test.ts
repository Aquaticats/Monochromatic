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
