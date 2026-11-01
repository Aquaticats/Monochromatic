import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type { Comment, } from '@oxlint/plugins';

import { parseTsdocComment, } from './tsdoc-comments.ts';

/** Builds host-shaped block comment for cache tests. */
function commentWithValue(value: string,): Comment {
  return { value, } as Comment;
}

await describe({
  name: parseTsdocComment.name,
  children: [
    it({
      name: 'reuses parsed facts for identical comment bodies',
      fn: async () => {
        const value = '*\n * @param value - Input.\n ';
        const first = parseTsdocComment(commentWithValue(value,),);
        const second = parseTsdocComment(commentWithValue(value,),);

        expect(second,).toBe(first,);
        expect(second.docComment.params.blocks,).toHaveLength(1,);
      },
    },),
    it({
      name: 'does not share facts across distinct host comments',
      fn: async () => {
        const first = parseTsdocComment(commentWithValue('*\n * First.\n ',),);
        const second = parseTsdocComment(commentWithValue('*\n * Second.\n ',),);

        expect(second,).not.toBe(first,);
      },
    },),
  ],
},);
