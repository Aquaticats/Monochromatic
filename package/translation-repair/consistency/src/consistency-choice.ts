import type { JsonSchemaResponseFormat, } from '@monochromatic-dev/translation-repair-provider/ts';
import { shownLabelOf, } from './consistency-label-form.ts';
import {
  majorityChoiceOf,
  runOffFinalistsOf,
} from './consistency-majority.ts';
import { isJsonRecord, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import { rendersAsNothing, } from '@monochromatic-dev/translation-repair-typography/ts';

//region Consistency choice
// THE REPLY OF A JUDGE WHO ONLY CHOOSES: the run-off among the finalists
// of a first round that gave no strict majority, and the choice among the candidate texts of a receiving place. The
// first round asks for quotes too; these two ask for a reason and a choice, in
// that order (REASON BEFORE CHOICE, the owner's ruling of 2026-10-11), the
// reason declared first in the reply schema and written first on the sheet.
//
// THE REPLY AND THE STORED CHOICE SPELL THE REASON DIFFERENTLY (ledger B762). The reply calls it `assessment`, a name that
// sorts before `choice`, so a provider that returns a tool call's keys alphabetically still gives the reason first. The
// choice a gather stores (and the cache and the run's artifacts read back) keeps the name `reason` it was stored under, so
// no stored record changes spelling: `readConsistencyChoiceWire` is the one place the reply becomes a stored choice.

/**
 One judge's reply that only chooses.

 @example
 ```ts
 const wire: ConsistencyChoiceWire = { assessment: 'It reads as the refrain reads.', choice: 'CANDIDATE 2', };
 ```
 */
export type ConsistencyChoiceWire = {
  /**
   Why the judge chose as it did, written before the choice.
   */
  readonly assessment: string;

  /**
   Label of the place or candidate chosen.
   */
  readonly choice: string;
};

/**
 One judge's choice as the gather stores it and the counting reads it.

 @example
 ```ts
 const ballot: ConsistencyChoiceBallot = { reason: 'It reads as the refrain reads.', choice: 'CANDIDATE 2', };
 ```
 */
export type ConsistencyChoiceBallot = {
  /**
   Why the judge chose as it did.
   */
  readonly reason: string;

  /**
   Label of the place or candidate chosen.
   */
  readonly choice: string;
};

/**
 One heard choice with the seat that gave it.

 @example
 ```ts
 const voice: ConsistencyChoiceVoice = { seat: 'glm-5.3', reason: 'Reads best.', choice: 'OCCURRENCE 3', };
 ```
 */
export type ConsistencyChoiceVoice = ConsistencyChoiceBallot & {
  /**
   Seat that answered.
   */
  readonly seat: string;
};

/**
 Structured-output constraint for replies that only choose, built for one ask: `assessment` is declared first, and `choice` lists
 the labels the sheet showed as an `enum`, so a provider that enforces the schema can answer with no other.

 @param labels - labels the sheet showed, the only labels a reply may use

 @returns The constraint for this ask

 @example
 ```ts
 const format = consistencyChoiceResponseFormat({ labels: ['CANDIDATE 1', 'CANDIDATE 2',], },);
 ```
 */
export function consistencyChoiceResponseFormat(
  { labels, }: { readonly labels: readonly string[]; },
): JsonSchemaResponseFormat {
  return {
    type: 'json_schema',
    json_schema: {
      name: 'page_consistency_choice',
      schema: {
        type: 'object',
        required: [
          'assessment',
          'choice',
        ],
        additionalProperties: false,
        // REASON DECLARED BEFORE CHOICE, as the sheets' reply shape shows it, in the order the keys sort (ledger B762).
        properties: {
          assessment: { type: 'string', },
          choice: {
            type: 'string',
            enum: [...labels,],
          },
        },
      },
    },
  };
}

/**
 Whether a parsed reply has the shape of a choice, whatever it names.

 @param value - parsed provider value

 @returns Whether it carries an assessment and a choice

 @example
 ```ts
 isConsistencyChoiceWire(JSON.parse(text,),);
 ```
 */
export function isConsistencyChoiceWire(value: unknown,): value is ConsistencyChoiceWire {
  return isJsonRecord(value,) && ((typeof value.assessment) === 'string')
    && ((typeof value.choice) === 'string');
}

/**
 Whether a parsed stored value has the shape of a stored choice, whatever it names.

 @param value - parsed stored value, such as a voice read back from the cache

 @returns Whether it carries a reason and a choice

 @example
 ```ts
 isConsistencyChoiceBallot(JSON.parse(text,),);
 ```
 */
export function isConsistencyChoiceBallot(value: unknown,): value is ConsistencyChoiceBallot {
  return isJsonRecord(value,) && ((typeof value.reason) === 'string')
    && ((typeof value.choice) === 'string');
}

/**
 The reply as the choice a gather stores.

 @param wire - reply of the right shape

 @returns The choice, its assessment stored as the reason

 @example
 ```ts
 const ballot = readConsistencyChoiceWire({ wire: { assessment: 'It reads best.', choice: '3', }, },);
 ```
 */
export function readConsistencyChoiceWire(
  { wire, }: { readonly wire: ConsistencyChoiceWire; },
): ConsistencyChoiceBallot {
  return {
    reason: wire.assessment,
    choice: wire.choice,
  };
}

/**
 The choice with its label as the sheet showed it, when it names exactly one shown label in any form (its case, white space
 around it, a bare number, the word with the number); every other field of the reply is kept.

 @param ballot - choice of the right shape, with whatever else the reply carries

 @param labels - labels the sheet showed

 @returns The choice with the shown label, as a list of zero or one; none when the choice names no shown label or more than one

 @example
 ```ts
 const [canonical,] = canonicalChoiceOf({ ballot: { reason: 'r', choice: '3', }, labels: ['OCCURRENCE 3',], },);
 ```
 */
export function canonicalChoiceOf<BallotT extends ConsistencyChoiceBallot,>(
  {
    ballot,
    labels,
  }: {
    readonly ballot: BallotT;
    readonly labels: readonly string[];
  },
): readonly BallotT[] {
  /**
   The shown label the choice stands for.
   */
  const [label,] = shownLabelOf({
    name: ballot.choice,
    shown: labels,
  },);
  return (label === undefined)
    ? []
    : [
      {
        ...ballot,
        choice: label,
      },
    ];
}

/**
 Whether a choice can be counted against the labels a sheet showed: it gives a
 reason that shows a reader something and chooses a shown label, written as the
 label or in another form of it.

 @param ballot - choice of the right shape

 @param labels - labels the sheet showed

 @returns Whether the choice counts

 @example
 ```ts
 isUsableChoice({ ballot, labels: ['OCCURRENCE 3', 'OCCURRENCE 4',], });
 ```
 */
export function isUsableChoice(
  {
    ballot,
    labels,
  }: {
    readonly ballot: ConsistencyChoiceBallot;
    readonly labels: readonly string[];
  },
): boolean {
  return (!rendersAsNothing({ text: ballot.reason, },)) && (canonicalChoiceOf({
    ballot,
    labels,
  },)
    .length
    > 0);
}

/**
 Guard handed to the quorum gather, so a reply that cannot be counted is asked
 again like one that is not a choice.

 @param labels - labels the sheet showed

 @returns Guard accepting only choices that count

 @example
 ```ts
 const validate = usableChoiceFor({ labels: ['CANDIDATE 1', 'CANDIDATE 2',], });
 ```
 */
export function usableChoiceFor(
  { labels, }: { readonly labels: readonly string[]; },
): (value: unknown,) => value is ConsistencyChoiceWire {
  return function isUsableReply(value: unknown,): value is ConsistencyChoiceWire {
    return isConsistencyChoiceWire(value,) && isUsableChoice({
      ballot: readConsistencyChoiceWire({ wire: value, },),
      labels,
    },);
  };
}

/**
 The choices that can be counted against the labels a sheet showed, each with the shown label it names in place of the form it
 was written in, so a choice stored before the forms were read is counted as one stored since.

 @param voices - every heard choice with its seat

 @param labels - labels the sheet showed

 @returns The usable choices

 @example
 ```ts
 const usable = usableChoicesOf({ voices, labels, },);
 ```
 */
export function usableChoicesOf(
  {
    voices,
    labels,
  }: {
    readonly voices: readonly ConsistencyChoiceVoice[];
    readonly labels: readonly string[];
  },
): readonly ConsistencyChoiceVoice[] {
  return voices.flatMap(function counted(voice,): readonly ConsistencyChoiceVoice[] {
    return isUsableChoice({
      ballot: voice,
      labels,
    },)
      ? canonicalChoiceOf({
        ballot: voice,
        labels,
      },)
      : [];
  },);
}

/**
 The finalists of a run-off among the choices that can be counted, when no label has more than half of the usable choices:
 the labels sharing the highest count, or, when one label leads alone, that label and the labels with the second-highest count.
 A majority has no finalists.

 @param voices - every heard choice with its seat

 @param labels - labels the sheet showed, in sheet order

 @returns The finalists in sheet order, none when a label has a majority or fewer than two labels drew a choice

 @example
 ```ts
 const finalists = runOffChoiceFinalistsOf({ voices, labels: ['CANDIDATE 1', 'CANDIDATE 2', 'CANDIDATE 3',], },);
 ```
 */
export function runOffChoiceFinalistsOf(
  {
    voices,
    labels,
  }: {
    readonly voices: readonly ConsistencyChoiceVoice[];
    readonly labels: readonly string[];
  },
): readonly string[] {
  return runOffFinalistsOf({
    voices: usableChoicesOf({
      voices,
      labels,
    },),
    labels,
  },);
}

/**
 What a set of choices come to: the usable count and the label a strict
 majority of the usable choices named.

 @param voices - every heard choice with its seat

 @param labels - labels the sheet showed

 @returns How many choices counted, and the label more than half of them chose, as a list of zero or one

 @example
 ```ts
 const { usable, chosen, } = choiceTallyOf({ voices, labels, },);
 ```
 */
export function choiceTallyOf(
  {
    voices,
    labels,
  }: {
    readonly voices: readonly ConsistencyChoiceVoice[];
    readonly labels: readonly string[];
  },
): {
  readonly usable: number;
  readonly chosen: readonly string[];
} {
  /**
   The choices that count.
   */
  const usable = usableChoicesOf({
    voices,
    labels,
  },);
  return {
    usable: usable.length,
    chosen: majorityChoiceOf({ voices: usable, },),
  };
}

//endregion Consistency choice
