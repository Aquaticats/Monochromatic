/**
 Tests for model-id tokenization and date classification.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  classifyToken,
  isDateShapedRaw,
  NO_NUMERIC_RUN,
  parseModelId,
  readNumericRun,
  splitOnNonTokenCharacters,
  stripOrganizationPrefix,
} from '../dist/final/node/index.mjs';

//region Fixtures

/** Token classification case. */
type TokenCase = {
  /** Token under test. */
  token: string;
  /** Expected name-shape contribution, absent when the whole token is a version. */
  word?: string;
  /** Expected raw version components. */
  raws: readonly string[];
};

/**
 Classification controls covering every branch of {@link classifyToken}:
 whole-token numerics, `v` prefixes, alphabetic prefixes with dotted digits,
 trailing letters, parameter sizes, and an empty token.
 */
const TOKEN_CASES: readonly TokenCase[] = [
  { token: '5.2', raws: ['5', '2'], },
  { token: '5', raws: ['5'], },
  { token: 'v4', raws: ['4'], },
  { token: 'v4.1', raws: ['4', '1'], },
  { token: '0813', raws: ['0813'], },
  { token: '20260420', raws: ['20260420'], },
  { token: 'qwen3.8', word: 'qwen', raws: ['3', '8'], },
  { token: 'k2.7', word: 'k', raws: ['2', '7'], },
  { token: 'nvfp4', word: 'nvfp', raws: ['4'], },
  { token: 'o1', word: 'o', raws: ['1'], },
  { token: '4o', word: '4o', raws: [], },
  { token: '70b', word: '70b', raws: [], },
  { token: 'a12b', word: 'a12b', raws: [], },
  { token: '2.4t', word: '2.4t', raws: [], },
  { token: 'flash', word: 'flash', raws: [], },
  { token: 'meta.llama3', word: 'meta.llama3', raws: [], },
  { token: '', raws: [], },
  { token: '3.', word: '3.', raws: [], },
];

/** Date classification case. */
type DateCase = {
  /** Raw version component under test. */
  raw: string;
  /** Whether the component reads as a calendar date. */
  expected: boolean;
};

/**
 Date controls for every accepted width and for the near misses that must stay
 version numbers, including the leading-zero pair `0813` and `813`.
 */
const DATE_CASES: readonly DateCase[] = [
  { raw: '0813', expected: true, },
  { raw: '0528', expected: true, },
  { raw: '2512', expected: true, },
  { raw: '2407', expected: true, },
  { raw: '0324', expected: true, },
  { raw: '250715', expected: true, },
  { raw: '20260420', expected: true, },
  { raw: '20250805', expected: true, },
  { raw: '813', expected: false, },
  { raw: '2024', expected: false, },
  { raw: '13', expected: false, },
  { raw: '5', expected: false, },
  { raw: '9999', expected: false, },
  { raw: '20261332', expected: false, },
  { raw: '08x3', expected: false, },
];

/** Model-id parse case. */
type ParseCase = {
  /** Model id under test. */
  modelId: string;
  /** Expected name shape. */
  nameShape: string;
  /** Expected version components. */
  versionParts: readonly number[];
  /** Expected raw version text. */
  versionRaws: readonly string[];
  /** Expected date-shaped raw text. */
  dateRaws: readonly string[];
};

/**
 Parse controls pinning the shapes that decide retirements: tier words and
 parameter sizes stay in the name shape, alias markers and Hugging Face
 organizations are dropped, and glued versions split.
 */
const PARSE_CASES: readonly ParseCase[] = [
  {
    modelId: 'glm-5.2',
    nameShape: 'glm',
    versionParts: [5, 2],
    versionRaws: ['5', '2'],
    dateRaws: [],
  },
  {
    modelId: 'glm-5.3-flash',
    nameShape: 'glm-flash',
    versionParts: [5, 3],
    versionRaws: ['5', '3'],
    dateRaws: [],
  },
  {
    modelId: 'deepseek-v4-pro-0813',
    nameShape: 'deepseek-pro',
    versionParts: [4, 813],
    versionRaws: ['4', '0813'],
    dateRaws: ['0813'],
  },
  {
    modelId: 'hf:zai-org/GLM-5.3',
    nameShape: 'glm',
    versionParts: [5, 3],
    versionRaws: ['5', '3'],
    dateRaws: [],
  },
  {
    modelId: '~anthropic/claude-opus-latest',
    nameShape: 'anthropic-claude-opus-latest',
    versionParts: [],
    versionRaws: [],
    dateRaws: [],
  },
  {
    modelId: 'openai/gpt-4o',
    nameShape: 'openai-gpt-4o',
    versionParts: [],
    versionRaws: [],
    dateRaws: [],
  },
  {
    modelId: 'qwen3.8-flash',
    nameShape: 'qwen-flash',
    versionParts: [3, 8],
    versionRaws: ['3', '8'],
    dateRaws: [],
  },
  {
    modelId: 'qwen3.7-flash',
    nameShape: 'qwen-flash',
    versionParts: [3, 7],
    versionRaws: ['3', '7'],
    dateRaws: [],
  },
  {
    modelId: 'meta.llama3-1-70b-instruct-v1:0',
    nameShape: 'meta.llama3-70b-instruct',
    versionParts: [1, 1, 0],
    versionRaws: ['1', '1', '0'],
    dateRaws: [],
  },
  {
    modelId: 'llama-3.1-8b-instruct',
    nameShape: 'llama-8b-instruct',
    versionParts: [3, 1],
    versionRaws: ['3', '1'],
    dateRaws: [],
  },
  {
    modelId: 'llama-3.1-70b-instruct',
    nameShape: 'llama-70b-instruct',
    versionParts: [3, 1],
    versionRaws: ['3', '1'],
    dateRaws: [],
  },
  {
    modelId: 'gpt-4o-2024-11-20',
    nameShape: 'gpt-4o',
    versionParts: [2_024, 11, 20],
    versionRaws: ['2024', '11', '20'],
    dateRaws: [],
  },
  {
    modelId: 'anthropic.claude-opus-4-1-20250805-v1:0',
    nameShape: 'anthropic.claude-opus',
    versionParts: [4, 1, 20_250_805, 1, 0],
    versionRaws: ['4', '1', '20250805', '1', '0'],
    dateRaws: ['20250805'],
  },
];

/** Organization prefix case. */
type PrefixCase = {
  /** Model id under test. */
  modelId: string;
  /** Expected id after prefix stripping. */
  expected: string;
};

/** Prefix controls for the alias marker and the Hugging Face organization segment. */
const PREFIX_CASES: readonly PrefixCase[] = [
  { modelId: 'hf:zai-org/GLM-5.3', expected: 'glm-5.3', },
  { modelId: 'hf:moonshotai/Kimi-K3', expected: 'kimi-k3', },
  { modelId: '~anthropic/claude-opus-latest', expected: 'anthropic/claude-opus-latest', },
  { modelId: 'hf:orgless', expected: 'hf:orgless', },
  { modelId: 'gpt-6-luna', expected: 'gpt-6-luna', },
];

//endregion Fixtures

await describe({
  name: '',
  children: [
    describe({
      name: classifyToken.name,
      children: TOKEN_CASES.map(function createTokenCase(testCase,) {
        return it({
          name: `classifies ${testCase.token === '' ? 'the empty token' : testCase.token}`,
          fn: async function runTokenCase() {
            expect(classifyToken(testCase.token,),).toEqual({
              ...(testCase.word === undefined ? {} : { word: testCase.word, }),
              raws: testCase.raws,
            },);
          },
        },);
      },),
    },),
    describe({
      name: isDateShapedRaw.name,
      children: DATE_CASES.map(function createDateCase(testCase,) {
        return it({
          name: `returns ${String(testCase.expected,)} for ${testCase.raw}`,
          fn: async function runDateCase() {
            expect(isDateShapedRaw(testCase.raw,),).toBe(testCase.expected,);
          },
        },);
      },),
    },),
    describe({
      name: stripOrganizationPrefix.name,
      children: PREFIX_CASES.map(function createPrefixCase(testCase,) {
        return it({
          name: `returns ${testCase.expected} for ${testCase.modelId}`,
          fn: async function runPrefixCase() {
            expect(stripOrganizationPrefix(testCase.modelId,),).toBe(testCase.expected,);
          },
        },);
      },),
    },),
    describe({
      name: parseModelId.name,
      children: PARSE_CASES.map(function createParseCase(testCase,) {
        return it({
          name: `parses ${testCase.modelId}`,
          fn: async function runParseCase() {
            expect(parseModelId(testCase.modelId,),).toEqual({
              nameShape: testCase.nameShape,
              versionParts: testCase.versionParts,
              versionRaws: testCase.versionRaws,
              dateRaws: testCase.dateRaws,
            },);
          },
        },);
      },),
    },),
    describe({
      name: splitOnNonTokenCharacters.name,
      children: [
        it({
          name: 'splits on hyphens and keeps dotted versions together',
          fn: async function runHyphenSplit() {
            expect(splitOnNonTokenCharacters('glm-5.2',),).toEqual(['glm', '5.2'],);
          },
        },),
        it({
          name: 'splits on colons and slashes',
          fn: async function runSeparatorSplit() {
            expect(splitOnNonTokenCharacters('hf:zai-org/glm-5.3',),).toEqual([
              'hf',
              'zai',
              'org',
              'glm',
              '5.3',
            ],);
          },
        },),
        it({
          name: 'drops empty runs',
          fn: async function runEmptyRuns() {
            expect(splitOnNonTokenCharacters('',),).toEqual([],);
            expect(splitOnNonTokenCharacters('--',),).toEqual([],);
            expect(splitOnNonTokenCharacters('-a-',),).toEqual(['a'],);
          },
        },),
        it({
          name: 'keeps a trailing dot inside its token',
          fn: async function runTrailingDot() {
            expect(splitOnNonTokenCharacters('3.',),).toEqual(['3.'],);
          },
        },),
      ],
    },),
    describe({
      name: readNumericRun.name,
      children: [
        it({
          name: 'reads a dotted run to the end of the token',
          fn: async function runDottedRun() {
            expect(readNumericRun({ text: '3.8', start: 0, },),).toEqual({
              end: 3,
              raws: ['3', '8'],
            },);
          },
        },),
        it({
          name: 'reads a run starting after an alphabetic prefix',
          fn: async function runOffsetRun() {
            expect(readNumericRun({ text: 'qwen3.8', start: 4, },),).toEqual({
              end: 7,
              raws: ['3', '8'],
            },);
          },
        },),
        it({
          name: 'preserves leading zeros in the raw text',
          fn: async function runLeadingZeros() {
            expect(readNumericRun({ text: '0813', start: 0, },),).toEqual({
              end: 4,
              raws: ['0813'],
            },);
          },
        },),
        it({
          name: 'stops at a trailing letter',
          fn: async function runTrailingLetter() {
            expect(readNumericRun({ text: '70b', start: 0, },),).toEqual({
              end: 2,
              raws: ['70'],
            },);
          },
        },),
        it({
          name: 'reports no run when no digit starts it',
          fn: async function runNoDigits() {
            expect(readNumericRun({ text: 'abc', start: 0, },),).toBe(NO_NUMERIC_RUN,);
          },
        },),
        it({
          name: 'reports no run when a dot is not followed by a digit',
          fn: async function runDanglingDot() {
            expect(readNumericRun({ text: '3.', start: 0, },),).toBe(NO_NUMERIC_RUN,);
          },
        },),
        it({
          name: 'reads a run behind a version marker',
          fn: async function runMarkedRun() {
            expect(readNumericRun({ text: 'v4', start: 1, },),).toEqual({
              end: 2,
              raws: ['4'],
            },);
          },
        },),
      ],
    },),
    describe({
      name: 'family separation',
      children: [
        it({
          name: 'keeps parameter sizes in separate families',
          fn: async function runSizeSeparation() {
            expect(parseModelId('llama-3.1-8b-instruct',).nameShape,).not.toBe(
              parseModelId('llama-3.1-70b-instruct',).nameShape,
            );
          },
        },),
        it({
          name: 'keeps tier variants in separate families',
          fn: async function runTierSeparation() {
            expect(parseModelId('glm-5.3-flash',).nameShape,).not.toBe(
              parseModelId('glm-5.3',).nameShape,
            );
          },
        },),
        it({
          name: 'puts glued versions of the same tier in one family',
          fn: async function runGluedVersionGrouping() {
            expect(parseModelId('qwen3.7-flash',).nameShape,).toBe(
              parseModelId('qwen3.8-flash',).nameShape,
            );
          },
        },),
      ],
    },),
  ],
},);
