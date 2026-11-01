/**
 Structured deep-readonly classification evidence and diagnostic rendering.
 
 @module
 */

import { isIdentifierText, } from 'typescript/ast/scanner';

/**
 Source ownership of one writable declaration.
 */
export type WritableDeclarationOwner =
  | 'default-library'
    | 'external-library'
    | 'unresolved'
    | 'workspace';

/**
 One access step from parameter type to writable state.
 */
export type WritablePathSegment =
  | {
    readonly kind: 'index';
    readonly keyType: string;
  }
    | {
      readonly kind: 'property';
      readonly name: string;
    };

/**
 Kind of writable state reached at one complete path.
 */
export type WritablePathKind = 'array' | 'index' | 'property' | 'tuple';

/**
 Complete structured cause for one mutable classification.
 */
export type WritablePath = {
  readonly kind: WritablePathKind;
  readonly segments: readonly WritablePathSegment[];
  readonly declarationOwners: readonly WritableDeclarationOwner[];
};

/**
 Semantic readonly classification used by rule diagnostics.
 
 @example
 ```ts
 const result: ReadonlyClassification = {
   kind: 'mutable',
   writablePaths: [{
     kind: 'property',
     segments: [{ kind: 'property', name: 'value' }],
     declarationOwners: ['workspace'],
   }],
 };
 ```
 */
export type ReadonlyClassification =
  | { readonly kind: 'deep-readonly'; }
    | {
      readonly kind: 'mutable';
      readonly writablePaths: readonly WritablePath[];
    }
    | {
      readonly kind: 'opaque-capability';
      readonly reason: string;
    }
    | {
      readonly kind: 'projected-readonly-capability';
      readonly reason: string;
    };

/**
 Deep-readonly singleton result.
 */
export const DEEP_READONLY: ReadonlyClassification = { kind: 'deep-readonly', };

/**
 Builds mutable classification from one writable cause.
 
 @param path - Complete writable path relative to current classified type.
 
 @returns mutable classification retaining structured path evidence.
 
 @example
 ```ts
 mutableReadonlyClassification({
   kind: 'tuple',
   segments: [],
   declarationOwners: [],
 });
 ```
 */
export function mutableReadonlyClassification(
  path: WritablePath,
): ReadonlyClassification {
  return {
    kind: 'mutable',
    writablePaths: [path,],
  };
}

/**
 Prepends one access segment to every mutable path in classification.
 
 @param classification - Child classification reached through segment.
 
 @param segment - Parent access step entering child type.
 
 @returns same non-mutable classification or prefixed mutable evidence.
 
 @example
 ```ts
 prefixReadonlyClassification({
   classification,
   segment: { kind: 'property', name: 'child' },
 });
 ```
 */
export function prefixReadonlyClassification({
  classification,
  segment,
}: {
  readonly classification: ReadonlyClassification;
  readonly segment: WritablePathSegment;
}): ReadonlyClassification {
  if (classification.kind !== 'mutable')
    return classification;
  return {
    kind: 'mutable',
    writablePaths: classification.writablePaths
      .map(function prefixPath(path,): WritablePath {
        return {
          ...path,
          segments: [
            segment,
            ...path.segments,
          ],
        };
      },),
  };
}

/**
 Stable identity for de-duplicating equivalent writable paths.
 
 @param path - Structured writable cause.
 
 @returns JSON identity preserving cause kind and access segments.
 */
function writablePathIdentity(path: WritablePath,): string {
  return JSON.stringify([
    path.kind,
    path.segments,
  ],);
}

/**
 Renders one property segment as JavaScript-like property access.
 
 @param name - Exact semantic property name.
 
 @param first - Whether segment begins rendered path.
 
 @returns dot access for identifiers or bracketed JSON string for other names.
 */
function renderPropertySegment({
  name,
  first,
}: {
  readonly name: string;
  readonly first: boolean;
}): string {
  if (isIdentifierText(name,))
    return first ? name : `.${name}`;
  return `[${JSON.stringify(name,)}]`;
}

/**
 Renders complete structured writable path.
 
 @param segments - Access steps from parameter to writable state.
 
 @returns JavaScript-like path without diagnostic delimiters.
 
 @example
 ```ts
 renderWritablePath([
   { kind: 'property', name: 'children' },
   { kind: 'index', keyType: 'number' },
   { kind: 'property', name: 'type' },
 ]); // children[number].type
 ```
 */
export function renderWritablePath(
  segments: readonly WritablePathSegment[],
): string {
  return segments.reduce(
    function appendSegment(
      rendered,
      segment,
      index,
    ): string {
      if (segment.kind === 'index')
        return `${rendered}[${segment.keyType}]`;
      return `${rendered}${renderPropertySegment({
        name: segment.name,
        first: index === 0,
      },)}`;
    },
    '',
  );
}

/**
 Renders one structured cause for diagnostic reason text.
 
 @param path - Complete writable cause.
 
 @returns quoted path and mutability statement.
 */
function writablePathReason(path: WritablePath,): string {
  /**
   Rendered access path,
   empty only when parameter type itself is mutable collection syntax.
   */
  const rendered = renderWritablePath(path.segments,);
  if (path.kind === 'array') {
    return rendered.length === 0
      ? 'parameter type uses mutable `Array`'
      : `\`${rendered}\` uses mutable \`Array\``;
  }
  if (path.kind === 'tuple') {
    return rendered.length === 0
      ? 'parameter type uses a mutable tuple'
      : `\`${rendered}\` uses a mutable tuple`;
  }
  if (rendered.length === 0)
    return path.kind === 'index'
      ? 'parameter index signature is writable'
      : 'parameter property is writable';
  return `\`${rendered}\` is writable`;
}

/**
 De-duplicates and sorts writable paths without presentation truncation.
 
 @param paths - Structured mutable causes from every reachable branch.
 
 @returns deterministic complete distinct cause list.
 
 @example
 ```ts
 normalizeWritablePaths(paths);
 ```
 */
export function normalizeWritablePaths(
  paths: readonly WritablePath[],
): readonly WritablePath[] {
  /**
   First path retained for each exact structured identity.
   */
  const pathsByIdentity = new Map<string, WritablePath>();
  paths.forEach(function retainPath(path,): void {
    /**
     Stable identity shared by same rendered cause across type branches.
     */
    const identity = writablePathIdentity(path,);
    /**
     Earlier equivalent path whose declaration ownership must be preserved.
     */
    const existing = pathsByIdentity.get(identity,);
    pathsByIdentity.set(
      identity,
      existing === undefined
        ? path
        : {
          ...path,
          declarationOwners: [...new Set([
            ...existing.declarationOwners,
            ...path.declarationOwners,
          ],),].toSorted(),
        },
    );
  },);
  return [...pathsByIdentity.values(),]
    .toSorted(function byRenderedPath(
      left,
      right,
    ): number {
      /**
       Left rendered identity compared by runtime-independent UTF-16 code units.
       */
      const leftReason = writablePathReason(left,);
      /**
       Right rendered identity compared by runtime-independent UTF-16 code units.
       */
      const rightReason = writablePathReason(right,);
      if (leftReason < rightReason)
        return -1;
      if (leftReason > rightReason)
        return 1;
      return 0;
    },);
}

/**
 Renders every distinct writable cause on one physical diagnostic line.
 
 @param classification - Mutable classification carrying path evidence.
 
 @returns semicolon-separated complete reason list.
 
 @example
 ```ts
 readonlyMutableReason(classification);
 ```
 */
export function readonlyMutableReason(
  classification: Extract<ReadonlyClassification, { readonly kind: 'mutable'; }>,
): string {
  return normalizeWritablePaths(classification.writablePaths,)
    .map(writablePathReason,)
    .join('; ',);
}

/**
 Tests whether mutable evidence is exclusively structural property or index data.
 
 @param classification - Candidate mutable classification.
 
 @returns whether deep structural projection applies to every cause.
 
 @example
 ```ts
 mutableClassificationIsStructural(classification);
 ```
 */
export function mutableClassificationIsStructural(
  classification: ReadonlyClassification,
): boolean {
  if (classification.kind !== 'mutable')
    return false;
  /**
   Mutable paths narrowed from classification branch.
   */
  const { writablePaths, } = classification;
  if (writablePaths.length === 0)
    return false;
  return writablePaths
    .every(function structural(path,): boolean {
      return (path.kind === 'property') || (path.kind === 'index');
    },);
}

/**
 Tests whether any writable declaration belongs outside workspace source.
 
 @param classification - Candidate mutable classification.
 
 @returns whether at least one external or default-library declaration is reached.
 
 @example
 ```ts
 mutableClassificationHasExternalDeclaration(classification);
 ```
 */
export function mutableClassificationHasExternalDeclaration(
  classification: ReadonlyClassification,
): boolean {
  if (classification.kind !== 'mutable')
    return false;
  return classification
    .writablePaths
    .some(function externalPath(path,): boolean {
      return path.declarationOwners
        .some(function externalOwner(owner,): boolean {
          return (owner === 'default-library') || (owner === 'external-library');
        },);
    },);
}

/**
 Tests whether any writable declaration belongs to workspace source.
 
 @param classification - Candidate mutable classification.
 
 @returns whether at least one exact workspace declaration is writable.
 
 @example
 ```ts
 mutableClassificationHasWorkspaceDeclaration(classification);
 ```
 */
export function mutableClassificationHasWorkspaceDeclaration(
  classification: ReadonlyClassification,
): boolean {
  if (classification.kind !== 'mutable')
    return false;
  return classification
    .writablePaths
    .some(function workspacePath(path,): boolean {
      /**
       Ownership categories for current writable leaf.
       */
      const { declarationOwners, } = path;
      return declarationOwners.includes('workspace',);
    },);
}
