import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { _isStrictPrefix, } from '@monochromatic-dev/toml-edit-core';

await describe({
  name: _isStrictPrefix.name,
  children: [
    it({
      name: 'accepts a matching shorter path',
      fn: async () => {
        expect(_isStrictPrefix({ candidate: ['a', 0,], path: ['a', 0, 'x',], },),).toBe(true,);
        expect(_isStrictPrefix({ candidate: [], path: ['a',], },),).toBe(true,);
      },
    },),
    it({
      name: 'rejects equal and longer paths',
      fn: async () => {
        expect(_isStrictPrefix({ candidate: ['a',], path: ['a',], },),).toBe(false,);
        expect(_isStrictPrefix({ candidate: [], path: [], },),).toBe(false,);
        expect(_isStrictPrefix({ candidate: ['a', 'b',], path: ['a',], },),).toBe(false,);
      },
    },),
    it({
      name: 'rejects a partially matching candidate',
      fn: async () => {
        expect(_isStrictPrefix({ candidate: ['a', 'x',], path: ['a', 'b', 'c',], },),).toBe(false,);
      },
    },),
  ],
},);
