/**
 Unit tests for English morphology fallbacks.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  englishGerund,
  englishThirdSingular,
} from './morphology.ts';

/** One regular morphology fallback expectation. */
type MorphologyCase = Readonly<{
  /** Test name explaining rule branch. */
  readonly name: string;
  /** Verb base form passed to morphology helper. */
  readonly base: string;
  /** Expected derived surface. */
  readonly expected: string;
}>;

/** Function shape shared by English morphology helpers. */
type MorphologyHelper = (options: { readonly base: string; },) => string;

/** Gerund cases covering each fallback branch and documented examples. */
const GERUND_CASES: readonly MorphologyCase[] = [
  {
    name: 'changes final ie to ying',
    base: 'die',
    expected: 'dying',
  },
  {
    name: 'doubles final consonant after consonant-vowel-consonant run',
    base: 'run',
    expected: 'running',
  },
  {
    name: 'doubles final consonant before ing after stop',
    base: 'stop',
    expected: 'stopping',
  },
  {
    name: 'doubles final consonant in longer consonant-vowel-consonant bases',
    base: 'begin',
    expected: 'beginning',
  },
  {
    name: 'drops silent final e before ing',
    base: 'save',
    expected: 'saving',
  },
  {
    name: 'keeps final ee before ing',
    base: 'see',
    expected: 'seeing',
  },
  {
    name: 'keeps vowel-y ending before ing',
    base: 'play',
    expected: 'playing',
  },
  {
    name: 'keeps final x before ing',
    base: 'fix',
    expected: 'fixing',
  },
  {
    name: 'adds ing in default branch',
    base: 'want',
    expected: 'wanting',
  },
];

/** Third-person singular cases covering every fallback branch. */
const THIRD_SINGULAR_CASES: readonly MorphologyCase[] = [
  {
    name: 'adds es after final s',
    base: 'kiss',
    expected: 'kisses',
  },
  {
    name: 'adds es after final x',
    base: 'fix',
    expected: 'fixes',
  },
  {
    name: 'adds es after final z',
    base: 'buzz',
    expected: 'buzzes',
  },
  {
    name: 'adds es after final ch',
    base: 'watch',
    expected: 'watches',
  },
  {
    name: 'adds es after final sh',
    base: 'wash',
    expected: 'washes',
  },
  {
    name: 'changes consonant-y ending to ies',
    base: 'try',
    expected: 'tries',
  },
  {
    name: 'keeps vowel-y ending before s',
    base: 'play',
    expected: 'plays',
  },
  {
    name: 'adds s for one-letter y base without previous consonant',
    base: 'y',
    expected: 'ys',
  },
  {
    name: 'adds es after final o',
    base: 'go',
    expected: 'goes',
  },
  {
    name: 'adds s in default branch',
    base: 'save',
    expected: 'saves',
  },
];

/**
 Builds a morphology test for one expected fallback surface.
 
 @param options - case and helper wrapped for named-parameter calls
 
 @returns test descriptor for module-test
 */
function morphologyCaseIt(
  options: {
    readonly testCase: MorphologyCase;
    readonly helper: MorphologyHelper;
  },
): ReturnType<typeof it> {
  /** Case data and helper under test. */
  const { testCase, helper, } = options;
  return it({
    name: testCase.name,
    fn: async () => {
      expect(helper({ base: testCase.base, },),).toBe(testCase.expected,);
    },
  },);
}

await describe({
  name: '',
  children: [
    describe({
      name: englishGerund.name,
      children: GERUND_CASES.map(function mapGerundCase(
        testCase: MorphologyCase,
      ): ReturnType<typeof it> {
        return morphologyCaseIt({
          testCase,
          helper: englishGerund,
        },);
      },),
    },),

    describe({
      name: englishThirdSingular.name,
      children: THIRD_SINGULAR_CASES.map(function mapThirdSingularCase(
        testCase: MorphologyCase,
      ): ReturnType<typeof it> {
        return morphologyCaseIt({
          testCase,
          helper: englishThirdSingular,
        },);
      },),
    },),
  ],
},);
