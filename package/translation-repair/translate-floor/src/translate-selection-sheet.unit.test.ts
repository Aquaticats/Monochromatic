/**
 Tests for what the translate lane's judges are told.

 WHY A SHEET OF CONSTANTS IS WORTH PINNING. These strings are the only thing
 standing between a judge and the wrong reading of `nothing added`. Measured on
 one contested slice, three of six judges rejected a candidate for carrying a
 declared alias, every one of them having been shown the declared names. The
 separate names criterion did not stop them, because it reads as spelling
 guidance while `nothing added` reads as a rule. Losing the carve-out again
 would cost accurate detail on memorial pages and nothing here would fail.

 AND THE CARVE-OUT HAS ITS OWN FAILURE MODE, measured on the consolidation
 bed once it was written without a scope: a judge abstained from a whole slate
 because no candidate carried the declared LOCATION, and a shipped rendering
 signed a note left by a friend of the deceased with the deceased's own name,
 alias and city. Both directions are pinned here, because a sheet edit that
 fixes one by reopening the other would otherwise pass.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  TRANSLATE_FRONT_MATTER_CRITERIA,
  TRANSLATE_LINE_STRUCTURE_CRITERION,
  TRANSLATE_SELECTION_CRITERIA,
  translateSelectionCriteria,
  TRANSLATE_SELECTION_TASK,
} from '../dist/final/node/index.mjs';

/**
 Clause the line-structure criterion overrides, spelled as both carry it.

 WRITTEN OUT HERE rather than imported, because a case importing the constant
 would compare the sheet against itself and pass however the sentence was
 edited. The point is that this exact wording still reaches a judge.
 */
const OVERRIDDEN_CLAUSE = 'A SHAPE THE ORIGINAL DOES NOT HAVE IS NOT A FAULT';

/**
 Whether a governed criterion states the overridden clause without being the
 line-structure criterion itself: the predicate both the precedence case
 (by position) and the verbatim-clause case (by presence) check over a
 governed slice.

 @param criterion - criterion from a governed slice

 @returns Whether this criterion carries the overridden clause elsewhere
 */
function statesOverriddenClause(criterion: string,): boolean {
  return criterion.includes(OVERRIDDEN_CLAUSE,)
    && (criterion !== TRANSLATE_LINE_STRUCTURE_CRITERION);
}

/**
 Criteria joined, since a judge reads them as one list.
 */
const sheet = TRANSLATE_SELECTION_CRITERIA.join('\n',);

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: 'translate selection sheet',
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'CARVES declared names out of the rule that forbids additions',
          fn: async () => {
            /**
             Criterion carrying the prohibition judges actually applied.
             */
            const faithfulness = TRANSLATE_SELECTION_CRITERIA
              .find(function forbidsAdditions(line,): boolean {
                return line.includes('nothing added',);
              },) ?? '';
            expect(faithfulness,).not.toBe('',);
            // THE CARVE-OUT SITS INSIDE THAT CRITERION, not beside it, because the
            // rule being misapplied is the one that has to name the exception.
            expect(faithfulness,).toContain('a declared name or handle is not an addition',);
          },
        },),
        it({
          name: 'KEEPS what the page carries and the original is silent about inside the rule that forbids additions (class eighty-five, CuspariaKLSY6 2026-09-22: the archive gloss line stripped by every consolidation, the gate kept the present-tense standing to keep it)',
          fn: async () => {
            /**
             Criterion carrying the prohibition the slate judges applied to the gloss.
             */
            const faithfulness = TRANSLATE_SELECTION_CRITERIA
              .find(function forbidsAdditions(line,): boolean {
                return line.includes('nothing added',);
              },) ?? '';
            expect(faithfulness,).not.toBe('',);
            // THE CLAUSE SITS INSIDE THE FAITHFULNESS CRITERION, beside the declared-name
            // carve-out, because "nothing added" is the rule the judges cited when they
            // rewarded the candidate without the gloss.
            expect(faithfulness,).toContain('a gloss of a name or a term',);
            expect(faithfulness,).toContain('has added nothing',);
            expect(faithfulness,).toContain('has dropped page content',);
          },
        },),
        it({
          name: 'SCOPES the carve-out to a passage that refers to the person',
          fn: async () => {
            /**
             Criterion carrying the prohibition judges actually applied.
             */
            const faithfulness = TRANSLATE_SELECTION_CRITERIA
              .find(function forbidsAdditions(line,): boolean {
                return line.includes('nothing added',);
              },) ?? '';
            // WITHOUT THE SCOPE the carve-out reads as a licence to put the
            // archive's identity block into any passage at all.
            expect(faithfulness,).toContain('When a name is used TO REFER TO a person',);
            expect(faithfulness,).toContain('This does not authorize replacing a word, character, spelling, or sound',);
          },
        },),
        it({
          name: 'REFUSES to make a declared name content the passage owes',
          fn: async () => {
            /**
             Whether the sheet still tells a judge an unnamed person is a gap.
             */
            const owes = sheet.includes('has left something out',);
            expect(owes,).toBe(false,);
            expect(sheet.includes('has left nothing out',),).toBe(true,);
          },
        },),
        it({
          name: 'REFUSES to let an attribution line take this person\'s name',
          fn: async () => {
            // A note left BY A FRIEND of the deceased was signed with the dead
            // person's name, alias and city, and the sheet is where that was
            // licensed.
            expect(sheet.includes('attributing the passage to someone ELSE',),).toBe(true,);
          },
        },),
        it({
          name: 'REFUSES to let a spelling every candidate shares decide the slate',
          fn: async () => {
            // Measured twice at one slice: the identity block declares one form,
            // the archive's passage writes another, every candidate inherits it,
            // and a judge abstained from the whole slate. The incumbent that
            // survives that decline is where the spelling came from.
            expect(sheet.includes('cannot separate them',),).toBe(true,);
            expect(sheet.includes('rather than declining',),).toBe(true,);
          },
        },),
        it({
          name: 'REFUSES to make the original the authority on shape',
          fn: async () => {
            // Producers are floored on the page, which splits, merges and quotes
            // blocks the original runs as prose. A judge told the original is the
            // standard marks down exactly the renderings the guard demands.
            expect(sheet.includes('structure of the ORIGINAL',),).toBe(false,);
            expect(sheet.includes('A SHAPE THE ORIGINAL DOES NOT HAVE IS NOT A FAULT',),).toBe(true,);
          },
        },),
        it({
          name: 'KEEPS a shape rule a judge can check from the candidate alone',
          fn: async () => {
            // The existing translation is on the ballot anonymously and never
            // travels as labelled evidence, so a criterion naming the page would
            // name a text the judge cannot see.
            expect(sheet.includes('PAGE AS IT STANDS',),).toBe(false,);
            expect(sheet.includes('used consistently',),).toBe(true,);
          },
        },),
        it({
          name: 'KEEPS coverage and faithfulness ahead of fluency',
          fn: async () => {
            // The ordering the whole lane rests on: a candidate that reads better
            // while saying less must lose.
            const coverage = TRANSLATE_SELECTION_CRITERIA
              .findIndex(function isCoverage(line,): boolean {
                return line.includes('Complete coverage',);
              },);
            const fluency = TRANSLATE_SELECTION_CRITERIA
              .findIndex(function isFluency(line,): boolean {
                return line.includes('idiomatic English',);
              },);
            expect(coverage,).toBe(0,);
            expect(fluency,).toBe(TRANSLATE_SELECTION_CRITERIA.length - 1,);
          },
        },),
        it({
          name: 'NAMES the candidates as complete translations, not as edits',
          fn: async () => {
            expect(TRANSLATE_SELECTION_TASK.includes('complete English translation',),).toBe(true,);
          },
        },),
      ],
    },),

    describe({
      name: translateSelectionCriteria.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'HANDS BACK THE MEASURED ARRAY ITSELF FOR A PROSE SLICE, not a copy of it and not a rebuild. '
            + 'The judge-quality bench measured whether these judges can tell a faithful rendering from a fluent one, and it measured '
            + 'them on these exact criteria. Identity rather than equality is the assertion, because a rebuilt array '
            + 'that happened to match today is the thing that drifts tomorrow',
          fn: async () => {
            expect(translateSelectionCriteria({ lineStructured: false, },),).toBe(TRANSLATE_SELECTION_CRITERIA,);
          },
        },),
        it({
          name: 'USES DEDICATED YAML CRITERIA for front matter instead of prose shape rules',
          fn: async () => {
            const criteria = translateSelectionCriteria({
              lineStructured: false,
              syntax: 'front-matter',
            },);
            expect(criteria,).toBe(TRANSLATE_FRONT_MATTER_CRITERIA,);
            expect(criteria.some(function namesVisibleIdentity(criterion,): boolean {
              return criterion.includes('entry directory id',);
            },),).toBe(true,);
            expect(criteria.some(function preservesIdentityRelation(criterion,): boolean {
              return criterion.includes('name and info.alias are the same identity',);
            },),).toBe(true,);
            expect(criteria.some(function preservesCommentContributor(criterion,): boolean {
              return criterion.includes('established target contributor spelling',);
            },),).toBe(true,);
          },
        },),
        it({
          name: 'ADDS THE LINE-STRUCTURE CRITERION ONLY WHERE THE VERSE RULE GOVERNS, which is the whole gate. '
            + 'A sheet carrying it unconditionally would satisfy every other case here and would tell the judges of '
            + 'the archive\'s prose to count lines the ORIGINAL never broke',
          fn: async () => {
            expect(
              translateSelectionCriteria({ lineStructured: true, },)
                .includes(TRANSLATE_LINE_STRUCTURE_CRITERION,),
            ).toBe(true,);
            expect(
              translateSelectionCriteria({ lineStructured: false, },)
                .includes(TRANSLATE_LINE_STRUCTURE_CRITERION,),
            ).toBe(false,);
          },
        },),
        it({
          name: 'PLACES IT AHEAD OF THE RULE IT OVERRIDES, because these criteria are read most important first '
            + 'and a rule that defers to another has to be met after it. Landing behind the shape rule would leave '
            + 'a judge reading the general permission first and the exception to it second',
          fn: async () => {
            /**
             Criteria as a governed slice is given them.
             */
            const governed = translateSelectionCriteria({ lineStructured: true, },);

            expect(governed.indexOf(TRANSLATE_LINE_STRUCTURE_CRITERION,),).toBeLessThan(
              governed.findIndex(function permitsAnUnoriginalShape(criterion,): boolean {
                return statesOverriddenClause(criterion,);
              },),
            );
          },
        },),
        it({
          name: 'OVERRIDES A CLAUSE SOME JUDGE IS ACTUALLY GIVEN, spelled the same way in both criteria. An override '
            + 'naming a sentence that no longer appears anywhere is worse than no override at all: it reads as '
            + 'settled precedence while leaving the contradiction fixed for producers standing in front of the judge',
          fn: async () => {
            /**
             Criteria as a governed slice is given them.
             */
            const governed = translateSelectionCriteria({ lineStructured: true, },);

            // BOTH SIDES, because either one alone can hold the clause while the other
            // has been edited away from it.
            expect(TRANSLATE_LINE_STRUCTURE_CRITERION.includes(OVERRIDDEN_CLAUSE,),).toBe(true,);
            expect(
              governed.some(function statesTheRule(criterion,): boolean {
                return statesOverriddenClause(criterion,);
              },),
            ).toBe(true,);
          },
        },),
        it({
          name: 'LEAVES FLUENCY LAST AND COVERAGE FIRST ON A GOVERNED SLICE, so the ordering the lane rests on '
            + 'survives the insertion. A criterion added at either end would reorder the priorities every other '
            + 'case in this file pins',
          fn: async () => {
            /**
             Criteria as a governed slice is given them.
             */
            const governed = translateSelectionCriteria({ lineStructured: true, },);

            expect(governed.at(0,),).toBe(TRANSLATE_SELECTION_CRITERIA.at(0,),);
            expect(governed.at(-1,),).toBe(TRANSLATE_SELECTION_CRITERIA.at(-1,),);
            expect(governed.length,).toBe(TRANSLATE_SELECTION_CRITERIA.length + 1,);
          },
        },),
      ],
    },),
  ],
},);
