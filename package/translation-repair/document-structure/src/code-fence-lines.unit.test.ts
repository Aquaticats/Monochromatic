/**
 Tests for fenced code line flags.

 Every case asserts the WHOLE flag array rather than one position, because the
 defect this guards against is a state machine losing track of where a fence
 ends, and that shows up as a run of wrong flags rather than a wrong one.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { fencedLineFlags, } from '@monochromatic-dev/translation-repair-markup';
import { maskInvisibleLines, } from '../dist/final/node/index.mjs';

/**
 Splits a body the way the masker does, so fixtures read as text.

 @param text - body with newline terminators

 @returns Flag per line

 @example
 ```ts
 const flags = flagsOf('```\nx\n```\n',);
 ```
 */
function flagsOf(text: string,): readonly boolean[] {
  return fencedLineFlags({ lines: text.split('\n',), },);
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: fencedLineFlags.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'reports every line unfenced when the body carries no fence at all',
          fn: async () => {
            expect(flagsOf('Alpha.\n\nBeta.\n',),).toEqual([
              false,
              false,
              false,
              false,
            ],);
          },
        },),

        it({
          name: 'flags the opening and closing markers as well as the interior, so '
            + 'a caller that skips flagged lines never rewrites a marker either',
          fn: async () => {
            expect(flagsOf('Alpha.\n```\ncode\n```\nBeta.',),).toEqual([
              false,
              true,
              true,
              true,
              false,
            ],);
          },
        },),

        it({
          name: 'accepts tilde fences, which matter because a tilde fence is how a '
            + 'document holds a backtick fence as literal content',
          fn: async () => {
            expect(flagsOf('~~~\n```\n~~~\nAfter.',),).toEqual([
              true,
              true,
              true,
              false,
            ],);
          },
        },),

        it({
          name: 'needs three markers, so a run of two opens nothing and the lines '
            + 'after it stay available for masking',
          fn: async () => {
            expect(flagsOf('``\ncode\n``\n',),).toEqual([
              false,
              false,
              false,
              false,
            ],);
          },
        },),

        it({
          name: 'opens at three spaces of indent but not at four, because four '
            + 'columns is an indented code block and cannot carry a fence',
          fn: async () => {
            expect(flagsOf('   ```\ncode\n   ```\n',),).toEqual([
              true,
              true,
              true,
              false,
            ],);
            expect(flagsOf('    ```\ncode\n    ```\n',),).toEqual([
              false,
              false,
              false,
              false,
            ],);
          },
        },),

        it({
          name: 'refuses a tab-indented fence, since one tab is four columns and so '
            + 'exceeds the indent a fence may carry',
          fn: async () => {
            expect(flagsOf('\t```\ncode\n',),).toEqual([
              false,
              false,
              false,
            ],);
          },
        },),

        it({
          name: 'refuses to open a backtick fence whose info string carries a '
            + 'backtick, which is the ambiguity that separates a block from an '
            + 'inline code span',
          fn: async () => {
            expect(flagsOf('``` a ` b\nAlpha.\n',),).toEqual([
              false,
              false,
              false,
            ],);
          },
        },),

        it({
          name: 'will not close a backtick fence with a tilde one, so the marker '
            + 'that opened the block is the only one that can end it',
          fn: async () => {
            expect(flagsOf('```\ncode\n~~~\nstill code\n',),).toEqual([
              true,
              true,
              true,
              true,
              true,
            ],);
          },
        },),

        it({
          name: 'will not close with a run shorter than the one that opened, and '
            + 'accepts a longer one, which is how a fence holds its own marker',
          fn: async () => {
            expect(flagsOf('````\n```\ncode\n`````\nAfter.',),).toEqual([
              true,
              true,
              true,
              true,
              false,
            ],);
          },
        },),

        it({
          name: 'will not close on a marker that carries an info string, because '
            + 'only an opening fence may name a language',
          fn: async () => {
            expect(flagsOf('```\ncode\n``` ts\nstill code\n',),).toEqual([
              true,
              true,
              true,
              true,
              true,
            ],);
          },
        },),

        it({
          name: 'runs an unclosed fence to the end of the body, exactly as '
            + 'CommonMark reads it, so nothing after a stray marker is masked',
          fn: async () => {
            expect(flagsOf('Alpha.\n```\ncode\nmore code\n',),).toEqual([
              false,
              true,
              true,
              true,
              true,
            ],);
          },
        },),

        it({
          name: 'will not close on a marker trailed by an INVISIBLE character, '
            + 'which CommonMark reads as code content. Only spaces and tabs may '
            + 'follow a closing fence, and phrasing that check with trim() accepts '
            + 'U+FEFF, U+00A0, U+2028 and U+2029 as well, so the fence would close '
            + 'early and expose the next invisible-only line to masking. This is '
            + 'the same trap the masker itself exists to document',
          fn: async () => {
            for (const trailing of [
              '\u{FEFF}',
              '\u{00A0}',
              '\u{2028}',
              '\u{2029}',
            ]) {
              expect(flagsOf(`\`\`\`\ncode\n\`\`\`${trailing}\nstill code\n`,),).toEqual([
                true,
                true,
                true,
                true,
                true,
              ],);
            }
          },
        },),

        it({
          name: 'still closes on a marker trailed by ordinary spaces and tabs, '
            + 'which CommonMark does allow, so the stricter check did not simply '
            + 'stop closing fences from working',
          fn: async () => {
            expect(flagsOf('```\ncode\n```  \t\nAfter.',),).toEqual([
              true,
              true,
              true,
              false,
            ],);
          },
        },),
      ],
    },),

    describe({
      name: `${maskInvisibleLines.name} inside fenced code`,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'leaves an invisible-only line alone inside a fence, because there '
            + 'the character is CONTENT and blanking it rewrites the document being '
            + 'repaired, which is the one thing a length-preserving mask exists to '
            + 'avoid',
          fn: async () => {
            /**
             Zero-width space standing as the whole of a line inside a fence.
             */
            const text = '```\nalpha\n\u{200B}\nbeta\n```\n';

            expect(maskInvisibleLines({ text, },).masked,).toBe(text,);
          },
        },),

        it({
          name: 'still blanks an invisible-only line after the fence has closed, so '
            + 'the exemption ends where the code does',
          fn: async () => {
            expect(
              maskInvisibleLines({ text: '```\n\u{200B}\n```\nAlpha.\n\u{200B}\nBeta.\n', },).masked,
            ).toBe('```\n\u{200B}\n```\nAlpha.\n \nBeta.\n',);
          },
        },),
      ],
    },),
  ],
},);
