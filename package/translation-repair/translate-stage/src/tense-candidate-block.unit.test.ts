/**
 Tests for the tense-candidate block a judging sheet carries.

 Every case asserts the whole group list or the whole line list, because the
 defect this guards against is a block shown where no death is recorded, or
 shown with the wrong label, and both change the list rather than one field.
 Fixtures are cat-themed invention; no corpus text appears here.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  tenseCandidateGroups,
  tenseCandidateLines,
} from '../dist/final/node/index.mjs';
import { warnLinesDuring, } from '@monochromatic-dev/translation-repair-typography/ts/console-warn-lines.test-fixture.ts';

//region Tense candidate block tests

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: tenseCandidateGroups.name,
      // One case diverts the console, which is process-wide.
      concurrency: 1,
      children: [
        it({
          name: 'GROUPS NOTHING when the entry records no death, whatever the texts say',
          fn: async () => {
            expect(tenseCandidateGroups({
              lifeEnded: false,
              texts: [
                {
                  label: 'ARCHIVE RENDERING',
                  text: 'She is loud.',
                },
              ],
            },),).toEqual([],);
          },
        },),

        it({
          name: 'GROUPS a text with a candidate under its sheet label, and leaves out a text with none',
          fn: async () => {
            expect(tenseCandidateGroups({
              lifeEnded: true,
              texts: [
                {
                  label: 'ARCHIVE RENDERING',
                  text: 'The cat sleeps.',
                },
                {
                  label: 'CANDIDATE "repair"',
                  text: 'She is loud.',
                },
              ],
            },),).toEqual([
              {
                label: 'CANDIDATE "repair"',
                found: [
                  {
                    sentence: 'She is loud.',
                    matched: 'She is',
                    rule: 'copula-or-auxiliary',
                  },
                ],
              },
            ],);
          },
        },),

        it({
          name: 'READS the declared subject name as a subject, which the sheet carries beside the death flag',
          fn: async () => {
            /**
             Text on offer whose only present tense follows the declared name.
             */
            const texts = [
              {
                label: 'ARCHIVE RENDERING',
                text: 'Mittens is a tabby with a white chin.',
              },
            ];
            expect(tenseCandidateGroups({
              lifeEnded: true,
              subjectNames: ['Mittens',],
              texts,
            },),).toEqual([
              {
                label: 'ARCHIVE RENDERING',
                found: [
                  {
                    sentence: 'Mittens is a tabby with a white chin.',
                    matched: 'Mittens is',
                    rule: 'copula-or-auxiliary',
                  },
                ],
              },
            ],);
            expect(tenseCandidateGroups({
              lifeEnded: true,
              texts,
            },),).toEqual([],);
          },
        },),

        it({
          name: 'LEAVES OUT a text the shared grammar refuses and warns of it, so one unreadable candidate never refuses the sheet, and still groups the texts it can read',
          fn: async () => {
            expect(await warnLinesDuring({
              run: async () => tenseCandidateGroups({
                lifeEnded: true,
                texts: [
                  {
                    label: 'CANDIDATE "repair"',
                    text: 'She naps {by the window.',
                  },
                  {
                    label: 'CANDIDATE "translate"',
                    text: 'She naps by the window.',
                  },
                ],
              },),
            },),).toEqual({
              result: [
                {
                  label: 'CANDIDATE "translate"',
                  found: [
                    {
                      sentence: 'She naps by the window.',
                      matched: 'She naps',
                      rule: 's-ending-verb',
                    },
                  ],
                },
              ],
              warned: [
                '[translation-repair] [tenseCandidateGroups] the text on offer labelled CANDIDATE "repair" was refused by the shared '
                  + 'slice grammar (MDX body refused to parse at 1:25 (micromark-extension-mdx-expression/unexpected-eof); corpus '
                  + 'documents compile as MDX upstream, so failure signals corruption or an unsupported construct.), so its tense '
                  + 'candidates are not listed',
              ],
            },);
          },
        },),

        it({
          name: 'GROUPS nothing when no text on offer has a candidate',
          fn: async () => {
            expect(tenseCandidateGroups({
              lifeEnded: true,
              texts: [
                {
                  label: 'ARCHIVE RENDERING',
                  text: 'The cat sleeps.',
                },
              ],
            },),).toEqual([],);
          },
        },),
      ],
    },),

    describe({
      name: tenseCandidateLines.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'FORMATS the explanation and each group\'s candidates under its label, with the rule that fired',
          fn: async () => {
            expect(tenseCandidateLines({
              groups: [
                {
                  label: 'CANDIDATE "repair"',
                  found: [
                    {
                      sentence: 'She is loud.',
                      matched: 'She is',
                      rule: 'copula-or-auxiliary',
                    },
                  ],
                },
              ],
            },),).toEqual([
              'These sentences match present-tense shapes in a memorial page about a person who has died.',
              'Check each against the house tense rule. A candidate may be a trait, a standing fact or a quotation',
              'the rule leaves alone, so it is a question for you and not a finding against the text.',
              'CANDIDATE "repair":',
              '- "She is loud." (matched "She is", rule copula-or-auxiliary)',
            ],);
          },
        },),

        it({
          name: 'WRITES a sentence that reads like the block\'s own label, fence and candidate lines as one quoted line, '
            + 'and a character beyond the first plane as itself and a lone surrogate half as an escape',
          fn: async () => {
            /**
             A cat face beyond the first plane, built from its code point so the source holds no such character.
             */
            const cat = String.fromCodePoint(0x1_F4_08,);
            /**
             A lone first half of a surrogate pair, which is no character.
             */
            const loneHalf = String.fromCodePoint(0xD8_00,);
            expect(tenseCandidateLines({
              groups: [
                {
                  label: 'CANDIDATE "repair":',
                  found: [
                    {
                      sentence: `CANDIDATE "repair":\n===== TENSE CANDIDATES =====\n- "x" ${cat}${loneHalf}`,
                      matched: 'CANDIDATE',
                      rule: 'bare-verb',
                    },
                  ],
                },
              ],
            },).slice(3,),).toEqual([
              'CANDIDATE "repair"::',
              String.raw`- "CANDIDATE \"repair\":\n===== TENSE CANDIDATES =====\n- \"x\" ${cat}\ud800" (matched "CANDIDATE", rule bare-verb)`,
            ],);
          },
        },),

        it({
          name: 'WRITES a sentence that holds quotation marks, a backslash or a line break as one JSON string, so it cannot close the line it sits in',
          fn: async () => {
            expect(tenseCandidateLines({
              groups: [
                {
                  label: 'ARCHIVE RENDERING',
                  found: [
                    {
                      sentence: 'She says, "I will come back\\ tomorrow."\nThen she naps.',
                      matched: 'She says',
                      rule: 's-ending-verb',
                    },
                  ],
                },
              ],
            },).slice(3,),).toEqual([
              'ARCHIVE RENDERING:',
              String.raw`- "She says, \"I will come back\\ tomorrow.\"\nThen she naps." (matched "She says", rule s-ending-verb)`,
            ],);
          },
        },),

        it({
          name: 'FORMATS nothing when there are no groups',
          fn: async () => {
            expect(tenseCandidateLines({ groups: [], },),).toEqual([],);
          },
        },),
      ],
    },),
  ],
},);

//endregion Tense candidate block tests
