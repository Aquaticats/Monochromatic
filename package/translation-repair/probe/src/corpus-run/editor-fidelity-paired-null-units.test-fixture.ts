import {
  EDITOR_DAMAGE_KINDS,
  EDITOR_FIDELITY_POOL,
} from '../../dist/final/node/index.mjs';

//region Editor fidelity bench units
// THE ASKS OF A SIMULATED BENCH: the committed pool's trials in the order every arm walks them, and the asks of a bench that
// walks them pass by pass (every trial once, then every trial again), so that the leading asks of a bench the money stopped
// are the first pass whole and then the start of the second.

/**
 One trial of the pool in its order: the entry, the trial's handle and the kind of defect.

 @example
 ```ts
 const trial: PoolTrial = { entryId: 'tabby', trialId: 'tabby-s1/insertion', kind: 'insertion', };
 ```
 */
export type PoolTrial = {
  /**
   The entry the trial comes from.
   */
  readonly entryId: string;

  /**
   The trial's handle, the same in every pass.
   */
  readonly trialId: string;

  /**
   The kind of defect.
   */
  readonly kind: string;
};

/**
 One ask of a bench: a trial in one pass over the pool. Pass zero is the first time the trial is asked.

 @example
 ```ts
 const unit: BenchUnit = { entryId: 'tabby', trialId: 'tabby-s1/insertion', kind: 'insertion', pass: 0, };
 ```
 */
export type BenchUnit = PoolTrial & {
  /**
   Which pass over the pool this ask belongs to, from zero.
   */
  readonly pass: number;
};

/**
 The committed pool's trials in the order every arm walks them.

 @returns Trials, reference's damages in the order the specification lists them

 @example
 ```ts
 const order = poolTrialOrder();
 ```
 */
export function poolTrialOrder(): readonly PoolTrial[] {
  return EDITOR_FIDELITY_POOL.flatMap(function trialsOf(spec,): readonly PoolTrial[] {
    return spec.damages
      .filter(function builds(damage,): boolean {
        return EDITOR_DAMAGE_KINDS.some(function is(kind,): boolean {
          return kind === damage.kind;
        },);
      },)
      .map(function trialOf(damage,): PoolTrial {
        return {
          entryId: spec.entryId,
          trialId: `${spec.id}/${damage.kind}`,
          kind: damage.kind,
        };
      },);
  },);
}

/**
 The asks of a bench that walks trials pass by pass: every trial once, then every trial again, and so on, up to a number of
 asks.

 @param trials - trials of one pass, in order

 @param passes - how many passes the bench walks at most

 @param units - how many asks the bench makes, counted from the first

 @returns Leading asks, pass zero first

 @example
 ```ts
 const units = unitsOf({ trials: poolTrialOrder(), passes: 2, units: 120, },);
 ```
 */
export function unitsOf(
  {
    trials,
    passes,
    units,
  }: {
    readonly trials: readonly PoolTrial[];
    readonly passes: number;
    readonly units: number;
  },
): readonly BenchUnit[] {
  return Array.from(
    { length: passes, },
    function passAt(
      _unused,
      pass,
    ): readonly BenchUnit[] {
      return trials.map(function unitOf(trial,): BenchUnit {
        return {
          ...trial,
          pass,
        };
      },);
    },
  )
    .flat()
    .slice(
    0,
    units,
    );
}

//endregion Editor fidelity bench units
