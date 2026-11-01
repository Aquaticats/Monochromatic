import {
  dirname,
  join,
  parse,
} from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { ancestorDirectories, } from '../../dist/final/node/index.mjs';

await describe({
  name: ancestorDirectories.name,
  children: [
    it({
      name: 'yields the start and each ancestor through the filesystem root once',
      fn: async () => {
        const { root, } = parse(process.cwd(),);
        const start = join(
          root,
          'ancestor-probe-parent',
          'ancestor-probe-child',
        );
        expect([...ancestorDirectories(start,)],).toEqual([
          start,
          dirname(start,),
          root,
        ],);
      },
    },),
    it({
      name: 'yields a root starting point exactly once',
      fn: async () => {
        const { root, } = parse(process.cwd(),);
        expect([...ancestorDirectories(root,)],).toEqual([root,],);
      },
    },),
  ],
},);
