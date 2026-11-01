import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

import {
  isProviderName,
  type ProviderName,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';
import {
  isSpendReckoning,
  SPEND_MARKER,
  type SpendReckoning,
} from '@monochromatic-dev/translation-repair-provider/ts';
import { isWholeNumberText, } from '@monochromatic-dev/translation-repair-markup/ts';

//region Spend read
// Reads `SPEND` lines back out of run logs and totals them per model.
//
// THE OTHER HALF OF `spend-line.ts`, and it imports that module's marker rather
// than restating it, so the writer and the reader cannot drift.
//
// NO REGEX. The line is `[level] [iso] [tag] [tag] SPEND name=value ...`, which
// is one `indexOf` and two splits. Same shape and same discipline as
// `meter-sample-read.ts`, which reads the `METERS` line.
//
// ABSENCE IS A NAMED VALUE, never a nullish union: `'not-a-record'` for a line
// that is not one, `'unreadable'` for a line that is one and will not parse,
// and `'unreported'` for a count the provider never sent.
//
// PROSE MENTIONING THE MARKER IS NOT A RECORD. A run log carries this module's
// own commit messages, task notes and test output, any of which may write the
// word. A record is recognised by its first field parsing, which prose does not
// do, while a record truncated part way through a later field still does and is
// still counted. That trap was found the first time by `meter-sample-read.ts`,
// whose own summary line mentioned its marker and was counted as a hole.
//
// A LINE THAT IS A RECORD AND WILL NOT PARSE IS COUNTED, NOT DROPPED, for the
// same reason: interleaved writes from concurrent stages produce them, and a
// reader that skipped them would report a cleaner record than the one it read.

/**
 What `indexOf` returns for a marker that is not in the line.
 */
const NOT_FOUND = -1;

/**
 Value a count carries when the provider sent no usage block.
 */
const UNREPORTED = 'unreported';

/**
 Field opening every record, checked to tell a record from prose.
 */
const FIRST_FIELD = 'provider=';

/**
 One call's token counts, or the named absence standing for a provider that
 reported none.
 */
export type SpendCount = number | typeof UNREPORTED;

/**
 One call's cost in USD, or the named absence for a line that carried no
 `cost=` field: every line before 2026-09-03 and every line from a provider
 that does not bill in USD.
 */
export type SpendUsd = number | typeof UNREPORTED;

/**
 What one `SPEND` line said.

 @example
 ```ts
 const record: SpendRecord = {
   provider: 'hyper',
   model: 'qwen3.8-max',
   prompt: 5120,
   completion: 3072,
   costUsd: 'unreported',
   reckoning: 'reported',
 };
 ```
 */
export type SpendRecord = {
  /**
   Meter this call drew on.
   */
  readonly provider: ProviderName;

  /**
   Model as the serving provider names it.
   */
  readonly model: string;

  /**
   Tokens the request consumed.
   */
  readonly prompt: SpendCount;

  /**
   Tokens the answer produced, thinking included.
   */
  readonly completion: SpendCount;

  /**
   USD the wire reported for this call, where the provider bills in USD and
   the line carried the field.
   */
  readonly costUsd: SpendUsd;

  /**
   Why the counts and cost are reckoned rather than reported, or that the wire
   reported them. The writer marks an abandoned attempt and a bound with
   `estimated=` so a total can keep them apart, and this reader dropped the
   field until ledger P14, counting every such line as a reported call.
   */
  readonly reckoning: SpendReckoning | 'reported';
};

/**
 What one line turned out to be.
 */
export type SpendLineReading = SpendRecord | 'not-a-record' | 'unreadable';

/**
 One `name=value` field, split.
 */
type SpendField = {
  /**
   Text before the separator.
   */
  readonly name: string;

  /**
   Text after it, empty where the field carried none.
   */
  readonly value: string;
};

/**
 Splits a `name=value` field, reporting a piece that carries no separator.

 @param field - one space-delimited piece of the record tail

 @returns Name and value, or that this piece is not a field

 @example
 ```ts
 const pair = fieldOf({ field: 'provider=hyper', },);
 ```
 */
function fieldOf(
  { field, }: { readonly field: string; },
): SpendField | 'not-a-field' {
  /**
   Where the separator sits, or that this piece carries none.
   */
  const at = field.indexOf('=',);

  if (at === NOT_FOUND)
    return 'not-a-field';

  return {
    name: field.slice(
      0,
      at,
    ),
    value: field.slice(at + 1,),
  };
}

/**
 Reads a count field, keeping a provider's silence as a named value rather
 than folding it into zero.

 @param value - what the field carried

 @returns Count, the named absence, or that the field will not read

 @example
 ```ts
 const prompt = countOf({ value: '5120', },);
 ```
 */
function countOf(
  { value, }: { readonly value: string; },
): SpendCount | 'unreadable' {
  if (value === UNREPORTED)
    return UNREPORTED;

  // DIGITS AS THE WRITER SPELLS A TOKEN COUNT, whole and not negative.
  // `Number` read an empty field as zero, and a hexadecimal, an exponent or a
  // plus sign as a count `reportSpend` never writes (ledger B73).
  if (!isWholeNumberText({ text: value, },))
    return 'unreadable';

  return Number(value,);
}

/**
 Reads the cost field, keeping its absence as a named value.

 FRACTIONAL AND NOT NEGATIVE, unlike a token count: a call costs a fraction
 of a cent, and the wire writes it with nine decimals.

 SPELLED AS `String` SPELLS A NUMBER, which is how `reportSpend` writes it,
 exponent included for the smallest costs: `Number` also read a hexadecimal, a
 trailing zero and surrounding spaces as a cost the writer never writes, so
 only a value that reads back to its own text is a cost (ledger B73).

 @param value - what the field carried, empty where the line had none

 @returns USD, the named absence, or that the field will not read

 @example
 ```ts
 const cost = usdOf({ value: '0.000126255304', },);
 ```
 */
function usdOf(
  { value, }: { readonly value: string; },
): SpendUsd | 'unreadable' {
  if ((value === '') || (value === UNREPORTED))
    return UNREPORTED;

  /**
   Field read as a number, which is NaN for anything that is not one.
   */
  const parsed = Number(value,);

  if ((!Number.isFinite(parsed,)) || (String(parsed,) !== value))
    return 'unreadable';

  if (parsed < 0)
    return 'unreadable';

  return parsed;
}

/**
 Finds the marker where a logger prefix precedes it.

 @param line - one line of a run log

 @returns Where the marker word starts, or that the line carries none

 @example
 ```ts
 const at = spacedMarkerIn({ line: '[info] [t] SPEND provider=hyper', },);
 ```
 */
function spacedMarkerIn(
  { line, }: { readonly line: string; },
): number {
  /**
   Where the space in front of the marker sits.
   */
  const at = line.indexOf(` ${SPEND_MARKER}`,);

  return (at === NOT_FOUND) ? NOT_FOUND : (at + 1);
}

/**
 Collects the fields of a record tail, keyed by name.

 A `Map` RATHER THAN AN OBJECT, because the keys come off a log line and an
 object would let a line writing `__proto__=` reach the prototype. Nothing in
 a run log is supposed to do that, which is exactly why the reader must not
 depend on it not happening.

 A NAME WRITTEN TWICE IS REPORTED, NOT SETTLED BY ORDER. `reportSpend` writes
 each field once, so a tail naming one twice is a damaged record: two
 interleaved records sharing a line, or a line cut and resumed. Setting the
 name again kept the later value, which read one call out of such a line and
 lost the other while reporting a clean record.

 @param fields - space-delimited pieces of the tail

 @returns Every piece that split, keyed by name, or that the tail writes some
 name more than once

 @example
 ```ts
 const named = namedFields({ fields: ['provider=hyper',], },);
 ```
 */
function namedFields(
  { fields, }: { readonly fields: readonly string[]; },
): ReadonlyMap<string, string> | 'field-repeats' {
  /**
   Fields collected so far.
   */
  const named = new Map<string, string>();

  for (const field of fields) {
    /**
     Name and value of this piece.
     */
    const pair = fieldOf({ field, },);
    if (pair === 'not-a-field')
      continue;
    if (named.has(pair.name,))
      return 'field-repeats';
    named.set(
      pair.name,
      pair.value,
    );
  }

  return named;
}

/**
 Reads one log line as a spend record.

 @param line - one line of a run log, tag prefix and all

 @returns Record, or which kind of non-record this line is

 @example
 ```ts
 const read = readSpendLine({ line, },);
 ```
 */
export function readSpendLine(
  { line, }: { readonly line: string; },
): SpendLineReading {
  /**
   Where the marker sits, or that this is an ordinary line.

   ACCEPTED AT THE START OF A LINE OR AFTER A SPACE, so both forms read: the
   bare line `reportSpend` returns, and the same line once a logger has put
   its level, stamp and tags in front of it. Demanding the space would refuse
   the writer's own output.
   */
  const markerAt = line.startsWith(SPEND_MARKER,)
    ? 0
    : spacedMarkerIn({ line, },);

  if (markerAt === NOT_FOUND)
    return 'not-a-record';

  /**
   Fields written after the marker.
   */
  const fields = line
    .slice(markerAt + SPEND_MARKER.length,)
    .split(' ',);

  // PROSE IS NOT A TRUNCATED RECORD, decided on the first field alone.
  /**
   First field of the split, present since the split always yields one.
   */
  const firstField = nonNullishOrThrow(fields[0],);
  if (!firstField.startsWith(FIRST_FIELD,))
    return 'not-a-record';

  /**
   Every field that split, keyed by name.
   */
  const named = namedFields({ fields, },);

  // A RECORD, SINCE ITS FIRST FIELD READ, AND NOT ONE CALL'S: counted as a
  // hole rather than read for whichever value stood last.
  if (named === 'field-repeats')
    return 'unreadable';

  /**
   Provider the record names, before it is known to be one a record may carry.
   */
  const namedProvider = named.get('provider',);

  /**
   Provider named, checked against `PROVIDER_ORDER` through
   `isProviderName`, so a provider the pipeline gains is a provider a record
   may carry.
   */
  const provider = ((namedProvider !== undefined) && isProviderName(namedProvider,))
    ? namedProvider
    : undefined;

  /**
   Model named, absent on a record truncated before it.
   */
  const model = named.get('model',);

  if (provider === undefined)
    return 'unreadable';

  if ((model === undefined) || (model === ''))
    return 'unreadable';

  /**
   Tokens the request consumed, or that the field will not read.
   */
  const prompt = countOf({ value: named.get('prompt',) ?? '', },);

  /**
   Tokens the answer produced, or that the field will not read.
   */
  const completion = countOf({ value: named.get('completion',) ?? '', },);

  if (prompt === 'unreadable')
    return 'unreadable';

  if (completion === 'unreadable')
    return 'unreadable';

  /**
   USD the wire reported, or the named absence for a line without the field.
   */
  const costUsd = usdOf({ value: named.get('cost',) ?? '', },);

  if (costUsd === 'unreadable')
    return 'unreadable';

  /**
   Reckoning mark, absent on a line the wire reported.
   */
  const estimated = named.get('estimated',);

  // A MARK THIS PACKAGE NEVER WRITES IS A DAMAGED RECORD, not a reported
  // call: counting it as reported is the misreading ledger P14 fixed.
  if ((estimated !== undefined) && (!isSpendReckoning(estimated,)))
    return 'unreadable';

  return {
    provider,
    model,
    prompt,
    completion,
    costUsd,
    reckoning: estimated ?? 'reported',
  };
}

/**
 What one seat spent across every call a log recorded for it.

 @example
 ```ts
 const spend: SeatSpend = {
   provider: 'hyper',
   model: 'qwen3.8-max',
   calls: 13,
   promptTokens: 84_000,
   completionTokens: 51_065,
   unreportedCalls: 0,
 };
 ```
 */
export type SeatSpend = {
  /**
   Meter these calls drew on.
   */
  readonly provider: SpendRecord['provider'];

  /**
   Model as the serving provider names it.
   */
  readonly model: string;

  /**
   Calls the log recorded for this seat, reported or not.
   */
  readonly calls: number;

  /**
   Prompt tokens summed over the calls that reported any.
   */
  readonly promptTokens: number;

  /**
   Completion tokens summed over the calls that reported any, thinking
   included.
   */
  readonly completionTokens: number;

  /**
   Calls whose provider sent no usage block.

   CARRIED BESIDE THE TOTALS RATHER THAN FOLDED INTO THEM, because a total
   over reported calls only is a floor, and a reader has to be able to see how
   much of the run it is a floor over.
   */
  readonly unreportedCalls: number;

  /**
   USD summed over the calls whose line carried a cost.
   */
  readonly costUsd: number;

  /**
   Calls whose line carried a cost, so a USD total can be read as a floor
   over the rest.
   */
  readonly costedCalls: number;

  /**
   Calls whose counts and cost were reckoned rather than reported (ledger
   P14), carried beside the totals like the unreported ones, so a reader sees
   how much of the seat's figures rests on an estimate or a bound.
   */
  readonly reckonedCalls: number;
};

/**
 Everything a set of log lines said about what was spent.

 @example
 ```ts
 const tally = tallySpend({ lines, },);
 ```
 */
export type SpendTally = {
  /**
   One entry per provider and model pair, since the same model can be served
   by both and only one of the two is priced per token.
   */
  readonly seats: readonly SeatSpend[];

  /**
   Lines that carried the marker and a readable first field but would not
   parse, counted so a hole in the record cannot pass for an absence of spend.
   */
  readonly unreadableLines: number;
};

/**
 Totals every spend record in a log, per seat.

 @param lines - log lines in any order, records and prose mixed

 @returns Per-seat totals, sorted by completion tokens so the seat that cost
 the most reads first, plus how many records would not parse

 @example
 ```ts
 const tally = tallySpend({ lines: text.split('\n',), },);
 ```
 */
export function tallySpend(
  { lines, }: { readonly lines: readonly string[]; },
): SpendTally {
  /**
   What every line turned out to be, read once so the tally's counts cannot
   disagree about which lines were records.
   */
  const readings = lines.map(function readOne(line,): SpendLineReading {
    return readSpendLine({ line, },);
  },);

  /**
   Records that carried the marker and would not parse.
   */
  const unreadableLines = readings
    .filter(function damaged(reading,): boolean {
      return reading === 'unreadable';
    },)
    .length;

  /**
   Every line that read as a record.
   */
  const records = readings.filter(function isRecord(reading,): reading is SpendRecord {
    if (reading === 'not-a-record')
      return false;

    return reading !== 'unreadable';
  },);

  /**
   Running totals keyed by provider and model together.
   */
  const seats = new Map<string, SeatSpend>();

  for (const record of records) {
    /**
     Key pairing the provider with the model, since one model may be served
     by both and the two are billed differently.
     */
    const key = `${record.provider} ${record.model}`;

    /**
     Totals this seat had before this call.
     */
    const running = seats.get(key,) ?? {
      provider: record.provider,
      model: record.model,
      calls: 0,
      promptTokens: 0,
      completionTokens: 0,
      unreportedCalls: 0,
      costUsd: 0,
      costedCalls: 0,
      reckonedCalls: 0,
    };

    /**
     Whether this call reported anything at all.
     */
    const reported = (record.prompt !== UNREPORTED) || (record.completion !== UNREPORTED);

    /**
     Whether this call's line carried a cost.
     */
    const costed = record.costUsd !== UNREPORTED;

    seats.set(
      key,
      {
        provider: running.provider,
        model: running.model,
        calls: running.calls + 1,
        promptTokens: running.promptTokens
          + ((record.prompt === UNREPORTED) ? 0 : record.prompt),
        completionTokens: running.completionTokens
          + ((record.completion === UNREPORTED) ? 0 : record.completion),
        unreportedCalls: running.unreportedCalls + (reported ? 0 : 1),
        costUsd: running.costUsd + ((record.costUsd === UNREPORTED) ? 0 : record.costUsd),
        costedCalls: running.costedCalls + (costed ? 1 : 0),
        reckonedCalls: running.reckonedCalls + ((record.reckoning === 'reported') ? 0 : 1),
      },
    );
  }

  return {
    seats: [...seats.values(),].toSorted(function costliestFirst(
      left,
      right,
    ): number {
      return right.completionTokens - left.completionTokens;
    },),
    unreadableLines,
  };
}

//endregion Spend read
