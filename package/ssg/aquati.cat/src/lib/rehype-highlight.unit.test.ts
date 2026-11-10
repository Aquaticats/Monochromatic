/**
 Tests for the build-time Lezer highlighting pass.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { rehypeHighlight, } from '../../dist/final/node/index.mjs';

/**
 Tree type the plugin's transformer accepts, taken from the plugin so the
 fixture carries exactly the ownership marker the host interface declares.
 */
type TransformerTree = Parameters<ReturnType<typeof rehypeHighlight>>[0];

/**
 JSON document whose only key is `name`.
 */
const JSON_SOURCE = '{"name": "x"}';

/**
 Offsets of the `name` key inside {@link JSON_SOURCE}, quotes included.
 */
const KEY_RANGE = {
  from: 1,
  to: 7,
} as const;

/**
 Builds a hast tree holding one `<pre><code class="language-…">` block.
 
 @param language - language class suffix the build reads
 
 @param code - code block text
 
 @returns hast root wrapping the block
 
 @example
 ```ts
 codeBlockTree('json', '{"name": "x"}');
 ```
 */
function codeBlockTree(
  language: string,
  code: string,
): TransformerTree {
  return {
    type: 'root',
    children: [{
      type: 'element',
      tagName: 'pre',
      properties: {},
      children: [{
        type: 'element',
        tagName: 'code',
        properties: { className: [`language-${language}`,], },
        children: [{
          type: 'text',
          value: code,
        },],
      },],
    },],
  };
}

/**
 Reads one `data-hl-<group>` attribute from a rendered code block.
 
 @param root - tree the plugin annotated
 
 @param group - highlight group name
 
 @returns the attribute text, or an empty string when the group matched nothing
 
 @example
 ```ts
 highlightAttribute(root, 'property');
 // '1-7'
 ```
 */
function highlightAttribute(
  root: TransformerTree,
  group: string,
): string {
  /**
   The `<pre>` element the fixture wraps its code block in.
   */
  const [pre,] = root.children;
  if (pre?.type !== 'element')
    return '';
  /**
   First child of the `<pre>`, which is the code element the plugin annotated.
   */
  const [codeElement,] = pre.children;
  if (codeElement?.type !== 'element')
    return '';
  /**
   Attribute value for this group, coerced because hast properties are untyped.
   */
  const value = codeElement.properties[`data-hl-${group}`];

  return (typeof value) === 'string' ? value : '';
}

/**
 Checks whether a `from-to` attribute covers an offset range.
 
 @param attribute - semicolon-separated pairs the plugin emitted
 
 @param range - offsets that must fall inside one pair
 
 @returns `true` when one pair starts at `from` and ends at `to`
 
 @example
 ```ts
 coversRange('1-7;9-10', { from: 1, to: 7 });
 // true
 ```
 */
function coversRange(
  attribute: string,
  range: {
    readonly from: number;
    readonly to: number;
  },
): boolean {
  return attribute
    .split(';',)
    .includes(`${range.from}-${range.to}`,);
}

await describe({
  name: rehypeHighlight.name,
  children: [
    it({
      name: 'tags a JSON object key as a property',
      fn: async function tagsJsonKeyAsProperty(): Promise<void> {
        /**
         Tree whose single block is highlighted in place.
         */
        const root = codeBlockTree(
          'json',
          JSON_SOURCE,
        );

        rehypeHighlight()(root,);

        expect(coversRange(highlightAttribute(root, 'property',), KEY_RANGE,),).toBe(true,);
      },
    },),
    it({
      name: 'does not tag the JSON key as a string',
      fn: async function doesNotTagKeyAsString(): Promise<void> {
        /**
         Tree whose single block is highlighted in place.
         */
        const root = codeBlockTree(
          'json',
          JSON_SOURCE,
        );

        rehypeHighlight()(root,);

        expect(coversRange(highlightAttribute(root, 'string',), KEY_RANGE,),).toBe(false,);
      },
    },),
  ],
},);
