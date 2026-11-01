/**
 Tests how the coverage census reads the bundle text of one cold stretch
 (ledger T8): nothing but invariant throws, which it counts apart, or cold
 code. A stretch is an invariant throw only where every statement in it
 throws an `Error` whose message begins `unreachable:` or a class whose name
 ends in `InvariantError`; the braces, the `if` head and the `else` V8 folds
 into a stretch are structure, and anything else keeps the whole stretch
 cold. The texts are written as the unminified coverage build writes code;
 names are cat-themed invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { stretchReadingOf, } from '../../dist/final/node/index.mjs';

/**
 What the census reads of a stretch that is cold code.
 */
const COLD = { kind: 'cold', } as const;

/**
 What the census reads of a stretch that is one throw of an `Error` whose
 message begins `unreachable:`.
 */
const ONE_UNREACHABLE = {
  kind: 'invariant throws',
  thrown: ['Error',],
} as const;

await describe({
  name: stretchReadingOf.name,
  children: [
    it({
      name: 'READS A THROW OF AN ERROR WHOSE MESSAGE BEGINS "unreachable:" AS AN INVARIANT THROW, the message a string '
        + 'or a template holding substitutions, a cause beside it, a bracket or an escaped quote inside the message, '
        + 'and blank space around the statement',
      fn: async () => {
        expect([
          'throw new Error("unreachable: the cat always lands on its feet");',
          'throw new Error(\'unreachable: the basket holds a toy\');',
          `\n\tthrow new Error(\`unreachable: nap \${String(lap.warmth)} of \${baskets["wicker"]} is never \${\`cold \${depth}\`}\`);\n`,
          'throw new Error("unreachable: kneading raised something other than a purr", { cause: error });',
          String.raw`throw new Error("unreachable: the bowl ) holds \" a } quote", { cause: paw(")") });`,
          'throw new Error("unreachable: the cat is always fed")',
        ].map((text,) => stretchReadingOf({ text, },)),).toEqual([
          ONE_UNREACHABLE,
          ONE_UNREACHABLE,
          ONE_UNREACHABLE,
          ONE_UNREACHABLE,
          ONE_UNREACHABLE,
          ONE_UNREACHABLE,
        ],);
      },
    },),
    it({
      name: 'READS THE BRACES, THE IF HEAD AND THE ELSE V8 FOLDS INTO A STRETCH AS STRUCTURE: a braced throw, a throw '
        + 'behind its guarding if head, an else arm, and a brace closing a block that opened before the stretch',
      fn: async () => {
        expect([
          `{\n\t\tthrow new Error(\`unreachable: lap \${String(lap)} is always given\`);\n\t}`,
          '\n\t\tif (whisker < -5) throw new Error("unreachable: no whisker is that short");',
          ' else {\n\t\tthrow new Error("unreachable: the cat always purrs");\n\t}',
          ' else if (lap === null) throw new Error("unreachable: a lap is never null");',
          '\n\t\t\tthrow new Error("unreachable: the nap always ends");\n\t\t}',
        ].map((text,) => stretchReadingOf({ text, },)),).toEqual([
          ONE_UNREACHABLE,
          ONE_UNREACHABLE,
          ONE_UNREACHABLE,
          ONE_UNREACHABLE,
          ONE_UNREACHABLE,
        ],);
      },
    },),
    it({
      name: 'READS A THROW OF A CLASS WHOSE NAME ENDS IN InvariantError AS AN INVARIANT THROW whatever it is handed, '
        + 'naming the class',
      fn: async () => {
        expect([
          `throw new WhiskerInvariantError({ invariant: \`whisker \${String(index)} twitched with no cat under it\` });`,
          'throw new InvariantError();',
        ].map((text,) => stretchReadingOf({ text, },)),).toEqual([
          {
            kind: 'invariant throws',
            thrown: ['WhiskerInvariantError',],
          },
          {
            kind: 'invariant throws',
            thrown: ['InvariantError',],
          },
        ],);
      },
    },),
    it({
      name: 'NAMES EVERY THROW OF A STRETCH HOLDING SEVERAL, in the order the stretch holds them',
      fn: async () => {
        expect(stretchReadingOf({
          text: '{\n\t\tif (basket === void 0) throw new Error("unreachable: neither lap nor basket is given");\n'
            + '\t\tthrow new NapInvariantError({ fault: {\n\t\t\tkind: "no-lap",\n\t\t\tlaps: [lap]\n\t\t} });\n\t}',
        },),).toEqual({
          kind: 'invariant throws',
          thrown: ['Error', 'NapInvariantError',],
        },);
      },
    },),
    it({
      name: 'KEEPS A THROW OF AN ERROR COLD UNLESS ITS MESSAGE IS ONE STRING OR TEMPLATE BEGINNING "unreachable:": '
        + 'another message, the words later in the message, a template opening with a substitution, a literal '
        + 'beginning so with more joined to it, a name, no message, and the words with a capital',
      fn: async () => {
        expect([
          'throw new Error("the lap is missing");',
          'throw new Error("the lap is missing, unreachable: or so it reads");',
          `throw new Error(\`\${cat} unreachable: is asleep\`);`,
          'throw new Error("unreachable: " + cat);',
          'throw new Error(unreachableNap);',
          'throw new Error();',
          'throw new Error("Unreachable: the cat is asleep");',
        ].map((text,) => stretchReadingOf({ text, },)),).toEqual([
          COLD,
          COLD,
          COLD,
          COLD,
          COLD,
          COLD,
          COLD,
        ],);
      },
    },),
    it({
      name: 'KEEPS A THROW OF A CLASS NAMED OTHERWISE COLD, a message beginning "unreachable:" or not: a refusal, a '
        + 'built-in error other than Error, a name holding InvariantError before its end, a member of an object, '
        + 'a class a call picks, and a thrown value no class builds there',
      fn: async () => {
        expect([
          'throw new WhiskerRefusalError({ whisker });',
          'throw new TypeError("unreachable: a lap is always given");',
          'throw new InvariantErrorReport({ whisker });',
          'throw new errors.WhiskerInvariantError({ whisker });',
          'throw new (pickInvariantError())("unreachable: a lap is always given");',
          'throw error;',
          'throw nonNullishOrThrow(thrown);',
          'throw WhiskerInvariantError({ whisker });',
        ].map((text,) => stretchReadingOf({ text, },)),).toEqual([
          COLD,
          COLD,
          COLD,
          COLD,
          COLD,
          COLD,
          COLD,
          COLD,
        ],);
      },
    },),
    it({
      name: 'KEEPS A MIXED STRETCH COLD, an invariant throw beside anything else: a statement before it in its block, '
        + 'a return before or after it, the cold end of the condition guarding it, a catch head, a case label, the '
        + 'function holding it, something read off the thrown error, and an if whose condition stands in no brackets',
      fn: async () => {
        expect([
          '{\n\t\tlog("no lap");\n\t\tthrow new Error("unreachable: a lap is always given");\n\t}',
          'return toy;\n\tthrow new Error("unreachable: every basket holds a toy");\n',
          'throw new Error("unreachable: every basket holds a toy");\n\treturn toy;',
          '|| refused === cat)) throw new Error("unreachable: the cat read fed");',
          'catch (error) {\n\t\tthrow new Error("unreachable: kneading never fails", { cause: error });\n\t}',
          'default: throw new Error("unreachable: the lap is always warm");',
          'function never() {\n\t\tthrow new Error("unreachable: never called");\n\t}',
          'throw new Error("unreachable: the cat is asleep").stack;',
          'throw new Error("unreachable: the cat is asleep"), wake();',
          'if lap throw new Error("unreachable: a lap is always given");',
        ].map((text,) => stretchReadingOf({ text, },)),).toEqual([
          COLD,
          COLD,
          COLD,
          COLD,
          COLD,
          COLD,
          COLD,
          COLD,
          COLD,
          COLD,
        ],);
      },
    },),
    it({
      name: 'KEEPS A STRETCH HOLDING NO THROW COLD: no text, blank space, a brace, an else, and an if head with nothing '
        + 'thrown behind it',
      fn: async () => {
        expect([
          '',
          '\n\t',
          '}',
          ' else {\n\t}',
          'if (lap === void 0) {\n\t}',
          'throw new Error("unreachable: a lap is always given");\n\tif (basket === void 0) ',
        ].map((text,) => stretchReadingOf({ text, },)),).toEqual([
          COLD,
          COLD,
          COLD,
          COLD,
          COLD,
          COLD,
        ],);
      },
    },),
    it({
      name: 'KEEPS COLD WHAT ONE PASS CANNOT READ FOR CERTAIN: a slash that opens no comment, which is a division or '
        + 'a pattern whose quotes and brackets would be misread, here hiding a call after the throw; and a string, '
        + 'a template, a comment or a bracket that never closes',
      fn: async () => {
        expect([
          `throw new Error(\`unreachable: half of \${String(laps / 2)} laps\`);`,
          `throw new Error(\`unreachable: \${name.replace(/[")]/g, "")} is no cat\`); wake(")");`,
          'if (/^nap/.test(name)) throw new Error("unreachable: a nap is never named");',
          'throw new Error("unreachable: the cat is asleep',
          'throw new Error("unreachable: the cat is\nasleep");',
          `throw new Error(\`unreachable: the cat \${name} is asleep`,
          'throw new Error("unreachable: the cat is asleep"',
          'throw new Error("unreachable: the cat is asleep"); /* and stays so',
          'throw new Error("unreachable: the cat is asleep", { cause: paws[0 });',
        ].map((text,) => stretchReadingOf({ text, },)),).toEqual([
          COLD,
          COLD,
          COLD,
          COLD,
          COLD,
          COLD,
          COLD,
          COLD,
          COLD,
        ],);
      },
    },),
    it({
      name: 'SKIPS COMMENTS, which hold no statement: one before the throw, one inside its arguments and one after '
        + 'it on its line; a throw written only inside a comment is no throw, and a call after a line comment\'s '
        + 'line end, a rare one too, is still a call beside the throw',
      fn: async () => {
        expect([
          '\n\t/**\n\tWhy no cat reaches here: a "quote" and a ) bracket.\n\t*/\n'
            + '\tthrow new Error("unreachable: the cat is asleep", /* the cause */ { cause: error }); // never\n',
          '/* throw new Error("unreachable: the cat is asleep"); */',
          '// throw new Error("unreachable: the cat is asleep");',
          '// never\u2028wake();\n\tthrow new Error("unreachable: the cat is asleep");',
        ].map((text,) => stretchReadingOf({ text, },)),).toEqual([
          ONE_UNREACHABLE,
          COLD,
          COLD,
          COLD,
        ],);
      },
    },),
  ],
},);
