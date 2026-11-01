import {
  readdir,
  readFile,
} from 'node:fs/promises';
import {
  join,
} from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { plantedCommitControls, } from './native-planted-controls-commit.ts';
import { plantedEntryControls, } from './native-planted-controls-entry.ts';
import { plantedLandingControls, } from './native-planted-controls-landing.ts';

/**
 Package directory, one level above this script; every control's `file` is relative to it.
 */
const packageDirectory = join(import.meta.dirname, '..');

await describe({
  name: 'plantedControls',
  children: [
    it({
      name: 'every control replaces text that occurs exactly once and names a test that exists',
      fn: async () => {
        /**
         Every Rust source of the native wrapper, joined, to look test functions up in.
         */
        const sources = (await Promise.all((await readdir(join(packageDirectory, 'src', 'native')))
          .filter(function isRust(name) {
            return name.endsWith('.rs');
          })
          .map(async function readSource(name) {
            return await readFile(join(packageDirectory, 'src', 'native', name), 'utf8');
          })))
          .join('\n');
        /**
         Controls whose text or test the source no longer has, with the reason.
         */
        const stale: string[] = [];
        for (const control of [...plantedEntryControls, ...plantedCommitControls, ...plantedLandingControls]) {
          /**
           Text of the file the control plants its change in.
           */
          const text = await readFile(join(packageDirectory, control.file), 'utf8');
          /**
           How often the replaced text occurs; the harness refuses anything but once.
           */
          const occurrences = text.split(control.from).length - 1;
          /**
           Function name of the test that must notice the change.
           */
          const test = control.failing.split('::').at(-1) ?? control.failing;
          if (occurrences !== 1)
            stale.push(`${control.name}: its text occurs ${String(occurrences)} times in ${control.file}`);
          if (!sources.includes(`fn ${test}(`))
            stale.push(`${control.name}: no test named ${control.failing}`);
        }
        expect(stale).toEqual([]);
      },
    }),
  ],
});
