import {
  isJsonArray,
  isJsonRecord,
} from '@monochromatic-dev/translation-repair-document-structure/ts';

//region Ledger parse
// TURNS A LEDGER FILE ON DISK INTO A SHAPE THE READER CAN TRUST, refusing
// anything that does not carry what a contest needs.
//
// MODEL IDS ARE READ AS PLAIN STRINGS, deliberately, and not as the catalog
// union the writer held. A ledger is read to ask questions ABOUT the roster,
// including about a seat that has since been dropped, so narrowing a recorded
// id back into today's catalog would be a claim this reader cannot support and
// does not need: it only compares ids to each other and prints them.
//
// `weight` AND `selfVote` ARE NOT READ. Nothing downstream uses them. The
// reader works out who had a stake in each candidate from the producer lists,
// because it needs that for EVERY candidate on the slate and `selfVote` speaks
// only about the one its ballot named. Reading it as a cross-check would prove
// nothing either: `candidate-select.ts` and `candidate-ledger.ts` both derive
// their answer from `producerModelIds` on the same producer, in one process, so
// the two can never disagree.

/**
 Refusal raised when a ledger file does not hold a contest.

 @example
 ```ts
 throw new LedgerShapeError({ from: 'ledger/000001.json', field: 'ballots', },);
 ```
 */
export class LedgerShapeError extends Error {
  /**
   Declares this message safe to forward: it names the file and the field,
   and quotes neither.
   */
  readonly messageNamesOnly: true = true;

  /**
   Names the file and the field rather than quoting either.

   NAMES, NEVER QUOTES. A ledger file holds corpus wording, so a refusal that
   echoed the offending value could carry a passage into a log that a run
   directory's own access rules do not cover.

   @param from - file the value came from

   @param field - field that was missing or the wrong type
   */
  constructor(
    {
      from,
      field,
    }: {
      readonly from: string;
      readonly field: string;
    },
  ) {
    super(`ledger file ${from} has no usable ${field}`,);
    this.name = 'LedgerShapeError';
  }
}

/**
 One candidate as a ledger file records it.

 @example
 ```ts
 const shown: ReadCandidate = { index: 1, producers: ['minimax-m3',], rendered: 'text', };
 ```
 */
export type ReadCandidate = {
  /**
   One-based position the judges saw it at.
   */
  readonly index: number;

  /**
   Models behind it, composites expanded, as recorded.
   */
  readonly producers: readonly string[];

  /**
   Exactly the text the judges compared.
   */
  readonly rendered: string;
};

/**
 One ballot as a ledger file records it.

 @example
 ```ts
 const cast: ReadBallot = { modelId: 'minimax-m3', best: 2, reason: 'clearest', };
 ```
 */
export type ReadBallot = {
  /**
   Judge that cast it, as recorded rather than as the catalog spells it.
   */
  readonly modelId: string;

  /**
   One-based position named, zero for an abstention, and possibly a position
   the slate never held, which is recorded rather than corrected.
   */
  readonly best: number;

  /**
   Stated reason, verbatim.
   */
  readonly reason: string;
};

/**
 One judged contest as a ledger file records it.

 @example
 ```ts
 const round = parseLedgerRound({ value, from, },);
 ```
 */
export type ReadRound = {
  /**
   What the judges were deciding.
   */
  readonly task: string;

  /**
   When it was recorded.
   */
  readonly at: string;

  /**
   Slate as the judges saw it.
   */
  readonly candidates: readonly ReadCandidate[];

  /**
   Every ballot cast, faults included.
   */
  readonly ballots: readonly ReadBallot[];

  /**
   Position that won, or that the panel chose nothing.
   */
  readonly selectedIndex: number | 'declined';
};

/**
 Reads one field off a value that may not be an object at all.

 @param value - candidate object

 @param field - field wanted

 @returns Its value, absent where the value is not an object holding it

 @example
 ```ts
 const task = fieldOf({ value, field: 'task', },);
 ```
 */
function fieldOf(
  {
    value,
    field,
  }: {
    readonly value: unknown;
    readonly field: string;
  },
): unknown {
  // NARROWED BY THE HOUSE GUARD rather than asserted: `isJsonRecord` proves the
  // shape, so nothing here claims a type the value was never checked for.
  if (!isJsonRecord(value,))
    return undefined;

  return value[field];
}

/**
 Reads a required string field.

 @param value - candidate object

 @param field - field wanted

 @param from - file being read, named in any refusal

 @returns Its value

 @throws {@link LedgerShapeError} where the field is absent or not a string

 @example
 ```ts
 const task = stringField({ value, field: 'task', from, },);
 ```
 */
function stringField(
  {
    value,
    field,
    from,
  }: {
    readonly value: unknown;
    readonly field: string;
    readonly from: string;
  },
): string {
  /**
   Whatever sits at that field.
   */
  const found = fieldOf({
    value,
    field,
  },);

  if ((typeof found) !== 'string')
    throw new LedgerShapeError({
      from,
      field,
    },);

  return found;
}

/**
 Reads a required finite number field.

 @param value - candidate object

 @param field - field wanted

 @param from - file being read, named in any refusal

 @returns Its value

 @throws {@link LedgerShapeError} where the field is absent or not a number

 @example
 ```ts
 const index = numberField({ value, field: 'index', from, },);
 ```
 */
function numberField(
  {
    value,
    field,
    from,
  }: {
    readonly value: unknown;
    readonly field: string;
    readonly from: string;
  },
): number {
  /**
   Whatever sits at that field.
   */
  const found = fieldOf({
    value,
    field,
  },);

  if (((typeof found) !== 'number') || (!Number.isFinite(found,)))
    throw new LedgerShapeError({
      from,
      field,
    },);

  return found;
}

/**
 Reads a required array field.

 @param value - candidate object

 @param field - field wanted

 @param from - file being read, named in any refusal

 @returns Its elements, still unread

 @throws {@link LedgerShapeError} where the field is absent or not an array

 @example
 ```ts
 const ballots = arrayField({ value, field: 'ballots', from, },);
 ```
 */
function arrayField(
  {
    value,
    field,
    from,
  }: {
    readonly value: unknown;
    readonly field: string;
    readonly from: string;
  },
): readonly unknown[] {
  /**
   Whatever sits at that field.
   */
  const found = fieldOf({
    value,
    field,
  },);

  if (!isJsonArray(found,))
    throw new LedgerShapeError({
      from,
      field,
    },);

  return found;
}

/**
 Reads one candidate.

 @param value - candidate object

 @param from - file being read, named in any refusal

 @returns Candidate as recorded

 @throws {@link LedgerShapeError} on any field that is absent or wrongly typed

 @example
 ```ts
 const candidate = readCandidate({ value, from, },);
 ```
 */
function readCandidate(
  {
    value,
    from,
  }: {
    readonly value: unknown;
    readonly from: string;
  },
): ReadCandidate {
  return {
    index: numberField({
      value,
      field: 'index',
      from,
    },),
    producers: arrayField({
      value,
      field: 'producers',
      from,
    },)
      .map(function asId(id,): string {
        if ((typeof id) !== 'string')
          throw new LedgerShapeError({
            from,
            field: 'producers',
          },);

        return id;
      },),
    rendered: stringField({
      value,
      field: 'rendered',
      from,
    },),
  };
}

/**
 Reads one ballot.

 @param value - ballot object

 @param from - file being read, named in any refusal

 @returns Ballot as recorded

 @throws {@link LedgerShapeError} on any field that is absent or wrongly typed

 @example
 ```ts
 const ballot = readBallot({ value, from, },);
 ```
 */
function readBallot(
  {
    value,
    from,
  }: {
    readonly value: unknown;
    readonly from: string;
  },
): ReadBallot {
  return {
    modelId: stringField({
      value,
      field: 'modelId',
      from,
    },),
    best: numberField({
      value,
      field: 'best',
      from,
    },),
    reason: stringField({
      value,
      field: 'reason',
      from,
    },),
  };
}

/**
 Reads the winning position, which is either one or a refusal to pick.

 @param value - contest object

 @param from - file being read, named in any refusal

 @returns Winning position, or that the panel declined

 @throws {@link LedgerShapeError} where it is neither

 @example
 ```ts
 const selected = readSelected({ value, from, },);
 ```
 */
function readSelected(
  {
    value,
    from,
  }: {
    readonly value: unknown;
    readonly from: string;
  },
): number | 'declined' {
  /**
   Whatever the contest recorded as its outcome.
   */
  const found = fieldOf({
    value,
    field: 'selectedIndex',
  },);

  if (found === 'declined')
    return 'declined';

  return numberField({
    value,
    field: 'selectedIndex',
    from,
  },);
}

/**
 Reads one contest, refusing a file that does not hold one.

 REFUSES RATHER THAN FILLING IN. A ledger read to settle a roster question
 that quietly treated a truncated file as a contest with no ballots would
 report a seat as unjudged when the truth is that the record was lost.

 @param value - parsed JSON of one ledger file

 @param from - file it came from, named in any refusal

 @returns Contest as recorded

 @throws {@link LedgerShapeError} on any field that is absent or wrongly typed

 @example
 ```ts
 const round = parseLedgerRound({ value: parseRunJson({ text, from, },), from, },);
 ```
 */
export function parseLedgerRound(
  {
    value,
    from,
  }: {
    readonly value: unknown;
    readonly from: string;
  },
): ReadRound {
  return {
    task: stringField({
      value,
      field: 'task',
      from,
    },),
    at: stringField({
      value,
      field: 'at',
      from,
    },),
    candidates: arrayField({
      value,
      field: 'candidates',
      from,
    },)
      .map(function one(candidate,): ReadCandidate {
        return readCandidate({
          value: candidate,
          from,
        },);
      },),
    ballots: arrayField({
      value,
      field: 'ballots',
      from,
    },)
      .map(function one(ballot,): ReadBallot {
        return readBallot({
          value: ballot,
          from,
        },);
      },),
    selectedIndex: readSelected({
      value,
      from,
    },),
  };
}

//endregion Ledger parse
