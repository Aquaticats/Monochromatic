//region Audit sensitivity inputs
// The two renderings the rendering audit is asked about, and the ORACLE that
// says where the planted defect is.
//
// These are cat-themed invention. NO corpus text, licensed or otherwise, takes
// part in this check, and nothing here is written anywhere.
//
// THE ORACLE IS THE POINT OF THIS FILE. A live arm that only counted
// corroborated defects could not say WHY it found none: the auditors may have
// missed the defect, or found it and failed to anchor, or found and anchored it
// and been split apart by the matcher. Scoring each voice against a known span
// separates those three, and they call for three different fixes.

/**
 Original both arms are audited against.

 It denies that the cats eat canned food, and says the tabby sleeps on the
 windowsill. Both are load-bearing: the flipped rendering reverses the first
 and leaves the second alone, so a voice that flags the second is reporting
 something the original supports.
 */
export const SOURCE_TEXT = `三只猫住在书店的阁楼里。她们不吃罐头，每天傍晚只喝一碗温牛奶。
那只虎斑猫睡在窗台上，另外两只睡在书架上。`;

/**
 Rendering with the negation dropped, and nothing else changed.

 THE POSITIVE ARM. A dropped negator is the least deniable defect available:
 the original denies what this asserts, in one clause, with every other
 proposition rendered faithfully. An instrument that misses this cannot be
 trusted to find anything subtler, and one that reports defects elsewhere here
 is claiming damage the original supports.
 */
export const FLIPPED_CANDIDATE = `Three cats live in the attic of a bookshop. They eat canned food, and every evening they drink only one bowl of warm milk.
The tabby sleeps on the windowsill, and the other two sleep on the bookshelf.`;

/**
 Faithful rendering of the same original.

 THE CONTROL, and the more informative of the two arms. Corroborated findings
 here are false ones by construction, and the rate of them bounds what any
 later reading of this instrument may claim.
 */
export const CLEAN_CANDIDATE = `Three cats live in the attic of a bookshop. They do not eat canned food; every evening they drink only a bowl of warm milk.
The tabby sleeps on the windowsill, and the other two sleep on the bookshelf.`;

/**
 Span of the original the planted defect lives in.
 */
export const ORACLE_SOURCE_SPAN = '不吃罐头';

/**
 Span of the flipped rendering it lives in.
 */
export const ORACLE_CANDIDATE_SPAN = 'They eat canned food';

/**
 One arm of the instrument: a rendering to audit, its label and what a
 working instrument should conclude.

 @example
 ```ts
 const arm: AuditArm = { candidateText: FLIPPED_CANDIDATE, arm: 'flipped', expectation: 'defect', };
 ```
 */
export type AuditArm = {
  /**
   Rendering under audit.
   */
  readonly candidateText: string;

  /**
   Label for the arm.
   */
  readonly arm: string;

  /**
   What a working instrument should conclude, printed only; nothing branches
   on it.
   */
  readonly expectation: string;
};

/**
 Both arms, in the order they run.
 */
export const AUDIT_ARMS: readonly AuditArm[] = [
  {
    candidateText: FLIPPED_CANDIDATE,
    arm: 'flipped',
    expectation: 'agreement at either tier on the oracle span',
  },
  {
    candidateText: CLEAN_CANDIDATE,
    arm: 'clean',
    expectation: 'agreement at neither tier',
  },
];

//endregion Audit sensitivity inputs
