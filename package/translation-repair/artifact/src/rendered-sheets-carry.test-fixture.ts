/**
 The sheets of the page-level consistency pass beyond its first round, rendered
 for `rendered-sheets.test-fixture.ts` (ledger X17): the run-off among the
 finalists of a first round with no strict majority (the labels with the top
 count, and the labels next behind a lone leader), the writer asked to carry the
 chosen rendering to a receiving place, and the judge asked to choose among that
 place's candidates, in the first round and in the run-off.

 Fixtures are cat-themed invention; no corpus content appears here.

 @module
 */

import {
  buildCarryChoiceMessages,
  buildCarryWriterMessages,
  buildConsistencyRunOffMessages,
} from '@monochromatic-dev/translation-repair-consistency';

import {
  CHOSEN_PLACE,
  FIRST_VIEW,
  SECOND_VIEW,
  SENTENCE,
  THIRD_VIEW,
} from './rendered-sheets-carry-places.test-fixture.ts';
import { refrainCarrySheets, } from './rendered-sheets-carry-refrain.test-fixture.ts';
import {
  IDENTITY,
  joined,
  REFERENCES,
  type RenderedSheet,
  TRANSLATORS_NOTE,
} from '@monochromatic-dev/translation-repair-repair/ts/rendered-sheets-texts.test-fixture.ts';

//region Carry sheets

/**
 The sheets of the run-off, the writer and the choice among candidates, each
 also with a death recorded so the tense block is rendered once more, and the
 sheets of a refrain whose two places share one paragraph.

 @returns One entry per sheet

 @example
 ```ts
 const sheets = carrySheets();
 ```
 */
export function carrySheets(): readonly RenderedSheet[] {
  return [
    {
      name: 'page consistency run-off, repeated sentence',
      text: joined({
        messages: buildConsistencyRunOffMessages({
          subject: {
            repeat: SENTENCE,
            occurrences: [
              FIRST_VIEW,
              SECOND_VIEW,
            ],
            identityContext: IDENTITY,
            referenceContext: REFERENCES,
          },
          finalists: [
            'OCCURRENCE 1',
            'OCCURRENCE 2',
          ],
          renderings: [
            {
              kind: 'located',
              quote: 'The sunny window put the kitten to sleep',
              core: 'The sunny window put the kitten to sleep',
              start: 0,
            },
            {
              kind: 'located',
              quote: 'the sunlight on the sill lulled the kitten to sleep',
              core: 'the sunlight on the sill lulled the kitten to sleep',
              start: 16,
            },
          ],
        },),
      },),
    },
    {
      name: 'page consistency run-off, repeated sentence, death recorded',
      text: joined({
        messages: buildConsistencyRunOffMessages({
          subject: {
            repeat: SENTENCE,
            occurrences: [
              FIRST_VIEW,
              SECOND_VIEW,
            ],
            identityContext: IDENTITY,
            referenceContext: REFERENCES,
            lifeEnded: true,
            subjectNames: ['Mittens',],
          },
          finalists: [
            'OCCURRENCE 1',
            'OCCURRENCE 2',
          ],
          renderings: [
            {
              kind: 'located',
              quote: 'The sunny window put the kitten to sleep',
              core: 'The sunny window put the kitten to sleep',
              start: 0,
            },
            {
              kind: 'located',
              quote: 'the sunlight on the sill lulled the kitten to sleep',
              core: 'the sunlight on the sill lulled the kitten to sleep',
              start: 16,
            },
          ],
        },),
      },),
    },
    {
      // A run-off among a proper subset of the places: the sheet shows the finalists only, so the labels it names
      // are the first and third, never the second.
      name: 'page consistency run-off, repeated sentence, first and third of three places',
      text: joined({
        messages: buildConsistencyRunOffMessages({
          subject: {
            repeat: SENTENCE,
            occurrences: [
              FIRST_VIEW,
              SECOND_VIEW,
              THIRD_VIEW,
            ],
            identityContext: IDENTITY,
            referenceContext: REFERENCES,
          },
          finalists: [
            'OCCURRENCE 1',
            'OCCURRENCE 3',
          ],
          renderings: [
            {
              kind: 'located',
              quote: 'The sunny window put the kitten to sleep',
              core: 'The sunny window put the kitten to sleep',
              start: 0,
            },
            {
              kind: 'located',
              quote: 'the sunlight on the sill lulled the kitten to sleep',
              core: 'the sunlight on the sill lulled the kitten to sleep',
              start: 16,
            },
            {
              kind: 'located',
              quote: 'the sunny window put the kitten to sleep',
              core: 'the sunny window put the kitten to sleep',
              start: 9,
            },
          ],
        },),
      },),
    },
    {
      name: 'page consistency carry writer, repeated sentence',
      text: joined({
        messages: buildCarryWriterMessages({
          subject: {
            repeat: SENTENCE,
            chosen: {
              view: FIRST_VIEW,
              span: 'The sunny window put the kitten to sleep',
              paragraph: `The sunny window put the kitten to sleep. ${TRANSLATORS_NOTE}`,
            },
            receiving: [
              {
                view: SECOND_VIEW,
                span: 'the sunlight on the sill lulled the kitten to sleep',
              },
            ],
            paragraph: `In the evening, the sunlight on the sill lulled the kitten to sleep, and the fish snacks smiled too. ${TRANSLATORS_NOTE}`,
            keeping: [],
            before: '傍晚的阳光很暖。',
            after: '小鱼干也笑了。',
            lineStructured: false,
            identityContext: IDENTITY,
            referenceContext: REFERENCES,
          },
        },),
      },),
    },
    {
      name: 'page consistency carry writer, repeated sentence, death recorded',
      text: joined({
        messages: buildCarryWriterMessages({
          subject: {
            repeat: SENTENCE,
            chosen: {
              view: FIRST_VIEW,
              span: 'The sunny window put the kitten to sleep',
              paragraph: `The sunny window put the kitten to sleep. ${TRANSLATORS_NOTE}`,
            },
            receiving: [
              {
                view: SECOND_VIEW,
                span: 'the sunlight on the sill lulled the kitten to sleep',
              },
            ],
            paragraph: `In the evening, the sunlight on the sill lulled the kitten to sleep, and the fish snacks smiled too. ${TRANSLATORS_NOTE}`,
            keeping: [],
            lineStructured: true,
            identityContext: IDENTITY,
            referenceContext: REFERENCES,
            lifeEnded: true,
            subjectNames: ['Mittens',],
          },
        },),
      },),
    },
    {
      name: 'page consistency carry choice, repeated sentence',
      text: joined({
        messages: buildCarryChoiceMessages({
          subject: {
            repeat: SENTENCE,
            receiving: [
              {
                view: SECOND_VIEW,
                span: 'the sunlight on the sill lulled the kitten to sleep',
              },
            ],
            chosen: CHOSEN_PLACE,
            keeping: [],
            candidates: [
              {
                label: 'CANDIDATE 1',
                current: true,
                text: `In the evening, the sunlight on the sill lulled the kitten to sleep, and the fish snacks smiled too. ${TRANSLATORS_NOTE}`,
              },
              {
                label: 'CANDIDATE 2',
                current: false,
                text: `In the evening, the sunny window put the kitten to sleep, and the fish snacks smiled too. ${TRANSLATORS_NOTE}`,
              },
            ],
            identityContext: IDENTITY,
            referenceContext: REFERENCES,
          },
        },),
      },),
    },
    {
      // The run-off among finalists that are every candidate (an even split of two): its rules say a first round was split
      // between these candidates, so the sheet is never the first round's.
      name: 'page consistency carry choice run-off, repeated sentence, every candidate a finalist',
      text: joined({
        messages: buildCarryChoiceMessages({
          runOff: true,
          subject: {
            repeat: SENTENCE,
            receiving: [
              {
                view: SECOND_VIEW,
                span: 'the sunlight on the sill lulled the kitten to sleep',
              },
            ],
            chosen: CHOSEN_PLACE,
            keeping: [],
            candidates: [
              {
                label: 'CANDIDATE 1',
                current: true,
                text: `In the evening, the sunlight on the sill lulled the kitten to sleep, and the fish snacks smiled too. ${TRANSLATORS_NOTE}`,
              },
              {
                label: 'CANDIDATE 2',
                current: false,
                text: `In the evening, the sunny window put the kitten to sleep, and the fish snacks smiled too. ${TRANSLATORS_NOTE}`,
              },
            ],
            identityContext: IDENTITY,
            referenceContext: REFERENCES,
          },
        },),
      },),
    },
    {
      name: 'page consistency carry choice, repeated sentence, death recorded',
      text: joined({
        messages: buildCarryChoiceMessages({
          subject: {
            repeat: SENTENCE,
            receiving: [
              {
                view: SECOND_VIEW,
                span: 'the sunlight on the sill lulled the kitten to sleep',
              },
            ],
            chosen: CHOSEN_PLACE,
            keeping: [],
            candidates: [
              {
                label: 'CANDIDATE 1',
                current: true,
                text: `In the evening, the sunlight on the sill lulled Mittens to sleep, and the fish snacks smiled too. ${TRANSLATORS_NOTE}`,
              },
              {
                label: 'CANDIDATE 2',
                current: false,
                text: `In the evening, the sunny window put Mittens to sleep, and the fish snacks smiled too. ${TRANSLATORS_NOTE}`,
              },
            ],
            identityContext: IDENTITY,
            referenceContext: REFERENCES,
            lifeEnded: true,
            subjectNames: ['Mittens',],
          },
        },),
      },),
    },
    ...refrainCarrySheets(),
  ];
}

//endregion Carry sheets
