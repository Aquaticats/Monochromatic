import type {
  AnthropicCard,
  BedrockCard,
  DecisionsCard,
  DeepSeekCard,
  ModelCard,
  NeuralWattCard,
  OpenRouterCard,
  SeatHold,
  ServedCard,
  SyntheticCard,
  XiaomiCard,
} from './model-card.ts';
import { MODEL_CARDS, } from './model-cards.ts';
import {
  ANTHROPIC_SERVED_IDS,
  BEDROCK_SERVED_IDS,
  DEEPSEEK_SERVED_IDS,
  HYPER_SERVED_IDS,
  type HyperServedId,
  NEURALWATT_SERVED_IDS,
  OPENROUTER_SERVED_IDS,
  ROSTER_MODEL_IDS,
  type RosterModelId,
  SYNTHETIC_SERVED_IDS,
  XIAOMI_SERVED_IDS,
} from './roster-id.ts';

//region Model card derivation
// THE ONE PLACE THE CARDS ARE READ AS DATA. Every catalog, cap table and
// hold set is a projection of `MODEL_CARDS` through these helpers, so a
// card added or removed is seen everywhere at once and nowhere by hand.

/**
 Providers a card can carry a side for, in the order `roster-card` names
 them, and the one list its command line and refusals are written from.

 @example
 ```ts
 CARD_PROVIDERS.join('|',); // 'synthetic|hyper|openrouter|bedrock|neuralwatt|deepseek|xiaomi|anthropic'
 ```
 */
export const CARD_PROVIDERS = [
  'synthetic',
  'hyper',
  'openrouter',
  'bedrock',
  'neuralwatt',
  'deepseek',
  'xiaomi',
  'anthropic',
] as const;

/**
 Provider whose side of a card is being asked for.

 @example
 ```ts
 const provider: CardProvider = 'hyper';
 ```
 */
export type CardProvider = (typeof CARD_PROVIDERS)[number];

/**
 A card beside the roster id it sits under.

 @example
 ```ts
 const card: RosterCard | undefined = ROSTER_CARDS.find(function isMinimax(entry,) { return entry.id === 'minimax-m3'; },);
 ```
 */
export type RosterCard = ModelCard & {
  /**
   Roster identity, one per model however many providers reach it.
   */
  readonly id: RosterModelId;
};

/**
 A roster card that carries one provider's side.

 @example
 ```ts
 const card: ServingCard<'hyper'> = cardsServing({ provider: 'hyper', },)[0];
 ```
 */
export type ServingCard<Provider extends CardProvider> = RosterCard & Required<Pick<ModelCard, Provider>>;

/**
 Spelling one provider serves a card's model under.

 @example
 ```ts
 const servedId: ServedIdOf<'hyper'> = 'minimax-m3';
 ```
 */
export type ServedIdOf<Provider extends CardProvider> = NonNullable<ModelCard[Provider]>['id'];

/**
 Every provider's served spellings, keyed by provider, for the key check
 on a derived record.
 */
type ServedIdLists = {
  readonly [Provider in CardProvider]: readonly ServedIdOf<Provider>[];
};

/**
 Reader of one provider's side off a card that carries it, keyed by
 provider, so a generic provider can reach its side without indexing a
 generic key.
 */
type SideReaders = {
  readonly [Provider in CardProvider]: (card: ServingCard<Provider>,) => NonNullable<ModelCard[Provider]>;
};

/**
 Each provider's side reader.
 */
const SIDE_OF: SideReaders = {
  synthetic: function syntheticSide(card,): SyntheticCard {
    return card.synthetic;
  },
  hyper: function hyperSide(card,): ServedCard<HyperServedId> {
    return card.hyper;
  },
  openrouter: function openRouterSide(card,): OpenRouterCard {
    return card.openrouter;
  },
  bedrock: function bedrockSide(card,): BedrockCard {
    return card.bedrock;
  },
  neuralwatt: function neuralWattSide(card,): NeuralWattCard {
    return card.neuralwatt;
  },
  deepseek: function deepSeekSide(card,): DeepSeekCard {
    return card.deepseek;
  },
  xiaomi: function xiaomiSide(card,): XiaomiCard {
    return card.xiaomi;
  },
  anthropic: function anthropicSide(card,): AnthropicCard {
    return card.anthropic;
  },
};

/**
 Every provider's served spellings, as the roster lists them.
 */
const SERVED_IDS: ServedIdLists = {
  synthetic: SYNTHETIC_SERVED_IDS,
  hyper: HYPER_SERVED_IDS,
  openrouter: OPENROUTER_SERVED_IDS,
  bedrock: BEDROCK_SERVED_IDS,
  neuralwatt: NEURALWATT_SERVED_IDS,
  deepseek: DEEPSEEK_SERVED_IDS,
  xiaomi: XIAOMI_SERVED_IDS,
  anthropic: ANTHROPIC_SERVED_IDS,
};

/**
 Every card beside its id, in roster order.

 @example
 ```ts
 const cards = ROSTER_CARDS;
 ```
 */
export const ROSTER_CARDS: readonly RosterCard[] = ROSTER_MODEL_IDS
  .map(function withId(id,): RosterCard {
    return {
      id,
      ...MODEL_CARDS[id],
    };
  },);

/**
 Cards carrying one provider's side, in roster order.

 @param provider - whose side is wanted

 @returns Cards that provider serves, typed to carry that side

 @example
 ```ts
 const served = cardsServing({ provider: 'openrouter', },);
 ```
 */
export function cardsServing<Provider extends CardProvider>(
  { provider, }: { readonly provider: Provider; },
): readonly ServingCard<Provider>[] {
  return ROSTER_CARDS.filter(function serves(card,): card is ServingCard<Provider> {
    return card[provider] !== undefined;
  },);
}

/**
 Roster models held out of one role.

 @param hold - role hold to collect

 @returns Ids carrying that hold, in roster order

 @example
 ```ts
 const unmeasured = holdSet({ hold: 'writer-unmeasured', },);
 ```
 */
export function holdSet(
  { hold, }: { readonly hold: SeatHold; },
): ReadonlySet<RosterModelId> {
  return new Set(ROSTER_CARDS
    .filter(function held(card,): boolean {
      /**
       Roles this card is held out of.
       */
      const { holds, } = card;
      return holds.includes(hold,);
    },)
    .map(function toId(card,): RosterModelId {
      return card.id;
    },),);
}

/**
 Roster models only the decisions endpoint serves, in roster order: cards
 carrying a decisions side, which by construction carry no chat side.

 @example
 ```ts
 const seats = DECISION_ONLY_ROSTER_IDS;
 ```
 */
export const DECISION_ONLY_ROSTER_IDS: readonly RosterModelId[] = ROSTER_CARDS
  .filter(function decides(card,): boolean {
    return card.decisions !== undefined;
  },)
  .map(function toId(card,): RosterModelId {
    return card.id;
  },);

/**
 The decision-only seats, keyed for the per-call lookup.
 */
const DECISION_SEATS: ReadonlySet<RosterModelId> = new Set(DECISION_ONLY_ROSTER_IDS,);

/**
 Whether one roster model is a decision-only seat, which no chat client can
 take and only a stage with a decision adapter asks.

 @param modelId - roster model to look up

 @returns Whether its card carries a decisions side

 @example
 ```ts
 const decides = isDecisionSeat({ modelId: 'typesafe/jev-1.13', },);
 ```
 */
export function isDecisionSeat(
  { modelId, }: { readonly modelId: RosterModelId; },
): boolean {
  // A SET LOOKUP RATHER THAN A CARD READ, because the stage call asks this
  // of every seat it is handed and unit tests hand it seats with no card.
  return DECISION_SEATS.has(modelId,);
}

/**
 Raised when a decisions side is read off a card that carries none.

 @example
 ```ts
 throw new DecisionsCardMissingError({ modelId: 'minimax-m3', },);
 ```
 */
export class DecisionsCardMissingError extends Error {
  /**
   Declares this message safe to forward: it names a roster model and nothing else.
   */
  readonly messageNamesOnly: true = true;

  /**
   Builds the refusal naming the model.

   @param modelId - roster model whose card carries no decisions side

   @example
   ```ts
   new DecisionsCardMissingError({ modelId: 'minimax-m3', },);
   ```
   */
  public constructor({ modelId, }: { readonly modelId: RosterModelId; },) {
    super(`${modelId} is not a decision seat: its card carries no decisions side`,);
    this.name = 'DecisionsCardMissingError';
  }
}

/**
 The decisions side of one roster model's card.

 @param modelId - roster model to look up

 @returns Its decisions side

 @throws {@link DecisionsCardMissingError} when the card carries none

 @example
 ```ts
 const side = decisionsCardOf({ modelId: 'typesafe/jev-1.13', },);
 ```
 */
export function decisionsCardOf(
  { modelId, }: { readonly modelId: RosterModelId; },
): DecisionsCard {
  /**
   Side as the card carries it, or nothing.
   */
  const side = MODEL_CARDS[modelId]
    .decisions;
  if (side === undefined)
    throw new DecisionsCardMissingError({ modelId, },);
  return side;
}

/**
 Makes the check that a record built by `Object.fromEntries`, whose keys
 the type system widened to `string`, carries every key on a list and no
 other, which is what lets the record be read under the list's union.

 @param keys - keys the record must carry exactly

 @returns Predicate over a record with widened keys

 @example
 ```ts
 const isRosterKeyed = keyedBy({ keys: ROSTER_MODEL_IDS, },);
 ```
 */
export function keyedBy<Key extends string, Value>(
  { keys, }: { readonly keys: readonly Key[]; },
): (record: Readonly<Record<string, Value>>,) => record is Readonly<Record<Key, Value>> {
  return function isKeyed(record: Readonly<Record<string, Value>>,): record is Readonly<Record<Key, Value>> {
    /**
     Keys the record carries.
     */
    const carried = Object.keys(record,);
    return (carried.length === keys.length) && keys.every(function present(key,): boolean {
      return Object.hasOwn(
        record,
        key,
      );
    },);
  };
}

/**
 Record over exactly the given keys, each value read off a function.

 @param keys - every key, once

 @param of - value for one key

 @returns The record, read under the keys' union

 @throws When the keys repeat, which is the one way the built record can
 miss one

 @example
 ```ts
 const caps = recordOver({ keys: ROSTER_MODEL_IDS, of: function cap(): number { return 1; }, },);
 ```
 */
export function recordOver<Key extends string, Value>(
  {
    keys,
    of,
  }: {
    readonly keys: readonly Key[];
    readonly of: (key: Key,) => Value;
  },
): Readonly<Record<Key, Value>> {
  /**
   Record with its keys widened to `string`, as `Object.fromEntries` types it.
   */
  const built: Readonly<Record<string, Value>> = Object.fromEntries(keys.map(function entry(key,): readonly [
    Key,
    Value,
  ] {
    return [
      key,
      of(key,),
    ];
  },),);
  if (!keyedBy<Key, Value>({ keys, },)(built,))
    throw new RangeError(`keys repeat: ${keys.join(', ',)}`,);
  return built;
}

/**
 Record over entries whose ids must each appear once.

 @param entries - id and value pairs, in the order they were read

 @param owner - whose entries these are, so the refusal says where the repeat sits

 @returns The record, with its keys widened to `string`

 @throws When an id appears more than once, since the later row would replace
 the earlier and drop it without a word

 @example
 ```ts
 const record = recordOfDistinctIds({ entries: [['cat', 3,], ['kitten', 6,],], owner: 'tabby cards', },);
 ```
 */
export function recordOfDistinctIds<Value>(
  {
    entries,
    owner,
  }: {
    readonly entries: readonly (readonly [
      string,
      Value,
    ])[];
    readonly owner: string;
  },
): Readonly<Record<string, Value>> {
  /**
   Values by id; a map until handed back, as every record filled by a key is
   (ledger B77).
   */
  const byId = new Map<string, Value>();
  for (const [id, value,] of entries) {
    if (byId.has(id,))
      throw new RangeError(`${owner} carry the id ${id} more than once`,);
    byId.set(
      id,
      value,
    );
  }
  return Object.fromEntries(byId,);
}

/**
 Record over one provider's served spellings, each row read off the card
 that carries the spelling.

 @param provider - whose spellings key the record

 @param toRow - row for one card

 @returns The record, read under that provider's served-id union

 @throws {@link Error} naming an unreachable state when a served spelling
 on the roster's list has no card or a card's spelling is off the list,
 which only an edit to the cards or the list alone can cause

 @throws {@link RangeError} when two cards carry one spelling, which the key
 check after the build could not see since the later card would have
 replaced the earlier

 @example
 ```ts
 const rows = servedRecord({ provider: 'hyper', toRow: function row(card,): number { return card.hyper.maxOutputLength; }, },);
 ```
 */
export function servedRecord<Provider extends CardProvider, Row>(
  {
    provider,
    toRow,
  }: {
    readonly provider: Provider;
    readonly toRow: (card: ServingCard<Provider>,) => Row;
  },
): Readonly<Record<ServedIdOf<Provider>, Row>> {
  /**
   Cards this provider serves.
   */
  const served = cardsServing({ provider, },);
  /**
   Record with its keys widened to `string`, as `Object.fromEntries` types it.
   */
  const built: Readonly<Record<string, Row>> = recordOfDistinctIds({
    entries: served.map(function entry(card,): readonly [
      string,
      Row,
    ] {
      /**
       This provider's side of the card.
       */
      const side = SIDE_OF[provider](card,);
      return [
        side.id,
        toRow(card,),
      ];
    },),
    owner: `${provider} cards`,
  },);
  /**
   Spellings the roster lists for this provider.
   */
  const keys: readonly ServedIdOf<Provider>[] = SERVED_IDS[provider];
  // BOTH SIDES ARE THE PACKAGE'S OWN CONSTANTS (`MODEL_CARDS` and the served-id
  // lists of `roster-id.ts`), so no caller's input can make them disagree; an
  // edit to one of them alone does, and says so at the first import.
  if (!keyedBy<ServedIdOf<Provider>, Row>({ keys, },)(built,))
    throw new Error(`unreachable: ${provider} cards and served ids disagree: cards ${
      Object.keys(built,)
        .join(', ',)
    }; list ${keys.join(', ',)}`,);
  return built;
}

//endregion Model card derivation
