import { join, } from 'node:path';

import { isDecimalText, } from '@monochromatic-dev/translation-repair-markup/ts';
import { StatedRefusalError, } from '@monochromatic-dev/translation-repair-glossary/ts';

//region NeuralWatt ledger configuration
// WHERE THE ENERGY LEDGER LIVES AND HOW MUCH A DAY IT HOLDS, and the two ways
// reading it can fail. Modelled on `bedrock-ledger.ts`: the XDG state
// directory under the home the process runs as, because every launch uses a
// fresh runs directory and the free energy is spent across all of them.

/**
 Free energy the day holds, in watt-hours: the owner's 500 on the Max plan
 and 50 for the product-news email (`https://docs.neuralwatt.com/billing/daily-free-energy`,
 fetched 2026-10-20). The page says amounts can change from one promo to the
 next; the variable overrides this on the day they do.
 */
export const NEURALWATT_DAILY_WH = 550;

/**
 Environment variable overriding the day's allowance, in watt-hours: the
 owner's lever for a promo of a different size, and `0` for a day with no
 promo, when the provider reads dry from the start.
 */
export const NEURALWATT_DAILY_WH_VAR = 'TRANSLATION_REPAIR_NEURALWATT_DAILY_WH';

/**
 Environment variable overriding where the ledger file lives.
 */
export const NEURALWATT_LEDGER_PATH_VAR = 'TRANSLATION_REPAIR_NEURALWATT_LEDGER';

/**
 Where the ledger lives when nothing says otherwise: under the home the
 process runs as, never a spelled-out username.

 @param home - home directory, injectable so a test uses a disposable one

 @returns Absolute path of the ledger file

 @example
 ```ts
 defaultNeuralWattLedgerPath({ home: homedir(), },);
 ```
 */
export function defaultNeuralWattLedgerPath({ home, }: { readonly home: string; },): string {
  return join(
    home,
    '.local',
    'state',
    'translation-repair',
    'neuralwatt-energy.jsonl',
  );
}

/**
 Raised when the allowance override is present but is not a usable amount.

 @example
 ```ts
 throw new NeuralWattAllowanceOverrideError({ value: 'plenty', },);
 ```
 */
export class NeuralWattAllowanceOverrideError extends StatedRefusalError {
  /**
   Declares this message safe to forward: it names the variable and repeats
   the value the operator set in it.
   */
  override readonly messageNamesOnly: true = true;

  /**
   Names the variable, what it held, and why that cannot be an allowance.

   @param value - what the variable held, quoted back

   @example
   ```ts
   new NeuralWattAllowanceOverrideError({ value: '-5', },);
   ```
   */
  public constructor({ value, }: { readonly value: string; },) {
    super({
      says: `${NEURALWATT_DAILY_WH_VAR} must be a plain non-negative number of watt-hours; it holds ${
         JSON.stringify(value,)}`,
    },);
    this.name = 'NeuralWattAllowanceOverrideError';
  }
}

/**
 Raised when a ledger line will not read as an energy entry.

 NAMES THE FILE as well as the line, and is thrown rather than skipped: a
 ledger that silently dropped a line would report energy as unspent, and the
 next call could be billed to the owner's plan.

 @example
 ```ts
 throw new NeuralWattLedgerShapeError({ path, line: 3, detail: 'wh is not a non-negative number', },);
 ```
 */
export class NeuralWattLedgerShapeError extends Error {
  /**
   Declares this message safe to forward: a path, a line number and an
   authored phrase, never the line's content.
   */
  readonly messageNamesOnly: true = true;

  /**
   Names the file, the line and what was wrong with it.

   @param path - ledger file the line is in

   @param line - one-based line number in the file

   @param detail - authored phrase naming the broken expectation

   @example
   ```ts
   new NeuralWattLedgerShapeError({ path, line: 3, detail: 'not a JSON object', },);
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
    super(`NeuralWatt energy ledger ${path} line ${String(line,)} violated expectations: ${detail}`,);
    this.name = 'NeuralWattLedgerShapeError';
  }
}

/**
 The day's allowance: the variable's amount, or the built-in.

 @param env - process environment

 @returns Watt-hours a UTC day holds

 @throws {@link NeuralWattAllowanceOverrideError} when the variable holds
 something that is not a plain non-negative decimal

 @example
 ```ts
 const dailyWh = neuralWattDailyWhFrom({ env: process.env, },);
 ```
 */
export function neuralWattDailyWhFrom({ env, }: { readonly env: Readonly<NodeJS.ProcessEnv>; },): number {
  /**
   What the variable holds, absent or blank for the built-in.
   */
  const raw = env[NEURALWATT_DAILY_WH_VAR];
  if ((raw === undefined) || (raw.trim() === ''))
    return NEURALWATT_DAILY_WH;

  /**
   The override as a number, NaN where it is not one.
   */
  const parsed = Number(raw,);

  // A plain decimal, as an operator writes watt-hours: `Number` alone also
  // reads `0x10`, `1e3`, `+5` and ` 5` as amounts nobody wrote that way.
  if (!(isDecimalText({ text: raw, },) && Number.isFinite(parsed,)))
    throw new NeuralWattAllowanceOverrideError({ value: raw, },);
  return parsed;
}

/**
 Where the ledger lives: the variable's path, or the default under the home.

 @param env - process environment

 @param home - home directory the default hangs off

 @returns Absolute path

 @example
 ```ts
 const path = neuralWattLedgerPathFrom({ env: process.env, home: homedir(), },);
 ```
 */
export function neuralWattLedgerPathFrom(
  {
    env,
    home,
  }: {
    readonly env: Readonly<NodeJS.ProcessEnv>;
    readonly home: string;
  },
): string {
  /**
   What the variable holds, absent or blank for the default.
   */
  const raw = env[NEURALWATT_LEDGER_PATH_VAR];
  if ((raw === undefined) || (raw.trim() === ''))
    return defaultNeuralWattLedgerPath({ home, },);
  return raw;
}

//endregion NeuralWatt ledger configuration
