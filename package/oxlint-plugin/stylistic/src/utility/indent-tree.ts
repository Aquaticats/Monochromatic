import {
  at,
} from './range.ts';

//region Structural AST shapes
// Structural node/token kinds and record types the indentation model walks.
// Shaped accessors keep this module independent of the oxlint AST classes so
// unit tests can build stub trees as plain objects.

/**
 Structural AST node shape the indentation model walks: a kind tag plus the
 source span whose offsets order the walk.
 */
export type ExpectationNode = {
  readonly type: string;
  readonly start: number;
  readonly end: number;
};

/**
 Structural token shape the indentation model reads: a kind tag, its source
 text, and its span.
 */
export type ExpectationToken = {
  readonly type: string;
  readonly value: string;
  readonly start: number;
  readonly end: number;
};

/**
 One visited node with its parent's record index.
 
 Pre-order records stay sorted by `start`, which lets the token-owner merge
 walk records and tokens together in one forward pass.
 */
export type IndentNodeRecord = {
  readonly node: ExpectationNode;
  readonly parentIndex: number;
};
//endregion Structural AST shapes

//region Node collection
// Pre-order record collection and child enumeration via visitor keys.

/**
 Parameters for {@link collectIndentNodeRecords}.
 */
export type CollectIndentNodeRecordsParams = {
  readonly root: ExpectationNode;
  readonly visitorKeys: Readonly<Record<string, readonly string[]>>;
};

/**
 Narrows a property value to an AST node by shape.
 
 Shape checks keep metadata fields (`loc`, token arrays) out of the walk even
 when a visitor-key list names a property that holds them.
 
 @param value - property value read from a visited node
 
 @returns whether the value carries the {@link ExpectationNode} fields
 
 @example
 ```ts
 isExpectationNode({ type: 'Identifier', start: 0, end: 1 }); // true
 ```
 */
function isExpectationNode(value: unknown,): value is ExpectationNode {
  return ((typeof value) === 'object')
    && (value !== null)
    && ((typeof Reflect.get(
      value,
      'type',
    )) === 'string')
    && ((typeof Reflect.get(
      value,
      'start',
    )) === 'number')
    && ((typeof Reflect.get(
      value,
      'end',
    )) === 'number');
}

/**
 Parameters for {@link childNodesOf}.
 */
export type ChildNodesOfParams = {
  readonly node: ExpectationNode;
  readonly visitorKeys: Readonly<Record<string, readonly string[]>>;
};

/**
 Collects the child nodes `visitorKeys` names for `node`, in property order.
 
 @param node - node whose child slots are enumerated
 
 @param visitorKeys - visitor-key map naming each node kind's child properties
 
 @returns child nodes in parser source order
 
 @example
 ```ts
 childNodesOf({ node: root, visitorKeys }); // statement nodes of `root.body`
 ```
 */
export function childNodesOf({
  node,
  visitorKeys,
}: ChildNodesOfParams,): readonly ExpectationNode[] {
  /**
   Child property names this node kind declares; kinds without an entry have
   no child slots.
   */
  const keys = visitorKeys[node.type] ?? [];
  /**
   Child nodes collected so far, in property order.
   */
  const children: ExpectationNode[] = [];
  for (const key of keys) {
    /**
     Value of one visitor-key property; only node shapes join the walk, so
     metadata such as `loc` and token arrays stay out.
     */
    const value: unknown = Reflect.get(
      node,
      key,
    );
    if (Array.isArray(value,)) {
      for (const item of value) {
        if (isExpectationNode(item,))
          children.push(item,);
      }
      continue;
    }
    if (isExpectationNode(value,))
      children.push(value,);
  }
  return children;
}

/**
 Walks the AST from `root` in pre-order using `visitorKeys`.
 
 Parents always precede children and siblings keep parser order, which the
 token-owner merge relies on. The walk holds an explicit stack of open nodes
 instead of recursing, keeping stack depth constant on deeply nested input.
 
 @param root - node the walk starts from; record `0` is this node
 
 @param visitorKeys - visitor-key map naming each node kind's child properties
 
 @returns node records; record `0` is `root`
 
 @example
 ```ts
 const records = collectIndentNodeRecords({ root: ast, visitorKeys });
 ```
 */
export function collectIndentNodeRecords({
  root,
  visitorKeys,
}: CollectIndentNodeRecordsParams,): readonly IndentNodeRecord[] {
  /**
   Records in pre-order: the root first, then each child after its parent.
   */
  const records: IndentNodeRecord[] = [{
    node: root,
    parentIndex: -1,
  },];
  /**
   Open-node frames with child cursors; the stack replaces recursion in the
   structural walk.
   */
  const openFrames: {
    recordIndex: number;
    children: readonly ExpectationNode[];
    childIndex: number;
  }[] = [{
    recordIndex: 0,
    children: childNodesOf({
      node: root,
      visitorKeys,
    },),
    childIndex: 0,
  },];
  while (openFrames.length > 0) {
    /**
     Innermost open frame; its cursor decides whether to descend or pop.
     */
    const frame = at({
      arr: openFrames,
      index: openFrames.length - 1,
    },);
    if (frame.childIndex
      >= frame.children
        .length) {
      openFrames.pop();
      continue;
    }
    /**
     Child the frame's cursor points at; its record is appended next.
     */
    const child = at({
      arr: frame.children,
      index: frame.childIndex,
    },);
    frame.childIndex += 1;
    /**
     Record index the child takes, which is the current record count.
     */
    const recordIndex = records.length;
    records.push({
      node: child,
      parentIndex: frame.recordIndex,
    },);
    openFrames.push({
      recordIndex,
      children: childNodesOf({
        node: child,
        visitorKeys,
      },),
      childIndex: 0,
    },);
  }
  return records;
}
//endregion Node collection

//region Template spans
// Template-literal source bounds; interior lines stay exempt from checks.

/**
 Collects source spans of template literals whose content lines stay exempt.
 
 Only expression-position templates (`TemplateLiteral`) are spanned; their
 interior lines carry template tokens rather than statements.
 
 @param records - pre-order node records to scan
 
 @returns `[start, end)` span per template literal, in source order
 
 @example
 ```ts
 const spans = collectTemplateSpans(records);
 ```
 */
export function collectTemplateSpans(
  records: readonly IndentNodeRecord[],
): readonly (readonly [
  number,
  number
])[] {
  /**
   Accumulated spans, appended in record order.
   */
  const spans: (readonly [
    number,
    number
  ])[] = [];
  for (const record of records) {
    if (record.node
      .type
      === 'TemplateLiteral') {
      spans.push([
        record.node
          .start,
        record.node
          .end,
      ],);
    }
  }
  return spans;
}
//endregion Template spans

//region Token ownership
// Mapping each token to the AST record that owns it, with ESTree body
// wrappers collapsed onto their declarations.

/**
 ESTree body-wrapper kinds that start at their `{` while the declaration
 starts earlier. Collapsing each wrapper onto its declaration reproduces the
 flatter AST shape the indentation model was measured against, where the
 declaration owns the braces directly.
 */
const DECL_BODY_WRAPPER_KINDS = new Set([
  'ClassBody',
  'TSEnumBody',
  'TSInterfaceBody',
]);

/**
 Parameters for {@link effectiveRecordIndexOf}.
 */
export type EffectiveRecordIndexOfParams = {
  readonly records: readonly IndentNodeRecord[];
  readonly recordIndex: number;
};

/**
 Collapses body-wrapper records onto their declaration records.
 
 Class, interface, and enum bodies hang between the declaration and its
 members here, while the indentation model anchors braces and members at the
 declaration line directly.
 
 @param records - pre-order records whose ancestry is being collapsed
 
 @param recordIndex - candidate record index, `-1` for the no-record sentinel
 
 @returns `recordIndex` itself, or its nearest non-wrapper ancestor index
 
 @example
 ```ts
 const ownerIndex = effectiveRecordIndexOf({ records, recordIndex: bodyIndex });
 ```
 */
export function effectiveRecordIndexOf({
  records,
  recordIndex,
}: EffectiveRecordIndexOfParams,): number {
  /**
   Record the walk is testing; each wrapper step moves it one parent higher.
   */
  let current = recordIndex;
  while (current >= 0) {
    /**
     Current record; its kind decides whether the walk collapses it upward.
     */
    const record = at({
      arr: records,
      index: current,
    },);
    if (!DECL_BODY_WRAPPER_KINDS.has(record.node
      .type))
      return current;
    current = record.parentIndex;
  }
  return current;
}

/**
 Parameters for {@link tokenOwnerRecordIndexes}.
 */
export type TokenOwnerRecordIndexesParams = {
  readonly records: readonly IndentNodeRecord[];
  readonly tokens: readonly ExpectationToken[];
};

/**
 Maps each token to the record index of its innermost owning node, with body
 wrappers collapsed to their declarations.
 
 One merge pass over two offset-sorted lists: O(tokens + nodes) time and
 O(tokens) space, keeping lint hot paths linear.
 
 @param records - offset-sorted records, parents always before children
 
 @param tokens - offset-sorted tokens to map onto their owning records
 
 @returns owning record index per token, `-1` when no record contains it
 
 @example
 ```ts
 const owners = tokenOwnerRecordIndexes({ records, tokens });
 ```
 */
export function tokenOwnerRecordIndexes({
  records,
  tokens,
}: TokenOwnerRecordIndexesParams,): readonly number[] {
  /**
   Owning record index per token, appended in token order as the merge advances.
   */
  const owners: number[] = [];
  /**
   Record indices whose spans currently contain the merge cursor.
   */
  const containmentStack: number[] = [];
  /**
   End offset of the innermost containing record; call sites guard stack
   emptiness first, and `at` throws if they ever fail to.
   
   @returns source end offset of the top containment record
   */
  function topRecordEnd(): number {
    /**
     Index of the containment record whose end bounds the merge cursor.
     */
    const topIndex = at({
      arr: containmentStack,
      index: containmentStack.length - 1,
    },);
    return at({
      arr: records,
      index: topIndex,
    },)
      .node
      .end;
  }
  /**
   First record not yet pushed onto the containment stack; records are
   consumed in start order, one forward sweep.
   */
  let recordCursor = 0;
  for (
    let tokenIndex = 0;
    tokenIndex < tokens.length;
    tokenIndex += 1
  ) {
    /**
     Offset of the token being assigned, which record spans are compared to.
     */
    const tokenStart = at({
      arr: tokens,
      index: tokenIndex,
    },)
      .start;
    while (
      (recordCursor < records.length)
        && (at({
          arr: records,
          index: recordCursor,
        },)
          .node
          .start
          <= tokenStart)
    ) {
      while (
        (containmentStack.length > 0)
          && (topRecordEnd() <= tokenStart)
      ) {
        containmentStack.pop();
      }
      containmentStack.push(recordCursor);
      recordCursor += 1;
    }
    while (
      (containmentStack.length > 0)
        && (topRecordEnd() <= tokenStart)
    ) {
      containmentStack.pop();
    }
    /**
     Innermost record containing the token, `-1` when the stack is empty.
     */
    const innermostIndex = (containmentStack.length > 0)
      ? at({
        arr: containmentStack,
        index: containmentStack.length - 1,
      },)
      : -1;
    owners.push(effectiveRecordIndexOf({
      records,
      recordIndex: innermostIndex,
    },),);
  }
  return owners;
}
//endregion Token ownership
