import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import type { ChunkRepairOutcome, } from '@monochromatic-dev/translation-repair-assembly/ts';
import type { IssueClaim, } from '@monochromatic-dev/translation-repair-shared-contract/ts';

//region Non-translation evidence
// Deterministic contradiction check for wire-level non-translation votes.
// The category names a WHOLLY unrelated pair, so critics anchoring
// substantive content critique into the target text contradict the votes by
// the category's own definition. Deterministic evidence outranks model
// consensus here: the first real-corpus false block (Aniloviraw) drew four
// of seven critic votes AND a panel acceptance, proving model judgment
// noisy at both layers on divergence-heavy translated pairs.

/**
 Wire-level critical non-translation votes required before repair may be
 blocked: two answering critics, the ballots any decision needs, and no more.

 THE OWNER'S RULINGS OF 2026-10-25 set this at two ("floor of 3 - don't set a
 floor.", "Two ballots decide (Recommended)" and "Every weight floor"): a claim
 or contest is decided with two or more answering judges, and one judge alone
 never decides. It was three from the first real-corpus false block
 (AkiraComplex, commit `72f5537c1`), which discarded a faithful translation on
 TWO votes from only two of seven critics heard on an English-epigraph slice
 (English in both source and target, so the two heard critics misread it as
 source-equals-target non-translation), and it stopped two critics from
 blocking invented text in a later pass (XingZ6021, slices 68 and 69). Two
 votes cannot come from fewer than two heard critics, so a single critic can
 never block. What guards the AkiraComplex shape now is the deterministic
 contradiction check ({@link NON_TRANSLATION_CONTRADICTION_MIN}) and the
 good-translation anchor that vetoes a document-wide block; the vote count is no
 longer the guard. A false per-slice block leaves that slice unrepaired, with
 the archive's wording as it stands; the document-level dominance block is
 report-only (`repair/src/repair-translation.ts` logs it as "reported, not
 blocking" and adds a finding), so no block discards a translation.
 */
export const NON_TRANSLATION_BLOCK_VOTES = 2;

/**
 Validated content-critique claims anchored into target text at which
 non-translation votes stand contradicted.
 Derivation: the observed false block carried 44 such claims, while
 correct blocks come from skeleton or untranslated targets offering
 little translated content to critique; eight keeps an ensemble-scale
 margin (more than one claim per heard critic on a seven-critic roster)
 above that noise floor.
 */
export const NON_TRANSLATION_CONTRADICTION_MIN = 8;

/**
 Category leaves evidencing MISSING translation rather than defective
 translation; claims under these leaves anchor happily onto skeleton or
 untranslated targets, so they never count toward contradiction.
 */
const MISSING_TRANSLATION_LEAVES: ReadonlySet<string> = new Set([
  'omission',
  'untranslated',
  'non-translation',
]);

/**
 Verdict of deterministic evidence over pending non-translation votes.

 @example
 ```ts
 const evidence: NonTranslationEvidence = {
   contradicted: true,
   contradictionClaimCount: 44,
 };
 ```
 */
export type NonTranslationEvidence = {
  /**
   Whether votes reached the block threshold yet contradiction reached
   its floor, so repair proceeds and votes are dismissed as noise.
   */
  readonly contradicted: boolean;

  /**
   Validated content-critique claims anchored into target text.
   */
  readonly contradictionClaimCount: number;
};

/**
 Rules whether validated claims deterministically contradict wire-level
 non-translation votes.
 Counts claims outside the missing-translation leaves that anchor at
 least one span into the target; enough such claims prove the pair is a
 critiquable translation, not a wholly unrelated pairing.

 @param votes - wire-level critical non-translation votes heard

 @param claims - validated claims across every heard critic

 @returns Contradiction verdict plus supporting claim count

 @example
 ```ts
 const evidence = assessNonTranslationEvidence({
   votes: critic.nonTranslationVotes,
   claims: critic.claims,
 },);
 if (evidence.contradicted) proceedInsteadOfBlocking();
 ```
 */
export function assessNonTranslationEvidence(
  {
    votes,
    claims,
  }: {
    readonly votes: number;
    readonly claims: readonly IssueClaim[];
  },
): NonTranslationEvidence {
  /**
   Claims critiquing translated target content.
   */
  const contradictionClaimCount = claims
    .filter(function critiquesTargetContent(claim,) {
      /**
       Leaf segment of the category slug.
       */
      const leaf = claim.category
        .slice(claim.category
          .lastIndexOf('/',)
          + 1,);
      if (MISSING_TRANSLATION_LEAVES.has(leaf,))
        return false;
      return claim.spans
        .some(function anchorsTarget(span,) {
          return span.side === 'target';
        },);
    },)
    .length;

  return {
    contradicted: (votes >= NON_TRANSLATION_BLOCK_VOTES)
      && (contradictionClaimCount >= NON_TRANSLATION_CONTRADICTION_MIN),
    contradictionClaimCount,
  };
}

/**
 Screening result carrying the claims and findings the pipeline
 continues with after vote assessment.

 @example
 ```ts
 const screening: NonTranslationScreening = {
   contradicted: true,
   claims: [],
   findings: ['non-translation votes contradicted (4 votes, 44 content-critique claims); votes dismissed',],
 };
 ```
 */
export type NonTranslationScreening = {
  /**
   Whether votes stand contradicted; callers must never block when true.
   */
  readonly contradicted: boolean;

  /**
   Claims to continue with; contradicted votes take their
   non-translation claims along so envelopes never carry them.
   */
  readonly claims: readonly IssueClaim[];

  /**
   Contradiction record for the chunk's findings; empty when votes
   stand.
   */
  readonly findings: readonly string[];
};

/**
 Screens wire-level non-translation votes against deterministic
 evidence, dismissing contradicted votes together with their claims.

 @param votes - wire-level critical non-translation votes heard

 @param claims - validated claims across every heard critic

 @returns Continuing claims, contradiction findings, and verdict

 @example
 ```ts
 const screening = screenNonTranslationVotes({
   votes: critic.nonTranslationVotes,
   claims: critic.claims,
 },);
 ```
 */
export function screenNonTranslationVotes(
  {
    votes,
    claims,
  }: {
    readonly votes: number;
    readonly claims: readonly IssueClaim[];
  },
): NonTranslationScreening {
  /**
   Deterministic verdict over pending votes.
   */
  const evidence = assessNonTranslationEvidence({
    votes,
    claims,
  },);
  if (!evidence.contradicted) {
    return {
      contradicted: false,
      claims,
      findings: [],
    };
  }

  return {
    contradicted: true,
    claims: claims.filter(function keepsContentCritique(claim,) {
      return !claim.category
        .endsWith('/non-translation',);
    },),
    findings: [
      `non-translation votes contradicted (${String(votes,)} ${
        wordForCount({
          count: votes,
          one: 'vote',
          many: 'votes',
        },)
      }, ${
        String(evidence.contradictionClaimCount,)
      } content-critique ${
        wordForCount({
          count: evidence.contradictionClaimCount,
          one: 'claim',
          many: 'claims',
        },)
      }); votes dismissed`,
    ],
  };
}

/**
 Rules whether a slice's non-translation votes stand, so its characters
 count toward the document dominance block.
 Votes stand only at the {@link NON_TRANSLATION_BLOCK_VOTES} floor and only
 while deterministic evidence has not contradicted them; the floor itself
 carries the participation guard, since that many wire votes cannot come from
 fewer critics heard.

 @param votes - wire-level critical non-translation votes heard on the slice

 @param contradicted - whether deterministic evidence dismissed the votes

 @returns Whether the slice ships unchanged as standing non-translation

 @example
 ```ts
 const stands = nonTranslationVotesStand({
   votes: critic.nonTranslationVotes,
   contradicted: screening.contradicted,
 },);
 ```
 */
export function nonTranslationVotesStand(
  {
    votes,
    contradicted,
  }: {
    readonly votes: number;
    readonly contradicted: boolean;
  },
): boolean {
  return (votes >= NON_TRANSLATION_BLOCK_VOTES) && (!contradicted);
}

/**
 Whether a settled slice anchors confirmed good-translation content: it is
 not a standing non-translation and carries at least one accepted issue that
 critiques translated target text (a claim outside the missing-translation
 leaves anchoring a target span). Such an anchor proves the paired document
 is a translation being critiqued, not a wholly unrelated pairing. Callers
 guard slices not yet settled; an unsettled slice anchors nothing.

 @param outcome - slice's settled repair outcome

 @returns True when the slice proves translated content

 @example
 ```ts
 const anchors = sliceAnchorsTranslation({ outcome, },);
 ```
 */
export function sliceAnchorsTranslation(
  { outcome, }: { readonly outcome: ChunkRepairOutcome; },
): boolean {
  if (outcome.nonTranslationStanding)
    return false;
  return outcome.issues
    .some(function critiquesTarget(issue,) {
      if (issue.status !== 'accepted')
        return false;
      return issue.claims
        .some(function anchorsTargetContent(member,) {
          /**
           Leaf segment of the member claim's category slug.
           */
          const leaf = member.claim
            .category
            .slice(member.claim
              .category
              .lastIndexOf('/',)
              + 1,);
          if (MISSING_TRANSLATION_LEAVES.has(leaf,))
            return false;
          return member.claim
            .spans
            .some(function onTarget(span,) {
              return span.side === 'target';
            },);
        },);
    },);
}

/**
 Document-level dominance verdict over standing per-slice votes.

 @example
 ```ts
 const dominance: NonTranslationDominance = {
   blocked: true,
   standingChars: 900,
   totalChars: 1_100,
 };
 ```
 */
export type NonTranslationDominance = {
  /**
   Whether standing votes dominate the translation's characters, so the
   dominance block is reported (report-only: repair of the other slices goes on).
   */
  readonly blocked: boolean;

  /**
   Target characters inside slices whose votes stand.
   */
  readonly standingChars: number;

  /**
   Target characters across every slice considered so far.

   THE SLICED FRACTION, not the translation. A section the aligner refused to
   pair produces no slice, and slicing leaves one-sided sections out
   altogether, so both this and `standingChars` are sums over what was
   EXAMINED. Measured over the pinned corpus, slices cover 92.5% of an average
   English document, and two entries fall under half. Which denominator the
   block should read is a decision rather than a defect.
   */
  readonly totalChars: number;
};

/**
 Rules whether standing non-translation votes dominate a document.
 A wholly unrelated pair carries standing votes on most of its
 characters; a minority region (an untranslated passage, a divergent
 paragraph) degrades only its own slices and must never block the
 repairable remainder (settled architecture: degradation is never
 document-wide).
 A confirmed good-translation anchor anywhere vetoes the block outright,
 even when standing chars dominate: such an anchor proves the document IS a
 translation, so a standing-char majority then means asymmetric extra
 content (image/screenshot translations with no source-markdown counterpart)
 rather than an unrelated pairing. Every real-corpus block this session was
 this false shape (Aniloviraw, AkiraComplex, Arita, Mio all held clean
 anchors); the only true positive (a wholly unrelated invented pair) held
 none, so the anchor veto separates them without a tuned threshold and keeps
 the calibrated err-toward-not-blocking direction.

 @param slices - per-slice target sizes with their standing and anchor
 verdicts

 @returns Dominance verdict with the character tallies behind it

 @example
 ```ts
 const dominance = assessNonTranslationDominance({ slices: tallies, },);
 if (dominance.blocked) returnUnchanged();
 ```
 */
export function assessNonTranslationDominance(
  {
    slices,
  }: {
    readonly slices: readonly {
      readonly targetChars: number;
      readonly votesStand: boolean;
      readonly anchorsTranslation: boolean;
    }[];
  },
): NonTranslationDominance {
  /**
   Target characters inside standing-vote slices.
   */
  const standingChars = slices
    .filter(function stands(slice,) {
      return slice.votesStand;
    },)
    .reduce(
    function addChars(
      sum,
      slice,
    ) {
      return sum + slice.targetChars;
    },
    0,
    );

  /**
   Target characters across every slice.
   */
  const totalChars = slices.reduce(
    function addChars(
      sum,
      slice,
    ) {
      return sum + slice.targetChars;
    },
    0,
  );

  /**
   Whether any settled slice anchors confirmed translated content, which
   vetoes the block however the standing chars fall.
   */
  const anchored = slices
    .some(function anchors(slice,) {
      return slice.anchorsTranslation;
    },);

  return {
    blocked: ((standingChars * 2) > totalChars) && (!anchored),
    standingChars,
    totalChars,
  };
}

//endregion Non-translation evidence
