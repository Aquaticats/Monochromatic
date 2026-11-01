import type { JsonSchemaResponseFormat, } from '@monochromatic-dev/translation-repair-provider/ts';
import {
  shownLabelOf,
  shownLabelReaderOf,
} from './consistency-label-form.ts';
import {
  isJsonArray,
  isJsonRecord,
} from '@monochromatic-dev/translation-repair-document-structure/ts';
import { rendersAsNothing, } from '@monochromatic-dev/translation-repair-typography/ts';

//region Consistency wire
// The one question the page-level consistency pass asks about a repeated
// source wording: the page renders it in several places, the renderings
// differ, and a judge picks the one every place should follow, or says which
// places call for their own.
//
// THE JUDGE NAMES WORDS, NEVER WRITES THEM. The ballot quotes the exact words
// of each place's English that render the repeated wording, and the pass
// locates them in the place's own English (`consistency-fold.ts`): a quote is
// evidence of where it points, never text to splice (the owner's ruling of
// 2026-10-11). What carries the chosen rendering to another place is chosen
// among candidates by the bench and held to every floor
// (`corpus-run/pass-consistency-carry.ts`): the chosen words moved in by code,
// where the place's words can be moved whole, and the blocks the translate
// stage's writers write; no judge's quote and no model's text reaches the page
// without that choice and those floors.
//
// REASON BEFORE CHOICE (the owner's ruling of 2026-10-11): the reply schema
// declares the reason first, the sheet's reply shape shows it first, and the
// sheet says to write it before choosing.
//
// THE REPLY AND THE STORED BALLOT SPELL TWO KEYS DIFFERENTLY (ledger B762). The reply calls the reason `assessment` and the
// quotes `candidateQuotes`, names that sort before `choice`, so a provider that returns a tool call's keys alphabetically
// still gives the reason, then the quotes, then the choice and the places to keep. The ballot a gather stores (and the
// cache and the run's artifacts read back) keeps the names `reason` and `quotes` it was stored under, so no stored record
// changes spelling: `readConsistencyBallotWire` is the one place the reply becomes a stored ballot.

/**
 Word every place's label begins with.
 */
const LABEL_WORD = 'OCCURRENCE';

/**
 One place's quoted rendering, by label.

 @example
 ```ts
 const item: ConsistencyQuoteWire = { label: 'OCCURRENCE 1', quote: 'the cat naps', };
 ```
 */
export type ConsistencyQuoteWire = {
  /**
   Label of the place the quote is taken from.
   */
  readonly label: string;

  /**
   Exact words of that place's English that render the repeated wording, or
   an empty string when the judge cannot give them.
   */
  readonly quote: string;

  /**
   For a frame: the exact words inside the quote that render the varying
   slot. Absent for a repeated run with no slot.
   */
  readonly slot?: string;
};

/**
 One judge's reply about one repeated wording.

 @example
 ```ts
 const wire: ConsistencyBallotWire = { assessment: 'Both say it alike.', candidateQuotes: [], choice: 'OCCURRENCE 1', keep: [], };
 ```
 */
export type ConsistencyBallotWire = {
  /**
   Why the judge chose as it did, written before the choice.
   */
  readonly assessment: string;

  /**
   One quote per place.
   */
  readonly candidateQuotes: readonly ConsistencyQuoteWire[];

  /**
   Label of the place every place should follow.
   */
  readonly choice: string;

  /**
   Labels of the places whose surroundings call for different wording.
   */
  readonly keep: readonly string[];
};

/**
 One judge's ballot as the gather stores it and the counting reads it.

 @example
 ```ts
 const ballot: ConsistencyBallot = { reason: 'Both say it alike.', quotes: [], choice: 'OCCURRENCE 1', keep: [], };
 ```
 */
export type ConsistencyBallot = {
  /**
   Why the judge chose as it did.
   */
  readonly reason: string;

  /**
   One quote per place.
   */
  readonly quotes: readonly ConsistencyQuoteWire[];

  /**
   Label of the place every place should follow.
   */
  readonly choice: string;

  /**
   Labels of the places whose surroundings call for different wording.
   */
  readonly keep: readonly string[];
};

/**
 Label of the place at a position on the sheet.

 @param position - zero-based position among the sheet's places

 @returns Label, counting from one

 @example
 ```ts
 const label = consistencyLabel({ position: 0, },); // 'OCCURRENCE 1'
 ```
 */
export function consistencyLabel({ position, }: { readonly position: number; },): string {
  return `${LABEL_WORD} ${String(position + 1,)}`;
}

/**
 Labels of the first places of a sheet, in order.

 @param count - how many places the sheet shows

 @returns One label per place

 @example
 ```ts
 const labels = consistencyLabels({ count: 2, },); // ['OCCURRENCE 1', 'OCCURRENCE 2']
 ```
 */
export function consistencyLabels({ count, }: { readonly count: number; },): readonly string[] {
  return Array.from(
    { length: count, },
    function labelAt(
      _unused,
      position,
    ): string {
      return consistencyLabel({ position, },);
    },
  );
}

/**
 Whether a parsed quote has the shape of one.

 @param value - parsed item

 @returns Whether it carries a label and a quote, and a slot when it has one

 @example
 ```ts
 isConsistencyQuoteWire({ label: 'OCCURRENCE 1', quote: 'the cat naps', },);
 ```
 */
function isConsistencyQuoteWire(value: unknown,): value is ConsistencyQuoteWire {
  if (!isJsonRecord(value,))
    return false;
  if (((typeof value.label) !== 'string') || ((typeof value.quote) !== 'string'))
    return false;
  return (value.slot === undefined) || ((typeof value.slot) === 'string');
}

/**
 Whether a parsed reply has the shape of a ballot, whatever it names.

 @param value - parsed provider value

 @returns Whether it carries an assessment, a list of quotes, a choice and a list
 of kept labels

 @example
 ```ts
 isConsistencyBallotWire(JSON.parse(text,),);
 ```
 */
export function isConsistencyBallotWire(value: unknown,): value is ConsistencyBallotWire {
  if (!isJsonRecord(value,))
    return false;
  if (((typeof value.assessment) !== 'string') || ((typeof value.choice) !== 'string'))
    return false;
  if ((!isJsonArray(value.candidateQuotes,)) || (!isJsonArray(value.keep,)))
    return false;
  return value.candidateQuotes
    .every(isConsistencyQuoteWire,)
    && value.keep
      .every(function isLabel(entry,): boolean {
        return (typeof entry) === 'string';
      },);
}

/**
 Whether a parsed stored value has the shape of a stored ballot, whatever it names.

 @param value - parsed stored value, such as a voice read back from the cache

 @returns Whether it carries a reason, a list of quotes, a choice and a list of kept labels

 @example
 ```ts
 isConsistencyBallot(JSON.parse(text,),);
 ```
 */
export function isConsistencyBallot(value: unknown,): value is ConsistencyBallot {
  if (!isJsonRecord(value,))
    return false;
  if (((typeof value.reason) !== 'string') || ((typeof value.choice) !== 'string'))
    return false;
  if ((!isJsonArray(value.quotes,)) || (!isJsonArray(value.keep,)))
    return false;
  return value.quotes
    .every(isConsistencyQuoteWire,)
    && value.keep
      .every(function isLabel(entry,): boolean {
        return (typeof entry) === 'string';
      },);
}

/**
 The reply as the ballot a gather stores.

 @param wire - reply of the right shape

 @returns The ballot, its assessment stored as the reason and its candidate quotes as the quotes

 @example
 ```ts
 const ballot = readConsistencyBallotWire({ wire, },);
 ```
 */
export function readConsistencyBallotWire({ wire, }: { readonly wire: ConsistencyBallotWire; },): ConsistencyBallot {
  return {
    reason: wire.assessment,
    quotes: wire.candidateQuotes,
    choice: wire.choice,
    keep: wire.keep,
  };
}

/**
 The shown labels a list of names stands for, in order, as a list of one list: none when a name stands for no shown label (or
 for two), or when two names stand for one place.

 @param names - what a ballot gave as labels

 @param shown - labels the sheet showed

 @returns The shown label of each name, as a list of zero or one

 @example
 ```ts
 const [labels,] = shownOnceLabelsOf({ names: ['2',], shown: ['OCCURRENCE 1', 'OCCURRENCE 2',], },); // ['OCCURRENCE 2']
 ```
 */
function shownOnceLabelsOf(
  {
    names,
    shown,
  }: {
    readonly names: readonly string[];
    readonly shown: readonly string[];
  },
): readonly (readonly string[])[] {
  /**
   The lookup of a shown label by name, over the list read once for all the names.
   */
  const shownLabelFor = shownLabelReaderOf({ shown, },);
  /**
   The shown label each name stands for, none when it stands for none.
   */
  const found = names.map(function labelOf(name,): readonly string[] {
    return shownLabelFor(name,);
  },);
  /**
   The labels of the names that stood for one.
   */
  const labels = found.flat();
  return ((labels.length === names.length) && (new Set(labels,).size === labels.length)) ? [labels,] : [];
}

/**
 The ballot with the places it names written as the sheet showed them (its choice, its kept places and the label of each of its
 quotes, whatever case or form the judge wrote them in), when it can name them: every one stands for exactly one shown place and
 no place is kept twice or quoted twice. Every other field of the ballot is kept.

 @param ballot - ballot of the right shape, with whatever else the reply carries

 @param labels - labels the sheet showed

 @returns The ballot with the shown labels, as a list of zero or one

 @example
 ```ts
 const [canonical,] = canonicalBallotOf({ ballot, labels: consistencyLabels({ count: 2, },), },);
 ```
 */
export function canonicalBallotOf<BallotT extends ConsistencyBallot,>(
  {
    ballot,
    labels,
  }: {
    readonly ballot: BallotT;
    readonly labels: readonly string[];
  },
): readonly BallotT[] {
  /**
   The shown place the choice stands for.
   */
  const [chosen,] = shownLabelOf({
    name: ballot.choice,
    shown: labels,
  },);
  /**
   The shown places the kept places stand for.
   */
  const [kept,] = shownOnceLabelsOf({
    names: ballot.keep,
    shown: labels,
  },);
  /**
   The shown places the quotes were taken from.
   */
  const [quoted,] = shownOnceLabelsOf({
    names: ballot.quotes
      .map(function labelOf(quote,): string {
        return quote.label;
      },),
    shown: labels,
  },);
  if ((chosen === undefined) || (kept === undefined)
    || (quoted === undefined))
    return [];
  return [
    {
      ...ballot,
      choice: chosen,
      keep: kept,
      quotes: ballot.quotes
        .map(function quoteAt(
          quote,
          position,
        ): ConsistencyQuoteWire {
          return {
            ...quote,
            label: quoted[position] ?? quote.label,
          };
        },),
    },
  ];
}

/**
 Whether a ballot can be counted against the places a sheet showed: it gives
 a reason that shows a reader something, chooses a shown place, keeps only
 shown places, and quotes only shown places, each once, every place written as the
 sheet showed it or in another form of it.

 @param ballot - ballot of the right shape

 @param labels - labels the sheet showed

 @returns Whether the ballot counts

 @example
 ```ts
 isUsableConsistencyBallot({ ballot, labels: consistencyLabels({ count: 2, },), },);
 ```
 */
export function isUsableConsistencyBallot(
  {
    ballot,
    labels,
  }: {
    readonly ballot: ConsistencyBallot;
    readonly labels: readonly string[];
  },
): boolean {
  if (rendersAsNothing({ text: ballot.reason, },))
    return false;
  return canonicalBallotOf({
    ballot,
    labels,
  },)
    .length
    > 0;
}

/**
 Guard handed to the quorum gather, so a reply that cannot be counted is
 asked again like one that is not a ballot.

 @param labels - labels the sheet showed

 @returns Guard accepting only ballots that count

 @example
 ```ts
 const validate = usableConsistencyBallotFor({ labels: consistencyLabels({ count: 2, },), });
 ```
 */
export function usableConsistencyBallotFor(
  { labels, }: { readonly labels: readonly string[]; },
): (value: unknown,) => value is ConsistencyBallotWire {
  return function isUsableBallot(value: unknown,): value is ConsistencyBallotWire {
    return isConsistencyBallotWire(value,) && isUsableConsistencyBallot({
      ballot: readConsistencyBallotWire({ wire: value, },),
      labels,
    },);
  };
}

/**
 Structured-output constraint for consistency replies, built for one ask. `assessment` is declared first, `slot` is the one
 optional property, and the three fields that name a place (`choice`, each kept place and the label of each quote) list the
 labels the sheet showed as an `enum`, so a provider that enforces the schema can answer with no other.

 @param labels - labels the sheet showed, the only labels a reply may use

 @returns The constraint for this ask

 @example
 ```ts
 const format = consistencyResponseFormat({ labels: consistencyLabels({ count: 2, },), },);
 ```
 */
export function consistencyResponseFormat(
  { labels, }: { readonly labels: readonly string[]; },
): JsonSchemaResponseFormat {
  return {
    type: 'json_schema',
    json_schema: {
      name: 'page_consistency',
      schema: {
        type: 'object',
        required: [
          'assessment',
          'candidateQuotes',
          'choice',
          'keep',
        ],
        additionalProperties: false,
        // REASON DECLARED BEFORE CHOICE, as the sheet's reply shape shows it
        // (owner, 2026-10-11, "Reason before vote"), in the order the keys sort (ledger B762).
        properties: {
          assessment: { type: 'string', },
          candidateQuotes: {
            type: 'array',
            items: {
              type: 'object',
              required: [
                'label',
                'quote',
              ],
              additionalProperties: false,
              properties: {
                label: {
                  type: 'string',
                  enum: [...labels,],
                },
                quote: { type: 'string', },
                slot: { type: 'string', },
              },
            },
          },
          choice: {
            type: 'string',
            enum: [...labels,],
          },
          keep: {
            type: 'array',
            items: {
              type: 'string',
              enum: [...labels,],
            },
          },
        },
      },
    },
  };
}

/**
 What a sheet shows of the English of one place.

 @example
 ```ts
 const english: ConsistencyEnglish = { kind: 'present', text: 'The cat naps.', };
 ```
 */
export type ConsistencyEnglish =
  | {
    /**
     The place's slice carries English.
     */
    readonly kind: 'present';

    /**
     The slice's whole English as it now stands.
     */
    readonly text: string;
  }
    | {
    /**
     The place's slice carries no English.
     */
      readonly kind: 'absent';
    };
