import type { ProviderName, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import { StatedRefusalError, } from '@monochromatic-dev/translation-repair-glossary/ts';
import { isDecimalText, } from '@monochromatic-dev/translation-repair-markup/ts';

//region Spend ceiling
// HOW MUCH ONE RUN MAY SPEND ON EACH PROVIDER THAT BILLS IN USD before it stops
// starting entries.
//
// THE OWNER'S DECISION OF 2026-09-04, asked with two options after the
// OpenRouter fallback landed: the credits meter only stops a pass at zero, and
// auto top-up refills it, so a runaway would spend every top-up until someone
// noticed. A per-run ceiling bounds that to the ceiling plus whatever entries
// were already in flight, which finish; the scheduler's `stopBeforeNext` is
// asked before each entry and never mid-entry.
//
// TWENTY DOLLARS BUILT IN. The planning record of 2026-09-03 priced a day of
// three to four entries bought entirely on OpenRouter without Kimi-K3 at about
// 20 USD, and the auto top-up threshold recommended the same day is 20 USD, so
// a run that reaches this has bought a day's work; anything past it is worth an
// operator's glance. Raise it per launch for a deliberately larger run.
//
// PER PROVIDER, SINCE 2026-10-22, BY THE OWNER'S CHOICE OF "20 USD each" (asked against a
// separate-Xiaomi and a summed figure; worst case about 60 USD plus entries in flight): the same
// allowance applies to each of the providers in `SPEND_CEILING_PROVIDERS` on its own, and the run stops when any
// one reaches it. Xiaomi and DeepSeek joined OpenRouter when the direct routes
// landed ahead of it in the spending order: Xiaomi documents no balance, so
// nothing else bounds its spend, and DeepSeek's prepaid balance may be topped
// up. Each provider's total is its own, never a sum, so the number the owner
// chose for one provider stays the number for each and no new one is invented.
//
// UNSET AND EMPTY ARE THE BUILT-IN; ANYTHING ELSE UNREADABLE THROWS, exactly as
// `cap-override.ts` argues for the entry ceiling: an operator who set a bound
// believes the run is bounded the way they asked for. ZERO IS ALLOWED, and it
// means "start nothing": that is how the guard is shown to fire on a live run
// without spending a cent.

/**
 Environment variable overriding the per-run ceiling, in USD.
 */
export const SPEND_CEILING_VAR = 'TRANSLATION_REPAIR_RUN_SPEND_CEILING_USD';

/**
 Built-in per-run ceiling, in USD, on each provider in {@link SPEND_CEILING_PROVIDERS}.
 */
export const SPEND_CEILING_USD = 20;

/**
 Providers the ceiling meters, each against the ceiling alone: the ones that bill in USD per token and whose spend
 no other bound stops (Xiaomi has no balance, and OpenRouter and DeepSeek balances may be topped up).
 */
export const SPEND_CEILING_PROVIDERS: readonly ProviderName[] = [
  'openrouter',
  'deepseek',
  'xiaomi',
];

/**
 Raised when the override is present but is not a usable amount.
 */
export class SpendCeilingOverrideError extends StatedRefusalError {
  /**
   Declares this message safe to forward: it names the variable and repeats
   the value the operator set in it.
   */
  override readonly messageNamesOnly: true = true;

  /**
   Names the variable, what it held, and why that cannot be a ceiling.

   @param value - what the variable held, quoted back

   @example
   ```ts
   throw new SpendCeilingOverrideError({ value: 'plenty', },);
   ```
   */
  constructor({ value, }: { readonly value: string; },) {
    super({
      says: `${SPEND_CEILING_VAR} must be a non-negative number of USD; received ${JSON.stringify(value,)}.`
        + ' This is the ceiling that stops a run spending past its allowance, so an unreadable'
        + ' value is refused rather than quietly replaced by the default: an operator who set it'
        + ' believes the run is bounded the way they asked for.',
    },);
    this.name = 'SpendCeilingOverrideError';
  }
}

/**
 Reads the per-run ceiling this invocation runs under.

 @param fallback - built-in ceiling, used when nothing overrides it

 @param raw - override text, which the entry file reads from its launch's
 environment and hands in, so no library code reads the process's own (ledger
 B398). Unset and empty collapse to one value here, as in `cap-override.ts`:
 neither is an override

 @returns USD this run may spend on the metered provider before it stops
 starting entries

 @throws {@link SpendCeilingOverrideError} when the override is present and
 is not a finite non-negative number

 @example
 ```ts
 const ceiling = resolveSpendCeilingUsd({ fallback: SPEND_CEILING_USD, raw: process.env[SPEND_CEILING_VAR] ?? '', },);
 ```
 */
export function resolveSpendCeilingUsd(
  {
    fallback,
    raw,
  }: {
    readonly fallback: number;
    readonly raw: string;
  },
): number {
  if (raw.trim() === '')
    return fallback;

  /**
   Override read as a number; `Number` rather than `parseFloat` so a trailing
   word makes the whole value unreadable instead of being dropped.
   */
  const usd = Number(raw,);

  // A PLAIN DECIMAL, as an operator writes dollars, which carries no sign:
  // `Number` also read `0x10`, `1e1`, `+15`, ` 15` and `.5` as amounts nobody
  // wrote that way (ledger B73). A digit run past the largest double is still
  // a plain decimal, and reads as Infinity.
  if ((!isDecimalText({ text: raw, },)) || (!Number.isFinite(usd,)))
    throw new SpendCeilingOverrideError({ value: raw, },);

  return usd;
}

/**
 Names the allowance a launch overrode, or says nothing when it kept the
 built-in.

 A run must never hide which ceiling it ran under: one that stopped early
 for money reads the same as one that ran out of entries unless the
 allowance is on record.

 @param ceilingUsd - allowance this run resolved

 @returns Line for the run log, empty when the built-in is in force

 @example
 ```ts
 const note = spendCeilingOverrideNote({ ceilingUsd: 50, },);
 ```
 */
export function spendCeilingOverrideNote({ ceilingUsd, }: { readonly ceilingUsd: number; },): string {
  if (ceilingUsd === SPEND_CEILING_USD)
    return '';
  return `SPEND CEILING OVERRIDDEN by ${SPEND_CEILING_VAR}: new entries stop once this run has spent ${String(ceilingUsd,)} USD on any one of ${SPEND_CEILING_PROVIDERS.join(', ',)} rather than the built-in ${String(SPEND_CEILING_USD,)}`;
}

/**
 Whether a run has spent its allowance.

 AT OR PAST, so a ceiling of zero refuses the first entry.

 @param spentUsd - what the run has spent on the metered provider so far

 @param ceilingUsd - the run's allowance

 @returns Whether no further entry may start

 @example
 ```ts
 const stop = spendCeilingReached({ spentUsd: 20.4, ceilingUsd: 20, },);
 ```
 */
export function spendCeilingReached(
  {
    spentUsd,
    ceilingUsd,
  }: {
    readonly spentUsd: number;
    readonly ceilingUsd: number;
  },
): boolean {
  return spentUsd >= ceilingUsd;
}

/**
 Explains why the run is starting no more entries.

 @param provider - the metered provider whose spend reached the ceiling

 @param spentUsd - what the run has spent on that provider

 @param ceilingUsd - the run's allowance

 @returns Line naming both figures, the provider, and the dial that raises
 the allowance

 @example
 ```ts
 console.log(spendCeilingNote({ provider: 'xiaomi', spentUsd: 20.4, ceilingUsd: 20, },),);
 ```
 */
export function spendCeilingNote(
  {
    provider,
    spentUsd,
    ceilingUsd,
  }: {
    readonly provider: ProviderName;
    readonly spentUsd: number;
    readonly ceilingUsd: number;
  },
): string {
  return `SPEND CEILING reached: ${String(spentUsd,)} of ${String(ceilingUsd,)} USD spent on `
    + `${provider} this run; not starting new entries. Entries already running finish. `
    + `Raise ${SPEND_CEILING_VAR} for a larger run.`;
}

//endregion Spend ceiling
