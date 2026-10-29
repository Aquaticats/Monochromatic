/**
 Tests for attaching archive transcript blocks to source-matched media.

 Fixtures are cat-themed invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  parseDocument,
  type RepairDocument,
} from '@monochromatic-dev/translation-repair-shared-contract';
import { claimMediaAdjacentTargets, } from '../dist/final/node/index.mjs';

/**
 Literal site path placeholder, assembled so source does not resemble accidental interpolation.
 */
const PATH_TOKEN = [
  '$',
  '{path}',
].join('',);

/**
 Shared media marker.
 */
const LETTER = `<PhotoScroll photos={[ '${PATH_TOKEN}/photos/letter.webp']} />`;

/**
 Other media marker for ambiguous-gap fixture.
 */
const PORTRAIT = `<PhotoScroll photos={[ '${PATH_TOKEN}/photos/portrait.webp']} />`;

/**
 Parses fixture blocks separated as document paragraphs.

 @param texts - block text in document order

 @returns Parsed fixture document

 @example
 ```ts
 const document = parsed(['Cats nap.']);
 ```
 */
function parsed(texts: readonly string[],): RepairDocument {
  return parseDocument({ text: texts.join('\n\n',), });
}

await describe({
  name: claimMediaAdjacentTargets.name,
  children: [
    it({
      name: 'CLAIMS transcript run bounded by ordinary block and matching media marker so picture evidence reaches its quality slice',
      fn: async () => {
        const source = parsed([
          'About the cat.',
          LETTER,
          'Remember the cat.',
        ],);
        const target = parsed([
          'About the cat.',
          '<details>\n<summary>Letter</summary>\n> Translated letter.\n</details>',
          LETTER,
          'Remember the cat.',
        ],);
        expect(claimMediaAdjacentTargets({
          sourceBlocks: source.nodes,
          targetBlocks: target.nodes,
          targetContainers: target.containers,
          pairs: [
            { source: 0, target: 0, },
            { source: 1, target: 3, },
            { source: 2, target: 4, },
          ],
        },).pairs,).toEqual([
          { source: 0, target: 0, },
          { source: 1, target: 1, },
          { source: 1, target: 2, },
          { source: 1, target: 3, },
          { source: 2, target: 4, },
        ],);
      },
    },),
    it({
      name: 'LEAVES gap between distinct media markers unclaimed because adjacency cannot choose which picture supports it',
      fn: async () => {
        const source = parsed([
          LETTER,
          PORTRAIT,
        ],);
        const target = parsed([
          LETTER,
          '<details>\n<summary>Caption</summary>\n> Ambiguous.\n</details>',
          PORTRAIT,
        ],);
        expect(claimMediaAdjacentTargets({
          sourceBlocks: source.nodes,
          targetBlocks: target.nodes,
          targetContainers: target.containers,
          pairs: [
            { source: 0, target: 0, },
            { source: 1, target: 3, },
          ],
        },).pairs,).toEqual([
          { source: 0, target: 0, },
          { source: 1, target: 3, },
        ],);
      },
    },),
    it({
      name: 'LEAVES ordinary prose beside one matching media marker unclaimed because adjacency without explicit details container is not transcript proof',
      fn: async () => {
        const source = parsed([LETTER,],);
        const target = parsed([
          LETTER,
          'An unrelated archive paragraph.',
        ],);
        expect(claimMediaAdjacentTargets({
          sourceBlocks: source.nodes,
          targetBlocks: target.nodes,
          targetContainers: target.containers,
          pairs: [{ source: 0, target: 0, },],
        },).pairs,).toEqual([{ source: 0, target: 0, },],);
      },
    },),
    it({
      name: 'LEAVES ordinary unclaimed prose untouched when no paired blocks share media',
      fn: async () => {
        const source = parsed(['About the cat.',],);
        const target = parsed([
          'About the cat.',
          'An inherited aside.',
        ],);
        expect(claimMediaAdjacentTargets({
          sourceBlocks: source.nodes,
          targetBlocks: target.nodes,
          targetContainers: target.containers,
          pairs: [{ source: 0, target: 0, },],
        },).findings,).toEqual([]);
      },
    },),
  ],
},);
