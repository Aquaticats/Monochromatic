/**
 Test-only helpers that hand a real AST node from rolldown's parser to a case.
 
 The package's exported extractors take AST nodes, never source text, so a case
 needs the parser's own output. Each helper parses one source string and returns
 the node the case asked for, throwing when the source does not carry it.
 
 Imported from `./test-support.ts`, which the lint rule for test-only fixtures
 allows a test to reach through source.
 
 @module
 */

import {
  type ESTree,
  parseSync,
  Visitor,
} from 'rolldown/utils';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

/**
 Name the parser reports in diagnostics; the helpers only need a stable label.
 */
const SOURCE_NAME = 'test-support-case.ts';

/**
 Parses one module and walks it with `visitor`.
 
 @param code - module source to parse
 
 @param visitor - visitor the parser walks the module with
 
 @throws Error when the source does not parse
 */
function visitModule({
  code,
  visitor,
}: {
  readonly code: string;
  readonly visitor: Visitor;
},): void {
  /**
   Parsed module the visitor walks.
   */
  const parsed = parseSync(
    SOURCE_NAME,
    code,
  );
  visitor.visit(parsed.program,);
}

/**
 Returns the import attributes of the source's first static declaration,
 whether it is an import, a named re-export or a wildcard re-export.
 
 @param code - module source holding a static `import` with an attribute clause
 
 @returns the declaration's attribute nodes, in source order
 
 @throws Error when the source carries no static import with attributes
 
 @example
 ```ts
 const attributes = importAttributesOf("import x from './a.sql' with { type: 'text' };");
 attributes.length; // 1
 ```
 */
export function importAttributesOf(code: string,): readonly ESTree.ImportAttribute[] {
  /**
   Holder the visitor writes the first attribute list into.
   */
  const found: { attributes?: readonly ESTree.ImportAttribute[]; } = {};
  visitModule({
    code,
    visitor: new Visitor({
      ImportDeclaration(node: ForeignBorrowed<ESTree.ImportDeclaration>,): void {
        found.attributes ??= node.attributes;
      },
      ExportNamedDeclaration(node: ForeignBorrowed<ESTree.ExportNamedDeclaration>,): void {
        found.attributes ??= node.attributes;
      },
      ExportAllDeclaration(node: ForeignBorrowed<ESTree.ExportAllDeclaration>,): void {
        found.attributes ??= node.attributes;
      },
    },),
  },);
  if (found.attributes === undefined)
    throw new Error(`no static import with attributes in: ${code}`,);
  return found.attributes;
}

/**
 Returns the source literal of the source's first static declaration.
 
 @param code - module source holding a static `import`
 
 @returns the declaration's specifier literal
 
 @throws Error when the source carries no static import
 
 @example
 ```ts
 const source = stringLiteralSourceOf("import x from './a.sql' with { type: 'text' };");
 source.value; // './a.sql'
 ```
 */
export function stringLiteralSourceOf(code: string,): ESTree.StringLiteral {
  /**
   Holder the visitor writes the first specifier into.
   */
  const found: { source?: ESTree.StringLiteral; } = {};
  visitModule({
    code,
    visitor: new Visitor({
      ImportDeclaration(node: ForeignBorrowed<ESTree.ImportDeclaration>,): void {
        found.source ??= node.source;
      },
    },),
  },);
  if (found.source === undefined)
    throw new Error(`no static import in: ${code}`,);
  return found.source;
}

/**
 Returns the options expression of the source's first dynamic import.
 
 @param code - module source holding an `import()` call
 
 @returns the call's second argument
 
 @throws Error when the source carries no dynamic import with options
 
 @example
 ```ts
 const options = dynamicImportOptionsOf("import('./a.sql', { with: { type: 'text' } });");
 options.type; // 'ObjectExpression'
 ```
 */
export function dynamicImportOptionsOf(code: string,): ESTree.Expression {
  /**
   Holder the visitor writes the first options expression into.
   */
  const found: { options?: ESTree.Expression; } = {};
  visitModule({
    code,
    visitor: new Visitor({
      ImportExpression(node: ForeignBorrowed<ESTree.ImportExpression>,): void {
        if ((found.options === undefined) && (node.options !== null))
          found.options = node.options;
      },
    },),
  },);
  if (found.options === undefined)
    throw new Error(`no dynamic import with options in: ${code}`,);
  return found.options;
}
