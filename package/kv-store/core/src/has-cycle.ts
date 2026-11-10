//region Cycle detection walks one root-to-node path at a time

/**
 Marker property keying the traversal entry that closes a node's subtree.
 A module-private symbol keeps a real value's own property from being read as one.
 */
const SUBTREE_COMPLETE: unique symbol = Symbol('kv-store/has-cycle subtree complete',);

/**
 Traversal stack entry pushed under a node's children, popped once every child
 has been walked, which is when the node leaves the ancestor path.
 */
type SubtreeComplete = { readonly [SUBTREE_COMPLETE]: object; };

/**
 Narrow a traversal stack entry to the marker that closes a node's subtree.
 
 @param entry - value popped from the traversal stack
 
 @returns whether `entry` is the close marker, which carries the node whose subtree ended
 */
function isSubtreeComplete(entry: unknown,): entry is SubtreeComplete {
  return ((typeof entry) === 'object')
    && (entry !== null)
    && (SUBTREE_COMPLETE in entry);
}

//endregion

/**
 Detect whether a value contains circular references.
 
 A reference counts as a cycle only when it points back at an ancestor of the
 node being walked, so a value whose properties share one subobject stays
 acyclic and keeps that sharing. Uses an explicit traversal stack, so
 arbitrarily deep values cannot overflow the call stack. Returns `false`
 immediately for non-object / `null` inputs.
 
 Internal helper for {@link serializeValue}; not part of the package public API.
 
 @param value - value to inspect for cyclic references
 
 @returns `true` when value contains at least one cycle
 
 @example
 Cyclic object detection:
 ```ts
 const obj: Record<string, unknown> = {};
 obj.self = obj;
 hasCycle(obj); // true
 ```
 
 @example
 Acyclic values, including shared references:
 ```ts
 const shared = { a: 1 };
 hasCycle({ a: 1, b: { c: 2 } }); // false
 hasCycle({ x: shared, y: shared }); // false
 hasCycle(42); // false
 hasCycle(null); // false
 ```
 */
export function hasCycle(value: unknown,): boolean {
  if (((typeof value) !== 'object') || (value === null))
    return false;

  /**
   Objects on the path from the root to the node being walked.
   A back-edge into this set is a cycle; a reference to an object that was
   already walked and left is a shared reference, which an acyclic value has.
   */
  const path = new WeakSet();

  /**
   Depth-first traversal stack: nodes still to walk, interleaved with the
   {@link SUBTREE_COMPLETE} markers that pop their node back off {@link path}.
   */
  const stack: unknown[] = [value,];

  while (stack.length
    > 0) {
    /**
     Next node of the depth-first walk, or the marker closing an ancestor's subtree.
     */
    const entry = stack.pop();

    if (isSubtreeComplete(entry,)) {
      path.delete(entry[SUBTREE_COMPLETE],);
      continue;
    }

    if (((typeof entry) !== 'object') || (entry === null))
      continue;
    if (path.has(entry,))
      return true;

    path.add(entry,);
    stack.push({ [SUBTREE_COMPLETE]: entry, },);
    for (const child of Object.values(entry,))
      stack.push(child,);
  }

  return false;
}
