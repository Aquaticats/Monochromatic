/**
 Publication and reconciliation errors.
 
 @module
 */

import type { InputPosition, } from './model.ts';
import type { CreatedIssue, } from './publisher-model.ts';

/**
 Reports terminal publication response or retry failure.
 */
export class IssuePublicationError extends Error {
  /**
   Creates publication failure.
   
   @param message - Safe status or response-shape diagnostic.
   
   @param options - Optional native error cause metadata.
   
   @example
   ```ts
   const error = new IssuePublicationError('Issue creation failed');
   ```
   */
  public constructor(
    message: string,
    options?: ErrorOptions,
  ) {
    super(
      message,
      options,
    );
    this.name = 'IssuePublicationError';
  }
}

/**
 Reports multiple exact post-failure matches without mutating any match.
 */
export class AmbiguousReconciliationError extends IssuePublicationError {
  /**
   Every exact matching Issue URL.
   */
  public readonly urls: readonly string[];

  /**
   Creates multiple-match reconciliation failure.
   
   @param urls - Every exact title-and-body match above high-water mark.
   
   @example
   ```ts
   const error = new AmbiguousReconciliationError({ urls: ['https://example.test/1'] });
   ```
   */
  public constructor({ urls, }: { readonly urls: readonly string[]; },) {
    super(`ambiguous create reconciliation found ${String(urls.length,)} exact matches: ${urls.join(', ',)}`,);
    this.name = 'AmbiguousReconciliationError';
    this.urls = urls;
  }
}


/**
 Carries partial created results and exact input position of stopping failure.
 */
export class PublicationStoppedError extends IssuePublicationError {
  /**
   Confirmed Issues created before failure.
   */
  public readonly created: readonly CreatedIssue[];

  /**
   Input position whose publication stopped the run.
   */
  public readonly position: InputPosition;

  /**
   Creates positioned partial publication failure.
   
   @param created - Confirmed Issues created before failure.
   
   @param position - Input position of failed Issue.
   
   @param cause - Original terminal publication error.
   
   @example
   ```ts
   const error = new PublicationStoppedError({ created: [], position, cause });
   ```
   */
  public constructor({
    created,
    position,
    cause,
  }: {
    readonly created: readonly CreatedIssue[];
    readonly position: InputPosition;
    readonly cause: unknown;
  },) {
    super(
      `publication stopped at ${position.kind} ${String(position.value,)}: ${String(cause,)}`,
      { cause, },
    );
    this.name = 'PublicationStoppedError';
    this.created = created;
    this.position = position;
  }
}


/**
 Carries confirmed creations after first handled publication interrupt.
 */
export class PublicationInterruptedError extends IssuePublicationError {
  /**
   Confirmed Issues created before interrupt settled.
   */
  public readonly created: readonly CreatedIssue[];

  /**
   Position active or next when publication stopped.
   */
  public readonly position: InputPosition;

  /**
   Creates handled publication interruption.
   
   @param created - Confirmed Issues retained after interrupt.
   
   @param position - Input position active or next at stop boundary.
   
   @example
   ```ts
   const error = new PublicationInterruptedError({ created: [], position });
   ```
   */
  public constructor({
    created,
    position,
  }: {
    readonly created: readonly CreatedIssue[];
    readonly position: InputPosition;
  },) {
    super(`Issue creation interrupted at ${position.kind} ${String(position.value,)}`,);
    this.name = 'PublicationInterruptedError';
    this.created = created;
    this.position = position;
  }
}
