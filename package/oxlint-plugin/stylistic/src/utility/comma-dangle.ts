import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import type {
  Context,
  Fixer,
  Node,
  Token,
} from '@oxlint/plugins';

/**
 Token value required at list tails.
 */
const COMMA = ',';

/**
 Sentinel for absent AST node values without nullish unions.
 */
export const NO_NODE: unique symbol = Symbol('oxlint-stylistic:comma-dangle:no-node',);

/**
 Sentinel for absent token lookups without nullish unions.
 */
const NO_TOKEN: unique symbol = Symbol('oxlint-stylistic:comma-dangle:no-token',);

/**
 Closing delimiters that may follow comma-delimited lists.
 */
const CLOSE_DELIMITERS: ReadonlySet<string> = new Set([
  ')',
  ']',
  '}',
  '>',
],);

/**
 AST node or absence sentinel accepted by comma-dangle checks.
 */
export type OptionalNode = Node | typeof NO_NODE;

/**
 Object value with string-keyed unknown fields.
 */
type FieldRecord = Record<string, unknown>;

/**
 Token view needed for delimiter checks.
 */
type TokenValue = {
  /**
   Token text.
   */
  readonly value: string;
};

/**
 Parameters for trailing comma checks.
 */
export type CheckTrailingCommaParams = {
  /**
   Rule context used for token lookup and reporting.
   */
  readonly context: Context;
  /**
   Container node whose closing delimiter bounds the list.
   */
  readonly container: Node;
  /**
   Last concrete item in the list, or sentinel for empty lists.
   */
  readonly lastItem: OptionalNode;
  /**
   Whether token before container close delimiter supplies insertion point.
   */
  readonly useContainerPenultimateToken?: boolean;
};

/**
 Parameters for last field node lookup.
 */
type LastFieldNodeParams = {
  /**
   Node owning list field.
   */
  readonly node: Node;
  /**
   Field name whose array value supplies list items.
   */
  readonly fieldName: string;
};

/**
 Parameters for following token lookup.
 */
type TokenAfterParams = {
  /**
   Rule context used for token lookup.
   */
  readonly context: Context;
  /**
   Node or token anchoring lookup.
   */
  readonly target: Node | Readonly<Token>;
};

/**
 Parameters for last token lookup.
 */
type LastTokenParams = {
  /**
   Rule context used for token lookup.
   */
  readonly context: Context;
  /**
   Node whose last token is requested.
   */
  readonly node: Node;
};

/**
 Parameters for container-tail token lookup.
 */
type ContainerPenultimateTokenParams = {
  /**
   Rule context used for token lookup.
   */
  readonly context: Context;
  /**
   Container node whose closing delimiter ends the list.
   */
  readonly container: Node;
};

/**
 Parameters for insertion token lookup.
 */
type InsertionTokenParams = {
  /**
   Rule context used for token lookup.
   */
  readonly context: Context;
  /**
   Container node whose close delimiter may bound insertion.
   */
  readonly container: Node;
  /**
   Last concrete item in list.
   */
  readonly lastItem: Node;
  /**
   Whether token before close delimiter supplies insertion point.
   */
  readonly useContainerPenultimateToken: boolean;
};

/**
 Reports whether a value is an object with readable fields.
 
 @param value - candidate runtime value
 
 @returns whether value exposes string-keyed fields
 
 @example
 ```ts
 if (!isFieldRecord(value)) return;
 ```
 */
function isFieldRecord(value: unknown,): value is FieldRecord {
  return ((typeof value) === 'object') && (value !== null);
}

/**
 Returns readable fields from an oxlint AST node.
 
 @param node - visitor node whose structural fields are needed
 
 @returns node viewed as string-keyed runtime fields
 
 @example
 ```ts
 const fields = fieldsOf(node);
 ```
 */
function fieldsOf(node: ForeignBorrowed<Node>,): FieldRecord {
  // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- oxlint visitor nodes expose runtime fields that generic Node omits
  return node as unknown as FieldRecord;
}

/**
 Returns node-type discriminant when present.
 
 @param node - oxlint AST node
 
 @returns node type, or empty string for malformed stubs
 
 @example
 ```ts
 if (nodeType(node) === 'RestElement') return;
 ```
 */
function nodeType(node: ForeignBorrowed<Node>,): string {
  /**
   Runtime `type` field from oxlint visitor node.
   */
  const { type, } = fieldsOf(node,);
  return ((typeof type) === 'string') ? type : '';
}

/**
 Reports whether a value looks like an Oxlint AST node.
 
 @param value - candidate runtime value
 
 @returns whether value carries node discriminant and range offsets
 
 @example
 ```ts
 if (isNode(value)) return value;
 ```
 */
function isNode(value: unknown,): value is Node {
  if (!isFieldRecord(value,))
    return false;

  return (((typeof value.type) === 'string')
    && ((typeof value.start) === 'number'))
    && ((typeof value.end) === 'number');
}

/**
 Reports whether a node is a JavaScript rest element.
 
 @param node - candidate list tail
 
 @returns whether trailing comma would be invalid after node
 
 @example
 ```ts
 if (isRestElement(lastItem)) return;
 ```
 */
function isRestElement(node: ForeignBorrowed<Node>,): boolean {
  return nodeType(node,) === 'RestElement';
}

/**
 Reports whether a token closes a comma-delimited list.
 
 @param token - token after insertion point
 
 @returns whether token is one of `)`, `]`, `}`, or `>`
 
 @example
 ```ts
 if (!isCloseDelimiter(token)) return;
 ```
 */
function isCloseDelimiter(token: TokenValue,): boolean {
  return CLOSE_DELIMITERS.has(token.value,);
}

/**
 Reports whether a node is a named import specifier.
 
 @param value - candidate specifier value
 
 @returns whether value represents `ImportSpecifier`
 
 @example
 ```ts
 const named = specifiers.filter(function keep(specifier) { return isImportSpecifier(specifier); });
 ```
 */
function isImportSpecifier(value: unknown,): value is Node {
  if (!isNode(value,))
    return false;

  return nodeType(value,) === 'ImportSpecifier';
}

/**
 Returns last AST node in a possibly sparse list.
 
 Holes in array expressions and patterns arrive as `null`; malformed or absent
 values are skipped through the runtime node guard.
 
 @param items - list-like values from oxlint AST
 
 @returns last concrete node, or sentinel when list is empty
 
 @example
 ```ts
 const lastItem = lastNode(node.elements);
 ```
 */
function lastNode(items?: readonly unknown[],): OptionalNode {
  if (items === undefined)
    return NO_NODE;

  for (let loopIndex = items.length - 1; loopIndex >= 0; loopIndex -= 1) {
    /**
     Candidate item from reverse scan.
     */
    const item = items[loopIndex];
    if (isNode(item,))
      return item;
  }

  return NO_NODE;
}

/**
 Returns last node from a named array field.
 
 @param node - owner node
 
 @param fieldName - field whose array value supplies items
 
 @returns last concrete node, or sentinel when field is absent or empty
 
 @example
 ```ts
 const lastParam = lastFieldNode({ node, fieldName: 'params' });
 ```
 */
export function lastFieldNode({
  node,
  fieldName,
}: ForeignBorrowed<LastFieldNodeParams>,): OptionalNode {
  /**
   Runtime field value, expected to be an array for supported lists.
   */
  const value = fieldsOf(node,)[fieldName];
  if (!Array.isArray(value,))
    return NO_NODE;

  return lastNode(value,);
}

/**
 Returns last named import specifier in an import declaration.
 
 Default and namespace specifiers are outside braces and do not form the named
 comma list this rule enforces.
 
 @param node - import declaration node
 
 @returns last named import specifier, or sentinel when none exists
 
 @example
 ```ts
 const lastSpecifier = lastNamedImportSpecifier(node);
 ```
 */
export function lastNamedImportSpecifier(node: ForeignBorrowed<Node>,): OptionalNode {
  /**
   Import declaration specifier field.
   */
  const { specifiers, } = fieldsOf(node,);
  if (!Array.isArray(specifiers,))
    return NO_NODE;

  /**
   Named specifiers enclosed by import braces.
   */
  const namedSpecifiers = specifiers.filter(function keepNamedSpecifier(
    specifier,
  ): boolean {
    return isImportSpecifier(specifier,);
  },);
  return lastNode(namedSpecifiers,);
}

/**
 Returns last enum member from an enum declaration.
 
 @param node - enum declaration node
 
 @returns last enum member, or sentinel when enum body is empty
 
 @example
 ```ts
 const lastMember = lastEnumMember(node);
 ```
 */
export function lastEnumMember(node: ForeignBorrowed<Node>,): OptionalNode {
  /**
   Enum body value carrying members in oxlint AST.
   */
  const { body, } = fieldsOf(node,);
  if (!isFieldRecord(body,))
    return NO_NODE;

  /**
   Enum member list from body.
   */
  const { members, } = body;
  if (!Array.isArray(members,))
    return NO_NODE;

  return lastNode(members,);
}

/**
 Returns dynamic import item that occupies final argument position.
 
 `import(source, options)` checks `options`; `import(source)` checks `source`.
 
 @param node - dynamic import expression node
 
 @returns source or options node, or sentinel for malformed AST
 
 @example
 ```ts
 const lastItem = lastImportExpressionItem(node);
 ```
 */
export function lastImportExpressionItem(node: ForeignBorrowed<Node>,): OptionalNode {
  /**
   Dynamic import options expression, present only for two-argument imports.
   */
  const { options, } = fieldsOf(node,);
  if (isNode(options,))
    return options;

  /**
   Dynamic import source expression.
   */
  const { source, } = fieldsOf(node,);
  if (isNode(source,))
    return source;

  return NO_NODE;
}

/**
 Returns token after a node or token, or sentinel when absent.
 
 @param params - context and target for lookup
 
 @returns following syntax token, or sentinel
 
 @example
 ```ts
 const next = tokenAfter({ context, target: node });
 ```
 */
function tokenAfter(params: ForeignBorrowed<Readonly<TokenAfterParams>>,): Token | typeof NO_TOKEN {
  /**
   Rule context and lookup target.
   */
  const {
    context,
    target,
  } = params;
  /**
   Following syntax token, absent at file end.
   */
  const token = context.sourceCode
    .getTokenAfter(target,);
  return token ?? NO_TOKEN;
}

/**
 Returns last token in a node, or sentinel when absent.
 
 @param params - context and node for lookup
 
 @returns last syntax token, or sentinel
 
 @example
 ```ts
 const token = lastToken({ context, node });
 ```
 */
function lastToken(params: ForeignBorrowed<Readonly<LastTokenParams>>,): Token | typeof NO_TOKEN {
  /**
   Rule context and node.
   */
  const {
    context,
    node,
  } = params;
  /**
   Last syntax token in node, absent for malformed stubs.
   */
  const token = context.sourceCode
    .getLastToken(node,);
  return token ?? NO_TOKEN;
}

/**
 Returns token before a container's closing delimiter.
 
 Binding patterns carry type annotations whose node ranges extend past the
 pattern's own brace, so a trailing annotation is skipped to reach the list
 close delimiter the comma actually belongs to.
 
 @param params - context and container for lookup
 
 @returns penultimate container token, or sentinel
 
 @example
 ```ts
 const token = containerPenultimateToken({ context, container });
 ```
 */
function containerPenultimateToken(
  params: ForeignBorrowed<Readonly<ContainerPenultimateTokenParams>>,
): Token | typeof NO_TOKEN {
  /**
   Rule context and container node.
   */
  const {
    context,
    container,
  } = params;
  /**
   Type annotation node extending a binding pattern's range, when present.
   */
  const { typeAnnotation, } = fieldsOf(container,);
  /**
   Container close delimiter: the pattern's own brace when an annotation follows, else the container tail.
   */
  const closeToken = isNode(typeAnnotation,)
    ? context.sourceCode
      .getTokenBefore(typeAnnotation,)
    : context.sourceCode
      .getLastToken(container,);
  if (closeToken === null)
    return NO_TOKEN;
  /**
   Token before close delimiter, absent for malformed stubs.
   */
  const token = context.sourceCode
    .getTokenBefore(closeToken,);
  return token ?? NO_TOKEN;
}

/**
 Chooses token that should receive inserted comma.
 
 Container-tail lists use token before close delimiter so trailing array holes
 keep their existing comma. Non-tail lists inspect token after last item because
 import/export declarations span beyond their brace or attribute list.
 
 @param context - rule context for token lookup
 
 @param container - container node whose closing delimiter may bound insertion
 
 @param lastItem - last concrete list item
 
 @param useContainerPenultimateToken - whether container-tail strategy applies
 
 @returns insertion token, comma token when already present, or sentinel
 
 @example
 ```ts
 const insertionToken = insertionTokenForList({ context, container, lastItem, useContainerPenultimateToken: true });
 ```
 */
function insertionTokenForList({
  context,
  container,
  lastItem,
  useContainerPenultimateToken,
}: ForeignBorrowed<InsertionTokenParams>,): Token | typeof NO_TOKEN {
  if (useContainerPenultimateToken) {
    /**
     Token immediately before container close delimiter.
     */
    const insertionToken = containerPenultimateToken({
      context,
      container,
    },);
    if (insertionToken === NO_TOKEN)
      return NO_TOKEN;
    if (insertionToken.value === COMMA)
      return insertionToken;
    /**
     Token after insertion point must be close delimiter.
     */
    const boundaryToken = tokenAfter({
      context,
      target: insertionToken,
    },);
    if ((boundaryToken === NO_TOKEN) || (!isCloseDelimiter(boundaryToken,)))
      return NO_TOKEN;
    return insertionToken;
  }

  /**
   Token after last item determines whether comma already exists.
   */
  const boundaryToken = tokenAfter({
    context,
    target: lastItem,
  },);
  if (boundaryToken === NO_TOKEN)
    return NO_TOKEN;
  if (boundaryToken.value === COMMA)
    return boundaryToken;
  if (!isCloseDelimiter(boundaryToken,))
    return NO_TOKEN;

  return lastToken({
    context,
    node: lastItem,
  },);
}

/**
 Reports and fixes a missing trailing comma for one list.
 
 @param context - rule context for token lookup and reporting
 
 @param container - container node whose closing delimiter bounds list
 
 @param lastItem - list tail or absence sentinel
 
 @param useContainerPenultimateToken - whether container-tail strategy applies
 
 @example
 ```ts
 checkTrailingComma({ context, container: node, lastItem });
 ```
 
 @mutates context - Emits Oxlint diagnostics through foreign rule context.
 */
export function checkTrailingComma({
  context,
  container,
  lastItem,
  useContainerPenultimateToken = false,
}: ForeignBorrowed<CheckTrailingCommaParams>,): void {
  if (lastItem === NO_NODE)
    return;
  if (isRestElement(lastItem,))
    return;

  /**
   Token that already is a comma or should receive one.
   */
  const insertionToken = insertionTokenForList({
    context,
    container,
    lastItem,
    useContainerPenultimateToken,
  },);
  if (insertionToken === NO_TOKEN)
    return;
  if (insertionToken.value === COMMA)
    return;

  context.report({
    node: lastItem,
    messageId: 'missingComma',
    fix(fixer: ForeignBorrowed<Fixer>,): ReturnType<Fixer['insertTextAfter']> {
      return fixer.insertTextAfter(
        insertionToken,
        COMMA,
      );
    },
  },);
}
