/**
 Tests for the leaf-asset exclusion predicates used by fingerprinting.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  containsHiddenDirectory,
  isLeafExcluded,
} from './postprocess-excludes.ts';

/**
 Cases exercising the hidden-segment scan, each paired with its expected answer.
 */
const HIDDEN_DIRECTORY_CASES = [
  {
    path: 'a/.git/HEAD',
    expected: true,
    reason: 'a hidden directory segment inside the path',
  },
  {
    path: '.envrc',
    expected: false,
    reason: 'a dotfile with no leading slash',
  },
  {
    path: 'a/./b',
    expected: false,
    reason: 'a current-directory segment, where the dot is followed by a slash',
  },
  {
    path: 'a/.cache/b/file.png',
    expected: true,
    reason: 'a hidden segment with a deeper tail',
  },
  {
    path: 'a/.git',
    expected: false,
    reason: 'a dotfile at the end of the path, with no closing slash',
  },
  {
    path: 'dist/img/.hidden/x.avif',
    expected: true,
    reason: 'a hidden segment after a nested prefix of plain segments',
  },
] as const;

/**
 Cases exercising the leaf exclusion predicate, each paired with its expected answer.
 */
const LEAF_EXCLUDED_CASES = [
  {
    path: 'dist/foo.html',
    expected: true,
    reason: 'an excluded suffix',
  },
  {
    path: 'dist/node_modules/x',
    expected: true,
    reason: 'an excluded substring',
  },
  {
    path: 'dist/foo.avif',
    expected: false,
    reason: 'a fingerprinted asset',
  },
  {
    path: 'dist/final/a.png',
    expected: true,
    reason: 'the intermediate output directory',
  },
  {
    path: 'dist/.git/HEAD',
    expected: true,
    reason: 'a hidden directory segment',
  },
  {
    path: 'dist/styles.css',
    expected: true,
    reason: 'the generated stylesheet, which the HTML references by hash',
  },
] as const;

await describe({
  name: '',
  children: [
    describe({
      name: containsHiddenDirectory.name,
      children: HIDDEN_DIRECTORY_CASES.map(function hiddenCase({ path, expected, reason, },) {
        return it({
          name: `returns ${String(expected,)} for ${path} (${reason})`,
          fn: async function detectsHiddenSegment(): Promise<void> {
            expect(containsHiddenDirectory(path,),).toBe(expected,);
          },
        },);
      },),
    },),
    describe({
      name: isLeafExcluded.name,
      children: LEAF_EXCLUDED_CASES.map(function excludedCase({ path, expected, reason, },) {
        return it({
          name: `returns ${String(expected,)} for ${path} (${reason})`,
          fn: async function excludesLeaf(): Promise<void> {
            expect(isLeafExcluded(path,),).toBe(expected,);
          },
        },);
      },),
    },),
  ],
},);
