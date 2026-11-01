/**
 Whether external formal positions are mapped from argument positions, or indexed as if identical.
 
 Its own test because no shape in the fixture corpus reaches this through a diagnostic. Doing so needs
 an installed package with a locked version whose shipped implementation provably mutates a formal,
 invoked with a spread. A mutant restoring the previous actual-position indexing survived the whole
 suite, so the mapping had to be exercised directly or it would have landed untested.
 
 What the mapping being wrong costs is a dropped fact rather than an invented one. A proven mutation of
 a later formal read an index that did not exist and recorded nothing, so an offer stood where the
 external analyzer had already disproved it.
 
 @module
 */

import { fileURLToPath, } from 'node:url';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import type { CallExpression, } from 'typescript/ast';
import {
  isCallExpression,
  isFunctionDeclaration,
} from 'typescript/ast/is';

import {
  asEffectSlot,
  closeSemanticBridge,
  formalArgumentIndexes,
  openSemanticFile,
} from '../dist/final/node/index.mjs';

/**
 Repository path the overlay stands in for, so project discovery succeeds.
 */
const OVERLAY_PATH = fileURLToPath(new URL(
  '../../../test-fixture/oxlint-no-restricted-syntax/src/readonly-structural-store-invalid.ts',
  import.meta.url,
),);

/**
 Source declaring one plain callee and one rest callee, called four ways.
 
 Each call initializes a distinctly named local, so a case can find its call by that name rather than
 by counting call expressions.
 */
const SOURCE = `
export function plainCallee(first: string, second: string, third: string,): string {
  return first + second + third;
}

export function restCallee(head: string, ...tail: readonly string[],): string {
  return head + tail.join('',);
}

export function driver(alpha: string, beta: string, gamma: string,): readonly string[] {
  const positional = plainCallee(alpha, beta, gamma,);
  const spreadTail = plainCallee(alpha, ...[beta, gamma,],);
  const spreadWhole = plainCallee(...[alpha, beta, gamma,],);
  const restReceiving = restCallee(alpha, beta, gamma,);
  return [positional, spreadTail, spreadWhole, restReceiving,];
}
`;

/**
 Origins standing in for what each argument position carries, one distinct slot per position.
 
 Distinct values on purpose, so a mapping that reads the wrong position produces a visibly wrong
 answer rather than a coincidentally right one.
 */
const ARGUMENT_ORIGINS = [
  [asEffectSlot(10,),],
  [asEffectSlot(20,),],
  [asEffectSlot(30,),],
];

/**
 Semantic session over the overlay, shared by every case.
 */
const session = openSemanticFile({
  fileName: OVERLAY_PATH,
  sourceText: SOURCE,
  hasBOM: false,
},);

/**
 Names the call expression initializing one named local.
 
 @param callName - Local the call initializes.
 
 @returns call expression that local is initialized with.
 
 @throws when the name does not resolve to a call.
 
 @example
 ```ts
 callNamed('positional');
 ```
 */
function callNamed(callName: string,): CallExpression {
  /**
   Offset of the callee name written just after the local's assignment.
   */
  const at = SOURCE.indexOf('Callee(', SOURCE.indexOf(`const ${callName} =`,),);
  /**
   Call expression the node at that offset belongs to.
   */
  const call = session.nodeAtOffset(at,)
    .parent;
  if (!isCallExpression(call,))
    throw new Error(`${callName} did not resolve to a call expression`,);
  return call;
}

/**
 Maps one named call against one named callee's formals.
 
 @param calleeName - Declaration whose formals order the arguments.
 
 @param callName - Local the call initializes.
 
 @returns caller origins by formal position.
 
 @throws when the callee name does not resolve to a function declaration.
 
 @example
 ```ts
 mapCall({ calleeName: 'plainCallee', callName: 'positional' });
 ```
 */
function mapCall({
  calleeName,
  callName,
}: {
  readonly calleeName: string;
  readonly callName: string;
},): readonly (readonly number[])[] {
  /**
   Declaration owning the requested callee name.
   */
  const declaration = session.nodeAtOffset(
    SOURCE.indexOf(`function ${calleeName}`,) + 'function '.length,
  )
    .parent;
  if (!isFunctionDeclaration(declaration,))
    throw new Error(`${calleeName} did not resolve to a function declaration`,);
  return formalArgumentIndexes({
    declaration,
    call: callNamed(callName,),
    allArgumentIndexes: ARGUMENT_ORIGINS,
  },);
}

await describe({
  name: 'external formal-to-actual mapping',
  concurrency: 1,
  children: [
    it({
      name: 'maps a plain positional call one formal to one actual',
      fn: async () => {
        /* The case that predates the mapping and must be unchanged by it, since every offer standing
         * today rests on this shape answering exactly as it did. */
        expect(mapCall({
          calleeName: 'plainCallee',
          callName: 'positional',
        },),).toEqual([
          [asEffectSlot(10,),],
          [asEffectSlot(20,),],
          [asEffectSlot(30,),],
        ],);
      },
    },),
    it({
      name: 'gives every formal at or past a spread every actual from the spread onward',
      fn: async () => {
        /* One syntactic argument supplies an unknown number of formals, so positional correspondence
         * is gone from the spread onward. Formals two and three may each receive the spread, and the
         * actual-position indexing answered nothing at all for formal three because the call writes
         * only two arguments. That silence is what dropped a proven mutation. */
        expect(mapCall({
          calleeName: 'plainCallee',
          callName: 'spreadTail',
        },),).toEqual([
          [asEffectSlot(10,),],
          [asEffectSlot(20,),],
          [asEffectSlot(20,),],
        ],);
      },
    },),
    it({
      name: 'gives every formal the spread when the whole argument list is one spread',
      fn: async () => {
        expect(mapCall({
          calleeName: 'plainCallee',
          callName: 'spreadWhole',
        },),).toEqual([
          [asEffectSlot(10,),],
          [asEffectSlot(10,),],
          [asEffectSlot(10,),],
        ],);
      },
    },),
    it({
      name: 'gives a rest formal every actual from its own position onward',
      fn: async () => {
        /* The second failure direction. Indexing by formal position charged the rest formal the
         * second actual alone and missed the third. */
        expect(mapCall({
          calleeName: 'restCallee',
          callName: 'restReceiving',
        },),).toEqual([
          [asEffectSlot(10,),],
          [asEffectSlot(20,), asEffectSlot(30,),],
        ],);
      },
    },),
    it({
      name: 'answers empty for a declaration whose formals cannot be read',
      fn: async () => {
        /* A declaration with no formal list orders nothing, so the mapping declines rather than
         * guessing, and the applier charges every argument's origins instead. */
        expect(formalArgumentIndexes({
          declaration: session.sourceFile,
          call: callNamed('positional',),
          allArgumentIndexes: ARGUMENT_ORIGINS,
        },),).toEqual([],);
      },
    },),
  ],
},);
closeSemanticBridge();
