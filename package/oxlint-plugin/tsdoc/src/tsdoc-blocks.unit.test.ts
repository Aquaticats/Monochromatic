import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type { Comment, } from '@oxlint/plugins';
import type { ReadonlyDeep, } from 'type-fest';

import { splitDocComment, } from './tsdoc-blocks.ts';

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

await describe({
  name: '',
  children: [
    describe({
      name: splitDocComment.name,
      children: [
        //region @param extraction

        it({
          name: 'extracts a param name and its description',
          fn: async () => {
            const { params, } = splitDocComment({
              comment: commentFromBody(['@param a - first',],),
            },);
            expect(params.blocks,).toEqual([
              {
                parameterName: 'a',
                hasDescription: true,
              },
            ],);
          },
        },),

        it({
          name: 'marks a name-only param as missing a description',
          fn: async () => {
            const { params, } = splitDocComment({
              comment: commentFromBody(['@param name',],),
            },);
            expect(params.blocks,).toEqual([
              {
                parameterName: 'name',
                hasDescription: false,
              },
            ],);
          },
        },),

        it({
          name: 'treats a bare hyphen as a missing parameter name',
          fn: async () => {
            const { params, } = splitDocComment({
              comment: commentFromBody(['@param - desc',],),
            },);
            expect(params.blocks,).toEqual([
              {
                parameterName: '',
                hasDescription: true,
              },
            ],);
          },
        },),

        it({
          name: 'keeps multiple param blocks in source order',
          fn: async () => {
            const { params, } = splitDocComment({
              comment: commentFromBody([
                '@param a - first',
                '',
                '@param b - second',
              ],),
            },);
            expect(params.blocks
              .map(function name(block,): string {
                return block.parameterName;
              },),).toEqual([
                'a',
                'b',
              ],);
          },
        },),

        it({
          name: 'ignores a @param inside a fenced example block',
          fn: async () => {
            const { params, hasExampleTag, } = splitDocComment({
              comment: commentFromBody([
                '@param a - real',
                '',
                '@example',
                '```ts',
                '@param fake - x',
                '```',
              ],),
            },);
            expect(params.blocks,).toHaveLength(1,);
            expect(params.blocks[0]?.parameterName,).toBe('a',);
            expect(hasExampleTag,).toBe(true,);
          },
        },),

        //endregion @param extraction

        //region @mutates extraction

        it({
          name: 'extracts mutation targets and descriptions',
          fn: async () => {
            const { mutates, } = splitDocComment({
              comment: commentFromBody([
                '@mutates values - Clears caller-owned entries.',
                '',
                '@mutates cache - Replaces cached values.',
              ],),
            },);
            expect(mutates.blocks,).toEqual([
              {
                parameterName: 'values',
                hasDescription: true,
                lineOffset: 1,
              },
              {
                parameterName: 'cache',
                hasDescription: true,
                lineOffset: 3,
              },
            ],);
          },
        },),

        it({
          name: 'records missing mutation target and description independently',
          fn: async () => {
            const { mutates, } = splitDocComment({
              comment: commentFromBody([
                '@mutates - Has rationale.',
                '',
                '@mutates values',
              ],),
            },);
            expect(mutates.blocks,).toEqual([
              {
                parameterName: '',
                hasDescription: true,
                lineOffset: 1,
              },
              {
                parameterName: 'values',
                hasDescription: false,
                lineOffset: 3,
              },
            ],);
          },
        },),

        it({
          name: 'ignores mutation tags inside fenced examples',
          fn: async () => {
            const { mutates, } = splitDocComment({
              comment: commentFromBody([
                '@example',
                '```ts',
                '@mutates fake - Not documentation.',
                '```',
              ],),
            },);
            expect(mutates.blocks,).toEqual([],);
          },
        },),

        //endregion @mutates extraction

        //region @returns extraction

        it({
          name: 'records a returns block with a description',
          fn: async () => {
            const { returnsBlock, } = splitDocComment({
              comment: commentFromBody(['@returns sum of inputs',],),
            },);
            expect(returnsBlock,).toEqual({ hasDescription: true, },);
          },
        },),

        it({
          name: 'records a bare returns block as missing a description',
          fn: async () => {
            const { returnsBlock, } = splitDocComment({
              comment: commentFromBody(['@returns',],),
            },);
            expect(returnsBlock,).toEqual({ hasDescription: false, },);
          },
        },),

        it({
          name: 'leaves returnsBlock undefined when no @returns tag exists',
          fn: async () => {
            const { returnsBlock, } = splitDocComment({
              comment: commentFromBody(['Summary only.',],),
            },);
            expect(returnsBlock,).toBeUndefined();
          },
        },),

        //endregion @returns extraction

        //region Tag-presence flags

        it({
          name: 'detects an inline @inheritDoc tag',
          fn: async () => {
            const { hasInheritDocTag, } = splitDocComment({
              comment: commentFromBody(['{@inheritDoc Foo}',],),
            },);
            expect(hasInheritDocTag,).toBe(true,);
          },
        },),

        it({
          name: 'detects the @internal modifier',
          fn: async () => {
            const { hasInternalModifier, } = splitDocComment({
              comment: commentFromBody([
                'Internal helper.',
                '',
                '@internal',
              ],),
            },);
            expect(hasInternalModifier,).toBe(true,);
          },
        },),

        it({
          name: 'reports no presence flags for a plain summary',
          fn: async () => {
            const doc = splitDocComment({
              comment: commentFromBody(['Just a summary.',],),
            },);
            expect(doc.hasExampleTag,).toBe(false,);
            expect(doc.hasInheritDocTag,).toBe(false,);
            expect(doc.hasInternalModifier,).toBe(false,);
          },
        },),

        it({
          name: 'does not treat @example mentioned in prose as a block tag',
          fn: async () => {
            const doc = splitDocComment({
              comment: commentFromBody([
                'Directly exported function missing @example.',
                '',
                '@param a - first',
              ],),
            },);
            expect(doc.hasExampleTag,).toBe(false,);
          },
        },),

        //endregion Tag-presence flags
      ],
    },),
  ],
},);
