import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import { launchTerminal, } from '@monochromatic-dev/terminal-exec-cli';

await describe({
  name: 'terminal-exec lib (built bundle smoke test)',
  children: [
    //region Re-export integrity: load the built bundle by-name, check the export type without invoking it

    // Importing launch.mjs and reading typeof executes the lib bundle but never CALLS launchTerminal (which would
    // spawn a real terminal emulator). The bin entry (index.mjs, execvp-replaces the process) is never run by any test.
    it({
      name: 'exposes launchTerminal as a function (without invoking it)',
      fn: async () => {
        expect(typeof launchTerminal,).toBe('function',);
      },
    },),

    //endregion Re-export integrity
  ],
},);
