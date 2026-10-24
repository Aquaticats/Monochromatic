import { textOf, } from './claude-schema-call-forms.test-fixture.ts';
import {
  ARGUMENTS,
  CONTEST_IMPORT,
  spotAt,
} from './claude-schema-destination-forms.test-fixture.ts';
import type { ShapeCase, } from './claude-schema-unread-shape-forms.test-fixture.ts';

//region A builder's result in a substitution of a tagged template
// The case of the third total for a template tagged with a function. The substitutions of a tagged template are the
// arguments of the tag function, which can keep them, so a value that goes into one is reported like an argument of any
// call; the substitutions of a plain template are read into a string, which carries nothing the schema could travel in,
// so a value that goes into one ends the walk without a report.

/**
 The character that opens a substitution in a template, which an imaginary file's text spells out beside a brace.
 */
const DOLLAR = '$';

/**
 The position the check gives a value that is a substitution of a tagged template.
 */
const TAGGED_SUBSTITUTION = 'TaggedTemplateExpression.quasi';

/**
 The import of the sender, as a specifier written in a file under `imaginary/src`.
 */
const SENDER_IMPORT =
  'import { gatherThroughCache, } from \'../../corpus-pass/src/corpus-run/pass-consistency-gather.ts\';';

/**
 The tag function the imaginary files declare, which returns its one substitution.
 */
const TAG_DECLARATION = 'const tag = (_strings: TemplateStringsArray, value: unknown,) => value;';

/**
 The case of a value in a tagged template and in a plain one.
 */
export const TAGGED_TEMPLATE_CASES: readonly ShapeCase[] = [
  {
    name: 'REPORTS A BUILDER\'S RESULT IN A SUBSTITUTION OF A TAGGED TEMPLATE, tagged by a name or by a member, nested in '
      + 'another tagged template, held by a constant at the top level, or the object a sender takes, and nothing for a '
      + 'substitution of a plain template: a tag function is a call that can keep what it is given',
    files: [
      {
        file: 'corpus-pass/src/corpus-run/pass-consistency-gather.ts',
        text: textOf([
          'export async function gatherThroughCache({ responseFormat, }: { readonly responseFormat: unknown; },) {',
          '  return responseFormat;',
          '}',
        ],),
      },
      {
        file: 'imaginary/src/tagged-stage.ts',
        text: textOf([
          CONTEST_IMPORT,
          '',
          TAG_DECLARATION,
          '',
          'export function tagged() {',
          `  return tag\`${DOLLAR}{contestResponseFormat(${ARGUMENTS},)}\`;`,
          '}',
          '',
          'export function raw() {',
          `  return String.raw\`${DOLLAR}{contestResponseFormat(${ARGUMENTS},)}\`;`,
          '}',
          '',
          'export function nested() {',
          `  return tag\`a${DOLLAR}{tag\`b${DOLLAR}{contestResponseFormat(${ARGUMENTS},)}\`}\`;`,
          '}',
          '',
          'export function member() {',
          `  return (console as any).tag\`${DOLLAR}{contestResponseFormat(${ARGUMENTS},)}\`;`,
          '}',
          '',
          'export function plain() {',
          `  return \`${DOLLAR}{contestResponseFormat(${ARGUMENTS},)}\`;`,
          '}',
        ],),
      },
      {
        file: 'imaginary/src/tagged-constant-stage.ts',
        text: textOf([
          CONTEST_IMPORT,
          '',
          TAG_DECLARATION,
          '',
          `const TAGGED = contestResponseFormat(${ARGUMENTS},);`,
          '',
          `export const KEPT = tag\`${DOLLAR}{TAGGED}\`;`,
        ],),
      },
      {
        file: 'imaginary/src/tagged-sender-stage.ts',
        text: textOf([
          CONTEST_IMPORT,
          SENDER_IMPORT,
          '',
          'export async function tagged() {',
          `  return await gatherThroughCache\`${DOLLAR}{{ responseFormat: contestResponseFormat(${ARGUMENTS},), }}\`;`,
          '}',
          '',
          'export async function plain() {',
          `  return await gatherThroughCache(\`${DOLLAR}{{ responseFormat: contestResponseFormat(${ARGUMENTS},), }}\`);`,
          '}',
        ],),
      },
    ],
    found: [
      'imaginary/src/tagged-constant-stage.ts',
      'imaginary/src/tagged-sender-stage.ts',
      'imaginary/src/tagged-stage.ts',
    ],
    builders: [],
    spots: [
      spotAt({
        file: 'imaginary/src/tagged-stage.ts',
        line: 6,
        position: TAGGED_SUBSTITUTION,
      },),
      spotAt({
        file: 'imaginary/src/tagged-stage.ts',
        line: 10,
        position: TAGGED_SUBSTITUTION,
      },),
      spotAt({
        file: 'imaginary/src/tagged-stage.ts',
        line: 14,
        position: TAGGED_SUBSTITUTION,
      },),
      spotAt({
        file: 'imaginary/src/tagged-stage.ts',
        line: 18,
        position: TAGGED_SUBSTITUTION,
      },),
      spotAt({
        file: 'imaginary/src/tagged-constant-stage.ts',
        line: 5,
        position: TAGGED_SUBSTITUTION,
      },),
      spotAt({
        file: 'imaginary/src/tagged-sender-stage.ts',
        line: 5,
        position: TAGGED_SUBSTITUTION,
      },),
    ],
  },
];

//endregion A builder's result in a substitution of a tagged template
