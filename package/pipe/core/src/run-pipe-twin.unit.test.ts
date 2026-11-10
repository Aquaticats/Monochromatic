/**
 The synchronous and asynchronous dispatch ladders state how they relate.
 
 `runPipe` and `runPipeAsync` apply the same steps in the same order and differ only by awaiting
 each application, so nothing stops an edit to one ladder from silently skipping the other.
 Issue 1675 recorded that silence. Two things answer it: each declaration's TSDoc states the
 arrangement, and the prologue both sides need (logger tagging, argument widening, gap and
 overflow validation) lives in `prepare-run.ts` instead of being copied into each ladder.
 
 This suite reads the module as text because neither ladder is part of the public surface a
 built import could reach. It fails when either statement disappears, when a ladder stops
 calling the shared prologue or repeats it, when one ladder drops an arity branch or step label
 the other keeps, or when only one side awaits its applications.
 */
import { readFileSync, } from 'node:fs';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

//region Source scanning

/**
 Execution module holding both ladders.
 */
const runModulePath = new URL('run.ts', import.meta.url,);

/**
 Declaration that opens the synchronous ladder.
 */
const syncDeclaration = 'export function runPipe(';

/**
 Declaration that opens the asynchronous ladder, which follows the synchronous one.
 */
const asyncDeclaration = 'export async function runPipeAsync(';

/**
 Phrase the arrangement statement must carry: the sides are written twice on purpose.
 */
const handWrittenMarker = 'hand-written';

/**
 Function both ladders call for the prologue they share.
 */
const sharedPrologue = 'prepareRun';

/**
 Prologue helpers a ladder must not repeat inline once the shared call exists.
 */
const prologueHelpers = [
  'assertContiguousSteps(',
  'assertNoOverflowStep(',
  'tagged(',
] as const;

/**
 Every arity an `fnN` guard dispatches, `fn2` through the nine-step cap.
 */
const guardedArities = [
  2,
  3,
  4,
  5,
  6,
  7,
  8,
  9,
] as const;

/**
 Step count of the highest branch, which a ladder applies without a guard.
 */
const fallThroughStepCount = guardedArities.length + 1;

/**
 Every step count a ladder logs: one per guarded branch, then the fall-through.
 */
const loggedStepCounts = [
  ...guardedArities.map(function branchStepCount(arity,) {
    return arity - 1;
  },),
  fallThroughStepCount,
];

/**
 Source text of the execution module.
 
 @returns module text the assertions scan
 */
function runModuleSource(): string {
  return readFileSync(runModulePath, 'utf8',);
}

/**
 Code of each ladder, split at the documentation the asynchronous declaration carries.
 
 Starting the asynchronous stretch at its declaration keeps the synchronous stretch free of the
 asynchronous prose, so an `await` in that documentation never reads as a synchronous await.
 
 @param source - module text as written
 
 @returns the synchronous ladder's text and the asynchronous ladder's
 */
function ladderCodes(source: string,): {
  readonly sync: string;
  readonly async: string;
} {
  return {
    sync: source.slice(
      declarationIndex(source, syncDeclaration,),
      documentationStart(source, asyncDeclaration,),
    ),
    async: source.slice(declarationIndex(source, asyncDeclaration,),),
  };
}

/**
 How often one text occurs in one stretch of the module.
 
 @param source - module text under inspection
 @param needle - literal text to count
 
 @returns occurrences of `needle`, without overlap
 */
function occurrences(source: string, needle: string,): number {
  return source.split(needle,).length - 1;
}

/**
 Index where one declaration opens in the module text.
 
 @param source - module text under inspection
 @param declaration - text opening the declaration
 
 @returns offset of the declaration's first character
 */
function declarationIndex(source: string, declaration: string,): number {
  const index = source.indexOf(declaration,);
  expect(index,).toBeGreaterThanOrEqual(0,);
  return index;
}

/**
 Index where the documentation above one declaration opens.
 
 @param source - module text as written
 @param declaration - text opening the documented declaration
 
 @returns offset of the documentation's opening marker
 */
function documentationStart(source: string, declaration: string,): number {
  const closeIndex = source.lastIndexOf('*/', declarationIndex(source, declaration,),);
  expect(closeIndex,).toBeGreaterThanOrEqual(0,);
  const openIndex = source.lastIndexOf('/**', closeIndex,);
  expect(openIndex,).toBeGreaterThanOrEqual(0,);
  return openIndex;
}

/**
 TSDoc block immediately above one declaration.
 
 @param source - module text as written
 @param declaration - text opening the declaration whose documentation is wanted
 
 @returns documentation text between its opening and closing markers
 */
function declarationDocumentation(source: string, declaration: string,): string {
  const openIndex = documentationStart(source, declaration,);
  return source.slice(openIndex, source.indexOf('*/', openIndex,),);
}

/**
 Debug label a ladder logs for one step count, singular only at one.
 
 @param count - number of steps the branch applies
 
 @returns label text as it appears in the module
 */
function stepLabel(count: number,): string {
  return `'${String(count,)} ${count === 1 ? 'step' : 'steps'}'`;
}

//endregion

await describe({
  name: '',
  children: [
    describe({
      name: 'runPipe and runPipeAsync twin arrangement',
      children: [
        it({
          name: 'states in each declaration documentation that both ladders are hand-written',
          fn: async () => {
            const source = runModuleSource();
            const syncDocumentation = declarationDocumentation(source, syncDeclaration,);
            const asyncDocumentation = declarationDocumentation(source, asyncDeclaration,);

            expect(syncDocumentation,).toContain(handWrittenMarker,);
            expect(syncDocumentation,).toContain('runPipeAsync',);

            expect(asyncDocumentation,).toContain(handWrittenMarker,);
            expect(asyncDocumentation,).toContain('runPipe',);
          },
        },),

        it({
          name: 'names the shared prologue in each declaration documentation',
          fn: async () => {
            const source = runModuleSource();

            expect(declarationDocumentation(source, syncDeclaration,),).toContain(
              sharedPrologue,
            );
            expect(declarationDocumentation(source, asyncDeclaration,),).toContain(
              sharedPrologue,
            );
          },
        },),

        it({
          name: 'calls the shared prologue exactly once in each ladder instead of repeating it',
          fn: async () => {
            const ladders = ladderCodes(runModuleSource(),);

            for (const ladder of [
              ladders.sync,
              ladders.async,
            ]) {
              expect(occurrences(ladder, `${sharedPrologue}(`,),).toBe(1,);
              for (const helper of prologueHelpers)
                expect(occurrences(ladder, helper,),).toBe(0,);
            }
          },
        },),

        it({
          name: 'dispatches every arity guard and step label on both ladders',
          fn: async () => {
            const ladders = ladderCodes(runModuleSource(),);

            for (const ladder of [
              ladders.sync,
              ladders.async,
            ]) {
              for (const arity of guardedArities)
                expect(occurrences(ladder, `if (fn${String(arity,)} === undefined)`,),).toBe(
                  1,
                );
              for (const count of loggedStepCounts)
                expect(occurrences(ladder, stepLabel(count,),),).toBe(1,);
            }
          },
        },),

        it({
          name: 'awaits every application on the asynchronous ladder and none on the synchronous one',
          fn: async () => {
            const ladders = ladderCodes(runModuleSource(),);

            expect(ladders.sync,).not.toContain('await',);
            for (const arity of [
              1,
              ...guardedArities,
            ]) {
              expect(ladders.sync,).toContain(`fn${String(arity,)}(`,);
              expect(ladders.async,).toContain(`await fn${String(arity,)}(`,);
            }
          },
        },),
      ],
    },),
  ],
},);
