/**
 Tests for comparing the provider's offering against the compiled catalog.

 The case that matters is the alias. The provider serves ids that are not
 distinct models, and admitting one would let a single model take two seats on
 a voting panel, so one opinion would be counted as two independent
 confirmations. The other case that matters is a catalog id the provider has
 dropped: that already happened twice on 2026-08-05 and cost a lost voice per
 call, silently, because 404 is not a transient status.

 Model ids here are the real ones, since the rule under test is about their
 relationships. No corpus text is involved.

 @module
 */

import {
  caught,
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { SYNTHETIC_MODELS, } from '@monochromatic-dev/translation-repair-provider';
import { ArtifactParseError, } from '@monochromatic-dev/translation-repair-artifact-pool';
import {
  CATALOG_MODEL_IDS,
  compareCatalog,
  decodeModelList,
  formatCatalogReport,
} from '../../dist/final/node/index.mjs';
import {
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';

/**
 Catalog the comparisons run against.
 */
const CATALOG: readonly string[] = [
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_WITHHELD,
];

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: 'CATALOG_MODEL_IDS',
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'IS the compiled Synthetic catalog, so a model removed from the catalog is removed from the drift '
            + 'check the same day, rather than a copy that reported a departed model as expected',
          fn: async () => {
            expect([...CATALOG_MODEL_IDS,].toSorted(),).toEqual(Object.keys(SYNTHETIC_MODELS,).toSorted(),);
            expect(CATALOG_MODEL_IDS.includes('hf:zai-org/GLM-4.7-Flash',),).toBe(false,);
          },
        },),
      ],
    },),

    describe({
      name: compareCatalog.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'reports an id the provider no longer serves, which is the failure '
            + 'that already happened: two ids began answering 404, 404 is not in '
            + 'the transient retry set, and every stage holding one lost a voice '
            + 'per call while nothing said why',
          fn: async () => {
            expect(compareCatalog({
              served: [{ id: SEAT_HYPER_OPENROUTER_VISION_EDITOR, huggingFaceId: 'zai-org/GLM-5.3-Flash', },],
              catalog: CATALOG,
            },).missing,).toStrictEqual([SEAT_SYNTHETIC_VISION_WITHHELD,],);
          },
        },),

        it({
          name: 'counts an ALIAS as an alias rather than as a new model, because '
            + 'seating it would let one model vote twice on a panel and a single '
            + 'opinion would read as two independent confirmations',
          fn: async () => {
            /**
             Provider list carrying an alias onto a seated model.
             */
            const comparison = compareCatalog({
              served: [
                {
                  id: SEAT_HYPER_OPENROUTER_VISION_EDITOR,
                  huggingFaceId: 'zai-org/GLM-5.3-Flash',
                },
                {
                  id: SEAT_SYNTHETIC_VISION_WITHHELD,
                  huggingFaceId: 'moonshotai/Kimi-K3',
                },
                {
                  id: 'syn:large:vision',
                  huggingFaceId: 'moonshotai/Kimi-K3',
                },
              ],
              catalog: CATALOG,
            },);

            expect(comparison.unlisted,).toHaveLength(0,);
            expect(comparison.aliases
              .map(function toId(model,) {
                return model.id;
              },),).toStrictEqual(['syn:large:vision',],);
          },
        },),

        it({
          name: 'reports a genuinely new model as UNLISTED, which is the only kind '
            + 'that could judge an issue independently: critics, panel and judges '
            + 'are all the same roster, so every seated model already ruled on '
            + 'every issue',
          fn: async () => {
            expect(compareCatalog({
              served: [
                {
                  id: SEAT_HYPER_OPENROUTER_VISION_EDITOR,
                  huggingFaceId: 'zai-org/GLM-5.3-Flash',
                },
                {
                  id: SEAT_SYNTHETIC_VISION_WITHHELD,
                  huggingFaceId: 'moonshotai/Kimi-K3',
                },
                {
                  id: 'hf:someone/Newcomer-1',
                  huggingFaceId: 'someone/Newcomer-1',
                },
              ],
              catalog: CATALOG,
            },).unlisted
              .map(function toId(model,) {
                return model.id;
              },),).toStrictEqual(['hf:someone/Newcomer-1',],);
          },
        },),

        it({
          name: 'counts TWO aliases onto one new model as one new model and one '
            + 'alias, so a provider that exposes a model under several names '
            + 'cannot inflate how many independent voices are available',
          fn: async () => {
            /**
             One new model served under two ids.
             */
            const comparison = compareCatalog({
              served: [
                {
                  id: 'hf:someone/Newcomer-1',
                  huggingFaceId: 'someone/Newcomer-1',
                },
                {
                  id: 'syn:newcomer',
                  huggingFaceId: 'someone/Newcomer-1',
                },
              ],
              catalog: CATALOG,
            },);

            expect(comparison.unlisted,).toHaveLength(1,);
            expect(comparison.aliases,).toHaveLength(1,);
          },
        },),

        it({
          name: 'reports nothing in any direction when the provider serves exactly '
            + 'the catalog, so a clean check is distinguishable from one that '
            + 'failed to compare anything',
          fn: async () => {
            /**
             Provider list matching the catalog exactly.
             */
            const comparison = compareCatalog({
              served: [
                {
                  id: SEAT_HYPER_OPENROUTER_VISION_EDITOR,
                  huggingFaceId: 'zai-org/GLM-5.3-Flash',
                },
                {
                  id: SEAT_SYNTHETIC_VISION_WITHHELD,
                  huggingFaceId: 'moonshotai/Kimi-K3',
                },
              ],
              catalog: CATALOG,
            },);

            expect(comparison.missing,).toHaveLength(0,);
            expect(comparison.unlisted,).toHaveLength(0,);
            expect(comparison.aliases,).toHaveLength(0,);
          },
        },),

        it({
          name: 'CLAIMS an underlying model under either spelling, so an alias onto one already seated is a '
            + 'second seat rather than a new voice',
          fn: async () => {
            // Neither catalog id is served under its own spelling, so both are
            // missing, and each served model is one the catalog seats: the
            // catalog spells one with the `hf:` prefix and one without.
            expect(compareCatalog({
              served: [{
                id: 'glm-5.3-seat',
                huggingFaceId: 'zai-org/GLM-5.3-Flash',
              }, {
                id: 'mao-1-seat',
                huggingFaceId: 'cats/Mao-1',
              },],
              catalog: ['hf:zai-org/GLM-5.3-Flash', 'cats/Mao-1',],
            },),).toEqual({
              unlisted: [],
              aliases: [{
                id: 'glm-5.3-seat',
                huggingFaceId: 'zai-org/GLM-5.3-Flash',
              }, {
                id: 'mao-1-seat',
                huggingFaceId: 'cats/Mao-1',
              },],
              missing: ['hf:zai-org/GLM-5.3-Flash', 'cats/Mao-1',],
            },);
          },
        },),
      ],
    },),

    describe({
      name: decodeModelList.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'reads id and underlying model from a well-formed list',
          fn: async () => {
            expect(decodeModelList({
              body: {
                data: [
                  {
                    id: 'syn:large:text',
                    hugging_face_id: 'zai-org/GLM-5.2',
                  },
                ],
              },
            },),).toStrictEqual([
              {
                id: 'syn:large:text',
                huggingFaceId: 'zai-org/GLM-5.2',
              },
            ],);
          },
        },),

        it({
          name: 'leaves the underlying model EMPTY when the provider states none, '
            + 'rather than inventing one, since an id with no stated model is its '
            + 'own identity and guessing would merge two distinct voices',
          fn: async () => {
            expect(decodeModelList({ body: { data: [{ id: 'hf:someone/Newcomer-1', },], }, },),)
              .toStrictEqual([
                {
                  id: 'hf:someone/Newcomer-1',
                  huggingFaceId: '',
                },
              ],);
          },
        },),

        it({
          name: 'THROWS on a body with no data array rather than reporting an '
            + 'empty offering, because an empty list reads as "the provider serves '
            + 'nothing" and would mark the whole catalog missing',
          fn: async () => {
            expect(function decodeGarbage() {
              return decodeModelList({ body: { models: [], }, },);
            },).toThrow();
          },
        },),

        it({
          name: 'THROWS on an entry whose id is not a string, so a half-read list '
            + 'cannot quietly under-report what the provider serves',
          fn: async () => {
            expect(function decodeBadEntry() {
              return decodeModelList({ body: { data: [{ id: 7, },], }, },);
            },).toThrow();
          },
        },),

        it({
          name: 'REFUSES a models body that is no object and an entry that is no object, naming where, since '
            + 'nothing there names a served model',
          fn: async () => {
            /**
             What the read throws for a body that is a number.
             */
            const bodyRefusal = caught(function readsBody(): void {
              decodeModelList({ body: 5, },);
            },);
            expect(bodyRefusal,).toBeInstanceOf(ArtifactParseError,);
            expect(String(bodyRefusal,),).toBe('ArtifactParseError: artifact parse failed at models: expected an object.',);
            /**
             What the read throws for an entry that is a number.
             */
            const entryRefusal = caught(function readsEntry(): void {
              decodeModelList({ body: { data: [5,], }, },);
            },);
            expect(entryRefusal,).toBeInstanceOf(ArtifactParseError,);
            expect(String(entryRefusal,),).toBe(
              'ArtifactParseError: artifact parse failed at models.data[]: expected an object.',
            );
          },
        },),
      ],
    },),

    describe({
      name: formatCatalogReport.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'REPORTS the owner blocklist under both spellings, the unlisted model it does not block, the '
            + 'catalog ids the provider dropped and the alias, line for line',
          fn: async () => {
            const comparison = compareCatalog({
              served: [{
                id: 'qwen3.8-max',
                huggingFaceId: 'qwen/qwen3.8-max',
              }, {
                id: 'glm-5.2-seat',
                huggingFaceId: 'zai-org/GLM-5.2',
              }, {
                id: 'mao-1',
                huggingFaceId: 'cats/Mao-1',
              }, {
                id: 'glm-5.3-second',
                huggingFaceId: 'zai-org/GLM-5.3-Flash',
              },],
              catalog: ['cats/Mao-2', 'hf:zai-org/GLM-5.3-Flash',],
            },);
            // The unblocked model's line ends at its spelling: a blocked one
            // carries the same opening and then its reason.
            expect(formatCatalogReport({ comparison, },),).toBe([
              'MISSING from the provider but still in the catalog: 2',
              '  cats/Mao-2  <- every call on this loses a voice to a 404',
              '  hf:zai-org/GLM-5.3-Flash  <- every call on this loses a voice to a 404',
              'UNLISTED distinct models the provider serves: 3',
              '  qwen3.8-max  (qwen/qwen3.8-max)  BLOCKED by owner: absurd cost in money',
              '  glm-5.2-seat  (zai-org/GLM-5.2)  BLOCKED by owner: too outdated',
              '  mao-1  (cats/Mao-1)',
              'ALIASES onto models already seated: 1',
              '  glm-5.3-second -> zai-org/GLM-5.3-Flash',
            ].join('\n',),);
          },
        },),
      ],
    },),
  ],
},);
