/**
 Tests for the page-level facts a preparation hands every judging sheet and
 every cache key: the declared names, the cited pages, the recorded death and
 the subject the translation names.

 Every case asserts the whole value read, over a real preparation. Fixtures are
 cat-themed invention; no corpus text appears here.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { preparedSheetFactsOf, } from '@monochromatic-dev/translation-repair-corpus-source';
import { prepareDocumentPair, } from '../dist/final/node/index.mjs';

//region Prepared sheet facts tests

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: preparedSheetFactsOf.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'CARRIES every fact a page has: the declared names, the recorded death, the name the translation declares and the cited pages',
          fn: async () => {
            expect(preparedSheetFactsOf({
              prepared: prepareDocumentPair({
                sourceText: '---\nname: 咪咪\n---\n\n猫睡觉。\n',
                targetText: '---\nname: Mittens\n---\n\nThe cat naps.\n',
                referenceContext: '- reference 1 https://blog.example/mittens ("In memory of Mittens"): Mittens was a tabby.',
                sourceEntryInfo: { info: { died: '2025-02-26', }, },
              },),
            },),).toEqual({
              identityContext: '- name: ORIGINAL declares "咪咪", TRANSLATION declares "Mittens"\n'
                + '- died: the date this person died, as the corpus records it: "2025-02-26".',
              lifeEnded: true,
              subjectNames: ['Mittens',],
              referenceContext: '- reference 1 https://blog.example/mittens ("In memory of Mittens"): Mittens was a tabby.',
            },);
          },
        },),

        it({
          name: 'CARRIES every name the translation declares for its subject, the aliases after the name, a string alias split at its commas',
          fn: async () => {
            expect(preparedSheetFactsOf({
              prepared: prepareDocumentPair({
                sourceText: '---\nname: 咪咪\n---\n\n猫睡觉。\n',
                targetText: '---\nname: Mittens\ninfo:\n  alias: Whiskers, Tom\n---\n\nThe cat naps.\n',
                sourceEntryInfo: { info: { died: '2025-02-26', }, },
              },),
            },).subjectNames,).toEqual([
              'Mittens',
              'Whiskers',
              'Tom',
            ],);
          },
        },),

        it({
          name: 'CARRIES every name the translation declares for its subject where the alias is a list',
          fn: async () => {
            expect(preparedSheetFactsOf({
              prepared: prepareDocumentPair({
                sourceText: '---\nname: 咪咪\n---\n\n猫睡觉。\n',
                targetText: '---\nname: Mittens\ninfo:\n  alias:\n    - Whiskers\n    - Tom\n---\n\nThe cat naps.\n',
                sourceEntryInfo: { info: { died: '2025-02-26', }, },
              },),
            },).subjectNames,).toEqual([
              'Mittens',
              'Whiskers',
              'Tom',
            ],);
          },
        },),

        it({
          name: 'LEAVES OUT each fact the page lacks, the death included',
          fn: async () => {
            expect(preparedSheetFactsOf({
              prepared: prepareDocumentPair({
                sourceText: '猫睡觉。\n',
                targetText: 'The cat naps.\n',
              },),
            },),).toEqual({},);
          },
        },),

        it({
          name: 'LEAVES OUT the subject name where no death is recorded, though the translation declares one',
          fn: async () => {
            expect(preparedSheetFactsOf({
              prepared: prepareDocumentPair({
                sourceText: '---\nname: 咪咪\n---\n\n猫睡觉。\n',
                targetText: '---\nname: Mittens\n---\n\nThe cat naps.\n',
              },),
            },),).toEqual({
              identityContext: '- name: ORIGINAL declares "咪咪", TRANSLATION declares "Mittens"',
            },);
          },
        },),
      ],
    },),
  ],
},);

//endregion Prepared sheet facts tests
