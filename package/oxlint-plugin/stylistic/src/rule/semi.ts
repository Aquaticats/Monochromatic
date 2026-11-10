import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import type {
  Context,
  CreateOnceRule,
  Fixer,
  Node,
  Token,
  VisitorWithHooks,
} from '@oxlint/plugins';

//region Constants
// The terminator itself, the loop left slots that use separators instead, and
// the export-default declarations their own grammar terminates.

/**
 Token value that terminates statement-like syntax under the always-semi rule.
 */
const SEMICOLON = ';';

/**
 Parent node types whose variable-declaration left slot does not accept a trailing terminator.
 */
const FOR_LEFT_PARENT_TYPES = new Set([
  'ForInStatement',
  'ForOfStatement',
],);

/**
 Export-default declaration node types that are self-terminated by their own grammar.
 */
const EXPORT_DEFAULT_DECLARATION_SEMI_EXEMPT_TYPES = new Set([
  'ClassDeclaration',
  'FunctionDeclaration',
  'TSInterfaceDeclaration',
],);
//endregion Constants

//region Structured node views
// The parent links and declaration payloads the rule reads, which oxlint's bare
// `Node` shape omits.

/**
 AST node with a string discriminant.
 */
type TypedNode = Node & {
  /**
   ESTree or Oxlint node type.
   */
  readonly type: string;
};

/**
 AST parent shape needed to identify loop-init variable declarations.
 */
type VariableDeclarationParent = TypedNode & {
  /**
   Initializer slot of `for (...)`, when parent is a `ForStatement`.
   */
  readonly init?: unknown;
  /**
   Left slot of `for (... in/of ...)`, when parent is a for-in/of statement.
   */
  readonly left?: unknown;
};

/**
 AST node carrying a parent link.
 */
type NodeWithParent = Node & {
  /**
   Parent AST node, absent only for root-like synthetic nodes.
   */
  readonly parent?: VariableDeclarationParent;
};

/**
 Export-default declaration node shape needed for semicolon exemptions.
 */
type ExportDefaultDeclarationNode = Node & {
  /**
   Declaration payload of `export default ...`.
   */
  readonly declaration: TypedNode;
};

/**
 Export-named declaration node shape needed to distinguish declaration exports.
 */
type ExportNamedDeclarationNode = Node & {
  /**
   Declaration payload, null for `export { value }`, and undefined for malformed stubs.
   */
  readonly declaration?: unknown;
};
//endregion Structured node views

//region Token lookup and loop-slot exclusions
// A node's last token, and the two grammar positions where a missing semicolon
// is correct rather than a violation.

/**
 Parameters for {@link lastTokenOf}.
 */
type LastTokenOfParams = {
  /**
   Rule context used for source-code token lookup.
   */
  readonly context: Context;
  /**
   AST node requiring a last token.
   */
  readonly node: Node;
};

/**
 Parameters for {@link isForStatementInitializer}.
 */
type IsForStatementInitializerParams = {
  /**
   Variable declaration being checked.
   */
  readonly node: Node;
  /**
   Parent node that may own the declaration as a `for` initializer.
   */
  readonly parent: VariableDeclarationParent;
};

/**
 Parameters for {@link isForInOrOfLeft}.
 */
type IsForInOrOfLeftParams = {
  /**
   Variable declaration being checked.
   */
  readonly node: Node;
  /**
   Parent node that may own the declaration as a for-in/of left side.
   */
  readonly parent: VariableDeclarationParent;
};

/**
 Parameters for {@link checkForSemicolon}.
 */
type CheckForSemicolonParams = {
  /**
   Rule context with token lookup and reporting APIs.
   */
  readonly context: Context;
  /**
   Statement-like AST node being checked.
   */
  readonly node: Node;
};

/**
 Returns the last syntax token for a node, throwing when Oxlint supplies none.
 
 @param params - rule context and AST node
 
 @returns last token inside node
 
 @throws when Oxlint cannot find a token for node
 
 @example
 ```ts
 const lastToken = lastTokenOf({ context, node });
 ```
 */
function lastTokenOf(params: ForeignBorrowed<Readonly<LastTokenOfParams>>,): Token {
  /**
   Rule context used for source-code token lookup.
   */
  const { context, } = params;
  /**
   AST node requiring a last token.
   */
  const { node, } = params;
  /**
   Last token from Oxlint's token store, or null for malformed synthetic nodes.
   */
  const lastToken = context.sourceCode
    .getLastToken(node,);
  if (lastToken === null) {
    throw new Error('Expected node to contain at least one token.',);
  }

  return lastToken;
}

/**
 Checks whether a variable declaration occupies a `for` initializer slot.
 
 @param params - variable declaration and parent node
 
 @returns whether declaration is the `for` initializer
 
 @example
 ```ts
 if (isForStatementInitializer({ node, parent })) return false;
 ```
 */
function isForStatementInitializer(params: ForeignBorrowed<Readonly<IsForStatementInitializerParams>>,): boolean {
  /**
   Variable declaration being checked.
   */
  const { node, } = params;
  /**
   Parent node that may be a `ForStatement`.
   */
  const { parent, } = params;
  return (parent.type === 'ForStatement') && (parent.init === node);
}

/**
 Checks whether a variable declaration occupies a `for-in` or `for-of` left slot.
 
 @param params - variable declaration and parent node
 
 @returns whether declaration is the left side of a for-in/of statement
 
 @example
 ```ts
 if (isForInOrOfLeft({ node, parent })) return false;
 ```
 */
function isForInOrOfLeft(params: ForeignBorrowed<Readonly<IsForInOrOfLeftParams>>,): boolean {
  /**
   Variable declaration being checked.
   */
  const { node, } = params;
  /**
   Parent node that may be a for-in/of statement.
   */
  const { parent, } = params;
  return FOR_LEFT_PARENT_TYPES.has(parent.type,) && (parent.left === node);
}
//endregion Token lookup and loop-slot exclusions

//region Per-node checks
// Whether one node kind needs a terminator at all, asked before any token is
// looked up.

/**
 Decides whether a variable declaration should have a statement terminator.
 
 `for` initializer and for-in/of left declarations are excluded because
 those grammar positions use loop separators rather than statement-ending
 semicolons.
 
 @param node - variable declaration being checked
 
 @returns whether semi should inspect declaration
 
 @example
 ```ts
 if (shouldCheckVariableDeclaration(node)) checkForSemicolon(node);
 ```
 */
function shouldCheckVariableDeclaration(node: ForeignBorrowed<Node>,): boolean {
  /**
   Parent link from Oxlint's AST, narrowed to the loop fields this rule reads.
   */
  const { parent, } = node as NodeWithParent;
  if (parent === undefined)
    return true;

  if (isForStatementInitializer({
    node,
    parent,
  },))
    return false;

  if (isForInOrOfLeft({
    node,
    parent,
  },))
    return false;

  return true;
}

/**
 Decides whether an export-default declaration should have a semicolon.
 
 Class declarations, function declarations, and TypeScript interfaces are
 self-terminating in `export default` form. Expression exports such as
 `export default value` remain statement-like and require `;`.
 
 @param node - export-default declaration being checked
 
 @returns whether semi should inspect declaration
 
 @example
 ```ts
 if (shouldCheckExportDefaultDeclaration(node)) checkForSemicolon(node);
 ```
 */
function shouldCheckExportDefaultDeclaration(node: ForeignBorrowed<Node>,): boolean {
  /**
   Export-default declaration payload, narrowed to its discriminant.
   */
  // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- oxlint Node omits declaration fields exposed by this visitor node
  const { declaration, } = node as ExportDefaultDeclarationNode;

  return !EXPORT_DEFAULT_DECLARATION_SEMI_EXEMPT_TYPES.has(
    declaration.type,
  );
}
//endregion Per-node checks

//region Reporting
// Report a missing terminator, and insert it after the node's last token.

/**
 Checks a semicolon-terminated node and reports when its last token is not `;`.
 
 @param params - rule context and statement-like AST node
 
 @example
 ```ts
 checkForSemicolon({ context, node });
 ```
 
 @mutates params - Emits Oxlint diagnostics through params.context.
 */
function checkForSemicolon(params: ForeignBorrowed<Readonly<CheckForSemicolonParams>>,): void {
  /**
   Rule context with token lookup and reporting APIs.
   */
  const { context, } = params;
  /**
   Statement-like AST node being checked.
   */
  const { node, } = params;
  /**
   Last syntax token determines whether the node is already terminated.
   */
  const lastToken = lastTokenOf({
    context,
    node,
  },);
  if (lastToken.value === SEMICOLON)
    return;

  context.report({
    node,
    messageId: 'missingSemi',
    fix(fixer: ForeignBorrowed<Fixer>,): ReturnType<Fixer['insertTextAfter']> {
      return fixer.insertTextAfter(
        lastToken,
        SEMICOLON,
      );
    },
  },);
}
//endregion Reporting

//region Rule
// The exported rule: metadata, one binding per visitor concern, and the map
// from node kind to check.

/**
 Requires semicolons at the end of statement-like declarations and expressions.
 
 This mirrors the `@stylistic/semi` rule's default `"always"` mode without
 exposing any configuration: enabling `stylistic/semi` always requires
 explicit semicolons, does not support `"never"`, and does not support
 omit-last-in-one-line options.
 
 @example
 ```ts
 // Bad
 const value = 1
 export default value
 
 // Good
 const value = 1;
 export default value;
 ```
 */
export const semi: CreateOnceRule = {
  meta: {
    type: 'layout',
    fixable: 'code',
    schema: [],
    docs: {
      description: 'Require semicolons instead of ASI.',
      recommended: true,
    },
    messages: {
      missingSemi: 'Missing semicolon.',
    },
  },
  createOnce(context: ForeignBorrowed<Context>,): VisitorWithHooks {
    /**
     Helper bound to this rule context for visitor callbacks.
     
     @param node - statement-like AST node being checked
     */
    function check(node: ForeignBorrowed<Node>,): void {
      checkForSemicolon({
        context,
        node,
      },);
    }

    /**
     VariableDeclaration visitor skips loop header declarations.
     
     @param node - variable declaration being checked
     */
    function checkVariableDeclaration(node: ForeignBorrowed<Node>,): void {
      if (!shouldCheckVariableDeclaration(node,))
        return;

      check(node,);
    }

    /**
     ExportNamedDeclaration visitor skips declarations that terminate themselves.
     
     @param node - export-named declaration being checked
     */
    function checkExportNamedDeclaration(node: ForeignBorrowed<Node>,): void {
      /**
       Export-named declaration payload; null for `export { value }`.
       */
      const { declaration, } = node as ExportNamedDeclarationNode;
      if (declaration === null) {
        check(node,);
        return;
      }

      if (declaration === undefined)
        check(node,);
    }

    /**
     ExportDefaultDeclaration visitor skips class/function/interface declarations.
     
     @param node - export-default declaration being checked
     */
    function checkExportDefaultDeclaration(node: ForeignBorrowed<Node>,): void {
      if (!shouldCheckExportDefaultDeclaration(node,))
        return;

      check(node,);
    }

    return {
      VariableDeclaration: checkVariableDeclaration,
      ExpressionStatement: check,
      ReturnStatement: check,
      ThrowStatement: check,
      DoWhileStatement: check,
      DebuggerStatement: check,
      BreakStatement: check,
      ContinueStatement: check,
      ImportDeclaration: check,
      ExportAllDeclaration: check,
      ExportNamedDeclaration: checkExportNamedDeclaration,
      ExportDefaultDeclaration: checkExportDefaultDeclaration,
      PropertyDefinition: check,
      TSAbstractPropertyDefinition: check,
      AccessorProperty: check,
      TSAbstractAccessorProperty: check,
      TSDeclareFunction: check,
      TSEmptyBodyFunctionExpression: check,
      TSExportAssignment: check,
      TSImportEqualsDeclaration: check,
      TSTypeAliasDeclaration: check,
    };
  },
};
//endregion Rule
