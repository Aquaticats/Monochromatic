import { textOf, } from './claude-schema-call-forms.test-fixture.ts';
import { CONTEST_IMPORT, } from './claude-schema-destination-forms.test-fixture.ts';
import {
  SENDER_FILE,
  SENDER_IMPORT,
} from './claude-schema-forwarder-forms.test-fixture.ts';
import type { ShapeCase, } from './claude-schema-unread-shape-forms.test-fixture.ts';

//region Names made from a function's parameters
// A function that hands a sender a builder call made from its own parameters takes the arguments of the schema from its
// callers, so the files that call it are listed. The names made from a parameter are the names the parameters bind, the
// `arguments` object, and the names a constant, an assignment, a pattern an assignment takes apart, a loop over a value
// or a `catch` clause binds from code that reads one of them, to the end of the chain. A chain of members whose
// member is computed starts with the same name as one whose member is not. The names a value stored in an object, a
// list or a map is read through are read too (`claude-schema-derived-container-forms.test-fixture.ts`). What stays
// unread, and is pinned silent here: a value that goes through a field of `this` (`this.a = args`), since the field is
// read by whatever else holds the object, and a value that a called function throws.

/**
 The arguments of a call of the function a case declares, as the file that calls it writes them.
 */
export const DERIVED_ARGUMENTS = '{ schemaName: \'derived\', verdicts: [], asksArchive: false, }';

/**
 A case whose stage file declares a function and whose caller file calls it.

 @param name - name of the case

 @param signature - function's signature as written, from its name to the opening brace

 @param body - lines of the function's body, the last of which hands the sender a builder call

 @param call - expression the caller file keeps

 @param isListed - whether the caller file is listed with the stage file

 @returns Case that reports nothing

 @example
 ```ts
 const shapeCase = derivedCase({ name: 'x', signature: 'mk(a: any,)', body: [], call: 'mk(1,)', isListed: true, },);
 ```
 */
export function derivedCase(
  {
    name,
    signature,
    body,
    call,
    isListed,
  }: {
    readonly name: string;
    readonly signature: string;
    readonly body: readonly string[];
    readonly call: string;
    readonly isListed: boolean;
  },
): ShapeCase {
  return {
    name,
    files: [
      SENDER_FILE,
      {
        file: 'imaginary/src/derived-stage.ts',
        text: textOf([
          CONTEST_IMPORT,
          SENDER_IMPORT,
          `export async function ${signature} {`,
          ...body,
          '}',
        ],),
      },
      {
        file: 'imaginary/src/derived-caller.ts',
        text: textOf([
          'import { mk, } from \'./derived-stage.ts\';',
          `export const SENT = ${call};`,
        ],),
      },
    ],
    found: isListed
      ? [
        'imaginary/src/derived-caller.ts',
        'imaginary/src/derived-stage.ts',
      ]
      : ['imaginary/src/derived-stage.ts',],
    builders: [],
    spots: [],
  };
}

/**
 The line that hands the sender a builder call made from the given expression.

 @param argument - argument of the builder call

 @returns Line of a function body

 @example
 ```ts
 const line = sendsFrom({ argument: 'a', },);
 ```
 */
export function sendsFrom({ argument, }: { readonly argument: string; },): string {
  return `  return await gatherThroughCache({ responseFormat: (contestResponseFormat as any)(${argument},), },);`;
}

/**
 Cases of names made from a function's parameters.
 */
export const DERIVED_NAME_CASES: readonly ShapeCase[] = [
  derivedCase({
    name: 'LISTS THE CALLER OF A FUNCTION THAT BUILDS FROM A MEMBER OF ITS PARAMETER COMPUTED FROM A NUMBER',
    signature: 'mk(list: any,)',
    body: [sendsFrom({ argument: 'list[0]', },),],
    call: `mk([${DERIVED_ARGUMENTS},],)`,
    isListed: true,
  },),
  derivedCase({
    name: 'LISTS THE CALLER OF A FUNCTION THAT BUILDS FROM A MEMBER OF A NAME A FOR-IN LOOP BINDS, a key of its '
      + 'parameter',
    signature: 'mk(rec: any,)',
    body: [
      '  for (const key in rec) {',
      '    return await gatherThroughCache({ responseFormat: (contestResponseFormat as any)(rec[key],), },);',
      '  }',
    ],
    call: `mk({ a: ${DERIVED_ARGUMENTS}, },)`,
    isListed: true,
  },),
  derivedCase({
    name: 'LISTS THE CALLER OF A FUNCTION THAT BUILDS FROM THE ARGUMENTS OBJECT, read by index',
    signature: 'mk()',
    body: [
      sendsFrom({ argument: 'arguments[0]', },),
    ],
    call: `mk(${DERIVED_ARGUMENTS},)`,
    isListed: true,
  },),
  derivedCase({
    name: 'LISTS THE CALLER OF A FUNCTION THAT BUILDS FROM A NAME ASSIGNED FROM ITS PARAMETER, not declared from it',
    signature: 'mk(args: any,)',
    body: [
      '  let a: any;',
      '  a = args;',
      sendsFrom({ argument: 'a', },),
    ],
    call: `mk(${DERIVED_ARGUMENTS},)`,
    isListed: true,
  },),
  derivedCase({
    name: 'LISTS THE CALLER OF A FUNCTION THAT BUILDS FROM A NAME A FOR-OF LOOP OVER ITS PARAMETER BINDS',
    signature: 'mk(list: any[],)',
    body: [
      '  for (const a of list) {',
      '    return await gatherThroughCache({ responseFormat: (contestResponseFormat as any)(a,), },);',
      '  }',
    ],
    call: `mk([${DERIVED_ARGUMENTS},],)`,
    isListed: true,
  },),
  derivedCase({
    name: 'LISTS THE CALLER OF A FUNCTION THAT BUILDS FROM THE PARAMETER OF A CATCH CLAUSE THAT CATCHES ITS PARAMETER',
    signature: 'mk(args: any,)',
    body: [
      '  try {',
      '    throw args;',
      '  } catch (caught) {',
      '    return await gatherThroughCache({ responseFormat: (contestResponseFormat as any)(caught,), },);',
      '  }',
    ],
    call: `mk(${DERIVED_ARGUMENTS},)`,
    isListed: true,
  },),
  derivedCase({
    name: 'LISTS THE CALLER OF A FUNCTION THAT BUILDS FROM A NAME A PATTERN ASSIGNMENT TAKES FROM ITS PARAMETER, in an '
      + 'array pattern and in an object pattern',
    signature: 'mk(args: any,)',
    body: [
      '  let a: any;',
      '  let b: any;',
      '  [a] = [args];',
      '  ({ b } = { b: args, });',
      '  return await gatherThroughCache({ responseFormat: (contestResponseFormat as any)(a, b,), },);',
    ],
    call: `mk(${DERIVED_ARGUMENTS},)`,
    isListed: true,
  },),
  derivedCase({
    name: 'DOES NOT LIST THE CALLER OF A FUNCTION THAT BUILDS FROM A FIELD OF THIS THAT ITS PARAMETER WAS STORED IN, a '
      + 'limit of the reading',
    signature: 'mk(this: any, args: any,)',
    body: [
      '  this.a = args;',
      sendsFrom({ argument: 'this.a', },),
    ],
    call: `mk(${DERIVED_ARGUMENTS},)`,
    isListed: false,
  },),
  derivedCase({
    name: 'DOES NOT LIST THE CALLER OF A FUNCTION THAT BUILDS FROM A VALUE THAT NO NAME OF ITS PARAMETERS REACHES',
    signature: 'mk(args: any,)',
    body: [
      '  let other: any;',
      '  other = 1;',
      '  void args;',
      sendsFrom({ argument: 'other', },),
    ],
    call: `mk(${DERIVED_ARGUMENTS},)`,
    isListed: false,
  },),
];

//endregion Names made from a function's parameters
