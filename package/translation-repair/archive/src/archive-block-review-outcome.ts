//region Archive block review outcome

/**
 Settled review of one unclaimed archive block.
 */
export type ArchiveBlockReviewOutcome = {
  /**
   Whether exact archived text stands or a different replacement was selected.
   */
  readonly kind: 'retained' | 'revised';
  /**
   Original or selected replacement text.
   */
  readonly text: string;
  /**
   Review and selection evidence.
   */
  readonly findings: readonly string[];
};

//endregion Archive block review outcome
