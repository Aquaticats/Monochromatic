import { homedir, } from 'node:os';
import { join, } from 'node:path';

import { tagged, } from '@monochromatic-dev/logger-core/ts';

import {
  contextRoot,
  readTextOrEmptyIfMissing,
  wordForCount,
} from '@monochromatic-dev/translation-repair-typography/ts';
import {
  errorName,
  isDecimalText,
} from '@monochromatic-dev/translation-repair-markup/ts';
import { isJsonRecord, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import { StatedRefusalError, } from '@monochromatic-dev/translation-repair-glossary/ts';
import { appendLedgerLine, } from './ledger-line-append.ts';
import {
  isSpendReckoning,
  type SpendReckoning,
} from './spend-line.ts';

//region Bedrock ledger
// THE METER THIS PROVIDER DOES NOT HAVE. Synthetic reports quotas, Charm Hyper
// a balance and OpenRouter its credits, each on an endpoint a bearer key may
// read; Amazon Bedrock exposes no balance to its API key, and the owner's
// credits are promotional ones the account spends first and then bills past.
// The owner, 2026-09-07: "I have 200USD of credits", "you're allowed to use it
// as much as you like. But I will NEVER top it up." So the line past which
// spending would reach the owner's card is this package's to keep, and it is
// kept here: every priced call appends one line to a durable file, and the
// meter is the credit minus the sum of those lines.
//
// APPEND-ONLY JSON LINES rather than a rewritten total, so two processes
// writing at once (a pass and a budget sample) cannot lose each other's
// calls, and so a reader can see what was bought and when. A line that will
// not parse is a thrown error naming the file and the line's number, not a
// skipped line: a ledger that silently dropped a line would report money as
// unspent.
//
// DURABLE OUTSIDE THE RUNS DIRECTORY, since every launch uses a fresh runs
// directory and the credit is spent across all of them: the XDG state
// directory under the home the process runs as, overridable by variable.

/**
 Logger root for the ledger.
 */
const l = contextRoot({ tag: 'translation-repair', },);

/**
 Decimals the spend is logged to: a call costs fractions of a cent.
 */
const SPENT_LOG_DECIMALS = 4;

/**
 Credit the owner stated on 2026-09-07, in USD.
 */
export const BEDROCK_CREDIT_USD = 200;

/**
 Environment variable overriding the credit line, in USD, for the day the
 owner reads a different balance off the console.
 */
export const BEDROCK_CREDIT_USD_VAR = 'TRANSLATION_REPAIR_BEDROCK_CREDIT_USD';

/**
 Environment variable overriding where the ledger file lives.
 */
export const BEDROCK_LEDGER_PATH_VAR = 'TRANSLATION_REPAIR_BEDROCK_LEDGER';

/**
 Where the ledger lives when nothing says otherwise: under the home the
 process runs as, never a spelled-out username.

 @param home - home directory, injectable so a test uses a disposable one

 @returns Absolute path of the ledger file

 @example
 ```ts
 defaultBedrockLedgerPath({ home: homedir(), },);
 ```
 */
export function defaultBedrockLedgerPath(
  { home, }: { readonly home: string; },
): string {
  return join(
    home,
    '.local',
    'state',
    'translation-repair',
    'bedrock-spend.jsonl',
  );
}

/**
 Raised when the credit override is present but is not a usable amount.

 @example
 ```ts
 throw new BedrockCreditOverrideError({ value: 'plenty', },);
 ```
 */
export class BedrockCreditOverrideError extends StatedRefusalError {
  /**
   Declares this message safe to forward: it names the variable and repeats
   the value the operator set in it.
   */
  override readonly messageNamesOnly: true = true;

  /**
   Names the variable, what it held, and why that cannot be a credit.

   @param value - what the variable held, quoted back

   @example
   ```ts
   new BedrockCreditOverrideError({ value: '', },);
   ```
   */
  public constructor({ value, }: { readonly value: string; },) {
    super({
      says: `${BEDROCK_CREDIT_USD_VAR} must be a non-negative number of USD; it holds ${JSON.stringify(value,)}`,
    },);
    this.name = 'BedrockCreditOverrideError';
  }
}

/**
 Raised when a ledger line will not read as a spend entry.

 NAMES THE FILE as well as the line: the ledger is repaired by hand (the
 module note says why a line is never skipped), and the variable can move it
 off its default, so the line number alone does not say which file to open.

 @example
 ```ts
 throw new BedrockLedgerShapeError({ path, line: 3, detail: 'usd is not a number', },);
 ```
 */
export class BedrockLedgerShapeError extends Error {
  /**
   Declares this message safe to forward: the ledger file's path, which the
   home or the operator's variable names, a line number and an authored
   phrase, never the line's content.
   */
  readonly messageNamesOnly: true = true;

  /**
   Names the file, the line and what was wrong with it.

   @param path - ledger file the line is in

   @param line - one-based line number in the ledger file

   @param detail - authored phrase naming the broken expectation

   @example
   ```ts
   new BedrockLedgerShapeError({ path, line: 3, detail: 'not a JSON object', },);
   ```
   */
  public constructor(
    {
      path,
      line,
      detail,
    }: {
      readonly path: string;
      readonly line: number;
      readonly detail: string;
    },
  ) {
    super(`Bedrock spend ledger ${path} line ${String(line,)} violated expectations: ${detail}`,);
    this.name = 'BedrockLedgerShapeError';
  }
}

/**
 One priced call, as the ledger keeps it.

 @example
 ```ts
 const entry: BedrockLedgerEntry = { at: '2026-09-07T20:00:00.000Z', model: 'google.gemma-4-e2b', usd: 0.0001, promptTokens: 90, completionTokens: 10, };
 ```
 */
export type BedrockLedgerEntry = {
  /**
   When the call completed, ISO 8601.
   */
  readonly at: string;

  /**
   Model as this provider spells it.
   */
  readonly model: string;

  /**
   What the call cost, in USD, off the catalog's prices.
   */
  readonly usd: number;

  /**
   Prompt tokens the provider reported.
   */
  readonly promptTokens: number;

  /**
   Completion tokens the provider reported.
   */
  readonly completionTokens: number;

  /**
   Why the counts and cost are reckoned rather than reported, absent on a
   call whose usage the provider reported: an attempt the endpoint accepted
   and never finished is noted at the most it could have been billed
   (ledger P1).
   */
  readonly estimated?: SpendReckoning;
};

/**
 What the ledger says is left, which is this provider's whole budget signal.

 @example
 ```ts
 const credits: BedrockCredits = { creditUsd: 200, spentUsd: 1.5, reckonedUsd: 0.2, remainingUsd: 198.5, calls: 12, };
 ```
 */
export type BedrockCredits = {
  /**
   Credit line the spend is measured against.
   */
  readonly creditUsd: number;

  /**
   Sum of every line in the ledger.
   */
  readonly spentUsd: number;

  /**
   The part of the spend that lines marked `estimated` hold: attempts priced
   at their bound or reckoned off what they delivered rather than off
   reported usage (ledger P1), so a reader sees how much of what is left
   rests on reckoning.
   */
  readonly reckonedUsd: number;

  /**
   Credit minus spend, which may go below zero once the last calls in
   flight settle.
   */
  readonly remainingUsd: number;

  /**
   How many lines the ledger holds.
   */
  readonly calls: number;
};

/**
 The durable spend record and the meter read off it.

 @example
 ```ts
 const ledger: BedrockLedger = createBedrockLedger({ path, creditUsd: 200, },);
 ```
 */
export type BedrockLedger = {
  /**
   Where the file lives, for the log line and the record.
   */
  readonly path: string;

  /**
   Appends one priced call.
   */
  readonly note: (entry: BedrockLedgerEntry,) => Promise<void>;

  /**
   Reads the credit, the spend and what is left.
   */
  readonly read: () => Promise<BedrockCredits>;
};

/**
 Credit line the environment names, or the owner's figure.

 @param env - process environment

 @returns Credit in USD

 @throws {@link BedrockCreditOverrideError} when the variable is set to
 something that is not a non-negative number

 @example
 ```ts
 const creditUsd = bedrockCreditUsdFrom({ env: process.env, },);
 ```
 */
export function bedrockCreditUsdFrom(
  { env, }: { readonly env: Readonly<NodeJS.ProcessEnv>; },
): number {
  /**
   What the variable holds, absent or empty for the built-in.
   */
  const raw = env[BEDROCK_CREDIT_USD_VAR];
  if ((raw === undefined) || (raw.trim() === ''))
    return BEDROCK_CREDIT_USD;

  /**
   The override as a number, NaN where it is not one.
   */
  const parsed = Number(raw,);

  /**
   Whether the override reads as an amount at all: a plain decimal, as an
   operator writes dollars, which carries no sign, and finite. `Number` also
   read `0x10`, `1e1`, `+15`, ` 15` and `.5` as amounts nobody wrote that way
   (ledger B73).
   */
  const isAmount = isDecimalText({ text: raw, },) && Number.isFinite(parsed,);
  if (!isAmount)
    throw new BedrockCreditOverrideError({ value: raw, },);
  return parsed;
}

/**
 Where the ledger lives: the variable's path, or the default under the home.

 @param env - process environment

 @param home - home directory the default hangs off

 @returns Absolute path

 @example
 ```ts
 const path = bedrockLedgerPathFrom({ env: process.env, home: homedir(), },);
 ```
 */
export function bedrockLedgerPathFrom(
  {
    env,
    home,
  }: {
    readonly env: Readonly<NodeJS.ProcessEnv>;
    readonly home: string;
  },
): string {
  /**
   What the variable holds, absent or empty for the default.
   */
  const raw = env[BEDROCK_LEDGER_PATH_VAR];
  if ((raw === undefined) || (raw.trim() === ''))
    return defaultBedrockLedgerPath({ home, },);
  return raw;
}

/**
 Reads one ledger line as an entry, naming the file and the line when it will
 not read.

 @param path - ledger file, for the error

 @param text - one line of the file

 @param line - its one-based number, for the error

 @returns Entry the line holds

 @throws {@link BedrockLedgerShapeError} when the line is not an entry

 @example
 ```ts
 const entry = entryOf({ path, text, line: 1, },);
 ```
 */
function entryOf(
  {
    path,
    text,
    line,
  }: {
    readonly path: string;
    readonly text: string;
    readonly line: number;
  },
): BedrockLedgerEntry {
  /**
   Parsed line, unknown until its shape is checked.
   */
  const parsed: unknown = (function parse(): unknown {
    try {
      return JSON.parse(text,);
    } catch (error) {
      throw new BedrockLedgerShapeError({
        path,
        line,
        detail: `not valid JSON (${errorName({ error, },)})`,
      },);
    }
  })();
  if (!isJsonRecord(parsed,))
    throw new BedrockLedgerShapeError({
      path,
      line,
      detail: 'not a JSON object',
    },);

  /**
   Fields as the line carries them, each still unknown.
   */
  const {
    at,
    model,
    usd,
    promptTokens,
    completionTokens,
    estimated,
  } = parsed;
  if ((typeof at) !== 'string')
    throw new BedrockLedgerShapeError({
      path,
      line,
      detail: 'at is not a string',
    },);
  if ((typeof model) !== 'string')
    throw new BedrockLedgerShapeError({
      path,
      line,
      detail: 'model is not a string',
    },);
  /**
   Whether the cost is a finite non-negative number.
   */
  const usdIsAmount = ((typeof usd) === 'number')
    && Number.isFinite(usd,)
    && (usd >= 0);
  if (!usdIsAmount)
    throw new BedrockLedgerShapeError({
      path,
      line,
      detail: 'usd is not a non-negative number',
    },);
  if (((typeof promptTokens) !== 'number') || ((typeof completionTokens) !== 'number'))
    throw new BedrockLedgerShapeError({
      path,
      line,
      detail: 'token counts are not numbers',
    },);
  if (estimated === undefined) {
    return {
      at,
      model,
      usd,
      promptTokens,
      completionTokens,
    };
  }
  if (!isSpendReckoning(estimated,))
    throw new BedrockLedgerShapeError({
      path,
      line,
      detail: 'estimated is not a reckoning this package writes',
    },);
  return {
    at,
    model,
    usd,
    promptTokens,
    completionTokens,
    estimated,
  };
}

/**
 Builds the ledger over one file and one credit line.

 @param path - ledger file, created on the first note

 @param creditUsd - credit line the spend is measured against

 @returns Ledger surface

 @example
 ```ts
 const ledger = createBedrockLedger({ path: bedrockLedgerPathFrom({ env: process.env, home: homedir(), },), creditUsd: bedrockCreditUsdFrom({ env: process.env, },), },);
 ```
 */
export function createBedrockLedger(
  {
    path,
    creditUsd,
  }: {
    readonly path: string;
    readonly creditUsd: number;
  },
): BedrockLedger {
  /**
   Appends one line, creating the directory on the way.

   @param entry - priced call
   */
  async function note(entry: BedrockLedgerEntry,): Promise<void> {
    await appendLedgerLine({
      path,
      entry,
    },);
  }

  /**
   Sums the file, absent for a file that was never written.

   @returns Credit, spend and what is left
   */
  async function read(): Promise<BedrockCredits> {
    /**
     Logger pre-tagged with this function's name.
     */
    const rl = tagged({
      tag: read.name,
      l,
    },);

    /**
     Whole file, or nothing where none exists yet.
     */
    const text = await readTextOrEmptyIfMissing({ path, },);

    /**
     Entries the file holds, in order.
     */
    const entries = text
      .split('\n',)
      .map(function trim(lineText,): string {
        return lineText.trim();
      },)
      .flatMap(function toEntry(
        lineText,
        index,
      ): readonly BedrockLedgerEntry[] {
        if (lineText === '')
          return [];
        return [entryOf({
          path,
          text: lineText,
          line: index + 1,
        },),];
      },);

    /**
     Everything the ledger says was spent.
     */
    const spentUsd = entries.reduce(
      function addUsd(
        running,
        entry,
      ): number {
        return running + entry.usd;
      },
      0,
    );

    /**
     The part of that spend reckoned rather than reported.
     */
    const reckonedUsd = entries.reduce(
      function addReckoned(
        running,
        entry,
      ): number {
        return (entry.estimated === undefined) ? running : running + entry.usd;
      },
      0,
    );

    /**
     What is left of the credit.
     */
    const remainingUsd = creditUsd - spentUsd;

    rl.debug(
      `${path}: ${String(entries.length,)} ${
        wordForCount({
          count: entries.length,
          one: 'call',
          many: 'calls',
        },)
      }, ${spentUsd.toFixed(SPENT_LOG_DECIMALS,)} USD spent of ${
        String(creditUsd,)
      } (${reckonedUsd.toFixed(SPENT_LOG_DECIMALS,)} of it reckoned), ${remainingUsd.toFixed(2,)} USD left`,
    );
    return {
      creditUsd,
      spentUsd,
      reckonedUsd,
      remainingUsd,
      calls: entries.length,
    };
  }

  return {
    path,
    note,
    read,
  };
}

/**
 Ledger at the path and credit the environment names.

 @param env - process environment

 @returns Ledger surface

 @example
 ```ts
 const ledger = bedrockLedgerFromEnv({ env: process.env, },);
 ```
 */
export function bedrockLedgerFromEnv(
  { env, }: { readonly env: Readonly<NodeJS.ProcessEnv>; },
): BedrockLedger {
  return createBedrockLedger({
    path: bedrockLedgerPathFrom({
      env,
      home: homedir(),
    },),
    creditUsd: bedrockCreditUsdFrom({ env, },),
  },);
}

//endregion Bedrock ledger
