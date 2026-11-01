/**
 Error raised when `rust-catalog.toml` fails validation.

 @module
 */

import type { RustCatalogIssue, } from './types.ts';

/**
 Formats one issue as an indented diagnostic block.

 @param issue - problem to describe

 @returns lines naming the location, the problem, and the remedy

 @example
 ```ts
 formatIssue({ location: 'catalog.tokio', problem: 'declares `features`', remedy: 'move it', });
 ```
 */
function formatIssue(issue: RustCatalogIssue,): string {
  return [
    `- ${issue.location}: ${issue.problem}.`,
    `  Fix: ${issue.remedy}.`,
  ].join('\n',);
}

/**
 Thrown when `rust-catalog.toml` holds entries or settings the catalog cannot use.

 Every problem found in one pass is reported together, so a contributor can
 fix the whole file at once.

 @example
 ```ts
 try {
   parseRustCatalog({ source, sourcePath: 'rust-catalog.toml', });
 } catch (error) {
   if (error instanceof RustCatalogError)
     console.error(error.issues.length);
   throw error;
 }
 ```
 */
export class RustCatalogError extends Error {
  /**
   Every validation problem found, in source order.
   */
  readonly issues: readonly RustCatalogIssue[];

  /**
   Builds the error from the issues of one validation pass.

   @param sourcePath - file the issues were found in

   @param issues - problems to report
   */
  constructor(
    {
      sourcePath,
      issues,
    }: {
      readonly sourcePath: string;
      readonly issues: readonly RustCatalogIssue[];
    },
  ) {
    super(
      [
        `${sourcePath} has ${String(issues.length,)} problem(s) the Rust dependency catalog cannot accept:`,
        ...issues.map(formatIssue,),
        'See doc/decision/rust-dependency-catalog.md for the catalog format.',
      ].join('\n',),
    );
    this.name = 'RustCatalogError';
    this.issues = issues;
  }
}
