import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  classifyNameInSourceText,
  type ExportedNameKind,
  isAllCapsSnakeName,
} from '../dist/final/node/index.mjs';

//region Case tables

/**
 One name-shape case: binding name in, whether the constant-import shape holds.
 */
type NameShapeCase = {
  /**
   What the case proves about the shape walk.
   */
  readonly name: string;
  /**
   Binding name to classify.
   */
  readonly inspected: string;
  /**
   Whether the name has the all-caps snake-case shape.
   */
  readonly expected: boolean;
};

/**
 Name-shape cases covering the leading-character rule, the tail grammar,
 the empty name, and every rejected character class the anchored
 `^[A-Z][A-Z0-9_]*$` pattern distinguished.
 */
const NAME_SHAPE_CASES: readonly NameShapeCase[] = [
  {
    name: 'accepts a single uppercase letter',
    inspected: 'A',
    expected: true,
  },
  {
    name: 'accepts an all-caps snake-case constant',
    inspected: 'FEED_PAGE_SIZE',
    expected: true,
  },
  {
    name: 'accepts digits and underscores in the tail',
    inspected: 'A1_B2',
    expected: true,
  },
  {
    name: 'accepts a zero digit in the tail, the edge of the digit range',
    inspected: 'AZ09_',
    expected: true,
  },
  {
    name: 'accepts a trailing underscore',
    inspected: 'A_',
    expected: true,
  },
  {
    name: 'rejects the empty name the leading character class required',
    inspected: '',
    expected: false,
  },
  {
    name: 'rejects a lowercase leading character',
    inspected: 'a',
    expected: false,
  },
  {
    name: 'rejects camel case',
    inspected: 'FeedPageSize',
    expected: false,
  },
  {
    name: 'rejects a leading underscore',
    inspected: '_A',
    expected: false,
  },
  {
    name: 'rejects a leading digit',
    inspected: '1A',
    expected: false,
  },
  {
    name: 'rejects a hyphen in the tail',
    inspected: 'A-b',
    expected: false,
  },
  {
    name: 'rejects a space in the tail',
    inspected: 'A B',
    expected: false,
  },
  {
    name: 'rejects a dollar sign the old character class never held',
    inspected: 'A$b',
    expected: false,
  },
  {
    name: 'rejects a non-ASCII letter outside the ASCII ranges',
    inspected: 'A\u00E9',
    expected: false,
  },
  {
    name: 'rejects a non-ASCII uppercase look-alike',
    inspected: '\u00C5',
    expected: false,
  },
  {
    name: 'rejects a lone surrogate in the tail',
    inspected: 'A\uD800',
    expected: false,
  },
];

/**
 One classification case: source text and inspected name in, export tag out.
 */
type ClassificationCase = {
  /**
   What the case proves about the scans.
   */
  readonly name: string;
  /**
   Source text of the imported file the scans read.
   */
  readonly content: string;
  /**
   Name inspected in the source's export surface.
   */
  readonly inspected: string;
  /**
   Tag the classifier must report.
   */
  readonly expected: ExportedNameKind;
};

/**
 Classification cases covering every declaration head branch, the
 whitespace-run grammar, the line-start anchor, the re-export brace scan
 with its resume rules, identifier-token edges, the precedence order, and
 the defect fixes the differential probe pinned against the replaced
 patterns.
 */
const CLASSIFICATION_CASES: readonly ClassificationCase[] = [
  //region Function declarations

  {
    name: 'reports a plain function declaration as callable',
    content: 'export function target() {}',
    inspected: 'target',
    expected: 'callable',
  },
  {
    name: 'reports an async function declaration as callable',
    content: 'export async function target() {}',
    inspected: 'target',
    expected: 'callable',
  },
  {
    name: 'reports a generator declaration as callable',
    content: 'export function* target() {}',
    inspected: 'target',
    expected: 'callable',
  },
  {
    name: 'reports an async generator declaration as callable',
    content: 'export async function* target() {}',
    inspected: 'target',
    expected: 'callable',
  },
  {
    name: 'accepts spaces around the generator star',
    content: 'export function * target() {}',
    inspected: 'target',
    expected: 'callable',
  },
  {
    name: 'rejects a generator star tight against the name',
    content: 'export function*target () {}',
    inspected: 'target',
    expected: 'unknown',
  },
  {
    name: 'rejects async tight against function',
    content: 'export asyncfunction target() {}',
    inspected: 'target',
    expected: 'unknown',
  },
  {
    name: 'rejects export tight against the keyword',
    content: 'exportfunction target() {}',
    inspected: 'target',
    expected: 'unknown',
  },
  {
    name: 'accepts async on its own line',
    content: 'export async\nfunction target() {}',
    inspected: 'target',
    expected: 'callable',
  },
  {
    name: 'accepts a newline inside the whitespace run after export',
    content: 'export \n function target() {}',
    inspected: 'target',
    expected: 'callable',
  },
  {
    name: 'accepts repeated spaces around the keyword',
    content: 'export  function  target  () {}',
    inspected: 'target',
    expected: 'callable',
  },
  {
    name: 'accepts tabs as the whitespace run',
    content: 'export\tfunction\ttarget() {}',
    inspected: 'target',
    expected: 'callable',
  },
  {
    name: 'accepts a no-break space as the whitespace run',
    content: 'export\u00A0function\u00A0target() {}',
    inspected: 'target',
    expected: 'callable',
  },
  {
    name: 'accepts a line separator as the whitespace run',
    content: 'export\u2028function target() {}',
    inspected: 'target',
    expected: 'callable',
  },
  {
    name: 'accepts a byte-order mark as the whitespace run',
    content: 'export\uFEFFfunction target() {}',
    inspected: 'target',
    expected: 'callable',
  },
  {
    name: 'reports a function declaration ending at the text end as callable',
    content: 'export function target',
    inspected: 'target',
    expected: 'callable',
  },

  //endregion Function declarations

  //region Class and const declarations

  {
    name: 'reports a class declaration as callable',
    content: 'export class target {}',
    inspected: 'target',
    expected: 'callable',
  },
  {
    name: 'reports a const declaration as const',
    content: 'export const target = 1;',
    inspected: 'target',
    expected: 'const',
  },
  {
    name: 'reports a const without a space before the initializer as const',
    content: 'export const target=1;',
    inspected: 'target',
    expected: 'const',
  },
  {
    name: 'reports a const declaration ending at the text end as const',
    content: 'export const target',
    inspected: 'target',
    expected: 'const',
  },

  //endregion Class and const declarations

  //region Line-start anchor

  {
    name: 'reports a declaration on a later line as callable',
    content: 'prefix();\nexport function target() {}',
    inspected: 'target',
    expected: 'callable',
  },
  {
    name: 'accepts a CRLF line ending before the declaration',
    content: 'x\r\nexport const target = 1;',
    inspected: 'target',
    expected: 'const',
  },
  {
    name: 'rejects a lone carriage return as a line start, the anchor the replaced pattern drew',
    content: 'x\rexport const target = 1;',
    inspected: 'target',
    expected: 'unknown',
  },
  {
    name: 'rejects a second declaration sharing the line of the first',
    content: 'export const a=1; export const target=2;',
    inspected: 'target',
    expected: 'unknown',
  },
  {
    name: 'rejects a declaration behind a line-comment marker',
    content: '// export function target() {}',
    inspected: 'target',
    expected: 'unknown',
  },
  {
    name: 'accepts a declaration inside a block comment, the raw-text heuristic the pattern scan also applied',
    content: '/*\nexport function target() {}\n*/',
    inspected: 'target',
    expected: 'callable',
  },
  {
    name: 'rejects a default-export declaration head',
    content: 'export default function target() {}',
    inspected: 'target',
    expected: 'unknown',
  },

  //endregion Line-start anchor

  //region Re-export brace lists

  {
    name: 'reports a brace-list re-export as reexport',
    content: "export { target } from './m'",
    inspected: 'target',
    expected: 'reexport',
  },
  {
    name: 'reports a renamed re-export specifier as reexport',
    content: "export { helper as target } from './m'",
    inspected: 'target',
    expected: 'reexport',
  },
  {
    name: 'accepts a brace list with no spaces and a quote tight after from',
    content: "export {target}from'./m'",
    inspected: 'target',
    expected: 'reexport',
  },
  {
    name: 'accepts newlines inside the brace list',
    content: "export\n{\n  target,\n} from './m';",
    inspected: 'target',
    expected: 'reexport',
  },
  {
    name: 'accepts a mid-line re-export and a double-quoted source',
    content: 'x; export { target } from "./m"',
    inspected: 'target',
    expected: 'reexport',
  },
  {
    name: 'accepts no-break spaces inside the brace list',
    content: "export {\u00A0target\u00A0} from './m'",
    inspected: 'target',
    expected: 'reexport',
  },
  {
    name: 'accepts tabs around the from keyword',
    content: "export { target }\tfrom\t'./m'",
    inspected: 'target',
    expected: 'reexport',
  },
  {
    name: 'accepts a re-export opener inside a wider brace list',
    content: "export { export { target } from './x' } from './y'",
    inspected: 'target',
    expected: 'reexport',
  },
  {
    name: 'accepts an opener inside a longer word, the unanchored scan the pattern also applied',
    content: "reexport { target } from './m'",
    inspected: 'target',
    expected: 'reexport',
  },
  {
    name: 'accepts a closing brace in an earlier comment before the opener',
    content: "/* } */ export { target } from './m'",
    inspected: 'target',
    expected: 'reexport',
  },
  {
    name: 'reports a name held only by a comment inside the brace list, the raw-text heuristic',
    content: "export { /* target */ x } from './m'",
    inspected: 'target',
    expected: 'reexport',
  },
  {
    name: 'rejects a closing brace inside a string before the name',
    content: "export { '}' as target } from './m'",
    inspected: 'target',
    expected: 'unknown',
  },
  {
    name: 'rejects a name sitting after the closing brace',
    content: "export { x } /* target */ from './m'",
    inspected: 'target',
    expected: 'unknown',
  },
  {
    name: 'rejects a brace list without a from clause',
    content: 'export { target };',
    inspected: 'target',
    expected: 'unknown',
  },
  {
    name: 'rejects a from clause without a quoted source',
    content: 'export { target } from ./m',
    inspected: 'target',
    expected: 'unknown',
  },
  {
    name: 'rejects an uppercase FROM keyword',
    content: "export { target } FROM './m'",
    inspected: 'target',
    expected: 'unknown',
  },
  {
    name: 'rejects a type-only re-export the value scan never classified',
    content: "export type { target } from './m'",
    inspected: 'target',
    expected: 'unknown',
  },
  {
    name: 'rejects unterminated openers with no closing brace anywhere',
    content: 'export { export { export { target',
    inspected: 'target',
    expected: 'unknown',
  },

  //endregion Re-export brace lists

  //region Identifier-token edges

  {
    name: 'rejects a declaration of a longer name sharing the prefix',
    content: 'export function targetX() {}',
    inspected: 'target',
    expected: 'unknown',
  },
  {
    name: 'rejects a declaration whose name continues with a zero digit',
    content: 'export function target0() {}',
    inspected: 'target',
    expected: 'unknown',
  },
  {
    name: 'rejects a declaration of a longer name sharing the suffix',
    content: 'export function xtarget() {}',
    inspected: 'target',
    expected: 'unknown',
  },
  {
    name: 'rejects a name followed by a dollar-joined longer identifier the old word boundary accepted',
    content: 'export function target$bar() {}',
    inspected: 'target',
    expected: 'unknown',
  },
  {
    name: 'rejects a name followed by a non-ASCII letter the old word boundary accepted',
    content: 'export function target\u00E9() {}',
    inspected: 'target',
    expected: 'unknown',
  },
  {
    name: 'accepts a no-break space directly after a declared name',
    content: 'export function target\u00A0() {}',
    inspected: 'target',
    expected: 'callable',
  },
  {
    name: 'accepts a line separator directly after a declared name',
    content: 'export function target\u2028() {}',
    inspected: 'target',
    expected: 'callable',
  },
  {
    name: 'rejects a brace-list run longer than the name',
    content: "export { target0 } from './m'",
    inspected: 'target',
    expected: 'unknown',
  },

  //endregion Identifier-token edges

  //region Defect fixes for regex-meta identifiers

  {
    name: 'classifies a name holding a dollar sign the interpolated anchor never matched',
    content: 'export function a$b() {}',
    inspected: 'a$b',
    expected: 'callable',
  },
  {
    name: 'classifies a name starting with a dollar sign',
    content: 'export const $jq = 1;',
    inspected: '$jq',
    expected: 'const',
  },
  {
    name: 'classifies a non-ASCII name the trailing word boundary never matched',
    content: 'export function caf\u00E9() {}',
    inspected: 'caf\u00E9',
    expected: 'callable',
  },
  {
    name: 'classifies a non-ASCII name in a brace list',
    content: "export { caf\u00E9 } from './m'",
    inspected: 'caf\u00E9',
    expected: 'reexport',
  },
  {
    name: 'refuses a trailing-dollar name against text without it, the degenerate anchor match the old pattern produced',
    content: 'export function ab',
    inspected: 'ab$',
    expected: 'unknown',
  },
  {
    name: 'classifies the reserved-word-shaped name from',
    content: "export { from } from './m'",
    inspected: 'from',
    expected: 'reexport',
  },
  {
    name: 'classifies the reserved-word-shaped name async',
    content: 'export function async() {}',
    inspected: 'async',
    expected: 'callable',
  },

  //endregion Defect fixes for regex-meta identifiers

  //region Precedence

  {
    name: 'prefers a function declaration over a re-export appearing earlier',
    content: "export { target } from './a';\nexport function target() {}",
    inspected: 'target',
    expected: 'callable',
  },
  {
    name: 'prefers a function declaration over a const appearing earlier',
    content: 'export const target = 1;\nexport function target() {}',
    inspected: 'target',
    expected: 'callable',
  },
  {
    name: 'prefers a class declaration over a re-export appearing earlier',
    content: "export { target } from './a';\nexport class target {}",
    inspected: 'target',
    expected: 'callable',
  },
  {
    name: 'prefers a const declaration over a re-export appearing earlier',
    content: "export { target } from './a';\nexport const target = 1;",
    inspected: 'target',
    expected: 'const',
  },

  //endregion Precedence

  //region Unknown fallback

  {
    name: 'reports empty text as unknown',
    content: '',
    inspected: 'target',
    expected: 'unknown',
  },
  {
    name: 'reports text without exports as unknown',
    content: 'const target = 1;\nfunction other() {}',
    inspected: 'target',
    expected: 'unknown',
  },

  //endregion Unknown fallback
];

//endregion Case tables

//region Adversarial input

/**
 One opener unit of the adversarial input family the audit measured: an
 `export {` opener that no closing brace ever terminates.
 */
const ADVERSARIAL_OPENER_UNIT = 'export { filler filler ';

/**
 Repetitions building about one megabyte of unterminated openers.
 */
const ADVERSARIAL_OPENER_REPEATS = 43_479;

//endregion Adversarial input

await describe({
  name: '',
  children: [
    describe({
      name: isAllCapsSnakeName.name,
      children: NAME_SHAPE_CASES.map(function mapNameShapeCase({
        name,
        inspected,
        expected,
      },) {
        return it({
          name,
          fn: async function testNameShapeCase(): Promise<void> {
            expect(isAllCapsSnakeName(inspected,),).toBe(expected,);
          },
        },);
      },),
    },),
    describe({
      name: classifyNameInSourceText.name,
      children: [
        ...CLASSIFICATION_CASES.map(function mapClassificationCase({
          name,
          content,
          inspected,
          expected,
        },) {
          return it({
            name,
            fn: async function testClassificationCase(): Promise<void> {
              expect(classifyNameInSourceText({
                content,
                name: inspected,
              },),).toBe(expected,);
            },
          },);
        },),
        it({
          name: 'classifies a megabyte of unterminated re-export openers without rescanning per opener',
          fn: async function testAdversarialOpeners(): Promise<void> {
            /**
             About one megabyte of unterminated openers, the input family the audit measured.
             
             The case bounds running time by construction rather than assertion: the replaced
             pattern was quadratic here, about four times per doubling up to 1010 ms at 240 KB,
             so a rescanning regression would need tens of seconds on this input while the
             brace-resume scan finishes in milliseconds.
             */
            const adversarialContent = ADVERSARIAL_OPENER_UNIT.repeat(ADVERSARIAL_OPENER_REPEATS,);
            expect(classifyNameInSourceText({
              content: adversarialContent,
              name: 'zzz_absent',
            },),).toBe('unknown',);
          },
        },),
      ],
    },),
  ],
},);
