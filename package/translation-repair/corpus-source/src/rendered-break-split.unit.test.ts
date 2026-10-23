/**
 Checks the instruction and the measured facts of a source-only passage with explicit breaks apart (R4): the
 instruction is the same text for every such passage so it can sit in a cached system prompt, and the counts of the
 call are in the facts alone.

 @module
 */

import { describe, expect, it, } from '@monochromatic-dev/module-test/ts';
import {
  renderedBreakContract,
  renderedBreakFacts,
} from '../dist/final/node/index.mjs';

/** One block holding one authored visible break. */
const ONE_BREAK = '> 猫醒了。  \n> 鸟唱了。';

/** One block holding two authored visible breaks. */
const TWO_BREAKS = '> 猫醒了。  \n> 鸟唱了。  \n> 鱼游了。';

/** Words every measured line holds, and no instruction does. */
const MEASURED = 'explicit breaks by top-level block';

await describe({
  name: 'the rendered-break instruction and facts, apart',
  children: [
    it({
      name: 'IS THE SAME TEXT FOR PASSAGES WITH DIFFERENT BREAK COUNTS and holds no measured count',
      fn: async () => {
        /** Instruction for the passage with one break. */
        const one = renderedBreakContract({ sourceText: ONE_BREAK, archiveText: '', },);
        /** Instruction for the passage with two breaks. */
        const two = renderedBreakContract({ sourceText: TWO_BREAKS, archiveText: '', },);
        expect(one.length,).toBeGreaterThan(0,);
        expect(two,).toBe(one,);
        expect(one,).not.toContain(MEASURED,);
      },
    },),
    it({
      name: 'IS EMPTY where the rule does not apply: archive wording, metadata, no break, unreadable source',
      fn: async () => {
        expect(renderedBreakContract({ sourceText: ONE_BREAK, archiveText: 'The cat wakes.', },),).toBe('',);
        expect(renderedBreakContract({ sourceText: ONE_BREAK, archiveText: '', syntax: 'front-matter', },),).toBe('',);
        expect(renderedBreakContract({ sourceText: '> 猫醒了。\n> 鸟唱了。', archiveText: '', },),).toBe('',);
        expect(renderedBreakContract({ sourceText: '<Cat value={', archiveText: '', },),).toBe('',);
      },
    },),
    it({
      name: 'HOLDS THE COUNTS OF THE CALL and none of the instruction',
      fn: async () => {
        /** Facts for the passage with two breaks and one shown candidate. */
        const facts = renderedBreakFacts({
          sourceText: TWO_BREAKS,
          archiveText: '',
          renderings: [{ label: 'Candidate 1', text: '> The cat wakes.<br/>The bird sings.', },],
        },);
        expect(facts,).toContain(`ORIGINAL ${MEASURED}: [2]`,);
        expect(facts,).toContain(`"Candidate 1" ${MEASURED}: [1]`,);
        expect(facts,).not.toContain('RENDERED LINE STRUCTURE',);
      },
    },),
    it({
      name: 'IS EMPTY exactly where the instruction is empty',
      fn: async () => {
        expect(renderedBreakFacts({ sourceText: ONE_BREAK, archiveText: 'The cat wakes.', },),).toBe('',);
        expect(renderedBreakFacts({ sourceText: ONE_BREAK, archiveText: '', syntax: 'front-matter', },),).toBe('',);
        expect(renderedBreakFacts({ sourceText: '> 猫醒了。\n> 鸟唱了。', archiveText: '', },),).toBe('',);
        expect(renderedBreakFacts({ sourceText: '<Cat value={', archiveText: '', },),).toBe('',);
      },
    },),
  ],
},);
