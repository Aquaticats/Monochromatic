/**
 Unit tests for GitHub URL argument validation helpers.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  isPrintableAscii,
  validateEndpointFragment,
  validatePositionalPathArgument,
  validateReferenceNumber,
  validateTokenArgument,
  type TokenValidation,
} from '../dist/final/node/index.mjs';

//region Fixtures

/**
 Argument name used across validation cases.
 */
const LABEL = 'release tag';

/**
 One validation case pairing an input with its complete expected result.
 */
type ValidationCase = {
  /**
   Case name.
   */
  readonly name: string;
  /**
   Candidate argument value.
   */
  readonly value: string;
  /**
   Complete expected validation result.
   */
  readonly expected: TokenValidation;
};

/**
 Successful validation fixture.
 */
const SAFE: TokenValidation = { safe: true, };

/**
 Empty-value refusal shared by every validator.
 */
const EMPTY_REFUSAL: TokenValidation = {
  safe: false,
  reason: `${LABEL} is empty`,
};

/**
 Token validation cases covering emptiness, flag-shaped input, and the GitHub naming charset.
 */
const TOKEN_CASES: readonly ValidationCase[] = [
  {
    name: 'accepts a plain login',
    value: 'cli',
    expected: SAFE,
  },
  {
    name: 'accepts dots, dashes, and underscores',
    value: 'a.b-c_d',
    expected: SAFE,
  },
  {
    name: 'refuses an empty value',
    value: '',
    expected: EMPTY_REFUSAL,
  },
  {
    name: 'refuses a value that would parse as a gh flag',
    value: '-cli',
    expected: {
      safe: false,
      reason: `${LABEL} "-cli" would parse as a gh flag`,
    },
  },
  {
    name: 'refuses a value outside the GitHub naming charset',
    value: 'ow!ner',
    expected: {
      safe: false,
      reason: `${LABEL} "ow!ner" contains characters GitHub does not allow in it`,
    },
  },
  {
    name: 'refuses a path separator inside one positional argument',
    value: 'cli/cli',
    expected: {
      safe: false,
      reason: `${LABEL} "cli/cli" contains characters GitHub does not allow in it`,
    },
  },
];

/**
 Reference number cases covering digit-only content and the accepted number width.
 */
const REFERENCE_NUMBER_CASES: readonly ValidationCase[] = [
  {
    name: 'accepts an issue number',
    value: '13118',
    expected: SAFE,
  },
  {
    name: 'accepts the widest accepted number',
    value: '123456789',
    expected: SAFE,
  },
  {
    name: 'refuses a number wider than the accepted width',
    value: '1234567890',
    expected: {
      safe: false,
      reason: `${LABEL} reference "1234567890" is not a number`,
    },
  },
  {
    name: 'refuses a form path segment',
    value: 'new',
    expected: {
      safe: false,
      reason: `${LABEL} reference "new" is not a number`,
    },
  },
  {
    name: 'refuses a signed number',
    value: '-1',
    expected: {
      safe: false,
      reason: `${LABEL} reference "-1" is not a number`,
    },
  },
  {
    name: 'refuses a decimal number',
    value: '1.2',
    expected: {
      safe: false,
      reason: `${LABEL} reference "1.2" is not a number`,
    },
  },
];

/**
 Endpoint fragment cases covering emptiness and the printable ASCII invariant.
 */
const ENDPOINT_FRAGMENT_CASES: readonly ValidationCase[] = [
  {
    name: 'accepts a repository endpoint with a query',
    value: '/repos/cli/cli?per_page=2',
    expected: SAFE,
  },
  {
    name: 'accepts a dotted tag',
    value: 'v2.101.0',
    expected: SAFE,
  },
  {
    name: 'refuses an empty fragment',
    value: '',
    expected: EMPTY_REFUSAL,
  },
  {
    name: 'refuses a fragment carrying a delete character',
    value: 'a\u007Fb',
    expected: {
      safe: false,
      reason: `${LABEL} contains characters outside printable ASCII`,
    },
  },
  {
    name: 'refuses a fragment carrying a newline',
    value: 'a\nb',
    expected: {
      safe: false,
      reason: `${LABEL} contains characters outside printable ASCII`,
    },
  },
  {
    name: 'refuses a fragment carrying a space',
    value: 'a b',
    expected: {
      safe: false,
      reason: `${LABEL} contains characters outside printable ASCII`,
    },
  },
];

/**
 Positional path argument cases covering slashed values and flag-shaped input.
 */
const POSITIONAL_PATH_CASES: readonly ValidationCase[] = [
  {
    name: 'accepts a slashed tag',
    value: 'release/1.0',
    expected: SAFE,
  },
  {
    name: 'accepts a build metadata tag',
    value: 'v1.0+build.2',
    expected: SAFE,
  },
  {
    name: 'refuses an empty tag',
    value: '',
    expected: EMPTY_REFUSAL,
  },
  {
    name: 'refuses a tag that would parse as a gh flag',
    value: '-rc1',
    expected: {
      safe: false,
      reason: `${LABEL} "-rc1" would parse as a gh flag`,
    },
  },
  {
    name: 'refuses a tag carrying a control character',
    value: 'a\u0001b',
    expected: {
      safe: false,
      reason: `${LABEL} contains characters outside printable ASCII`,
    },
  },
];

//endregion Fixtures

await describe({
  name: '',
  children: [
    //region Printable ASCII guard

    describe({
      name: isPrintableAscii.name,
      children: [
        it({
          name: 'accepts every character between exclusive space and inclusive tilde',
          fn: async () => {
            expect(isPrintableAscii('!~AZaz09/?.&=%+,;:@',),).toBe(true,);
          },
        },),
        it({
          name: 'accepts an empty value',
          fn: async () => {
            expect(isPrintableAscii('',),).toBe(true,);
          },
        },),
        it({
          name: 'refuses a space',
          fn: async () => {
            expect(isPrintableAscii('a b',),).toBe(false,);
          },
        },),
        it({
          name: 'refuses a control character',
          fn: async () => {
            expect(isPrintableAscii('a\u0000b',),).toBe(false,);
          },
        },),
        it({
          name: 'refuses a delete character',
          fn: async () => {
            expect(isPrintableAscii('a\u007Fb',),).toBe(false,);
          },
        },),
        it({
          name: 'refuses a non-ASCII character',
          fn: async () => {
            expect(isPrintableAscii('a\u00E9b',),).toBe(false,);
          },
        },),
        it({
          name: 'refuses a supplementary plane character as one whole code point',
          fn: async () => {
            expect(isPrintableAscii('a\u{1F600}b',),).toBe(false,);
          },
        },),
      ],
    },),

    //endregion Printable ASCII guard

    //region Argument validators

    describe({
      name: validateTokenArgument.name,
      children: TOKEN_CASES
        .map(function mapTokenCase(tokenCase: ValidationCase,) {
          return it({
            name: tokenCase.name,
            fn: async () => {
              expect(validateTokenArgument({
                value: tokenCase.value,
                label: LABEL,
              },),).toEqual(tokenCase.expected,);
            },
          },);
        },),
    },),
    describe({
      name: validateReferenceNumber.name,
      children: REFERENCE_NUMBER_CASES
        .map(function mapReferenceCase(referenceCase: ValidationCase,) {
          return it({
            name: referenceCase.name,
            fn: async () => {
              expect(validateReferenceNumber({
                value: referenceCase.value,
                label: LABEL,
              },),).toEqual(referenceCase.expected,);
            },
          },);
        },),
    },),
    describe({
      name: validateEndpointFragment.name,
      children: ENDPOINT_FRAGMENT_CASES
        .map(function mapFragmentCase(fragmentCase: ValidationCase,) {
          return it({
            name: fragmentCase.name,
            fn: async () => {
              expect(validateEndpointFragment({
                value: fragmentCase.value,
                label: LABEL,
              },),).toEqual(fragmentCase.expected,);
            },
          },);
        },),
    },),
    describe({
      name: validatePositionalPathArgument.name,
      children: POSITIONAL_PATH_CASES
        .map(function mapPositionalCase(positionalCase: ValidationCase,) {
          return it({
            name: positionalCase.name,
            fn: async () => {
              expect(validatePositionalPathArgument({
                value: positionalCase.value,
                label: LABEL,
              },),).toEqual(positionalCase.expected,);
            },
          },);
        },),
    },),

    //endregion Argument validators
  ],
},);
