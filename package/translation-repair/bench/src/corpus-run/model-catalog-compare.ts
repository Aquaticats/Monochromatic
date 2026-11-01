import { ArtifactParseError, } from '@monochromatic-dev/translation-repair-artifact-pool/ts';
import {
  isJsonArray,
  isJsonRecord,
} from '@monochromatic-dev/translation-repair-document-structure/ts';
import { blocklistVerdictFor, } from '@monochromatic-dev/translation-repair-stage/ts';
import { SYNTHETIC_MODELS, } from '@monochromatic-dev/translation-repair-provider/ts';

//region Model catalog comparison
// The pure half of the drift check: decoding what the provider returned and
// comparing it against the compiled catalog. Kept apart from the fetch so the
// comparison is testable without a network.
//
// ALIASES ARE THE SUBTLE PART. The provider serves ids that are not distinct
// models: `syn:large:text` and the rest each point at a model already listed,
// which the endpoint states in its own `hugging_face_id` field. Admitting one
// would let a single model occupy two seats on a voting panel, so one opinion
// would count as two independent confirmations. Deduplication is therefore on
// `hugging_face_id`, never on `id`.

/**
 Ids the pipeline compiles against on this provider, read off the catalog.

 DERIVED RATHER THAN WRITTEN OUT. The copy this replaced still listed a model
 removed on 2026-08-24, so the instrument built to catch catalog removals
 reported the departed model as expected and would have read a served one as
 new; a list read off `SYNTHETIC_MODELS` moves with the catalog. Synthetic's
 endpoint is what the fetch compares against, so the Hyper-only ids are not
 in it on purpose.
 */
export const CATALOG_MODEL_IDS: readonly string[] = Object.keys(SYNTHETIC_MODELS,);

/**
 One model as the provider describes it.

 @example
 ```ts
 const model: ServedModel = { id: 'syn:large:text', huggingFaceId: 'zai-org/GLM-5.2', };
 ```
 */
export type ServedModel = {
  /**
   Id a request names.
   */
  readonly id: string;

  /**
   Underlying model, which is what makes two ids the same voice. Empty when
   the provider stated none, which leaves the id its own identity.
   */
  readonly huggingFaceId: string;
};

/**
 What the comparison found.

 @example
 ```ts
 const comparison = compareCatalog({ served, catalog: CATALOG_MODEL_IDS, },);
 ```
 */
export type CatalogComparison = {
  /**
   Distinct models the provider serves that the catalog does not list.

   The interesting set: a model here holds no role in a run, so it is the
   only kind of model that could judge an issue independently.
   */
  readonly unlisted: readonly ServedModel[];

  /**
   Catalog ids the provider no longer serves.

   Each one costs a lost voice per call, silently, because 404 is not a
   transient status and the retry set does not cover it.
   */
  readonly missing: readonly string[];

  /**
   Served ids that resolve onto a model already counted, which must never
   take a second seat on a panel.
   */
  readonly aliases: readonly ServedModel[];
};

/**
 Reads one entry of the provider's model list.

 @param entry - single list element as parsed

 @returns Model, with an empty underlying id when none was stated

 @throws {@link Error} when the entry carries no usable id, since a list this
 report trusts must not be half-read

 @example
 ```ts
 const model = decodeModel({ entry, },);
 ```
 */
function decodeModel(
  {
    entry,
  }: {
    readonly entry: unknown;
  },
): ServedModel {
  if (!isJsonRecord(entry,))
    throw new ArtifactParseError({
      path: 'models.data[]',
      reason: 'an object',
    },);

  /**
   Requestable id, and the underlying model when the provider stated one.
   */
  const {
    id,
    hugging_face_id: huggingFaceId,
  } = entry;
  if ((typeof id) !== 'string')
    throw new ArtifactParseError({
      path: 'models.data[].id',
      reason: 'a string',
    },);

  return {
    id,
    huggingFaceId: ((typeof huggingFaceId) === 'string') ? huggingFaceId : '',
  };
}

/**
 Reads the provider's model-list response.

 @param body - parsed response body

 @returns Every model listed, in the order given

 @throws {@link Error} when the body carries no `data` array

 @example
 ```ts
 const served = decodeModelList({ body, },);
 ```
 */
export function decodeModelList(
  {
    body,
  }: {
    readonly body: unknown;
  },
): readonly ServedModel[] {
  if (!isJsonRecord(body,))
    throw new ArtifactParseError({
      path: 'models',
      reason: 'an object',
    },);

  /**
   Listed models, as the OpenAI-compatible shape names them.
   */
  const { data, } = body;
  if (!isJsonArray(data,))
    throw new ArtifactParseError({
      path: 'models.data',
      reason: 'an array',
    },);

  return data.map(function toModel(entry,): ServedModel {
    return decodeModel({ entry, },);
  },);
}

/**
 Compares what the provider serves against what the pipeline compiles against.

 @param served - every model the provider currently lists

 @param catalog - ids the pipeline may call

 @returns Drift in both directions, and the aliases held out of both

 @example
 ```ts
 const comparison = compareCatalog({ served, catalog: CATALOG_MODEL_IDS, },);
 ```
 */
export function compareCatalog(
  {
    served,
    catalog,
  }: {
    readonly served: readonly ServedModel[];
    readonly catalog: readonly string[];
  },
): CatalogComparison {
  /**
   Underlying models the catalog already occupies, so an alias onto one of
   them is recognizable as a second seat rather than a new voice.
   */
  const claimed = new Set(catalog.map(function toUnderlying(id,): string {
    return id.startsWith('hf:',) ? id.slice('hf:'.length,) : id;
  },),);

  /**
   Catalog membership, for the missing check.
   */
  const listed = new Set(catalog,);

  /**
   Served ids, for the missing check.
   */
  const servedIds = new Set(served.map(function toId(model,): string {
    return model.id;
  },),);

  /**
   Underlying models already reported as unlisted, so two aliases onto one
   new model do not read as two new models.
   */
  const reported = new Set<string>();

  /**
   Models the provider serves that hold no seat, and aliases onto ones that
   do, split in one pass over the served list.
   */
  const split = served.reduce(
    function classify(
      into: {
        readonly unlisted: ServedModel[];
        readonly aliases: ServedModel[];
      },
      model,
    ) {
      if (listed.has(model.id,))
        return into;
      if (claimed.has(model.huggingFaceId,)) {
        into.aliases
          .push(model,);
        return into;
      }
      if ((model.huggingFaceId !== '') && reported.has(model.huggingFaceId,)) {
        into.aliases
          .push(model,);
        return into;
      }
      if (model.huggingFaceId !== '')
        reported.add(model.huggingFaceId,);
      into.unlisted
        .push(model,);
      return into;
    },
    {
      unlisted: [],
      aliases: [],
    },
  );

  return {
    unlisted: split.unlisted,
    aliases: split.aliases,
    missing: catalog.filter(function isGone(id,): boolean {
      return !servedIds.has(id,);
    },),
  };
}

/**
 Renders the comparison for a human.

 @param comparison - what the comparison found

 @returns Report text

 @example
 ```ts
 console.log(formatCatalogReport({ comparison, },),);
 ```
 */
export function formatCatalogReport(
  {
    comparison,
  }: {
    readonly comparison: CatalogComparison;
  },
): string {
  /**
   Every line of the report, joined at the end.
   */
  const lines = [
    `MISSING from the provider but still in the catalog: ${String(comparison.missing
      .length,)}`,
    ...comparison.missing
      .map(function toLine(id,): string {
        return `  ${id}  <- every call on this loses a voice to a 404`;
      },),
    `UNLISTED distinct models the provider serves: ${String(comparison.unlisted
      .length,)}`,
    ...comparison.unlisted
      .map(function toLine(model,): string {
        /**
         Owner's verdict under the served spelling.
         */
        const direct = blocklistVerdictFor({ id: model.id, },);
        /**
         Verdict under either spelling, so an alias cannot slip a blocked
         model back into the candidate listing.
         */
        const verdict = direct.blocked
          ? direct
          : blocklistVerdictFor({ id: `hf:${model.huggingFaceId}`, },);
        return verdict.blocked
          ? `  ${model.id}  (${model.huggingFaceId})  BLOCKED by owner: ${verdict.reason}`
          : `  ${model.id}  (${model.huggingFaceId})`;
      },),
    `ALIASES onto models already seated: ${String(comparison.aliases
      .length,)}`,
    ...comparison.aliases
      .map(function toLine(model,): string {
        return `  ${model.id} -> ${model.huggingFaceId}`;
      },),
  ];

  return lines.join('\n',);
}

//endregion Model catalog comparison
