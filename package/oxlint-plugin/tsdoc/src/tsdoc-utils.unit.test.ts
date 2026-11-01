import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  IGNORED_EXTENSIONS,
  shouldIgnoreFile,
} from './tsdoc-utils.ts';

await describe({
  name: shouldIgnoreFile.name,
  children: [
    ...IGNORED_EXTENSIONS.map(function mapExt(ext,) {
      return it({
        name: `returns true for ${ext} extension`,
        fn: async () => {
          expect(shouldIgnoreFile(`/some/path/file${ext}`,),).toBe(true,);
        },
      },);
    },),
    it({
      name: 'returns false for plain .ts files',
      fn: async () => {
        expect(shouldIgnoreFile('/some/path/file.ts',),).toBe(false,);
      },
    },),
    it({
      name: 'returns false for .tsx files',
      fn: async () => {
        expect(shouldIgnoreFile('/some/path/component.tsx',),).toBe(false,);
      },
    },),
    it({
      name: 'returns false for empty string',
      fn: async () => {
        expect(shouldIgnoreFile('',),).toBe(false,);
      },
    },),
    it({
      name: 'checks suffix, not substring',
      fn: async () => {
        // A file named "test.ts.bak" should not be ignored
        expect(shouldIgnoreFile('/path/file.test.ts.bak',),).toBe(false,);
      },
    },),
  ],
},);
