/**
 Tests for attached / trailing comment resolution.
 
 @module
 */

import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  _isAttachedGap as isAttachedGap,
  parseTomlEdit,
  tomlGetCommentAfter,
  tomlGetCommentsBefore,
} from '@monochromatic-dev/toml-edit-core';

/** Length of the repeated-whitespace gaps exercising the long-input path. */
const longRunLength = 100_000;

/** Number of contiguous comment lines exercising the long comment block. */
const longBlockSize = 1_000;

await describe({
  name: 'tomlGetCommentsBefore / tomlGetCommentAfter',
  children: [
    it({
      name: 'two consecutive # lines attach as a block',
      fn: async () => {
        const source = '# one\n# two\nkey = 1\n';
        const edit = parseTomlEdit({ source, },);
        const attached = tomlGetCommentsBefore({ edit, path: ['key',], },);
        expect(attached.length,).toBe(2,);
        expect(nonNullishOrThrow(attached[0],).value,).toBe(' one',);
        expect(nonNullishOrThrow(attached[1],).value,).toBe(' two',);
      },
    },),

    it({
      name: 'locates comments on an indexed array-of-tables header',
      fn: async () => {
        const edit = parseTomlEdit({
          source: '[[foo]]\nname="a"\n# note\n[[foo]]\nname="b"\n',
        },);
        const comments = tomlGetCommentsBefore({ edit, path: ['foo', 1,], },);
        expect(comments.map(function valueOf(comment,) {
          return comment.value;
        },),).toStrictEqual([' note',],);
        expect(tomlGetCommentsBefore({ edit, path: ['foo', 0,], },),).toStrictEqual([],);
      },
    },),

    it({
      name: 'a blank line between comment and key breaks attachment',
      fn: async () => {
        const source = '# header\n\nkey = 1\n';
        const edit = parseTomlEdit({ source, },);
        const attached = tomlGetCommentsBefore({ edit, path: ['key',], },);
        expect(attached.length,).toBe(0,);
      },
    },),

    it({
      name: 'trailing inline comment on same line',
      fn: async () => {
        const source = 'key = 1  # trailing\n';
        const edit = parseTomlEdit({ source, },);
        const trailing = tomlGetCommentAfter({ edit, path: ['key',], },);
        expect(trailing.comment,).not.toBe(undefined,);
        expect(nonNullishOrThrow(trailing.comment,).value,).toBe(' trailing',);
      },
    },),

    it({
      name: 'trailing comment attaches when the hash immediately follows a value',
      fn: async () => {
        const edit = parseTomlEdit({ source: 'key = 1#tail\n', },);
        expect(tomlGetCommentAfter({ edit, path: ['key',], },).comment?.value,).toBe('tail',);
      },
    },),

    it({
      name: 'trailing comment attaches at end of file without a newline',
      fn: async () => {
        const edit = parseTomlEdit({ source: 'key = 1 #tail', },);
        expect(tomlGetCommentAfter({ edit, path: ['key',], },).comment?.value,).toBe('tail',);
      },
    },),

    it({
      name: 'does not mistake a preceding comment for a trailing comment',
      fn: async () => {
        const edit = parseTomlEdit({ source: '# before\nkey = 1\n', },);
        expect(tomlGetCommentAfter({ edit, path: ['key',], },).comment,).toBe(undefined,);
      },
    },),

    it({
      name: 'does not attach a comment on the following line',
      fn: async () => {
        const edit = parseTomlEdit({ source: 'key = 1\n# after\n', },);
        expect(tomlGetCommentAfter({ edit, path: ['key',], },).comment,).toBe(undefined,);
      },
    },),

    it({
      name: 'trailing inline comment absent yields no comment field',
      fn: async () => {
        const source = 'key = 1\n';
        const edit = parseTomlEdit({ source, },);
        expect(tomlGetCommentAfter({ edit, path: ['key',], },).comment,).toBe(undefined,);
      },
    },),
  ],
},);

await describe({
  name: isAttachedGap.name,
  children: [
    it({
      name: 'empty string is not a gap',
      fn: async () => {
        expect(isAttachedGap('',),).toBe(false,);
      },
    },),

    it({
      name: 'a single newline attaches',
      fn: async () => {
        expect(isAttachedGap('\n',),).toBe(true,);
      },
    },),

    it({
      name: 'spaces and tabs around one newline still attach',
      fn: async () => {
        expect(isAttachedGap(' \t \n \t ',),).toBe(true,);
      },
    },),

    it({
      name: 'leading-only and trailing-only whitespace attach',
      fn: async () => {
        expect(isAttachedGap('   \n',),).toBe(true,);
        expect(isAttachedGap('\n   ',),).toBe(true,);
      },
    },),

    it({
      name: 'two newlines (blank line) do not attach',
      fn: async () => {
        expect(isAttachedGap('\n\n',),).toBe(false,);
        expect(isAttachedGap(' \n \n ',),).toBe(false,);
      },
    },),

    it({
      name: 'whitespace with no newline does not attach',
      fn: async () => {
        expect(isAttachedGap('   ',),).toBe(false,);
        expect(isAttachedGap('\t',),).toBe(false,);
      },
    },),

    it({
      name: 'a non-whitespace char breaks the gap',
      fn: async () => {
        expect(isAttachedGap('a',),).toBe(false,);
        expect(isAttachedGap(' \n x',),).toBe(false,);
      },
    },),

    it({
      name: 'carriage return is not gap whitespace',
      fn: async () => {
        expect(isAttachedGap('\r\n',),).toBe(false,);
      },
    },),

    it({
      name: 'a long whitespace run with one newline attaches',
      fn: async () => {
        expect(isAttachedGap(`${' '.repeat(longRunLength,)}\n`,),).toBe(true,);
        expect(isAttachedGap(`\n${' '.repeat(longRunLength,)}`,),).toBe(true,);
      },
    },),

    it({
      name: 'a long whitespace run with two newlines does not attach',
      fn: async () => {
        expect(isAttachedGap(`${' '.repeat(longRunLength,)}\n\n`,),).toBe(false,);
      },
    },),
  ],
},);

await describe({
  name: 'attached comment block collection',
  children: [
    it({
      name: 'collects a long contiguous comment block',
      fn: async () => {
        const source = `${'# c\n'.repeat(longBlockSize,)}key = 1\n`;
        const edit = parseTomlEdit({ source, },);
        const attached = tomlGetCommentsBefore({ edit, path: ['key',], },);
        expect(attached.length,).toBe(longBlockSize,);
        expect(nonNullishOrThrow(attached[0],).value,).toBe(' c',);
        expect(nonNullishOrThrow(attached.at(-1,),).value,).toBe(' c',);
      },
    },),

    it({
      name: 'a blank line splits the block; only the lower run attaches',
      fn: async () => {
        const source = '# top1\n# top2\n\n# attached1\n# attached2\nkey = 1\n';
        const edit = parseTomlEdit({ source, },);
        const attached = tomlGetCommentsBefore({ edit, path: ['key',], },);
        expect(attached.length,).toBe(2,);
        expect(nonNullishOrThrow(attached[0],).value,).toBe(' attached1',);
        expect(nonNullishOrThrow(attached[1],).value,).toBe(' attached2',);
      },
    },),

    it({
      name: 'no preceding comment yields an empty block',
      fn: async () => {
        const edit = parseTomlEdit({ source: 'key = 1\n', },);
        expect(tomlGetCommentsBefore({ edit, path: ['key',], },).length,).toBe(0,);
      },
    },),

    it({
      name: 'a comment after the key does not attach',
      fn: async () => {
        const edit = parseTomlEdit({ source: 'key = 1\n# after\n', },);
        expect(tomlGetCommentsBefore({ edit, path: ['key',], },).length,).toBe(0,);
      },
    },),
  ],
},);
