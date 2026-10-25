import { textOf, } from './claude-schema-call-forms.test-fixture.ts';
import { CONTEST_IMPORT, } from './claude-schema-destination-forms.test-fixture.ts';
import {
  SENDER_FILE,
  SENDER_IMPORT,
} from './claude-schema-forwarder-forms.test-fixture.ts';
import type { SourceFile, } from './claude-schema-sources.test-fixture.ts';
import type { ShapeCase, } from './claude-schema-unread-shape-forms.test-fixture.ts';

//region A function that passes its parameter on to a function that hands a sender a builder call
// A function that makes a schema from what its caller passes and hands it to a sender is a forwarder, and so is a
// function that calls a forwarder with an argument that reads one of its own parameters: the files that call it supply
// the arguments of the schema as much as the files that call the first forwarder do. The forwarders are the fixpoint of
// that reading, as the wrappers are the fixpoint of the reading of a returned call, so a chain of any length is
// followed to its end.

/**
 The arguments of the call at the end of a chain, as the file that makes it writes them.
 */
const CHAIN_ARGUMENTS = '{ schemaName: \'chain\', verdicts: [], asksArchive: false, }';

/**
 The file of the first function of a chain, which hands the sender a builder call made from its parameter.
 */
const FIRST_LINK: SourceFile = {
  file: 'imaginary/src/hop-1.ts',
  text: textOf([
    CONTEST_IMPORT,
    SENDER_IMPORT,
    'export async function hop1(args: Parameters<typeof contestResponseFormat>[0],) {',
    '  return await gatherThroughCache({ responseFormat: contestResponseFormat(args,), },);',
    '}',
  ],),
};

/**
 The file of a function that passes its parameter on to the function of the file before it.

 @param hop - place of the function in the chain, from two

 @returns File that declares `hop<n>` over `hop<n - 1>`

 @example
 ```ts
 const file = linkAt({ hop: 2, },);
 ```
 */
function linkAt({ hop, }: { readonly hop: number; },): SourceFile {
  return {
    file: `imaginary/src/hop-${String(hop,)}.ts`,
    text: textOf([
      `import { hop${String(hop - 1,)}, } from './hop-${String(hop - 1,)}.ts';`,
      `export async function hop${String(hop,)}(args: unknown,) {`,
      `  return await hop${String(hop - 1,)}(args,);`,
      '}',
    ],),
  };
}

/**
 The file that calls the function at the end of a chain and keeps the answer.

 @param hop - place of the last function in the chain

 @returns File that calls `hop<n>` with the arguments of a schema

 @example
 ```ts
 const file = endOfChainAt({ hop: 3, },);
 ```
 */
function endOfChainAt({ hop, }: { readonly hop: number; },): SourceFile {
  return {
    file: 'imaginary/src/chain-caller.ts',
    text: textOf([
      `import { hop${String(hop,)}, } from './hop-${String(hop,)}.ts';`,
      'export async function run() {',
      `  const reply = await hop${String(hop,)}(${CHAIN_ARGUMENTS},);`,
      '  return Object.freeze(reply,);',
      '}',
    ],),
  };
}

/**
 The sources of a chain with the given number of functions, the sender beside them.

 @param length - number of functions in the chain, from two

 @returns Sender's file, the first function's file, one file for each function after it, and the file that calls
 the last

 @example
 ```ts
 const files = chainOfLength({ length: 3, },);
 ```
 */
function chainOfLength({ length, }: { readonly length: number; },): readonly SourceFile[] {
  return [
    SENDER_FILE,
    FIRST_LINK,
    ...Array.from(
      { length: length - 1, },
      function linkOf(
        _unused: unknown,
        index: number,
      ): SourceFile {
        return linkAt({ hop: index + 2, },);
      },
    ),
    endOfChainAt({ hop: length, },),
  ];
}

/**
 The paths of the imaginary files of a chain, sorted, which the search lists whole.

 @param length - number of functions in the chain

 @returns Paths of the files of the functions and of the file that calls the last

 @example
 ```ts
 const paths = pathsOfChain({ length: 3, },);
 ```
 */
function pathsOfChain({ length, }: { readonly length: number; },): readonly string[] {
  return [
    'imaginary/src/chain-caller.ts',
    ...Array.from(
      { length, },
      function pathOfLink(
        _unused: unknown,
        index: number,
      ): string {
        return `imaginary/src/hop-${String(index + 1,)}.ts`;
      },
    ),
  ].toSorted();
}

/**
 Cases of a function that calls a function that hands a sender a builder call.
 */
export const FORWARDER_CHAIN_CASES: readonly ShapeCase[] = [
  {
    name: 'LISTS THE FILE THAT CALLS A FUNCTION THAT PASSES ITS PARAMETER ON TO A FORWARDER, in a file of its own, '
      + 'since the arguments of the schema are written in the file that calls the second function',
    files: chainOfLength({ length: 2, },),
    found: pathsOfChain({ length: 2, },),
    builders: [],
    spots: [],
  },
  {
    name: 'LISTS THE FILE THAT CALLS A FUNCTION THAT PASSES ITS PARAMETER ON TO A FORWARDER IN ITS OWN FILE, so a '
      + 'function declared beside the forwarder and calling it is one too',
    files: [
      SENDER_FILE,
      {
        file: 'imaginary/src/wrap-stage.ts',
        text: textOf([
          CONTEST_IMPORT,
          SENDER_IMPORT,
          'export async function mk(args: Parameters<typeof contestResponseFormat>[0],) {',
          '  return await gatherThroughCache({ responseFormat: contestResponseFormat(args,), },);',
          '}',
          'export async function wrap(args: Parameters<typeof contestResponseFormat>[0],) {',
          '  return await mk(args,);',
          '}',
        ],),
      },
      {
        file: 'imaginary/src/wrap-caller.ts',
        text: textOf([
          'import { wrap, } from \'./wrap-stage.ts\';',
          'export async function run() {',
          `  const reply = await wrap(${CHAIN_ARGUMENTS},);`,
          '  return Object.freeze(reply,);',
          '}',
        ],),
      },
    ],
    found: [
      'imaginary/src/wrap-caller.ts',
      'imaginary/src/wrap-stage.ts',
    ],
    builders: [],
    spots: [],
  },
  {
    name: 'LISTS THE FILE THAT CALLS THE LAST OF TEN FUNCTIONS EACH OF WHICH PASSES ITS PARAMETER ON TO THE ONE BEFORE, '
      + 'since a chain has no length at which the files that call its end stop writing the arguments of the schema',
    files: chainOfLength({ length: 10, },),
    found: pathsOfChain({ length: 10, },),
    builders: [],
    spots: [],
  },
  {
    name: 'DOES NOT LIST THE FILE THAT CALLS A FUNCTION THAT CALLS A FORWARDER WITH NOTHING OF ITS PARAMETERS, since the '
      + 'arguments of the schema are written in the file of that function, which uses the forwarder and is listed',
    files: [
      SENDER_FILE,
      FIRST_LINK,
      {
        file: 'imaginary/src/hop-2.ts',
        text: textOf([
          'import { hop1, } from \'./hop-1.ts\';',
          'export async function hop2(question: string,) {',
          `  return { reply: await hop1(${CHAIN_ARGUMENTS},), question, };`,
          '}',
        ],),
      },
      endOfChainAt({ hop: 2, },),
    ],
    found: [
      'imaginary/src/hop-1.ts',
      'imaginary/src/hop-2.ts',
    ],
    builders: [],
    spots: [],
  },
];

//endregion A function that passes its parameter on to a function that hands a sender a builder call
