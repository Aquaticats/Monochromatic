/**
 Inert ownership-marker reporting.
 
 @module
 */

import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import type { Context, } from '@oxlint/plugins';
import type {
  Project,
  Type,
} from 'typescript/sync';

import { classifyReadonlyType, } from './readonly-classifier.ts';
import { isForeignBorrowedType, } from './foreign-borrowed-identity.ts';
import type { SemanticReportLocation, } from './semantic-location.ts';

/**
 Tests whether a parameter type is the shape this report is ever about.
 
 Every test here is a fact about the declared type alone, so it can be asked before foreign
 ownership is known. That ordering is what lets the verifier decline the proof for a parameter
 this report could not name whatever the proof said, and it stays an equivalence because
 `reportRedundantForeignBorrowed` applies the same tests before emitting anything.
 
 @param project - TypeScript project resolving marker identity.
 
 @param parameterType - Semantic parameter type possibly carrying marker.
 
 @returns whether a marker is present and its underlying type is already deeply readonly.
 
 @example
 ```ts
 redundantMarkerApplies({ project, parameterType });
 ```
 */
export function redundantMarkerApplies({
  project,
  parameterType,
}: {
  readonly project: Project;
  readonly parameterType: Type;
},): boolean {
  if (!isForeignBorrowedType({
    project,
    type: parameterType,
  },))
    return false;
  /**
   Marked underlying type supplied as marker type argument.
   */
  const underlying = parameterType
    .getAliasTypeArguments()
    .at(0,);
  if (underlying === undefined)
    return false;
  /**
   Classification of underlying type without marker exemption.
   */
  const underlyingClassification = classifyReadonlyType({
    checker: project.checker,
    project,
    type: underlying,
  },);
  return underlyingClassification.kind === 'deep-readonly';
}

/**
 Reports a ForeignBorrowed marker that no longer affects classification.
 
 A marker whose underlying type is already deeply readonly confers no
 mutable capability on any caller or callee;
 removing it changes no classification,
 so keeping it is misleading documentation.
 
 @param context - Foreign rule context receiving diagnostics.
 
 @param project - TypeScript project resolving marker identity.
 
 @param parameterType - Semantic parameter type possibly carrying marker.
 
 @param parameterSubject - Authored parameter text used in diagnostics.
 
 @param loc - Report location spanning parameter binding.
 
 @mutates context - Emits redundant-marker diagnostic through rule context.
 
 @example
 ```ts
 reportRedundantForeignBorrowed({ context, project, parameterType, parameterSubject, loc });
 ```
 */
export function reportRedundantForeignBorrowed({
  context,
  project,
  parameterType,
  parameterSubject,
  loc,
}: {
  readonly context: ForeignBorrowed<Context>;
  readonly project: Project;
  readonly parameterType: Type;
  readonly parameterSubject: string;
  readonly loc: SemanticReportLocation;
},): void {
  /* Repeated rather than assumed from the caller. The predicate is the guard this report has
   * always carried, and a caller that hoisted it for its own scheduling has not taken
   * responsibility for it. */
  if (!redundantMarkerApplies({
    project,
    parameterType,
  },))
    return;
  context.report({
    loc,
    messageId: 'redundantForeignBorrowed',
    data: { parameterSubject, },
  },);
}
