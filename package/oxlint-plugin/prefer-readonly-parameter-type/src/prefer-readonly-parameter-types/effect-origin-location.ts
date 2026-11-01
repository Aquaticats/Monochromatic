/**
 Source locations for unresolved-effect origins.
 
 Provenance facts stay location-enriched only at their origin call site;
 propagation copies facts unchanged,
 so the fixed point still terminates over a finite fact set while
 diagnostics name the one place a remediation applies.
 
 @module
 */

import type { Node, } from 'typescript/ast';

/**
 Repository-relative marker locating package sources.
 */
const PACKAGES_SEGMENT = '/packages/';

/**
 Shortens absolute path to repository-relative package path.
 
 @param fileName - Absolute source path reported by TypeScript.
 
 @returns repository-relative path when under packages, else full path.
 */
function shortSourcePath(fileName: string,): string {
  /**
   Offset of first package segment inside absolute path.
   */
  const packagesAt = fileName.indexOf(PACKAGES_SEGMENT,);
  if (packagesAt === (-1))
    return fileName;
  return fileName.slice(packagesAt + 1,);
}

/**
 Marker opening one bracketed origin-location suffix.
 */
const ORIGIN_SUFFIX_OPEN = ' [';

/**
 Strips bracketed origin-location suffix from provenance fact.
 
 Contract matching and identity checks compare boundary names,
 not origin locations.
 
 @param provenance - Possibly location-enriched provenance fact.
 
 @returns boundary name without origin location.
 
 @example
 ```ts
 originBoundaryName('JSON.stringify [package/a/src/b.ts:12]');
 ```
 */
export function originBoundaryName(provenance: string,): string {
  if (!provenance.endsWith(']',))
    return provenance;
  /**
   Offset of last bracketed suffix opening.
   */
  const openAt = provenance.lastIndexOf(ORIGIN_SUFFIX_OPEN,);
  if (openAt === (-1))
    return provenance;
  return provenance.slice(
    0,
    openAt,
  );
}

/**
 Describes origin call location for provenance facts.
 
 @param node - Originating call or callee expression.
 
 @returns repository-relative path with one-based line.
 
 @example
 ```ts
 effectOriginLocation({ node: call });
 ```
 */
export function effectOriginLocation({
  node,
}: {
  readonly node: Node;
},): string {
  /**
   Owning source file providing text and name.
   */
  const sourceFile = node.getSourceFile();
  /**
   Node start offset after leading trivia.
   */
  const start = node.getStart(sourceFile,);
  /**
   One-based line accumulated by linear newline scan.
   */
  const line = { value: 1, };
  for (let index = 0; index < start; index += 1) {
    if (sourceFile.text[index] === '\n')
      line.value += 1;
  }
  return `${shortSourcePath(sourceFile.fileName,)}:${String(line.value,)}`;
}
