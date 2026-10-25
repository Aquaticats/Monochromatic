import { nonNullishOrThrow, } from '@monochromatic-dev/module-or-throw/ts';

import {
  FIDELITY_DAMAGE_KINDS,
  type FidelityDamageKind,
} from '@monochromatic-dev/translation-repair-repair/ts';
import { StatedRefusalError, } from '@monochromatic-dev/translation-repair-glossary/ts';
import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import {
  type CommandLineOf,
  type FlagValue,
  idListFlag,
  wholeNumberFlag,
} from '@monochromatic-dev/translation-repair-run/ts';

//region Judge fidelity arguments
// What the fidelity probe is asked on its command line, kept beside the probe
// so the probe itself stays under the file-length cap.

/**
 How many trials one invocation runs by default.

 COUNTED IN ATTEMPTS, not in successes, so a failing roster cannot spend
 without bound while the count a reader checks stays small.
 */
export const DEFAULT_TRIAL_CAP = 16;

/**
 Defects built for every pair when the caller names none: every defect
 `FIDELITY_DAMAGE_KINDS` lists, in its order.

 @example
 ```ts
 const [first,] = DAMAGE_KINDS; // 'deletion'
 ```
 */
export const DAMAGE_KINDS: readonly FidelityDamageKind[] = FIDELITY_DAMAGE_KINDS;

/**
 Defects each `--damage` spelling asks for; every defect when the flag is
 not written.

 EVERY DEFECT BY DEFAULT, because one fixture alone leaves a habit
 unmeasured: the deletion cannot separate reading from preferring length, and
 the insertion alone would not say the roster sees an omission at all. An
 unlisted spelling is refused, rather than silently running something the
 caller did not ask for; so is `--damage` written last, which read as no
 spelling and ran every defect (ledger B73).

 A MAP RATHER THAN A PLAIN OBJECT, since the key is typed text: looked up on
 an object, `--damage constructor` found `Object.prototype.constructor` and
 handed a function on as the defects to build (ledger B75).

 BUILT FROM `DAMAGE_KINDS`, each defect asking for itself alone, so a defect
 the probe gains is a `--damage` spelling the moment it is listed.
 */
const DAMAGE_BY_NAME: ReadonlyMap<string, readonly FidelityDamageKind[]> = new Map(
  DAMAGE_KINDS.map(function entryOf(kind,): readonly [
    string,
    readonly FidelityDamageKind[],
  ] {
    return [
      kind,
      [kind,],
    ];
  },),
);

/**
 Spells the defects `--damage` takes the way its refusal names them, from the
 same list the map is built from.

 @returns The defects in list order, the last one joined by "or"

 @example
 ```ts
 const spelled = damageSpellings(); // 'deletion, insertion or alteration'
 ```
 */
function damageSpellings(): string {
  return `${DAMAGE_KINDS.slice(
    0,
    -1,
  )
    .join(', ',)} or ${nonNullishOrThrow(DAMAGE_KINDS.at(-1,),)}`;
}

/**
 Reads which defects `--damage` asks for.

 @param damage - what the flag carried, or that nobody wrote it

 @returns Every defect when the flag was not written, else the one named

 @throws StatedRefusalError when the flag names a defect this probe does not
 build

 @example
 ```ts
 const kinds = damageKindsOf({ damage: { kind: 'written', flag: '--damage', value: 'insertion', }, },);
 ```
 */
function damageKindsOf({ damage, }: { readonly damage: FlagValue; },): readonly FidelityDamageKind[] {
  if (damage.kind === 'unwritten')
    return DAMAGE_KINDS;
  /**
   Defects that spelling asks for, absent when it names none this probe builds.
   */
  const named = DAMAGE_BY_NAME.get(damage.value,);
  if (named === undefined)
    throw new StatedRefusalError({
      says: `--damage takes ${damageSpellings()}, not ${JSON.stringify(damage.value,)}`,
    },);
  return named;
}

/**
 Reads `--only`, `--cap`, `--damage` and `--context` from the command line.

 @internal

 @param line - the probe's command line, read whole by `reportingRefusals`
 and passed in so this is testable without a subprocess

 @returns Entry ids to trial, empty for every entry, the trial cap, and which
 defects to build

 @throws StatedRefusalError when a cap is not a whole number written in
 digits or is below zero, an entry filter names no entry, or a defect is one
 this probe does not build

 @example
 ```ts
 const { onlyIds, cap, damageKinds, } = readFidelityArguments({ line, },);
 ```
 */
export function readFidelityArguments(
  { line, }: { readonly line: CommandLineOf<'judge-fidelity-probe'>; },
): {
  readonly onlyIds: readonly string[];
  readonly cap: number;
  readonly damageKinds: readonly FidelityDamageKind[];
  readonly withContext: boolean;
} {
  /**
   Defects to build, read before the other flags so a bad one is refused
   first.
   */
  const damageKinds = damageKindsOf({ damage: line.flag('damage',), },);
  return {
    // Whether the sheet also carries the neighbouring sections' original,
    // which is the one thing that differs between a narrow run and a wide one.
    withContext: line.switched('context',),
    damageKinds,
    onlyIds: idListFlag({
      asked: line.flag('only',),
      naming: 'entry id',
    },),
    cap: wholeNumberFlag({
      asked: line.flag('cap',),
      unwritten: DEFAULT_TRIAL_CAP,
      leaveOffTo: `run the default of ${String(DEFAULT_TRIAL_CAP,)} ${
        wordForCount({
          count: DEFAULT_TRIAL_CAP,
          one: 'trial',
          many: 'trials',
        },)
      }`,
    },),
  };
}

//endregion Judge fidelity arguments
