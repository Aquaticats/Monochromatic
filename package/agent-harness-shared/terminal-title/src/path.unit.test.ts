/**
 Tests for smart terminal title paths.
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { terminalTitlePath, } from '@monochromatic-dev/agent-harness-shared-terminal-title';

await describe({
  name: terminalTitlePath.name,
  children: [
    it({
      name: 'keeps relative path context',
      fn: async () => {
        expect(terminalTitlePath({ filePath: 'src/index.ts', },),).toBe('src/index.ts',);
      },
    },),
    it({
      name: 'trims trivial current-directory prefix',
      fn: async () => {
        expect(terminalTitlePath({ filePath: './src/index.ts', },),).toBe('src/index.ts',);
      },
    },),
    it({
      name: 'uses cwd-relative path for absolute path inside cwd',
      fn: async () => {
        expect(
          terminalTitlePath({
            filePath: '/repo/src/index.ts',
            cwd: '/repo',
          },),
        ).toBe('src/index.ts',);
      },
    },),
    it({
      name: 'uses tail context for absolute path without cwd match',
      fn: async () => {
        expect(terminalTitlePath({ filePath: '/home/user/project/src/index.ts', },),).toBe(
          'src/index.ts',
        );
      },
    },),
  ],
},);
