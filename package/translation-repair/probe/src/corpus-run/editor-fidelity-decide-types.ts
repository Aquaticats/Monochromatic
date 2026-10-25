import type { PairedInterval, } from './editor-fidelity-paired.ts';

//region Editor fidelity decision types
// THE SHAPES THE DECISION RULE TAKES AND GIVES: the trials every arm shared, how one effort stands against the default and
// what the rule decided. Apart from the rule so that the rule, the lines it prints and these shapes each stay small.

/**
 One unit every arm shared, a trial in one pass over the pool: the entry it comes from, its kind of defect and whether
 each arm fixed it cleanly. A trial asked again in a later pass is a unit of its own in the same entry.

 @example
 ```ts
 const trial: SharedTrial = { entryId: 'tabby', kind: 'insertion', high: true, low: false, mediums: [false,], };
 ```
 */
export type SharedTrial = {
  /**
   The entry the trial comes from, the unit the test permutes within.
   */
  readonly entryId: string;

  /**
   The kind of defect.
   */
  readonly kind: 'insertion' | 'alteration';

  /**
   Whether the high effort fixed it cleanly, an ask that ended at the output-token cap counting as not.
   */
  readonly high: boolean;

  /**
   Whether the low effort fixed it cleanly.
   */
  readonly low: boolean;

  /**
   Whether each medium arm fixed it cleanly, one entry for each medium arm run.
   */
  readonly mediums: readonly boolean[];
};

/**
 The paired estimate for one kind of defect, a secondary figure.

 @example
 ```ts
 const kind: KindEstimate = { estimate: 0.1, trials: 40, };
 ```
 */
export type KindEstimate = {
  /**
   The paired difference per trial of the kind, as a fraction.
   */
  readonly estimate: number;

  /**
   Shared trials of the kind.
   */
  readonly trials: number;
};

/**
 How one effort stands against the default.

 @example
 ```ts
 const reading = decideEffort({ trials, }).high;
 ```
 */
export type EffortReading = {
  /**
   The effort read.
   */
  readonly effort: 'low' | 'high';

  /**
   The paired difference per shared trial as a fraction: the effort's fixes minus the mean of the medium arms'.
   */
  readonly estimate: number;

  /**
   The interval of the estimate, by the seeded cluster bootstrap over entries.
   */
  readonly interval: PairedInterval;

  /**
   The one-sided exact p-value.
   */
  readonly p: number;

  /**
   Whether the exact test is at or under the level, which is what shows the effort better.
   */
  readonly shown: boolean;

  /**
   The estimate and the trials for each kind of defect.
   */
  readonly kinds: {
    /**
     The insertion trials.
     */
    readonly insertion: KindEstimate;

    /**
     The alteration trials.
     */
    readonly alteration: KindEstimate;
  };
};

/**
 Which clause of the rule decided.
 */
export type DecidingClause = 'none-shown' | 'shown-better' | 'larger-estimate' | 'cheaper-on-equal';

/**
 What the rule decided over the shared trials.

 @example
 ```ts
 const verdict = decideEffort({ trials, },);
 ```
 */
export type EffortVerdict = {
  /**
   The effort the rule picks.
   */
  readonly verdict: 'low' | 'medium' | 'high';

  /**
   The clause that decided it.
   */
  readonly clause: DecidingClause;

  /**
   Shared trials the verdict is over.
   */
  readonly trials: number;

  /**
   Entries those trials come from.
   */
  readonly entries: number;

  /**
   How many medium arms were run.
   */
  readonly mediumArms: number;

  /**
   How the low effort stands.
   */
  readonly low: EffortReading;

  /**
   How the high effort stands.
   */
  readonly high: EffortReading;

  /**
   The lines the read step prints: a paired line and a kinds line for each effort, then the verdict.
   */
  readonly lines: readonly string[];

  /**
   The verdict line.
   */
  readonly said: string;
};

//endregion Editor fidelity decision types
