import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type { Comment, } from '@oxlint/plugins';
import type { ReadonlyDeep, } from 'type-fest';

import { collectStructuralMessages, } from './tsdoc-structural-messages.ts';

/** Builds a stub block comment whose `value` mimics a real TSDoc body. */
function commentFromBody(bodyLines: readonly string[],): ReadonlyDeep<Comment> {
  const value = [
    '*',
    ...bodyLines.map(function prefix(line,): string {
      return line === '' ? ' *' : ` * ${line}`;
    },),
    ' ',
  ].join('\n',);
  return { value, } as unknown as ReadonlyDeep<Comment>;
}

/** Extracts the message ids from a scanned comment body. */
function idsFor(bodyLines: readonly string[],): readonly string[] {
  return collectStructuralMessages({ comment: commentFromBody(bodyLines,), },)
    .map(function id(message,): string {
      return message.messageId;
    },);
}

await describe({
  name: '',
  children: [
    describe({
      name: collectStructuralMessages.name,
      children: [
        //region Missing hyphen

        it({
          name: 'flags a @param whose description lacks a hyphen',
          fn: async () => {
            expect(idsFor(['@param name description here',],),).toContain(
              'tsdoc-param-tag-missing-hyphen',
            );
          },
        },),

        it({
          name: 'accepts a @param with a hyphen separator',
          fn: async () => {
            expect(idsFor(['@param name - description',],),).toEqual([],);
          },
        },),

        it({
          name: 'does not flag a name-only @param',
          fn: async () => {
            expect(idsFor(['@param name',],),).toEqual([],);
          },
        },),

        //endregion Missing hyphen

        //region Inline tags

        it({
          name: 'flags an unclosed inline tag',
          fn: async () => {
            expect(idsFor(['See {@link Foo for more',],),).toContain(
              'tsdoc-inline-tag-missing-right-brace',
            );
          },
        },),

        it({
          name: 'flags an empty inline link tag',
          fn: async () => {
            expect(idsFor(['See {@link} here',],),).toContain(
              'tsdoc-link-tag-empty',
            );
          },
        },),

        it({
          name: 'accepts a well-formed inline tag',
          fn: async () => {
            expect(idsFor(['See {@link Foo} here',],),).toEqual([],);
          },
        },),

        it({
          name: 'ignores inline-tag syntax inside an inline code span',
          fn: async () => {
            expect(idsFor(['See `{@link}` for the shape',],),).toEqual([],);
          },
        },),

        //endregion Inline tags
      ],
    },),
  ],
},);
