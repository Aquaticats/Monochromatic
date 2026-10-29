/**
 Type surface of the thinking-dismissal bundle.
 The implementation is plain JavaScript with JSDoc;
 this declaration exists so TypeScript consumers and the test resolve it.
 */

/**
 One detection inside one thinking block: a hash and sentence index,
 never thinking text.
 */
export type DismissalHit = {
  readonly pattern: string;
  readonly blockHash: string;
  readonly sentenceIndex: number;
};

/**
 Folded projection state: reasoning-block count and the hits found so far.
 */
export type DismissalState = {
  readonly textBlocks: number;
  readonly hits: readonly DismissalHit[];
};

/** Cordis plugin name. */
export declare const name: string;

/** Cordis inject list; the projection registry is required. */
export declare const inject: readonly string[];

/**
 One committed session event,
 seen structurally as its kind and payload.
 */
export type DismissalEvent = {
  readonly type: string;
  readonly [key: string]: unknown;
};

/**
 Finds size-dismissal sentences in one thinking block.

 @param block - thinking text of one block

 @returns Hits in sentence order, empty when the block dismisses nothing
 */
export declare function findSizeDismissal(
  block: string,
): readonly DismissalHit[];

/**
 The `thinkingDismissal` projection unit registered on `ctx.sessionProjections`.
 */
export declare const thinkingDismissalProjection: {
  readonly key: string;
  readonly stateVersion: number;
  readonly init: () => DismissalState;
  readonly apply: (
    state: DismissalState,
    event: DismissalEvent,
  ) => DismissalState;
};

/**
 Registers the projection unit and the three recording points.

 @param ctx - registrant context carrying the projection registry
 */
export declare function apply(
  ctx: unknown,
): void;
