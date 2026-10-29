import {
  isRecord,
  type ReadonlyRecord,
} from '@monochromatic-dev/oxlint-plugin-shared/ts';

/**
 Node types that open their own `this`, `arguments`, `super`, and `new.target`,
 so nothing inside them reads the enclosing arrow's bindings.
 */
const REBINDING_FUNCTION_TYPES: ReadonlySet<unknown> = new Set([
  'FunctionDeclaration',
  'FunctionExpression',
  'TSDeclareFunction',
  'TSEmptyBodyFunctionExpression',
],);

/**
 Node types whose non-computed `key` holds a plain name, not an expression.
 */
const KEYED_MEMBER_TYPES: ReadonlySet<unknown> = new Set([
  'Property',
  'MethodDefinition',
  'PropertyDefinition',
],);

/**
 Sentinel naming no field; AST fields are never the empty string, so every field is walked.
 */
const NO_SKIPPED_FIELD = '';

/**
 Reads the key whose child is a plain name, not an expression, so it is never walked.

 Non-computed member properties and non-computed object or class keys hold names
 such as `arguments` that do not refer to the enclosing function's bindings.

 @param node - AST record being walked

 @returns the field to skip, or {@link NO_SKIPPED_FIELD} when every field is walked

 @example
 ```ts
 nameFieldOf({ type: 'MemberExpression', computed: false, },); // 'property'
 ```
 */
function nameFieldOf(node: ReadonlyRecord,): string {
  if (node.computed !== false) {
    return NO_SKIPPED_FIELD;
  }
  if (node.type === 'MemberExpression') {
    return 'property';
  }
  return KEYED_MEMBER_TYPES.has(node.type,) ? 'key' : NO_SKIPPED_FIELD;
}

/**
 Whether an AST record on its own reads an enclosing function's bindings.

 @param node - AST record being walked

 @returns whether `node` is `this`, `super`, `new.target`, or `arguments`

 @example
 ```ts
 isEnclosingBindingRead({ type: 'ThisExpression', },); // true
 ```
 */
function isEnclosingBindingRead(node: ReadonlyRecord,): boolean {
  if ((node.type === 'ThisExpression') || (node.type === 'Super')) {
    return true;
  }
  if (node.type === 'MetaProperty') {
    /**
     The `new` half of a `new.target` meta property; `import.meta` carries `import` instead.
     */
    const { meta, } = node;
    return isRecord(meta,) && (meta.name === 'new');
  }
  return (node.type === 'Identifier') && (node.name === 'arguments');
}

/**
 Whether an arrow function reads `this`, `arguments`, `super`, or `new.target` of its enclosing function.

 An arrow inherits those bindings and a `function` rebinds them, so such an arrow
 has no behavior-preserving function rewrite: `.bind(this)` keeps the runtime value
 but fails strict type-checking (TS2683) unless a `this` parameter is declared, and
 the enclosing `this` type cannot be named in general. Nested arrows are walked
 because they inherit too; nested non-arrow functions are skipped because they
 rebind. A class body is skipped except for computed member keys, which are evaluated in
 the enclosing scope.

 @param arrow - arrow function AST node; inspected as untyped data at runtime

 @returns whether the arrow depends on a binding its enclosing function owns

 @example
 ```ts
 readsEnclosingBinding({ type: 'ArrowFunctionExpression', params: [], body: { type: 'ThisExpression', }, },); // true
 ```
 */
export function readsEnclosingBinding(arrow: unknown,): boolean {
  /**
   Work stack of values still to inspect; a stack keeps deeply nested bodies off the call stack.
   */
  const pending: unknown[] = [arrow,];
  while (pending.length > 0) {
    /**
     Value taken off the work stack: an AST record, an array of them, or a leaf such as a string.
     */
    const current = pending.pop();
    if (Array.isArray(current,)) {
      for (const element of current) {
        pending.push(element,);
      }
      continue;
    }
    if (!isRecord(current,)) {
      continue;
    }
    if (isEnclosingBindingRead(current,)) {
      return true;
    }
    if (REBINDING_FUNCTION_TYPES.has(current.type,)) {
      continue;
    }
    if (current.type === 'ClassBody') {
      // Member values have their own `this`; only computed keys run in the enclosing scope.
      for (const member of Array.isArray(current.body,) ? current.body : []) {
        if (isRecord(member,) && (member.computed === true)) {
          pending.push(member.key,);
        }
      }
      continue;
    }
    /**
     Field holding a plain name for this node, left unwalked.
     */
    const skippedField = nameFieldOf(current,);
    for (const [field, child,] of Object.entries(current,)) {
      // `parent` points back up the tree.
      if ((field !== 'parent') && (field !== skippedField)) {
        pending.push(child,);
      }
    }
  }
  return false;
}
