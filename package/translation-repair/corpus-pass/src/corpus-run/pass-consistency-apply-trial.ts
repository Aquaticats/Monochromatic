import {
  type ConsistencyEdit,
  ConsistencyInvariantError,
} from '@monochromatic-dev/translation-repair-consistency/ts';
import {
  madeIn,
  type MadeSlice,
  unprovenOnPage,
} from './pass-consistency-apply-gains.ts';
import {
  type FloorsAsked,
  floorsQuestionOf,
  type GainReason,
  type SlicePlan,
} from './pass-consistency-apply-settled.ts';
import {
  gainSliceOf,
  idsOf,
  pageFrameOf,
  replacedAt,
  type ReadmitLedger,
} from './pass-consistency-apply-ledger.ts';
import type { ConsistencyFloorVerdict, } from './pass-consistency-floors.ts';

//region Pass consistency apply trial
// ONE SPLICE PUT BACK ON THE PAGE AS IT STANDS (`pass-consistency-apply-readmit.ts`). A TRIAL is that question asked of one
// splice the check left out; a SCREEN (`pass-consistency-apply-screen.ts`) spares the trials of the splices no page could carry.
// The splice is added to
// the replacements its slice makes, and the page is read with it made: the replacements must all still be made
// (the splice's words overlap none), the floors must pass the slice, the splice's own apostrophe must be proven
// there, and the apostrophes the splices that stand carry in must be proven still. The first of these that
// fails is the reason the splice stays out, and each is a reason that holds on the page as it stands. THE
// FLOORS' REFUSAL COMES BEFORE THE READINGS OF THE MARKS because it holds whatever the other splices do: a
// slice they refuse with the splice made is refused on every page, while a mark that could pair with the
// apostrophe is a fact of this page alone. Overlap comes first because no slice is made with a splice that
// overlaps one that stands, so the floors have no slice to judge.
//
// THE LAST REASON, THAT THE SPLICE WOULD UNPROVE AN APOSTROPHE THAT STANDS, is reached through spans that touch: the splice
// changes the character that stands directly beside the mark of a splice that stands (a comma becomes a letter, which makes
// that mark attached and no longer trailing). Two splices whose spans touch do not overlap (`applyConsistencyEdits`,
// `consistency-splice.ts`), and the test `pass-consistency-apply-hand-built.unit.test.ts` builds such a page by hand.
// Fuzzing pages whose marks and connectors vary, with the spans apart, never reached it.

/**
 What a trial of one splice came to.
 */
export type Trial = {
  /**
   Put back.
   */
  readonly kind: 'admit';

  /**
   The replacements the slice makes with the splice, in priority order.
   */
  readonly edits: readonly ConsistencyEdit[];

  /**
   The slice with them made.
   */
  readonly made: MadeSlice;

  /**
   What the floors said of the slice with them made.
   */
  readonly verdict: ConsistencyFloorVerdict;
} | {
  /**
   Left out.
   */
  readonly kind: 'out';

  /**
   The reason that holds with the splice made.
   */
  readonly reason: GainReason;
};

/**
 Whether two sets hold the same names.

 @param left - one set

 @param right - the other

 @returns Whether every name of one is in the other

 @example
 ```ts
 const same = holdSameIds({ left: new Set(['a',],), right: new Set(['a',],), },); // true
 ```
 */
function holdSameIds(
  {
    left,
    right,
  }: {
    readonly left: ReadonlySet<string>;
    readonly right: ReadonlySet<string>;
  },
): boolean {
  /**
   The names of the first set.
   */
  const members = Array.from(left,);
  return (left.size === right.size) && members.every(function isInRight(id,): boolean {
    return right.has(id,);
  },);
}

/**
 A slice with a splice added to the replacements it makes now.
 */
export type WithSplice = {
  /**
   The replacements the slice makes with the splice, in priority order.
   */
  readonly edits: readonly ConsistencyEdit[];

  /**
   The slice with them made.
   */
  readonly made: MadeSlice;

  /**
   Whether the splice's words overlap a replacement the slice makes, so that the page cannot hold the splice and every
   replacement that stood.
   */
  readonly overlaps: boolean;

  /**
   The slice's plan, which the floors are asked about.
   */
  readonly plan: SlicePlan;
};

/**
 Adds a splice to the replacements its slice makes now and makes them.

 @param ledger - what the step knows of the page

 @param position - the slice's place in the page

 @param edit - the splice

 @returns The replacements with the splice, the slice with them made, and whether the splice's words overlap one that stood

 @throws {@link ConsistencyInvariantError} when the slice has no plan, since only a slice with a plan leaves a splice out

 @example
 ```ts
 const { edits, made, overlaps, } = madeWithSplice({ ledger, position: 1, edit, },);
 ```
 */
export function madeWithSplice(
  {
    ledger,
    position,
    edit,
  }: {
    readonly ledger: ReadmitLedger;
    readonly position: number;
    readonly edit: ConsistencyEdit;
  },
): WithSplice {
  return madeWithSet({
    ledger,
    position,
    set: [edit,],
  },);
}

/**
 Adds some splices to the replacements their slice makes now and makes them.

 @param ledger - what the step knows of the page

 @param position - the slice's place in the page

 @param set - the splices, left out of the slice now

 @returns The replacements with the splices, the slice with them made, and whether the words of a splice overlap one that stood or
 one of the others, so that the page cannot hold them all

 @throws {@link ConsistencyInvariantError} when the slice has no plan, since only a slice with a plan leaves a splice out

 @example
 ```ts
 const { edits, made, overlaps, } = madeWithSet({ ledger, position: 1, set: [first, second,], },);
 ```
 */
export function madeWithSet(
  {
    ledger,
    position,
    set,
  }: {
    readonly ledger: ReadmitLedger;
    readonly position: number;
    readonly set: readonly ConsistencyEdit[];
  },
): WithSplice {
  /**
   What the step knows of the page.
   */
  const {
    worked,
    plans,
    made: ledgerMade,
    toMake,
  } = ledger;
  /**
   The slice.
   */
  const slice = worked[position];
  /**
   The slice's plan.
   */
  const plan = plans.get(position,);
  if ((slice === undefined) || (plan === undefined)) {
    throw new ConsistencyInvariantError({ invariant:
      `slice ${String(position,)} of the page left out a splice and has no plan, and only a slice with a plan leaves one out`, },);
  }
  /**
   The names of the replacements the slice makes now.
   */
  const ledgerIds = idsOf({ edits: toMake[position] ?? [], },);
  /**
   The names of the splices added.
   */
  const setIds = idsOf({ edits: set, },);
  /**
   The slice's replacements that were not left out for another group's words.
   */
  const { candidates, } = slice;
  /**
   The replacements the slice makes with the splices, in priority order.
   */
  const edits = candidates.filter(function isStanding({ id, },): boolean {
    return setIds.has(id,) || ledgerIds.has(id,);
  },);
  /**
   The slice with them made.
   */
  const made = madeIn({
    slice: gainSliceOf({ worked: slice, },),
    remaining: edits,
  },);
  /**
   The slice as it stood made before the splice.
   */
  const stood = ledgerMade[position];
  /**
   The replacements that stood made before, which must all still be made, with the splice and no other.
   */
  const before = (stood === undefined) ? [] : stood.edits;
  /**
   The names the slice must make with the splices.
   */
  const expected = new Set([
    ...idsOf({ edits: before, },),
    ...setIds,
  ],);
  return {
    edits,
    made,
    overlaps: !holdSameIds({
      left: expected,
      right: idsOf({ edits: made.edits, },),
    },),
    plan,
  };
}

/**
 What the floors say of a slice with a splice added to the replacements it makes, asked once of the same replacements.

 @param ledger - what the step knows of the page

 @param position - the slice's place in the page

 @param edit - the splice

 @param asked - what the floors said of each set of replacements, mutated by this function

 @param floorsOn - runs the floors on a slice with its replacements made

 @returns The floors' verdict on the slice with the splice made, none when the splice's words overlap a replacement that
 stood, since the page then cannot hold the splice and no slice is made with it

 @throws {@link ConsistencyInvariantError} when the slice has no plan, since only a slice with a plan leaves a splice out

 @example
 ```ts
 const [verdict,] = floorsWithSplice({ ledger, position: 1, edit, asked, floorsOn, },);
 ```
 */
export function floorsWithSplice(
  {
    ledger,
    position,
    edit,
    asked,
    floorsOn,
  }: {
    readonly ledger: ReadmitLedger;
    readonly position: number;
    readonly edit: ConsistencyEdit;
    readonly asked: Map<string, ConsistencyFloorVerdict>;
    readonly floorsOn: (arguments_: FloorsAsked,) => ConsistencyFloorVerdict;
  },
): readonly ConsistencyFloorVerdict[] {
  return floorsWithSet({
    ledger,
    position,
    set: [edit,],
    asked,
    floorsOn,
  },);
}

/**
 What the floors say of a slice with some splices added to the replacements it makes, asked once of the same replacements.

 @param ledger - what the step knows of the page

 @param position - the slice's place in the page

 @param set - the splices

 @param asked - what the floors said of each set of replacements, mutated by this function

 @param floorsOn - runs the floors on a slice with its replacements made

 @returns The floors' verdict on the slice with the splices made, none when the words of a splice overlap a replacement that
 stood or one of the others, since the page then cannot hold them and no slice is made with them

 @throws {@link ConsistencyInvariantError} when the slice has no plan, since only a slice with a plan leaves a splice out

 @example
 ```ts
 const [verdict,] = floorsWithSet({ ledger, position: 1, set: [first, second,], asked, floorsOn, },);
 ```
 */
export function floorsWithSet(
  {
    ledger,
    position,
    set,
    asked,
    floorsOn,
  }: {
    readonly ledger: ReadmitLedger;
    readonly position: number;
    readonly set: readonly ConsistencyEdit[];
    readonly asked: Map<string, ConsistencyFloorVerdict>;
    readonly floorsOn: (arguments_: FloorsAsked,) => ConsistencyFloorVerdict;
  },
): readonly ConsistencyFloorVerdict[] {
  /**
   The slice with the splices added.
   */
  const withSplice = madeWithSet({
    ledger,
    position,
    set,
  },);
  if (withSplice.overlaps)
    return [];
  /**
   The slice with the splice made, and its plan.
   */
  const {
    made,
    plan,
  } = withSplice;
  /**
   The facts of the slice.
   */
  const { facts, } = plan;
  /**
   The question the floors are asked about the slice with these replacements.
   */
  const question = floorsQuestionOf({
    sliceIndex: facts.sliceIndex,
    edits: withSplice.edits,
  },);
  /**
   What the floors said of the slice with these replacements, asked once.
   */
  const verdict = asked.get(question,) ?? floorsOn({
    plan: {
      ...plan,
      toMake: withSplice.edits,
    },
    candidateText: made.text,
  },);
  asked.set(
    question,
    verdict,
  );
  return [verdict,];
}

/**
 Tries one splice on the page as it stands.

 @param ledger - what the step knows of the page

 @param position - the slice's place in the page

 @param edit - the splice

 @param asked - what the floors said of each set of replacements, mutated by this function

 @param floorsOn - runs the floors on a slice with its replacements made

 @returns Put back, with what the slice then comes to, or left out with the reason that holds with it made: the splice's
 words overlap a replacement that stands (the slice cannot be made with it), else the floors refuse the slice with it, which
 holds whatever the other splices do, else its own apostrophe is unproven, else it would unprove the apostrophe of a splice
 that stands

 @throws {@link ConsistencyInvariantError} when the slice has no plan, since only a slice with a plan leaves a splice out

 @example
 ```ts
 const trial = trialOf({ ledger, position: 1, edit, asked, floorsOn, },);
 ```
 */
export function trialOf(
  {
    ledger,
    position,
    edit,
    asked,
    floorsOn,
  }: {
    readonly ledger: ReadmitLedger;
    readonly position: number;
    readonly edit: ConsistencyEdit;
    readonly asked: Map<string, ConsistencyFloorVerdict>;
    readonly floorsOn: (arguments_: FloorsAsked,) => ConsistencyFloorVerdict;
  },
): Trial {
  /**
   What the step knows of the page.
   */
  const {
    gained,
    made: ledgerMade,
  } = ledger;
  /**
   The slice with the splice added.
   */
  const {
    edits,
    made,
    overlaps,
  } = madeWithSplice({
    ledger,
    position,
    edit,
  },);
  if (overlaps) {
    return {
      kind: 'out',
      reason: { kind: 'overlaps', },
    };
  }
  /**
   What the floors say of the slice with the splice made, which holds whatever the other splices do.
   */
  const [verdict,] = floorsWithSplice({
    ledger,
    position,
    edit,
    asked,
    floorsOn,
  },);
  if (verdict === undefined) {
    throw new ConsistencyInvariantError({ invariant:
      `slice ${String(position,)} was made with a splice that overlaps nothing and the floors gave no verdict on it`, },);
  }
  if (verdict.kind === 'refused') {
    return {
      kind: 'out',
      reason: {
        kind: 'floors',
        findings: verdict.findings,
      },
    };
  }
  /**
   The splices whose mark is unproven on the page with the splice made.
   */
  const unproven = unprovenOnPage({
    made: replacedAt({
      list: ledgerMade,
      position,
      entry: made,
    },),
    gained,
    ...pageFrameOf({ ledger, },),
  },);
  /**
   The ones of this slice.
   */
  const unprovenHere = unproven[position] ?? [];
  /**
   Whether the splice's own mark is among them.
   */
  const isOwnMarkUnproven = unprovenHere.some(function isTheSplice({ id, },): boolean {
    return id === edit.id;
  },);
  if (isOwnMarkUnproven) {
    return {
      kind: 'out',
      reason: { kind: 'pairs', },
    };
  }
  /**
   Whether the splice would unprove the mark of a splice that stands.
   */
  const unprovesOthers = unproven.some(function holdsSome(unprovenEdits,): boolean {
    return unprovenEdits.length > 0;
  },);
  if (unprovesOthers) {
    return {
      kind: 'out',
      reason: { kind: 'unproves', },
    };
  }
  return {
    kind: 'admit',
    edits,
    made,
    verdict,
  };
}

//endregion Pass consistency apply trial
