import { isJsonRecord, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import { errorName, } from '@monochromatic-dev/translation-repair-markup/ts';
import { NEURALWATT_MODELS, } from './neuralwatt-catalog.ts';

//region NeuralWatt tracking aliases
// THE NAMES A CALL ON THE COVERED MODELS CAN BE MADE UNDER BESIDES THEIR OWN. A tracking alias such as `nw-flash` points
// at one model and moves to another when NeuralWatt ships a newer one, and every alias has a `-flex` twin
// (`https://docs.neuralwatt.com/guides/tracking-aliases`, fetched 2026-10-20). "Your usage shows the alias name you
// sent, listed under the model that served it", so a by-model usage row for a call made with `nw-flash` carries
// `requested_model: "nw-flash"`, and a count that takes only a card's served id misses it. The owner uses DeepSeek
// V4.1 Flash on the same account, so the meter would undercount, which is the unsafe side for a stop that keeps calls
// off the plan.
//
// WHAT AN ALIAS POINTS AT IS READ, NEVER KEPT IN A TABLE, because aliases move: `GET /v1/models` lists each with
// `metadata.huggingface_id` set to the model it currently points at, so an alias counts when that field equals the
// one of a covered model's own entry. A speed lane never counts ("Speed lanes are billed normally").
//
// WHEN THE LISTING CANNOT BE READ the count falls back to what the usage row itself says: every `nw-` row that is not
// a speed lane and was served by a covered model, the served name being compared by its part after the last slash,
// lowercase (`deepseek-ai/DeepSeek-V4.1-Flash` is the covered `deepseek-v4.1-flash`). That can count a call the
// listing would have said is elsewhere, the safe side again.

/**
 Suffix of the flex lane's name.
 */
const FLEX_SUFFIX = '-flex';

/**
 Suffix of the speed lane's name, which the free energy does not cover.
 */
const SPEED_SUFFIX = '-speed';

/**
 What the models listing said about the aliases of the covered models.

 @example
 ```ts
 const map: NeuralWattAliasMap = { known: true, names: new Set(['nw-flash', 'nw-flash-flex',],), };
 ```
 */
export type NeuralWattAliasMap =
  | {
    readonly known: true;

    /**
     The alias names, flex twins included, that point at a covered model.
     */
    readonly names: ReadonlySet<string>;
  }
    | {
      readonly known: false;

    /**
     Why the listing gave no map, an authored phrase.
     */
      readonly why: string;
    };

/**
 The model a listing entry points at.

 @param entry - one entry of the listing's `data`

 @returns The entry's `metadata.huggingface_id` in a list of one, or an empty list where it has none

 @example
 ```ts
 const [pointsAt,] = pointerOf({ entry, },);
 ```
 */
function pointerOf({ entry, }: { readonly entry: Readonly<Record<string, unknown>>; },): readonly string[] {
  /**
   The entry's metadata, whatever it holds.
   */
  const { metadata, } = entry;
  if (!isJsonRecord(metadata,))
    return [];

  /**
   The pointer, unchecked.
   */
  const pointer = metadata.huggingface_id;
  return ((typeof pointer) === 'string') ? [pointer as string,] : [];
}

/**
 Reads which names are aliases of a covered model out of a models listing.

 @param bodyText - body of a 200 reply of `GET /v1/models`

 @returns The names, or why the listing gives none

 @example
 ```ts
 const map = aliasNamesOf({ bodyText: reply.bodyText, },);
 ```
 */
export function aliasNamesOf({ bodyText, }: { readonly bodyText: string; },): NeuralWattAliasMap {
  /**
   Parsed body, or why it will not parse.
   */
  const parsed = (function parse(): { readonly value: unknown } | { readonly why: string } {
    try {
      return { value: JSON.parse(bodyText,), };
    }
    catch (error) {
      return { why: `the listing is not valid JSON (${errorName({ error, },)})`, };
    }
  })();
  if ('why' in parsed)
    return {
      known: false,
      why: parsed.why,
    };
  if (!isJsonRecord(parsed.value,))
    return {
      known: false,
      why: 'the listing is not a JSON object',
    };

  /**
   The listing's entries.
   */
  const { data, } = parsed.value;
  if (!Array.isArray(data,))
    return {
      known: false,
      why: 'the listing has no data list',
    };

  /**
   The entries that carry a name.
   */
  const entries = (data as readonly unknown[]).filter(function isNamed(entry,): entry is Readonly<Record<string, unknown>> & { readonly id: string } {
    return isJsonRecord(entry,) && ((typeof entry.id) === 'string');
  },);

  /**
   Names the cards serve.
   */
  const coveredIds = new Set<string>(Object.keys(NEURALWATT_MODELS,),);

  /**
   What the covered models themselves are listed as.
   */
  const coveredPointers = new Set<string>(entries
    .filter(function isCovered(entry,): boolean {
      return coveredIds.has(entry.id,);
    },)
    .flatMap(function pointers(entry,): readonly string[] {
      return pointerOf({ entry, },);
    },),);
  if (coveredPointers.size === 0)
    return {
      known: false,
      why: 'the listing lists neither covered model with a huggingface_id',
    };
  return {
    known: true,
    names: new Set<string>(entries
      .filter(function pointsAtCovered(entry,): boolean {
        return (!coveredIds.has(entry.id,))
          && (!entry.id
            .endsWith(SPEED_SUFFIX,))
          && pointerOf({ entry, },)
            .some(function isCovered(pointer,): boolean {
              return coveredPointers.has(pointer,);
            },);
      },)
      .flatMap(function withTwin(entry,): readonly string[] {
        return entry.id
          .endsWith(FLEX_SUFFIX,) ? [entry.id,] : [
            entry.id,
            `${entry.id}${FLEX_SUFFIX}`,
          ];
      },),),
  };
}

/**
 Whether a usage row for a call made under a `nw-` name was served by a covered model, for the count that has no map.

 @param requestedModel - the name the call used

 @param servedModelId - the backend model that served it, as the usage route spells it

 @returns Whether the row counts against the free energy

 @example
 ```ts
 servedByCoveredAlias({ requestedModel: 'nw-flash', servedModelId: 'deepseek-ai/DeepSeek-V4.1-Flash', },); // true
 ```
 */
export function servedByCoveredAlias(
  {
    requestedModel,
    servedModelId,
  }: {
    readonly requestedModel: string;
    readonly servedModelId: string;
  },
): boolean {
  if ((!requestedModel.startsWith('nw-',)) || requestedModel.endsWith(SPEED_SUFFIX,))
    return false;
  return Object.hasOwn(
    NEURALWATT_MODELS,
    servedModelId.split('/',)
      .at(-1,)
      ?.toLowerCase()
      ?? '',
  );
}

//endregion NeuralWatt tracking aliases
