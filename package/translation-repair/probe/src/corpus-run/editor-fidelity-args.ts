import { StatedRefusalError, } from '@monochromatic-dev/translation-repair-glossary/ts';
import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import {
  type CommandLineOf,
  type FlagValue,
  idListFlag,
  usdFlag,
  wholeNumberFlag,
} from '@monochromatic-dev/translation-repair-run/ts';
import {
  EDITOR_DAMAGE_KINDS,
  type EditorDamageKind,
} from '../editor-fidelity-trials.ts';

//region Editor fidelity arguments
// What the editor fidelity probe is asked on its command line, kept beside the probe so the probe itself stays under
// the file-length cap. The flags are the judge fidelity probe's, less `--context`, which has no meaning for an editor,
// and with the amount the run may spend.

/**
 How many trials one invocation runs by default, counted in attempts like the judge probe's own: enough for every trial
 of the frozen pool, so an arm run with no `--cap` asks them all and every arm sees the same ones.
 */
export const DEFAULT_EDITOR_TRIAL_CAP = 64;

/**
 Most a run may spend, in US dollars, when `--max-usd` is not written: the spend guard refuses the call that would take
 the spend measured from the replies, plus that call's ceiling, past it.
 */
export const DEFAULT_EDITOR_MAX_USD = 2;

/**
 Most the four arms of one editor fidelity bench may spend together, in US dollars: the owner's ruling for the bench, which
 the third round's proposed caps (37 USD in all) passed. The arms are separate processes, so nothing holds them together
 but the operator, who writes the spend the earlier arms measured as `--bench-spent-usd`; an arm whose `--max-usd` added
 to that passes this amount is refused before it reads or spends anything.
 */
export const EDITOR_BENCH_MAX_USD = 20;

/**
 Cents in a dollar, the unit amounts are compared in.
 */
const CENTS_PER_USD = 100;

/**
 Spells the defects `--damage` takes, the last one joined by "or".

 @returns The defects in list order

 @example
 ```ts
 // spelled is 'insertion or alteration'
 const spelled = damageSpellings();
 ```
 */
function damageSpellings(): string {
  return EDITOR_DAMAGE_KINDS.join(' or ',);
}

/**
 Reads which defects `--damage` asks for.

 @param damage - what the flag carried, or that nobody wrote it

 @returns Every defect the probe builds when the flag was not written, else the one named

 @throws StatedRefusalError when the flag names a deletion, with why it is not built, or a spelling this probe does
 not build

 @example
 ```ts
 const kinds = damageKindsOf({ damage: { kind: 'written', flag: '--damage', value: 'insertion', }, },);
 ```
 */
function damageKindsOf({ damage, }: { readonly damage: FlagValue; },): readonly EditorDamageKind[] {
  if (damage.kind === 'unwritten')
    return EDITOR_DAMAGE_KINDS;
  /**
   The defect that spelling names, absent when it names none this probe builds.
   */
  const named = EDITOR_DAMAGE_KINDS.find(function is(kind,): boolean {
    return kind === damage.value;
  },);
  if (named !== undefined)
    return [named,];
  if (damage.value === 'deletion') {
    throw new StatedRefusalError({
      says: `${damage.flag} deletion is not built for the editor probe: its fix is a sentence the editor must write, `
        + `which no deterministic test scores against the archive's wording; the probe takes ${
         damageSpellings()}`,
    },);
  }
  throw new StatedRefusalError({
    says: `${damage.flag} takes ${damageSpellings()}, not ${JSON.stringify(damage.value,)}`,
  },);
}

/**
 Writes a dollar amount in the fewest digits that hold its cents, so a sum of two amounts reads as typed.

 @param usd - amount to write

 @returns The amount to the cent, without trailing zeros

 @example
 ```ts
 // written is '20.5'
 const written = centsText({ usd: 20.5, },);
 ```
 */
function centsText({ usd, }: { readonly usd: number; },): string {
  return String(Math.round(usd * CENTS_PER_USD,) / CENTS_PER_USD,);
}

/**
 Refuses an arm whose largest spend, added to what the earlier arms measured, passes the bench total.

 @param maxUsd - most this arm may spend

 @param benchSpentUsd - what the earlier arms of the bench measured

 @throws StatedRefusalError when the two together pass the bench total, naming both, their sum, the total and what
 `--max-usd` may still be

 @example
 ```ts
 refuseOverBench({ maxUsd: 2, benchSpentUsd: 19, },);
 ```
 */
function refuseOverBench(
  {
    maxUsd,
    benchSpentUsd,
  }: {
    readonly maxUsd: number;
    readonly benchSpentUsd: number;
  },
): void {
  /**
   Whole cents, so a sum that reaches the total exactly is not refused for a rounding error of the sum of two decimals.
   */
  const askedCents = Math.round(maxUsd * CENTS_PER_USD,) + Math.round(benchSpentUsd * CENTS_PER_USD,);
  if (askedCents <= (EDITOR_BENCH_MAX_USD * CENTS_PER_USD))
    return;
  throw new StatedRefusalError({
    says: `the bench would spend up to ${centsText({ usd: askedCents / CENTS_PER_USD, },)} USD: --bench-spent-usd ${
      centsText({ usd: benchSpentUsd, },)
    } plus --max-usd ${centsText({ usd: maxUsd, },)} passes the ${
      String(EDITOR_BENCH_MAX_USD,)
    } USD the owner ruled for all the arms of the editor fidelity bench together; write a --max-usd of at most ${
      centsText({ usd: Math.max(
        0,
        EDITOR_BENCH_MAX_USD - benchSpentUsd,
      ), },)
    }, or stop the bench here`,
  },);
}

/**
 Reads `--only`, `--cap`, `--damage`, `--max-usd` and `--bench-spent-usd` from the command line.

 @internal

 @param line - the probe's command line, read whole by `reportingRefusals` and passed in so this is testable without a
 subprocess

 @returns Entry ids to trial, empty for every entry, the trial cap, the defects to build, the most the run may cost and
 what the earlier arms of the bench measured

 @throws StatedRefusalError when a cap is not a whole number written in digits or is below zero, an amount is not a
 plain decimal, a defect is one this probe does not build, `--bench-spent-usd` is not written, or the amounts together
 pass the bench total

 @example
 ```ts
 const { onlyIds, cap, damageKinds, maxUsd, benchSpentUsd, } = readEditorFidelityArguments({ line, },);
 ```
 */
export function readEditorFidelityArguments(
  { line, }: { readonly line: CommandLineOf<'editor-fidelity-probe'>; },
): {
  readonly onlyIds: readonly string[];
  readonly cap: number;
  readonly damageKinds: readonly EditorDamageKind[];
  readonly maxUsd: number;
  readonly benchSpentUsd: number;
} {
  /**
   Defects to build, read before the other flags so a bad one is refused first.
   */
  const damageKinds = damageKindsOf({ damage: line.flag('damage',), },);
  /**
   Most this arm may spend.
   */
  const maxUsd = usdFlag({
    asked: line.flag('max-usd',),
    unwritten: DEFAULT_EDITOR_MAX_USD,
  },);
  /**
   What the line says of the spend of the earlier arms.
   */
  const benchSpentAsked = line.flag('bench-spent-usd',);
  // NO DEFAULT: an arm that is not told what the earlier arms spent would hold the bench to a total it cannot see, and a
  // default of zero would let a later arm be launched without it. The first arm writes zero.
  if (benchSpentAsked.kind === 'unwritten') {
    throw new StatedRefusalError({
      says: '--bench-spent-usd is required: write what the arms already run measured (the sum of their GUARD total= '
        + 'lines, rounded up to the cent), or 0 for the first arm of a bench',
    },);
  }
  /**
   What the earlier arms of the bench measured, zero for the first.
   */
  const benchSpentUsd = usdFlag({
    asked: benchSpentAsked,
    unwritten: 0,
  },);
  refuseOverBench({
    maxUsd,
    benchSpentUsd,
  },);
  return {
    damageKinds,
    onlyIds: idListFlag({
      asked: line.flag('only',),
      naming: 'entry id',
    },),
    cap: wholeNumberFlag({
      asked: line.flag('cap',),
      unwritten: DEFAULT_EDITOR_TRIAL_CAP,
      leaveOffTo: `run the default of ${String(DEFAULT_EDITOR_TRIAL_CAP,)} ${
        wordForCount({
          count: DEFAULT_EDITOR_TRIAL_CAP,
          one: 'trial',
          many: 'trials',
        },)
      }`,
    },),
    maxUsd,
    benchSpentUsd,
  };
}

//endregion Editor fidelity arguments
