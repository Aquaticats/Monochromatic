import { tagged, } from '@monochromatic-dev/logger-core/ts';

import { isJsonRecord, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import { isIsoStampText, } from '@monochromatic-dev/translation-repair-markup/ts';
import { contextRoot, } from '@monochromatic-dev/translation-repair-typography/ts';

//region Quota snapshot
// Typed view over `GET /v2/quotas`. Only the fields budget decisions consume are modeled: the weekly credit limit (spend pacing)
// and, where a reply still carries one, the rolling five-hour request limit (throttling). Quota reads are free per provider docs,
// so drivers may poll without budget cost. A malformed body is a provider protocol failure and throws, unlike model-content
// defects which flow as data.
//
// THE FIVE-HOUR WINDOW IS OPTIONAL SINCE 2026-10-19 (ledger B671; GitHub issue 1779 tracks the unreadable meter). Synthetic's
// rate-limit page (`https://synthetic.new/rate-limits`, fetched 2026-10-19) describes only a weekly credit limit: "Every 202
// minutes (about 3.4 hours), you get back 2% of your total weekly credit limit." Its rolling five-hour request limit and its
// concurrency limits are gone from that page. A live reading of `GET /v2/quotas` on 2026-10-19 at 14:35 UTC returned an object
// whose top-level keys were exactly `subscription`, `search`, `freeToolCalls` and `weeklyTokenLimit`, each an object, with no
// `rollingFiveHourLimit`. The inner fields of `weeklyTokenLimit` were not observed in that reading, because the reader of the
// day refused the body before reaching them; `percentRemaining` and `nextRegenAt` are the names of the 2026-07-16 reading, the
// last in which the block was seen, and a refusal inside the block now says which keys it holds so the next live reading
// settles them.
//
// A BODY WITHOUT THE `rollingFiveHourLimit` KEY YIELDS A SNAPSHOT WITH NO FIVE-HOUR WINDOW. A body that carries the key is read as
// before: the block must be an object (`null` is refused) and each field must have its type, and the older two-block shape of
// 2026-07-16 stays readable, so an account whose reply still carries the window is honoured. The refusal of the pre-launch reading
// of 2026-10-18 (ledger B671) is consistent with a body that already lacked the key, though that body was not recorded. The
// maintainer of Synthetic wrote on 2026-09-15, in `slkiser/opencode-quota` issue 276, that a bearer API key returns the full quota
// JSON with `weeklyTokenLimit` and `rollingFiveHourLimit`; the reading of 2026-10-19 supersedes that for the five-hour block. The
// same issue records a key on an account the reporter took to be on the free tier answered with status 200 and the body `{}`,
// which this reader refuses for lack of a weekly block. A refusal says what arrived: the top-level key names with the JSON type
// of each, or the type of a body that is no object, and, when the fault is a field inside a block, that block's own key names with
// the JSON type of each; never a value (`QuotaShapeError`).
//
// THE WEEKLY BLOCK IS NAMED `planWeeklyLimit` SINCE A TIME BETWEEN 2026-10-22 22:20:53 UTC AND 22:21:58 UTC, AND THIS READER TAKES
// EITHER NAME. A corpus pass parsed a body with `weeklyTokenLimit` (420 characters) in a read that ended at 22:20:53 UTC on
// 2026-10-22 (the weekly credit read 34.3 % then), and was refused `weeklyTokenLimit is not an object; received
// subscription:object, search:object, planWeeklyLimit:object` by the read that ended at 22:21:58 UTC (285 characters) and by every read
// the log shows after it. A keyed `GET https://api.synthetic.new/v2/quotas` at 2026-10-23 03:28:16 UTC answered HTTP 200 with
// 269 characters whose top-level keys were `subscription`, `search` and `planWeeklyLimit`, each an object, and no
// `weeklyTokenLimit` and no `freeToolCalls`; the values of `planWeeklyLimit` were `nextRegenAt` (a string), `percentRemaining` (a
// number, 0 then) and `nextRegenPercent` (a number, 2 then). The credit read 34.3 % at the last parsed read, so a deploy is likelier
// than the credit reaching 0 % as the cause of the rename, but the log shows no credit after that read, so both names are read.
//
// THE RULES FOR THE TWO NAMES. A key whose value is `null` or that is missing counts as absent, so a deprecated key kept as `null`
// beside a valid block does not blind the meter; a key that holds anything else but an object is refused. Two blocks that read the
// same are accepted, and two that differ in `percentRemaining` are refused, since which one a reader should believe is then
// unknowable. Two that agree on `percentRemaining` and differ in `nextRegenAt` are read with no return and one warning naming both
// blocks and the field, since the return decides nothing. A body with no usable key under either name is refused, naming both.
//
// ONLY THE FIELDS A DECISION READS ARE MODELED. `nextRegenPercent` (what the next regeneration returns) is ignored as `subscription`
// is, so a mistyped value of it cannot refuse a body whose `percentRemaining` is sound. `nextRegenAt` decides nothing: only
// `percentRemaining` makes a meter dry, and the return is a field of a log line. So it is carried only as an instant the way the
// logger writes one (`isIsoStampText`), because the meter prints it. A value that is not one (missing, not a string, another
// spelling of a valid time) does NOT refuse the body, since a refused body reads `unreadable`, which routes as spendable, the
// failure this reading was fixed for. The window then keeps `percentRemaining` and has no `nextRegenAt` property at all (the raw
// string is never kept), the meter prints `syntheticWeeklyReturn=unreadable`, and one warning names the block and the field, never
// the value. See `doc/troubleshooting/synthetic-quota-weekly-block-renamed.md`.

/**
 Logger root for the quota reader.
 */
const l = contextRoot({ tag: 'translation-repair', },);

/**
 Budget-relevant quota state at one instant.
 Verified live response also carries `subscription`, `search`, and
 `freeToolCalls` blocks; they are deliberately unmodeled until something
 consumes them.

 THE FIVE-HOUR WINDOW MAY BE ABSENT, as in the reply of 2026-10-19 (see the
 region comment); the weekly block is always present, read from `weeklyTokenLimit` or `planWeeklyLimit`.

 @example
 ```ts
 const snapshot: QuotaSnapshot = {
   fiveHour: { remaining: 750, max: 750, limited: false, nextTickAt: '2026-07-16T22:55:29.000Z', },
   weekly: { percentRemaining: 99.8, nextRegenAt: '2026-07-17T00:12:58.000Z', },
 };
 const weeklyOnly: QuotaSnapshot = {
   weekly: { percentRemaining: 99.8, nextRegenAt: '2026-10-20T00:12:58.000Z', },
 };
 const renamed: QuotaSnapshot = {
   weekly: { percentRemaining: 0, nextRegenAt: '2026-10-23T05:03:47.000Z', },
 };
 ```
 */
export type QuotaSnapshot = {
  /**
   Rolling five-hour request limit, present only when the reply carries the
   block; regenerates 5% every 15 minutes where it exists.
   */
  readonly fiveHour?: {
    /**
     Price-weighted requests still available.
     */
    readonly remaining: number;

    /**
     Ceiling of the rolling window; scales with packs.
     */
    readonly max: number;

    /**
     Whether the provider currently throttles this account.
     */
    readonly limited: boolean;

    /**
     ISO timestamp of next regeneration tick.
     */
    readonly nextTickAt: string;
  };

  /**
   Weekly credit limit; regenerates 2% roughly every 3.4 hours.
   */
  readonly weekly: {
    /**
     Percent of weekly credits still available.
     */
    readonly percentRemaining: number;

    /**
     ISO timestamp of next credit regeneration, carried only as an instant spelled the way `Date.prototype.toISOString` writes
     one; ABSENT when the reply's value was missing, not a string or spelled another way (see the region comment).
     */
    readonly nextRegenAt?: string;
  };
};

/**
 Signals a `/quotas` body that refused to parse or lacked consumed fields;
 always a provider protocol failure, never a model defect.

 @example
 ```ts
 throw new QuotaShapeError({ detail: 'rollingFiveHourLimit.remaining is not a number', },);
 ```
 */
export class QuotaShapeError extends Error {
  /**
   Declares this message safe to forward: it names the field that failed its shape and the keys and types the body held, and the block's when the fault is inside one, never a value of the body.
   */
  readonly messageNamesOnly: true = true;

  /**
   Builds failure naming the field or parse step at fault.

   @param detail - which expectation the body violated

   @param cause - underlying parse error when JSON itself failed

   @param received - keys the body held and JSON type of each, or type of a body that is no object, followed for a fault
   inside a block by that block's own keys and the JSON type of each, so a reader of the log sees what arrived without a value;
   absent when the body was no JSON

   @example
   ```ts
   new QuotaShapeError({ detail: 'body is not valid JSON', cause: error, },);
   ```
   */
  public constructor(
    {
      detail,
      cause,
      received,
    }: {
      readonly detail: string;
      readonly cause?: unknown;
      readonly received?: string;
    },
  ) {
    super(
      `Synthetic /quotas body violated expectations: ${detail}${
        (received === undefined)
          ? ''
          : `; received ${received}`
      }`,
      // Conditional spread keeps cause absent when none was supplied.
      ...(cause === undefined
        ? []
        : [{ cause, },]),
    );
    this.name = 'QuotaShapeError';
  }
}

/**
 Parses body text as JSON, converting parse failures into shape errors.

 @param bodyText - raw response body

 @returns Parsed JSON value

 @throws {@link QuotaShapeError} when body is not valid JSON

 @example
 ```ts
 const parsed = parseQuotaJson({ bodyText, },);
 ```
 */
function parseQuotaJson({ bodyText, }: { readonly bodyText: string; },): unknown {
  try {
    return JSON.parse(bodyText,);
  }
  catch (error) {
    throw new QuotaShapeError({
      detail: 'body is not valid JSON',
      cause: error,
    },);
  }
}

/**
 How many top-level keys a refusal lists before it counts the rest.
 */
const RECEIVED_KEYS_LISTED = 20;

/**
 How many characters of a key name a refusal keeps.
 */
const RECEIVED_KEY_NAME_LENGTH = 40;

/**
 The first code point of printable ASCII, the space.
 */
const FIRST_PRINTABLE = 0x20;

/**
 The last code point of printable ASCII, the tilde.
 */
const LAST_PRINTABLE = 0x7E;

/**
 The JSON type of a parsed value.

 @param value - parsed JSON value

 @returns `null`, `array`, `object`, `string`, `number` or `boolean`

 @example
 ```ts
 const type = jsonTypeOf({ value: [], },); // 'array'
 ```
 */
function jsonTypeOf({ value, }: { readonly value: unknown; },): string {
  if (value === null)
    return 'null';
  return Array.isArray(value,) ? 'array' : (typeof value);
}

/**
 A key name as a refusal prints it: printable ASCII only, each other character as a question mark, and cut to a bounded length, since a
 key is provider text that reaches a log.

 @param key - top-level key of the refused body

 @returns Key name to print

 @example
 ```ts
 const printed = boundedKeyOf({ key: 'subscri\u0000ption', },); // 'subscri?ption'
 ```
 */
function boundedKeyOf({ key, }: { readonly key: string; },): string {
  return Array.from(
    key,
    function printable(character,): string {
      /**
       The code point of the character.
       */
      const code = character.codePointAt(0,) ?? 0;
      return ((code >= FIRST_PRINTABLE) && (code <= LAST_PRINTABLE)) ? character : '?';
    },
  )
    .slice(
    0,
    RECEIVED_KEY_NAME_LENGTH,
    )
    .join('',);
}

/**
 Describes the keys of one JSON object without a value of it: each key name with the JSON type of its value. A key name is cut to a
 bounded length and keeps printable ASCII only, and the keys listed are bounded.

 @param record - parsed JSON object, body or one block of it

 @returns `{}` for an empty object, `subscription:object, freeToolCalls:object` for keys, or the first twenty keys and a count of the rest

 @example
 ```ts
 const keys = describedKeysOf({ record: { percentRemaining: 87, nextRegenAt: 'soon', }, },); // 'percentRemaining:number, nextRegenAt:string'
 ```
 */
function describedKeysOf({ record, }: { readonly record: Readonly<Record<string, unknown>>; },): string {
  /**
   The keys and values of the object.
   */
  const entries = Object.entries(record,);
  if (entries.length === 0)
    return '{}';
  /**
   The keys listed, each with the type of its value.
   */
  const listed = entries
    .slice(
    0,
    RECEIVED_KEYS_LISTED,
    )
    .map(function described([key, value,],): string {
      return `${boundedKeyOf({ key, },)}:${jsonTypeOf({ value, },)}`;
    },);
  return (entries.length > RECEIVED_KEYS_LISTED)
    ? `${listed.join(', ',)}, and ${String(entries.length - RECEIVED_KEYS_LISTED,)} more`
    : listed.join(', ',);
}

/**
 Describes what a refused body held without a value of it: the top-level key names with the JSON type of each, or the type of a
 body that is no object. A key name is cut to a bounded length and keeps printable ASCII only, and the keys listed are bounded.

 @param parsed - parsed body

 @returns `{}` for an empty object, `subscription:object, freeToolCalls:object` for keys, or the type name for any other value

 @example
 ```ts
 const received = receivedShapeOf({ parsed: { subscription: { limit: 1, }, }, },); // 'subscription:object'
 ```
 */
function receivedShapeOf({ parsed, }: { readonly parsed: unknown; },): string {
  if (!isJsonRecord(parsed,))
    return jsonTypeOf({ value: parsed, },);
  return describedKeysOf({ record: parsed, },);
}

/**
 The keys a body has named its weekly block under, in the order they were first seen: `weeklyTokenLimit` until the live reading of
 2026-10-22 19:32 UTC and `planWeeklyLimit` from that of 2026-10-23 03:28 UTC.
 */
const WEEKLY_BLOCK_NAMES = [
  'weeklyTokenLimit',
  'planWeeklyLimit',
] as const;

/**
 A key a weekly block has been seen under.
 */
type WeeklyBlockName = typeof WEEKLY_BLOCK_NAMES[number];

/**
 Builds the refusal of a parsed body that violates the expected shape.

 @param detail - which expectation the body violated

 @param parsed - parsed body, described by its keys and their types and never by a value

 @returns Error to throw

 @example
 ```ts
 throw shapeRefusal({
   detail: 'neither weeklyTokenLimit nor planWeeklyLimit is an object',
   parsed,
 },);
 ```
 */
function shapeRefusal(
  {
    detail,
    parsed,
  }: {
    readonly detail: string;
    readonly parsed: unknown;
  },
): QuotaShapeError {
  return new QuotaShapeError({
    detail,
    received: receivedShapeOf({ parsed, },),
  },);
}

/**
 Builds the refusal of a field inside one block of a parsed body.

 DESCRIBES THE BLOCK AS WELL AS THE BODY. A field refused inside a weekly block says which keys that block holds and the JSON
 type of each, so the next live reading shows the block's real fields if they are not the ones this reader expects, with the same
 bounds as the body's description and never a value.

 @param detail - which expectation the block violated

 @param parsed - parsed body, described by its keys and their types

 @param blockName - key of the block at fault

 @param block - block at fault, described by its keys and their types

 @returns Error to throw

 @example
 ```ts
 throw blockRefusal({
   detail: 'planWeeklyLimit.percentRemaining is not a number',
   parsed,
   blockName: 'planWeeklyLimit',
   block: weekly,
 },);
 ```
 */
function blockRefusal(
  {
    detail,
    parsed,
    blockName,
    block,
  }: {
    readonly detail: string;
    readonly parsed: unknown;
    readonly blockName: 'rollingFiveHourLimit' | WeeklyBlockName;
    readonly block: Readonly<Record<string, unknown>>;
  },
): QuotaShapeError {
  return new QuotaShapeError({
    detail,
    received: `${receivedShapeOf({ parsed, },)}; ${blockName} holds ${describedKeysOf({ record: block, },)}`,
  },);
}

/**
 Validates the fields of a five-hour block and reads them out.

 @param parsed - parsed body, described in a refusal

 @param block - five-hour block of that body

 @returns Typed five-hour window

 @throws {@link QuotaShapeError} when a field is missing or mistyped

 @example
 ```ts
 const fiveHour = fiveHourWindowOf({ parsed, block, },);
 ```
 */
function fiveHourWindowOf(
  {
    parsed,
    block,
  }: {
    readonly parsed: Readonly<Record<string, unknown>>;
    readonly block: Readonly<Record<string, unknown>>;
  },
): NonNullable<QuotaSnapshot['fiveHour']> {
  /**
   Five-hour fields pulled out for type validation.
   */
  const {
    remaining,
    max,
    limited,
    nextTickAt,
  } = block;
  if ((typeof remaining) !== 'number')
    throw blockRefusal({
      detail: 'rollingFiveHourLimit.remaining is not a number',
      parsed,
      blockName: 'rollingFiveHourLimit',
      block,
    },);
  if ((typeof max) !== 'number')
    throw blockRefusal({
      detail: 'rollingFiveHourLimit.max is not a number',
      parsed,
      blockName: 'rollingFiveHourLimit',
      block,
    },);
  if ((typeof limited) !== 'boolean')
    throw blockRefusal({
      detail: 'rollingFiveHourLimit.limited is not a boolean',
      parsed,
      blockName: 'rollingFiveHourLimit',
      block,
    },);
  if ((typeof nextTickAt) !== 'string')
    throw blockRefusal({
      detail: 'rollingFiveHourLimit.nextTickAt is not a string',
      parsed,
      blockName: 'rollingFiveHourLimit',
      block,
    },);

  return {
    remaining,
    max,
    limited,
    nextTickAt,
  };
}

/**
 Validates the fields of a weekly block and reads them out.

 @param parsed - parsed body, described in a refusal

 @param blockName - key the block was found under, named in a refusal

 @param block - weekly block of that body

 @returns Typed weekly window, with no `nextRegenAt` where the block's value was no instant as the logger writes one

 @throws {@link QuotaShapeError} when `percentRemaining` is missing or not a number

 @example
 ```ts
 const weekly = weeklyWindowOf({ parsed, blockName: 'planWeeklyLimit', block, },);
 ```
 */
function weeklyWindowOf(
  {
    parsed,
    blockName,
    block,
  }: {
    readonly parsed: Readonly<Record<string, unknown>>;
    readonly blockName: WeeklyBlockName;
    readonly block: Readonly<Record<string, unknown>>;
  },
): QuotaSnapshot['weekly'] {
  /**
   Weekly fields pulled out for type validation; `nextRegenPercent` is not among them, since nothing reads it.
   */
  const {
    percentRemaining,
    nextRegenAt,
  } = block;
  if ((typeof percentRemaining) !== 'number')
    throw blockRefusal({
      detail: `${blockName}.percentRemaining is not a number`,
      parsed,
      blockName,
      block,
    },);
  // THE FIELD IS PRINTED INTO A LOG LINE LATER, so a string that is no instant is never carried (a space or a newline in it would
  // forge a field or a line); and it decides nothing, so it never refuses the body either.
  if (((typeof nextRegenAt) === 'string') && isIsoStampText({ text: nextRegenAt, },))
    return {
      percentRemaining,
      nextRegenAt,
    };

  return { percentRemaining, };
}

/**
 Warns that one weekly block's `nextRegenAt` is missing or no instant as the logger writes one, naming the block and the field and
 never the value.

 The tag stays `weeklyWindowOf`, the reader whose window lacks the return, so a reader of the log finds it under that name.

 @param blockName - key of the block whose return reads unreadable

 @example
 ```ts
 warnUnreadableReturn({ blockName: 'planWeeklyLimit', },);
 ```
 */
function warnUnreadableReturn(
  { blockName, }: { readonly blockName: WeeklyBlockName; },
): void {
  tagged({
    tag: weeklyWindowOf.name,
    l,
  },)
    .warn(
    `Synthetic /quotas ${blockName}.nextRegenAt is missing or no ISO-8601 instant as the logger writes one; `
      + 'the return reads unreadable',
    );
}

/**
 One weekly block found in a body: the key it was under, the block as delivered, and the window read from it.
 */
type WeeklyReading = {
  /**
   The key the block was found under.
   */
  readonly blockName: WeeklyBlockName;

  /**
   The block as delivered, kept to describe its keys in a refusal.
   */
  readonly block: Readonly<Record<string, unknown>>;

  /**
   The window read from the block.
   */
  readonly weekly: QuotaSnapshot['weekly'];
};

/**
 Warns once for each block of a body whose window carries no return, naming the block and the field.

 Used where the blocks do not disagree on the return, so each such block is its own cause.

 @param readings - blocks of the body, each with the window read from it

 @example
 ```ts
 warnUnreadableReturnsOf({ readings: [first,], },);
 ```
 */
function warnUnreadableReturnsOf(
  { readings, }: { readonly readings: readonly WeeklyReading[]; },
): void {
  for (const {
    blockName,
    weekly,
  } of readings) {
    if (weekly.nextRegenAt === undefined)
      warnUnreadableReturn({ blockName, },);
  }
}

/**
 Reads the weekly window from whichever key a body carries it under.

 BOTH KEYS ARE ACCEPTED (see the region comment): the block under either name, two blocks that read the same, two that agree on
 `percentRemaining` and differ in `nextRegenAt` (read with the return marked unreadable, and a warning), and a refusal for two that
 differ in `percentRemaining`. A key that is missing or `null` counts as absent; a key that holds anything else but an object is refused. A body with
 no usable key is refused naming both.

 @param parsed - parsed body

 @returns Typed weekly window

 @throws {@link QuotaShapeError} when no key holds a block, a key holds a value that is neither `null` nor an object,
 `percentRemaining` is missing or mistyped, or the two blocks of a body that carries both differ in `percentRemaining`

 @example
 ```ts
 const weekly = weeklyWindowFromBody({ parsed, },);
 ```
 */
function weeklyWindowFromBody(
  { parsed, }: { readonly parsed: Readonly<Record<string, unknown>>; },
): QuotaSnapshot['weekly'] {
  /**
   Each block the body carries under a weekly name, in the order of {@link WEEKLY_BLOCK_NAMES}, with the window read from it.
   */
  const readings = WEEKLY_BLOCK_NAMES.flatMap(function readCarried(blockName,): readonly WeeklyReading[] {
    /**
     The value under this key as delivered, missing and `null` both meaning no block.
     */
    const block = parsed[blockName];
    if ((block === undefined) || (block === null))
      return [];
    if (!isJsonRecord(block,))
      throw shapeRefusal({
        detail: `${blockName} is not an object`,
        parsed,
      },);

    return [
      {
        blockName,
        block,
        weekly: weeklyWindowOf({
          parsed,
          blockName,
          block,
        },),
      },
    ];
  },);
  /**
   The first block, which the key order makes `weeklyTokenLimit` where both are carried, and the second, present only for a body that
   carries both.
   */
  const [first, second,] = readings;
  if (first === undefined)
    throw shapeRefusal({
      detail: `neither ${WEEKLY_BLOCK_NAMES.join(' nor ',)} is an object`,
      parsed,
    },);
  /**
   The two windows, whose percents decide and whose returns only feed a log line.
   */
  const { weekly: firstWindow, } = first;
  if (second === undefined) {
    warnUnreadableReturnsOf({ readings: [first,], },);
    return firstWindow;
  }
  /**
   The second window, present only for a body that carries both names.
   */
  const { weekly: secondWindow, } = second;
  if (firstWindow.percentRemaining !== secondWindow.percentRemaining)
    throw new QuotaShapeError({
      detail: `${first.blockName} and ${second.blockName} are both present and read differently`,
      received: `${receivedShapeOf({ parsed, },)}; ${first.blockName} holds ${describedKeysOf({ record: first.block, },)}; `
        + `${second.blockName} holds ${describedKeysOf({ record: second.block, },)}`,
    },);
  if (firstWindow.nextRegenAt === secondWindow.nextRegenAt) {
    warnUnreadableReturnsOf({ readings: [
      first,
      second,
    ], },);
    return firstWindow;
  }

  // THE BLOCKS AGREE ON THE ONLY FIELD A DECISION READS, so the body is not refused for the log-only return: which instant is
  // right is unknowable, and neither is carried. ONE CAUSE, ONE WARNING: a block whose own return is missing or unspelled is part of
  // this disagreement and does not warn beside it.
  tagged({
    tag: weeklyWindowFromBody.name,
    l,
  },)
    .warn(
    `Synthetic /quotas ${first.blockName} and ${second.blockName} agree on percentRemaining and differ in nextRegenAt; `
      + 'the return reads unreadable',
    );

  return { percentRemaining: firstWindow.percentRemaining, };
}

/**
 Parses one `/quotas` body into the typed snapshot,
 validating every consumed field.

 THE WEEKLY BLOCK IS REQUIRED AND THE FIVE-HOUR BLOCK IS NOT. A body without the `rollingFiveHourLimit` key yields a snapshot with no
 `fiveHour` property; a body that carries the key must carry a well-formed block. The weekly block is read under `weeklyTokenLimit`
 or `planWeeklyLimit`.

 @param bodyText - raw 200-response body

 @returns Typed snapshot of budget-relevant quota state

 @throws {@link QuotaShapeError} when body is not JSON, the weekly block is missing or mistyped, or the five-hour block is present
 and mistyped

 @example
 ```ts
 const snapshot = parseQuotaSnapshot({ bodyText: reply.bodyText, },);
 if (snapshot.fiveHour?.limited) backOff(snapshot.fiveHour.nextTickAt,);
 ```
 */
export function parseQuotaSnapshot(
  { bodyText, }: { readonly bodyText: string; },
): QuotaSnapshot {
  /**
   Whole parsed body, probed field by field.
   */
  const parsed = parseQuotaJson({ bodyText, },);
  if (!isJsonRecord(parsed,))
    throw shapeRefusal({
      detail: 'body is not a JSON object',
      parsed,
    },);

  /**
   Whether the body carries the five-hour key at all, which is the only thing that makes the window optional: a present key must
   hold an object.
   */
  const carriesFiveHour = Object.hasOwn(
    parsed,
    'rollingFiveHourLimit',
  );

  /**
   Rolling five-hour block as delivered.
   */
  const { rollingFiveHourLimit: fiveHourBlock, } = parsed;
  if (carriesFiveHour && (!isJsonRecord(fiveHourBlock,)))
    throw shapeRefusal({
      detail: 'rollingFiveHourLimit is not an object',
      parsed,
    },);

  /**
   The five-hour window as a property to spread: empty when the body had no such key, since the `carriesFiveHour` check refuses a
   key that is present and no object.
   */
  const fiveHourEntry = isJsonRecord(fiveHourBlock,)
    ? {
      fiveHour: fiveHourWindowOf({
        parsed,
        block: fiveHourBlock,
      },),
    }
    : {};

  return {
    ...fiveHourEntry,
    weekly: weeklyWindowFromBody({ parsed, },),
  };
}

//endregion Quota snapshot
