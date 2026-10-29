import type { Logger, } from '@monochromatic-dev/logger-core/ts';

import type { ModelClient, } from '@monochromatic-dev/translation-repair-provider/ts';
import type {
  CarryBlock,
  CarryEdgeWords,
  ConsistencyPlan,
  ConsistencyRendering,
  SpliceOnlyKind,
} from '@monochromatic-dev/translation-repair-consistency/ts';
import type { DeclaredNamePair, } from '@monochromatic-dev/translation-repair-house-rule/ts';
import type { SliceCache, } from '@monochromatic-dev/translation-repair-translate-floor/ts';
import type { StageSettings, } from '@monochromatic-dev/translation-repair-stage/ts';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import type {
  ArtifactConsistencyCarry,
  ArtifactConsistencyRepeat,
  ArtifactConsistencyUnplaced,
  ArtifactConsistencyVerdict,
  ConsistencyOccurrenceOutcome,
} from '@monochromatic-dev/translation-repair-artifact/ts';
import type { ConsistencyCacheValue, } from './pass-consistency-cache.ts';
import type { EdgeVerdict, } from './pass-consistency-edge-refusal.ts';
import type { ConsistencyPlace, } from './pass-consistency-group.ts';

//region Pass consistency types
// The shapes the pass hands between settling a group's ballots and carrying the
// chosen rendering to the places that receive it, kept apart from both so
// neither imports the other for a type.

/**
 One replacement a group would make, before the floors have spoken.

 @example
 ```ts
 const edit: PendingEdit = { kind: 'splice', placePositions: [1,], sliceIndex: 3, start: 0, end: 12, text: 'the cat naps', };
 ```
 */
export type PendingEdit = {
  /**
   What the replacement is: the words of one code splice, or a writer's whole
   block, which can change words of other places that stand in the block.
   */
  readonly kind: 'splice' | 'block';

  /**
   Positions, among the group's places, of the receiving places this replacement
   carries the chosen rendering to: one for the words of a code splice, every
   receiving place of the block for a writer's block, which is rewritten once.
   */
  readonly placePositions: readonly number[];

  /**
   Slice the replacement is made in.
   */
  readonly sliceIndex: number;

  /**
   Offset in the slice's English where the replaced text starts: the words of a
   splice, or a writer's whole block.
   */
  readonly start: number;

  /**
   Exclusive offset where it ends.
   */
  readonly end: number;

  /**
   The text that goes in.
   */
  readonly text: string;
};

/**
 A place with its located rendering.

 @example
 ```ts
 const located: LocatedPlace = { place, rendering, };
 ```
 */
export type LocatedPlace = {
  /**
   The place a slice holds whole.
   */
  readonly place: ConsistencyPlace;

  /**
   Where its rendering stands in its English.
   */
  readonly rendering: Extract<ConsistencyRendering, { readonly kind: 'located'; }>;
};

/**
 A place the chosen rendering is to reach.

 @example
 ```ts
 const receiver: CarryReceiver = { position: 1, label: 'OCCURRENCE 2', place, rendering, plan: { kind: 'unavailable', }, };
 ```
 */
export type CarryReceiver = LocatedPlace & {
  /**
   Position of the place among the group's places.
   */
  readonly position: number;

  /**
   The label the sheet gave the place.
   */
  readonly label: string;

  /**
   What moving the chosen words in by code amounts to.
   */
  readonly plan: ConsistencyPlan;
};

/**
 Why a unit that stands in no kind of block cannot be given to a writer.

 - `'no-single-block'`: the words of its receiving places lie in no single block
   (they cross a paragraph boundary, or lie in a container no paragraph holds), so
   the unit is the whole English of the slice.
 - `'overlaps-outside-receiver'`: its block is a paragraph or heading that overlaps
   the located words of another receiving place of the group outside the unit
   whose plan is a replacement, which that place's splice replaces. A receiving
   place whose plan is unavailable has no splice and does not count, and a place
   whose words cross a block boundary has that plan when the chosen words hold a
   different number of block breaks than its own words do.

 @example
 ```ts
 const reason: UnwritableReason = 'no-single-block';
 ```
 */
export type UnwritableReason = 'no-single-block' | 'overlaps-outside-receiver';

/**
 The receiving places that one rewrite of one stretch of the English carries the
 chosen rendering to. Places are grouped by the stretch that holds them: the
 innermost block, a paragraph or a heading, that holds the words of each, or the
 whole English of the slice for words that lie in no single block. The writers
 rewrite the block once, with every receiving place in it, and the bench chooses
 once among the current block, the code splice of every place in it and each
 writer's block.

 @example
 ```ts
 const unit: CarryUnit = { block, writable: true, spliceOnlyIn: [], receivers: [receiver,], keeping: [], crossing: [], };
 ```
 */
export type CarryUnit = {
  /**
   The stretch of the slice's English the candidates replace.
   */
  readonly block: CarryBlock;

  /**
   Whether the block is a paragraph or a heading a writer can be given: false for
   the whole English of a slice, which holds words that lie in no single block,
   and which only the code splice and the current English stand for, and false
   for a paragraph whose rewrite would address words that another receiving place
   of the group has outside it.
   */
  readonly writable: boolean;

  /**
   Why the unit is not writable, where no kind of block in `spliceOnlyIn` says so:
   present for a unit that is not writable and stands in no such kind, absent for a
   writable unit and for a unit that stands in a kind of block.
   */
  readonly unwritableBecause?: UnwritableReason;

  /**
   The kind of block (a table, a code block, a setext heading, a preformatted
   block) that holds the words of the receiving places where no paragraph
   rewrite can carry words to them, as a list of zero or one; none for a
   paragraph, a heading and words in no single block.
   */
  readonly spliceOnlyIn: readonly SpliceOnlyKind[];

  /**
   The receiving places in the block, in page order, at least one.
   */
  readonly receivers: readonly CarryReceiver[];

  /**
   The English words of other places inside the block that must come out of any
   rewrite as they went in: the chosen place's words when it stands in the
   block, the words of a place the majority keeps for its context or that
   already reads as the chosen place does, and the located words of every place
   of every other group of the page, whatever became of that place.
   */
  readonly keeping: readonly string[];

  /**
   The stretches of the words of places of the group outside the unit whose words
   must stay as they are and that run over an edge of the block, each with the edge
   of the block it stands at: the chosen place, a place the majority keeps for its
   context, a place that already reads as the chosen place does, and a receiving
   place that has no splice (its plan is `unavailable`). A writer's block that does
   not hold the stretch at that edge would leave the place's words changed, although
   the record would have the place as kept as it was. A receiving place whose plan is
   `replace` and the places of other groups are not listed. None where no such place
   reaches into the block.
   */
  readonly crossing: readonly CarryEdgeWords[];
};

/**
 How the carry to the receiving places of one block ended.

 @example
 ```ts
 const carried: UnitCarried = { carries: [carry,], edits: [], findings: [], };
 ```
 */
export type UnitCarried = {
  /**
   The record of the carry, candidates, ballots and decision, once for each
   receiving place of the block in the order of the unit's receivers: the places
   of a block share one choice, so each label carries the same record.
   */
  readonly carries: readonly ArtifactConsistencyCarry[];

  /**
   The replacements the winning candidate makes, none when the English stands.
   */
  readonly edits: readonly PendingEdit[];

  /**
   What the carry reported, for the section's findings.
   */
  readonly findings: readonly string[];
};

/**
 Who is asked to write the receiving places' blocks.

 @example
 ```ts
 const writers: CarryWriters = { kind: 'not-asked', why: 'translators 0 of 8 reachable, floor 2', };
 ```
 */
export type CarryWriters =
  | {
    /**
     The translate stage's writer bench is asked.
     */
    readonly kind: 'asked';

    /**
     The writer bench, as the seat reading left it, in roster order.
     */
    readonly modelIds: readonly RosterModelId[];
  }
    | {
    /**
     The writers are not asked: the carry goes on with the current English and
     the code splice.
     */
      readonly kind: 'not-asked';

    /**
     Why, naming the bench and what the seat reading found of it.
     */
      readonly why: string;
    };

/**
 What a carry reads beside the two places.

 @example
 ```ts
 const context: CarryContext = { client, writers, judgeModelIds, cache, text, declared: [], declaredNames: [], signal, l, stageSettings, };
 ```
 */
export type CarryContext = {
  /**
   Provider client.
   */
  readonly client: ModelClient;

  /**
   Whether the translate stage's writer bench is asked, and which seats.
   */
  readonly writers: CarryWriters;

  /**
   The judge bench that chooses among the candidates.
   */
  readonly judgeModelIds: readonly RosterModelId[];

  /**
   Store an earlier answer is read from and a settled one kept in.
   */
  readonly cache: SliceCache<ConsistencyCacheValue>;

  /**
   Whole original.
   */
  readonly text: string;

  /**
   Declared identity lines of the page, when it declares any.
   */
  readonly identityContext?: string;

  /**
   Lines of the pages the original cites, when it cites any.
   */
  readonly referenceContext?: string;

  /**
   Whether the entry records a death.
   */
  readonly lifeEnded?: boolean;

  /**
   Names the translation declares for its subject.
   */
  readonly subjectNames?: readonly string[];

  /**
   Name pairs the front matter declares.
   */
  readonly declared: readonly DeclaredNamePair[];

  /**
   Declared name forms.
   */
  readonly declaredNames: readonly string[];

  /**
   The declared forms among them that name the subject, which the floors read
   as standing in for one another; absent when preparation named none.
   */
  readonly declaredSubjectForms?: readonly string[];

  /**
   Entry deadline and caller abort.
   */
  readonly signal: AbortSignal;

  /**
   Logger the stage writes to.
   */
  readonly l: Logger;

  /**
   Straggler windows the entry file read for this launch.
   */
  readonly stageSettings: StageSettings;
};

/**
 A group with its verdict, its places' renderings and outcomes, and its pending replacements.

 @example
 ```ts
 const settled: SettledGroup = { repeat, places, unplaced, verdict, renderings: [], outcomes: [], edits: [], carries: [], findings: [], };
 ```
 */
export type SettledGroup = {
  /**
   The repeated wording or frame.
   */
  readonly repeat: ArtifactConsistencyRepeat;

  /**
   The places a slice holds whole, in page order.
   */
  readonly places: readonly ConsistencyPlace[];

  /**
   The places no slice holds whole.
   */
  readonly unplaced: readonly ArtifactConsistencyUnplaced[];

  /**
   What was asked of the bench and what it answered.
   */
  readonly verdict: ArtifactConsistencyVerdict;

  /**
   Each place's rendering, in the order of the places.
   */
  readonly renderings: readonly ConsistencyRendering[];

  /**
   Each place's outcome so far, in the order of the places; `replaced` is
   provisional until the floors have spoken.
   */
  readonly outcomes: readonly ConsistencyOccurrenceOutcome[];

  /**
   For each place, in the order of the places, why the planner refused the splice at the edge of the span on the English
   around it, where the recorded words and marks alone let code move the chosen words in; `unrefused` for a place not so
   refused. Absent where no place was.
   */
  readonly leftBecause?: readonly EdgeVerdict[];

  /**
   The replacements the winner would make.
   */
  readonly edits: readonly PendingEdit[];

  /**
   How the chosen rendering was carried to each receiving place, in the order
   of the places.
   */
  readonly carries: readonly ArtifactConsistencyCarry[];

  /**
   What the gathers reported.
   */
  readonly findings: readonly string[];
};

/**
 A group whose ballots are counted and whose receiving places are still to be
 carried to.

 @example
 ```ts
 const decided: DecidedGroup = { ...settled, chosen: [], receivers: [], };
 ```
 */
export type DecidedGroup = SettledGroup & {
  /**
   The chosen place, as a list of zero or one.
   */
  readonly chosen: readonly LocatedPlace[];

  /**
   The places the chosen rendering is to reach, in the order of the places.
   */
  readonly receivers: readonly CarryReceiver[];
};

//endregion Pass consistency types
