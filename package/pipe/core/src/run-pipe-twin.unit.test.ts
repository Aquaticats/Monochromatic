/**
 The synchronous and asynchronous dispatch ladders state how they relate.
 
 `runPipe` and `runPipeAsync` apply the same steps in the same order and differ only by awaiting
 each application, so nothing stops an edit to one ladder from silently skipping the other.
 Issue 1675 recorded that silence. Two things answer it: each declaration's TSDoc states the
 arrangement, and the prologue both sides need (logger tagging, argument widening, gap and
 overflow validation) lives in `prepare-run.ts` instead of being copied into each ladder.
 
 This suite reads the module as text because neither ladder is part of the public surface a
 built import could reach. It fails when either statement disappears, when a ladder stops
 calling the shared prologue, or when the two ladders stop dispatching the same arities and
 step labels.
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
 Declaration that opens the asynchronous ladder.
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
 How many times the shared prologue's call appears, once per ladder.
 */
const laddersSharingPrologue = 2;

/**
 Prologue helpers a ladder must not repeat inline once the shared call exists.
 */
const prologueHelpers = [
  'assertContiguousSteps(',
  'assertNoOverflowStep(',
  'tagged(',
] as const;

/**
 Every arity an `fnN` guard dispatches, `fn2` through `fn9`.
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
 Every step count a ladder logs: one per guard's branch, plus the nine-step fall-through.
 */
const loggedStepCounts = [
  ...guardedArities.map(function branchStepCount(arity,) {
    return arity - 1;
  },),
  guardedArities.length,
];

/**
 Source text of the execution module.
 
 @returns module text the assertions scan
 */
function runModuleSource(): string {
  return readFileSync(runModulePath, 'utf8',);
}

/**
 How often one text occurs in the module.
 
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
 TSDoc block immediately above one declaration.
 
 @param source - module text as written
 @param declaration - text opening the declaration whose documentation is wanted
 
 @returns documentation text between its opening and closing markers
 */
function declarationDocumentation(source: string, declaration: string,): string {
  const closeIndex = source.lastIndexOf('*/', declarationIndex(source, declaration,),);
  expect(closeIndex,).toBeGreaterThanOrEqual(0,);
  const openIndex = source.lastIndexOf('/**', closeIndex,);
  expect(openIndex,).toBeGreaterThanOrEqual(0,);
  return source.slice(openIndex, closeIndex,);
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
          name: 'calls the shared prologue once per ladder instead of repeating it',
          fn: async () => {
            const source = runModuleSource();

            expect(occurrences(source, `${sharedPrologue}(`,),).toBe(
              laddersSharingPrologue,
            );
            for (const helper of prologueHelpers)
              expect(occurrences(source, helper,),).toBe(0,);
          },
        },),

        it({
          name: 'dispatches the same arity guards and step labels on both ladders',
          fn: async () => {
            const source = runModuleSource();

            for (const arity of guardedArities)
              expect(occurrences(source, `if (fn${String(arity,)} === undefined)`,),).toBe(
                laddersSharingPrologue,
              );
            for (const count of loggedStepCounts)
              expect(occurrences(source, stepLabel(count,),),).toBe(
                laddersSharingPrologue,
              );
          },
        },),
      ],
    },),
  ],
},);
