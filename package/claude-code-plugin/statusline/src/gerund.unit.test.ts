import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  findGerundInText,
  scanGerundMatches,
} from '../dist/final/node/statusline.mjs';

//region Case tables

/**
 One scanner case: lowercased text in, every gerund it holds out.
 */
type ScanCase = {
  /**
   What the case proves about the scanner.
   */
  readonly name: string;
  /**
   Already lowercased text, the form the scanner is documented to take.
   */
  readonly text: string;
  /**
   Gerunds in order of appearance.
   */
  readonly expected: readonly string[];
};

/**
 Scanner cases covering both match shapes, word edges, and the resume rule after a match.
 */
const SCAN_CASES: readonly ScanCase[] = [
  {
    name: 'reports every gerund of a sentence in order of appearance',
    text: 'compiling, then testing',
    expected: [
      'compiling',
      'testing',
    ],
  },
  {
    name: 'keeps a hyphenated compound together',
    text: 'lint-missing',
    expected: ['lint-missing',],
  },
  {
    name: 'reports every hyphenated compound of a sentence holding several',
    text: 're-running the type-checking pass',
    expected: [
      're-running',
      'type-checking',
    ],
  },
  {
    name: 'prefers the longer compound when the letters after a hyphen are a gerund',
    text: 'running-ing',
    expected: ['running-ing',],
  },
  {
    name: 'stops before a hyphen when the letters after it are not a gerund',
    text: 'running-x',
    expected: ['running',],
  },
  {
    name: 'resumes after a match instead of rescanning the letters inside it',
    text: 'foo-ing-bar',
    expected: ['foo-ing',],
  },
  {
    name: 'keeps a compound whose inner word could have matched on its own',
    text: 'foo-testing',
    expected: ['foo-testing',],
  },
  {
    name: 'starts a compound at the word after one that held no gerund',
    text: 'foo-bar-testing',
    expected: ['bar-testing',],
  },
  {
    name: 'falls back to the letters before a hyphen, then compounds the next word',
    text: 'testing-foo-compiling',
    expected: [
      'testing',
      'foo-compiling',
    ],
  },
  {
    name: 'reports two compounds separated by a word that held no gerund',
    text: 'foo-ing-bar-testing',
    expected: [
      'foo-ing',
      'bar-testing',
    ],
  },
  {
    name: 'treats a zero-width space as a word edge, since it is not a word character',
    text: 'testing\u200Bcompiling',
    expected: [
      'testing',
      'compiling',
    ],
  },
  {
    name: 'reports the whole word when a gerund ends it',
    text: 'xrunning',
    expected: ['xrunning',],
  },
  {
    name: 'reports words the noise filter drops later',
    text: 'bring string king',
    expected: [
      'bring',
      'string',
      'king',
    ],
  },
  {
    name: 'reports a stem shorter than the length filter accepts',
    text: 'aing',
    expected: ['aing',],
  },
  {
    name: 'reports a compound whose stem is a single letter',
    text: 'a-ing b-ing',
    expected: [
      'a-ing',
      'b-ing',
    ],
  },
  {
    name: 'treats a newline as a word edge',
    text: 'testing\ncompiling',
    expected: [
      'testing',
      'compiling',
    ],
  },
  {
    name: 'rejects a gerund fused to a digit',
    text: 'compiling9',
    expected: [],
  },
  {
    name: 'rejects a gerund fused to an underscore on either side',
    text: '_compiling then compiling_',
    expected: [],
  },
  {
    name: 'rejects a digit before a gerund-looking word',
    text: '9compiling',
    expected: [],
  },
  {
    name: 'needs a stem letter before the suffix',
    text: 'ing',
    expected: [],
  },
  {
    name: 'needs a stem letter before a suffix that follows punctuation',
    text: 'dot.ing',
    expected: [],
  },
  {
    name: 'allows only one hyphen inside a compound',
    text: 'run--ing',
    expected: [],
  },
  {
    name: 'expects already lowercased text, so a capital letter starts no candidate',
    text: 'Compiling',
    expected: [],
  },
  {
    name: 'reports nothing for empty text',
    text: '',
    expected: [],
  },
] as const;

/**
 One extractor case: prose in, displayed word out.
 */
type WordCase = {
  /**
   What the case proves about the extractor.
   */
  readonly name: string;
  /**
   Prose as a transcript would hold it, in any case.
   */
  readonly text: string;
  /**
   Capitalized word, empty when nothing survives the filters.
   */
  readonly expected: string;
};

/**
 Extractor cases covering case folding, ordering, and both filters.
 */
const WORD_CASES: readonly WordCase[] = [
  {
    name: 'lowercases for matching and capitalizes the result',
    text: 'I am COMPILING',
    expected: 'Compiling',
  },
  {
    name: 'capitalizes only the first letter of a compound',
    text: 'cherry-picking',
    expected: 'Cherry-picking',
  },
  {
    name: 'keeps the last surviving gerund of the text',
    text: 'refactoring then testing',
    expected: 'Testing',
  },
  {
    name: 'drops a match shorter than the minimum length',
    text: 'aing',
    expected: '',
  },
  {
    name: 'keeps a compound that reaches the minimum length',
    text: 'a-ing',
    expected: 'A-ing',
  },
  {
    name: 'drops noise words',
    text: 'something interesting',
    expected: '',
  },
  {
    name: 'drops hyphenated noise words',
    text: 'the missing lint-missing',
    expected: '',
  },
  {
    name: 'reaches past noise words to the next surviving gerund',
    text: 'waiting while compiling',
    expected: 'Compiling',
  },
  {
    name: 'drops filler verbs that name no activity',
    text: 'thinking about using something',
    expected: '',
  },
  {
    name: 'reports nothing when no word carries the suffix',
    text: 'no gerund here',
    expected: '',
  },
  {
    name: 'reports nothing for empty text',
    text: '',
    expected: '',
  },
  {
    name: 'reports nothing for whitespace',
    text: '   ',
    expected: '',
  },
] as const;

/**
 Count of filler repetitions that makes a text long enough for a per-character recursion to overflow
 the stack, which is what the linear scan must never do.
 */
const LONG_TEXT_FILLER_REPEATS = 20_000;

//endregion Case tables

//region Suites

/**
 Filler word holding no gerund, repeated to build the long text.
 */
const LONG_TEXT_FILLER = 'nonsense word ';

await describe({
  name: '',
  children: [
    describe({
      name: scanGerundMatches.name,
      children: [
        ...SCAN_CASES.map(function mapScanCase({
          name,
          text,
          expected,
        },) {
          return it({
            name,
            fn: async function testScanCase(): Promise<void> {
              expect(scanGerundMatches(text,),).toEqual(expected,);
            },
          },);
        },),
        it({
          name: 'scans a transcript-sized text without recursing once per character',
          fn: async function testLongText(): Promise<void> {
            /**
             Text whose only gerund sits at the very end, after hundreds of thousands of characters.
             
             The case bounds recursion depth, not running time. A scanner that recursed once per
             character raises `RangeError: Maximum call stack size exceeded` on this input, measured
             on a recursion of exactly that shape; one that halved its input per step would stay
             inside the stack and is not caught here.
             */
            const longText = `${LONG_TEXT_FILLER.repeat(LONG_TEXT_FILLER_REPEATS,)}compiling at the end`;
            expect(scanGerundMatches(longText,),).toEqual(['compiling',],);
          },
        },),
      ],
    },),
    describe({
      name: findGerundInText.name,
      children: WORD_CASES.map(function mapWordCase({
        name,
        text,
        expected,
      },) {
        return it({
          name,
          fn: async function testWordCase(): Promise<void> {
            expect(findGerundInText(text,),).toBe(expected,);
          },
        },);
      },),
    },),
  ],
},);

//endregion Suites
