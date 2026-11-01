/**
 Fail-closed errors from TypeScript semantic bridge.
 
 @module
 */

/**
 Stable bridge failure categories mapped to semantic diagnostics.
 
 @example
 ```ts
 const reason: SemanticBridgeFailureReason = 'project-not-found';
 ```
 */
export type SemanticBridgeFailureReason =
  | 'api-unavailable'
    | 'project-not-found'
    | 'source-file-not-found'
    | 'node-not-found'
    | 'analysis-incomplete'
    | 'readonly-capability-unavailable';

/**
 Error raised when unstable TypeScript bridge cannot provide required semantics.
 
 @example
 ```ts
 throw new SemanticBridgeError({
   reason: 'project-not-found',
   message: 'No configured project contains file.',
 });
 ```
 */
export class SemanticBridgeError extends Error {
  /**
   Machine-readable failure category used as diagnostic data.
   */
  readonly reason: SemanticBridgeFailureReason;

  /**
   Creates fail-closed semantic bridge error.
   
   @param reason - Stable category for diagnostic routing.
   
   @param message - Human-readable context retaining affected resource.
   */
  constructor({
    reason,
    message,
  }: {
    readonly reason: SemanticBridgeFailureReason;
    readonly message: string;
  },) {
    super(message,);
    this.name = 'SemanticBridgeError';
    this.reason = reason;
  }
}
