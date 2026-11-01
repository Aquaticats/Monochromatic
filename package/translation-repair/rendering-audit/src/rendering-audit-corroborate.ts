import { textsInCodePointOrder, } from '@monochromatic-dev/translation-repair-markup/ts';
import type {
  ScreenedFinding,
  SideReading,
} from './rendering-audit-screen.ts';
import { RenderingAuditInvariantError, } from './rendering-audit-invariant.ts';
import type { RenderingAuditCategory, } from './rendering-audit-wire.ts';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract/ts';

//region Rendering audit corroboration
// When two auditors are talking about the SAME DEFECT, as against the same
// sentence.
//
// ON FOCUS INTERVALS, not on quoted text. Two voices pointing at one dropped
// negation will not type the same characters: one quotes the clause, the other
// quotes the two words that carry it. What they have in common is a position in
// a document, so that is what this compares.
//
// EXACT INTERVALS CORROBORATE, AND NOTHING ELSE DOES. Overlap is reported and
// never merged, because a merge on overlap is a false claim of agreement: one
// sentence can carry two changed numbers, and two voices each finding a
// different one overlap perfectly while agreeing on nothing. A near miss says
// what it is, which is two claims about neighbouring text whose relationship a
// human or a later rule has to decide.
//
// NO TRANSITIVE CLUSTERING. If A overlaps B and B overlaps C, that does not put
// A and C in one defect: a single wide span would otherwise bridge two narrow
// findings that share no text at all. Every relation here is pairwise, and
// membership comes only from an exact key.
//
// EVERY MEMBER IS KEPT. A defect reports the claims it was built from, one per
// voice per claim, because the aggregate is what a later calibration re-reads,
// and a summary that discarded the voices could not answer a question nobody
// has asked yet.

/**
 How many distinct voices must find one defect for it to count as
 corroborated.

 TWO, not a majority, and the difference matters at this roster size: the
 question this instrument answers first is whether a defect is THERE, and a
 majority rule over six voices would discard a defect four of them missed. The
 tally keeps the count, so a stricter rule can be applied later without
 re-running anything.
 */
export const CORROBORATION_VOICES = 2;

/**
 One voice's claim, kept whole.

 @example
 ```ts
 const member: AuditMemberClaim = { modelId, finding, };
 ```
 */
export type AuditMemberClaim = {
  /**
   Auditor that made this claim.
   */
  readonly modelId: RosterModelId;

  /**
   What it claimed, screened.
   */
  readonly finding: ScreenedFinding;
};

/**
 One defect more than one voice located identically.

 @example
 ```ts
 const defect: CorroboratedDefect = { category: 'altered-polarity', voices: 2, members, };
 ```
 */
export type CorroboratedDefect = {
  /**
   Category every member named, since a differing category is a different
   defect here rather than the same one.
   */
  readonly category: RenderingAuditCategory;

  /**
   Where it is in the original.
   */
  readonly source: SideReading;

  /**
   Where it is in the candidate.
   */
  readonly candidate: SideReading;

  /**
   Distinct auditors that located it.
   */
  readonly voices: number;

  /**
   Every claim it was built from, including a voice claiming it twice.
   */
  readonly members: readonly AuditMemberClaim[];
};

/**
 Two claims that are about neighbouring text and are NOT the same defect.

 @example
 ```ts
 const near: NearMiss = { kind: 'overlapping-focus', left, right, };
 ```
 */
export type NearMiss = {
  /**
   Which way they nearly matched: the same span under different categories, or
   intersecting spans that are not the same span.
   */
  readonly kind: 'same-focus-different-category' | 'overlapping-focus';

  /**
   One claim.
   */
  readonly left: AuditMemberClaim;

  /**
   The other, always from a different voice.
   */
  readonly right: AuditMemberClaim;
};

/**
 One side's focus span, in the form two claims are compared by.

 NAMED ABSENCE rather than a nullish union: a side a category does not use is
 a thing this comparison knows about, and it must never intersect anything,
 which a missing value would leave to whoever remembered to check.

 @example
 ```ts
 const interval: FocusInterval = { kind: 'unused', };
 ```
 */
type FocusInterval = {
  /**
   Category does not rest on this side.
   */
  readonly kind: 'unused';
} | {
  /**
   Category rests on this side, and the focus sits here.
   */
  readonly kind: 'span';

  /**
   Where the focus begins.
   */
  readonly start: number;

  /**
   Where it ends, exclusive.
   */
  readonly end: number;
};

/**
 Reads one side's focus interval.

 @param reading - what one side of a finding rests on

 @returns Interval, or the named absence for a side the category does not use

 @example
 ```ts
 const interval = intervalOf({ reading: finding.source, },);
 ```
 */
function intervalOf(
  { reading, }: { readonly reading: SideReading; },
): FocusInterval {
  if (reading.kind === 'unused')
    return { kind: 'unused', };

  return {
    kind: 'span',
    start: reading.focus
      .start,
    end: reading.focus
      .end,
  };
}

/**
 Key under which two claims are the same defect.

 @param finding - screened finding

 @returns Category and both focus intervals

 @example
 ```ts
 const key = defectKey({ finding, },);
 ```
 */
function defectKey({ finding, }: { readonly finding: ScreenedFinding; },): string {
  return JSON.stringify([
    finding.category,
    intervalOf({ reading: finding.source, },),
    intervalOf({ reading: finding.candidate, },),
  ],);
}

/**
 Whether two intervals share any position.

 @param left - one claim's interval on a side, or the named absence where its
 category does not use that side

 @param right - the other claim's, on the same side

 @returns Whether both are spans and intersect

 @example
 ```ts
 const shared = intersects({ left: { kind: 'span', start: 0, end: 4, }, right: { kind: 'unused', }, },);
 ```
 */
function intersects(
  {
    left,
    right,
  }: {
    readonly left: FocusInterval;
    readonly right: FocusInterval;
  },
): boolean {
  // A SIDE NEITHER CLAIM USES IS NOT A SHARED POSITION. Two omissions both
  // leaving the candidate side unused agree about nothing there, and reading
  // that as an intersection would make every pair of one-sided claims touch.
  // No case can tell this line from its absence (an unused side has no
  // bounds, and comparing missing bounds is false); the type check needs it
  // to reach the bounds at all.
  if ((left.kind === 'unused') || (right.kind === 'unused'))
    return false;

  return (left.start < right.end) && (right.start < left.end);
}

/**
 Whether two findings point at exactly the same spans, whatever they call
 them.

 @param left - one finding

 @param right - the other

 @returns Whether both sides' intervals match exactly

 @example
 ```ts
 const same = sameSpans({ left, right, },);
 ```
 */
function sameSpans(
  {
    left,
    right,
  }: {
    readonly left: ScreenedFinding;
    readonly right: ScreenedFinding;
  },
): boolean {
  return (JSON.stringify(intervalOf({ reading: left.source, },),)
    === JSON.stringify(intervalOf({ reading: right.source, },),))
    && (JSON.stringify(intervalOf({ reading: left.candidate, },),)
      === JSON.stringify(intervalOf({ reading: right.candidate, },),));
}

/**
 Groups every claim by the defect it names.

 @param claims - every voice's claims

 @returns Defects reaching the corroboration threshold, most-agreed first

 @throws {@link RenderingAuditInvariantError} when a defect group holds no
 member, which cannot occur, since every group is opened by the claim it is
 keyed on

 @example
 ```ts
 const corroborated = corroborate({ claims, },);
 ```
 */
export function corroborate(
  { claims, }: { readonly claims: readonly AuditMemberClaim[]; },
): readonly CorroboratedDefect[] {
  /**
   Claims under each defect key, in arrival order.
   */
  const grouped = claims.reduce(
    function collect(
      groups: Map<string, readonly AuditMemberClaim[]>,
      claim,
    ): Map<string, readonly AuditMemberClaim[]> {
      /**
       Key this claim falls under.
       */
      const key = defectKey({ finding: claim.finding, },);
      groups.set(
        key,
        [
          ...groups.get(key,) ?? [],
          claim,
        ],
      );
      return groups;
    },
    new Map<string, readonly AuditMemberClaim[]>(),
  );

  return [...grouped.values(),]
    .map(function toDefect(members,): CorroboratedDefect {
      /**
       First member, which every member agrees with by construction of the
       key.
       */
      const [first,] = members;

      if (first === undefined)
        throw new RenderingAuditInvariantError({
          invariant: 'a defect group with no members cannot occur, since groups are built from claims',
        },);

      return {
        category: first.finding
          .category,
        source: first.finding
          .source,
        candidate: first.finding
          .candidate,
        // DISTINCT VOICES, so a voice claiming one defect twice in one answer
        // is one opinion rather than agreement with itself.
        voices: new Set(members.map(function toVoice(member,): string {
          return member.modelId;
        },),).size,
        members,
      };
    },)
    .filter(function reachedThreshold(defect,): boolean {
      return defect.voices >= CORROBORATION_VOICES;
    },)
    .toSorted(function byAgreement(
      left,
      right,
    ): number {
      return right.voices - left.voices;
    },);
}

/**
 One defect a group of voices agreed on WITHOUT quoting identical spans.

 A SECOND TIER, reported beside the strict count rather than folded into it,
 because the two answer different questions. The strict count asks whether
 voices picked out the same characters; this asks whether they were talking
 about the same thing. Both are worth having and neither should be mistaken
 for the other.

 MEASURED, NOT SUPPOSED: on the instrument's own positive control, three
 auditors independently found one dropped negator, all three called it
 `altered-polarity`, and they quoted `不吃`, `吃` and `不吃罐头`. Under the
 strict count that is zero corroborated defects. Unanimous agreement reported
 as nothing is a worse answer than the false merge the strict count exists to
 prevent, and a run earlier the same evening had two of those three voices
 landing on identical spans, so the strict count also turns on a coin flip.

 @example
 ```ts
 const agreed: OverlapAgreement = { category: 'altered-polarity', voices: 3, members, };
 ```
 */
export type OverlapAgreement = {
  /**
   Category every member named, since voices disagreeing about what KIND of
   defect it is have not agreed about the defect.
   */
  readonly category: RenderingAuditCategory;

  /**
   Distinct auditors in this group.
   */
  readonly voices: number;

  /**
   Every claim in it, so a reader can see the spans that were merged and judge
   the merge.
   */
  readonly members: readonly AuditMemberClaim[];
};

/**
 Whether two claims say the same thing about ONE side.

 A SIDE NEITHER USES is agreement by absence: both claims say their category
 rests on the other side, which is the same statement about where the defect
 is rather than a missing comparison.

 @param mine - one claim's interval on this side

 @param theirs - the other claim's

 @returns Whether they agree about this side

 @example
 ```ts
 const agrees = sideAgrees({ mine, theirs, },);
 ```
 */
function sideAgrees(
  {
    mine,
    theirs,
  }: {
    readonly mine: FocusInterval;
    readonly theirs: FocusInterval;
  },
): boolean {
  if ((mine.kind === 'unused') && (theirs.kind === 'unused'))
    return true;

  return intersects({
    left: mine,
    right: theirs,
  },);
}

/**
 Whether two claims are about the same thing, loosely enough to survive two
 voices choosing different widths.

 EVERY USED SIDE MUST TOUCH, not just one. Two claims agreeing about the
 original and pointing at different candidate clauses are two claims.

 @param left - one claim

 @param right - the other

 @returns Whether the categories match and every side they both use overlaps

 @example
 ```ts
 const same = aboutTheSameThing({ left, right, },);
 ```
 */
function aboutTheSameThing(
  {
    left,
    right,
  }: {
    readonly left: ScreenedFinding;
    readonly right: ScreenedFinding;
  },
): boolean {
  if (left.category !== right.category)
    return false;

  return sideAgrees({
    mine: intervalOf({ reading: left.source, },),
    theirs: intervalOf({ reading: right.source, },),
  },)
    && sideAgrees({
      mine: intervalOf({ reading: left.candidate, },),
      theirs: intervalOf({ reading: right.candidate, },),
    },);
}

/**
 Groups claims that are about the same thing, without merging through a third.

 PAIRWISE THROUGHOUT, which is what keeps this from collapsing into the false
 merge. Every member of a group must be about the same thing as every OTHER
 member, so a wide claim touching two narrow ones that share no text cannot
 pull them into one group: it forms a pair with each instead.

 @param claims - every voice's claims

 @returns Groups of at least {@link CORROBORATION_VOICES} distinct voices,
 most-agreed first

 @throws {@link RenderingAuditInvariantError} when an agreement group holds
 no member, which cannot occur, since every group is grown from the claim
 that seeds it

 @example
 ```ts
 const agreed = corroborateByOverlap({ claims, },);
 ```
 */
export function corroborateByOverlap(
  { claims, }: { readonly claims: readonly AuditMemberClaim[]; },
): readonly OverlapAgreement[] {
  /**
   One candidate group per claim, each holding every claim that agrees with
   the seed AND with everything already in it.
   */
  const grown = claims.map(function growFrom(seed,): readonly AuditMemberClaim[] {
    /**
     Members admitted so far, starting from the seed.
     */
    const group: AuditMemberClaim[] = [seed,];
    for (const claim of claims) {
      if (claim === seed)
        continue;

      /**
       Whether this claim is about the same thing as every member so far,
       which is what stops a group forming through a third claim.
       */
      const fits = group.every(function agrees(member,): boolean {
        return aboutTheSameThing({
          left: member.finding,
          right: claim.finding,
        },);
      },);

      if (fits)
        group.push(claim,);
    }
    return group;
  },);

  /**
   Groups that reached the threshold, one per distinct membership.
   */
  const byMembership = grown.reduce(
    function keepDistinct(
      groups: Map<string, readonly AuditMemberClaim[]>,
      group,
    ): Map<string, readonly AuditMemberClaim[]> {
      /**
       Voices in this group, which is what the threshold counts.
       */
      const voices = new Set(group.map(function toVoice(member,): string {
        return member.modelId;
      },),);

      if (voices.size < CORROBORATION_VOICES)
        return groups;

      groups.set(
        JSON.stringify(textsInCodePointOrder({ texts: group.map(function toKey(member,): string {
          return `${member.modelId}${defectKey({ finding: member.finding, },)}`;
        },), },),),
        group,
      );
      return groups;
    },
    new Map<string, readonly AuditMemberClaim[]>(),
  );

  return [...byMembership.values(),]
    .map(function toAgreement(members,): OverlapAgreement {
      /**
       First member, whose category every other member shares.
       */
      const [first,] = members;

      if (first === undefined)
        throw new RenderingAuditInvariantError({
          invariant: 'an overlap group with no members cannot occur, since groups are grown from a seed',
        },);

      return {
        category: first.finding
          .category,
        voices: new Set(members.map(function toVoice(member,): string {
          return member.modelId;
        },),).size,
        members,
      };
    },)
    .toSorted(function byAgreement(
      left,
      right,
    ): number {
      return right.voices - left.voices;
    },);
}

/**
 Finds pairs of claims that nearly agree, and says how.

 @param claims - every voice's claims

 @returns Pairs from different voices that overlap without matching

 @example
 ```ts
 const near = nearMisses({ claims, },);
 ```
 */
export function nearMisses(
  { claims, }: { readonly claims: readonly AuditMemberClaim[]; },
): readonly NearMiss[] {
  return claims.flatMap(function againstLater(
    left,
    position,
  ): readonly NearMiss[] {
    return claims
      .slice(position + 1,)
      .flatMap(function pair(right,): readonly NearMiss[] {
        // ONE VOICE'S OWN CLAIMS ARE NOT A NEAR MISS: a voice filing two
        // findings about one sentence is doing what atomicity asks of it.
        if (left.modelId === right.modelId)
          return [];

        if (defectKey({ finding: left.finding, },) === defectKey({ finding: right.finding, },))
          return [];

        if (sameSpans({
          left: left.finding,
          right: right.finding,
        },)) {
          return [{
            kind: 'same-focus-different-category',
            left,
            right,
          },];
        }

        /**
         Whether either side's spans touch at all.
         */
        const touching = intersects({
          left: intervalOf({ reading: left.finding
            .source, },),
          right: intervalOf({ reading: right.finding
            .source, },),
        },)
          || intersects({
            left: intervalOf({ reading: left.finding
              .candidate, },),
            right: intervalOf({ reading: right.finding
              .candidate, },),
          },);

        return touching
          ? [{
            kind: 'overlapping-focus',
            left,
            right,
          },]
          : [];
      },);
  },);
}

//endregion Rendering audit corroboration
