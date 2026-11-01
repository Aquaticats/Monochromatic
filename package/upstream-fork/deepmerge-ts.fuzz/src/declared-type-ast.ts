/**
 Declared-type AST for type-level fuzzing and its TypeScript emitter.

 The literal-inferred corpus (`./type-case-generate.ts`) never produces a
 declared union, optional key, readonly modifier, or index signature, yet
 every type-level defect found so far came from those. This AST describes
 such types: `./declared-type-arbitrary.ts` draws trees of it,
 `./declared-type-sample.ts` draws values that conform to them, and
 `./declared-type-generate.ts` emits `widen<T>(value)` inputs whose static
 types are the declared ones.

 Emitted types assume the file header declared by the generator: the symbol
 constants in {@link SYMBOL_NAMES}, the brand type `FuzzBrand`, and the
 generic alias `Box<T>`.

 @module
 */

//region AST

/**
 Scalar and leaf type kinds with no children.
 */
export type LeafNode = {
  readonly kind: 'boolean' | 'brand' | 'date' | 'null' | 'number' | 'regexp' | 'string' | 'undefined';
};

/**
 Literal type such as `1`, `'a'`, or `true`.
 */
export type LiteralNode = {
  readonly kind: 'literal';
  readonly value: boolean | number | string;
};

/**
 One declared property; `key` is a string key or a symbol constant name.
 */
export type PropNode = {
  readonly key: string;
  readonly symbolKey: boolean;
  readonly optional: boolean;
  readonly readonly: boolean;
  readonly type: TypeNode;
};

/**
 Index signature shape, or its absence.
 */
export type IndexNode =
  | { readonly keyKind: 'none'; }
    | {
      readonly keyKind: 'number' | 'string' | 'symbol' | 'template';
      readonly value: TypeNode
    };

/**
 Object type with an optional utility wrapper.
 */
export type ObjectNode = {
  readonly kind: 'object';
  readonly props: readonly PropNode[];
  readonly index: IndexNode;
  readonly wrapper: 'none' | 'partial' | 'readonly' | 'required';
};

/**
 Every declared type shape the fuzzer emits.
 */
export type TypeNode =
  | LeafNode
    | LiteralNode
    | ObjectNode
    | {
      readonly kind: 'array';
      readonly element: TypeNode;
      readonly readonly: boolean
    }
    | {
      readonly kind: 'box';
      readonly inner: TypeNode
    }
    | {
      readonly kind: 'intersection';
      readonly left: ObjectNode;
      readonly right: ObjectNode
    }
    | {
      readonly kind: 'map';
      readonly keyKind: 'number' | 'string';
      readonly value: TypeNode;
      readonly readonly: boolean
    }
    | {
      readonly kind: 'record';
      readonly keys: readonly string[];
      readonly value: TypeNode
    }
    | {
      readonly kind: 'set';
      readonly element: TypeNode;
      readonly readonly: boolean
    }
    | {
      readonly kind: 'tuple';
      readonly elements: readonly TypeNode[];
      readonly readonly: boolean
    }
    | {
      readonly kind: 'union';
      readonly members: readonly TypeNode[]
    };

/**
 Symbol constants the generated file declares as `unique symbol`s.
 */
export const SYMBOL_NAMES: readonly string[] = [
  'SYM_A',
  'SYM_B',
  'SYM_INDEX',
];

//endregion AST

//region Emitter

/**
 Index-signature key parameter source per key kind. The template entry is
 TypeScript source for a template-literal key type, not a JavaScript
 template, so it stays a plain string.
 */
const INDEX_KEY_PARAMETERS: Readonly<Record<'number' | 'string' | 'symbol' | 'template', string>> = {
  number: 'key: number',
  string: 'key: string',
  symbol: 'key: symbol',
  // oxlint-disable-next-line eslint/no-template-curly-in-string -- emitted TypeScript source for the key type `k${string}`, not a JavaScript template.
  template: 'key: `k${string}`',
};

/**
 Emit one property's key and modifiers.

 @param prop - Declared property.
 
 @param wrapper - Enclosing utility wrapper, which overrides optionality.

 @returns Source such as `readonly "a"?: number`.

 @example
 ```ts
 emitProp({ prop, wrapper: 'none', });
 ```
 */
function emitProp({
  prop,
  wrapper,
}: {
  readonly prop: PropNode;
  readonly wrapper: ObjectNode['wrapper']
},): string {
  /**
   Key source: computed symbol constant or quoted string.
   */
  const key = prop.symbolKey ? `[${prop.key}]` : JSON.stringify(prop.key,);
  return `${prop.readonly ? 'readonly ' : ''}${key}${prop.optional && (wrapper !== 'required') ? '?' : ''}: ${emitType(prop.type,)}`;
}

/**
 Emit an index signature, widened so every same-kind property conforms.

 @param node - Object type holding the signature.

 @returns Signature source, or an empty string without one.

 @example
 ```ts
 emitIndex(node); // '[key: string]: number | string'
 ```
 */
function emitIndex(node: ObjectNode,): string {
  /**
   Signature of this object.
   */
  const { index, } = node;
  if (index.keyKind === 'none')
    return '';
  /**
   Properties the signature constrains, whose types must fit its value type.
   */
  const covered = node.props
    .filter(function constrainedBy(prop: PropNode,) {
      if (index.keyKind === 'symbol')
        return prop.symbolKey;
      if (index.keyKind === 'string')
        return !prop.symbolKey;
      return false;
    },);
  /**
   Value type union of the declared value and every constrained property.
   */
  const value = [
    index.value,
    ...covered.map(function propType(prop: PropNode,) {
      return prop.type;
    },),
  ].map(emitType,)
    .join(' | ',);
  return `[${INDEX_KEY_PARAMETERS[index.keyKind]}]: ${value}`;
}

/**
 Emit an object literal type with its wrapper.

 @param node - Object type.

 @returns Type source such as `Partial<{ "a": number }>`.

 @example
 ```ts
 emitObject({ kind: 'object', props: [], index: { keyKind: 'none', }, wrapper: 'none', }); // '{  }'
 ```
 */
function emitObject(node: ObjectNode,): string {
  /**
   Member list of the object literal type.
   */
  const members = [
    ...node.props
      .map(function member(prop: PropNode,) {
        return emitProp({
          prop,
          wrapper: node.wrapper,
        },);
      },),
    emitIndex(node,),
  ].filter(function present(member: string,) {
    return member !== '';
  },);
  /**
   Bare object literal type.
   */
  const body = `{ ${members.join('; ',)} }`;
  return {
    none: body,
    partial: `Partial<${body}>`,
    readonly: `Readonly<${body}>`,
    required: `Required<${body}>`,
  }[node.wrapper];
}

/**
 Emit TypeScript source for a declared type.

 @param node - Type to emit.

 @returns Type source, parenthesized where precedence requires.

 @throws When a new node kind has no emitter branch.

 @example
 ```ts
 emitType({ kind: 'array', element: { kind: 'number', }, readonly: true, }); // 'readonly (number)[]'
 ```
 */
export function emitType(node: TypeNode,): string {
  if (node.kind === 'literal')
    return JSON.stringify(node.value,);
  if (node.kind === 'brand')
    return 'FuzzBrand';
  if (node.kind === 'date')
    return 'Date';
  if (node.kind === 'regexp')
    return 'RegExp';
  if ((node.kind === 'boolean') || (node.kind === 'null')
    || (node.kind === 'number')
    || (node.kind === 'string')
    || (node.kind === 'undefined'))
    return node.kind;
  if (node.kind === 'array')
    return `${node.readonly ? 'readonly ' : ''}(${emitType(node.element,)})[]`;
  if (node.kind === 'tuple')
    return `${node.readonly ? 'readonly ' : ''}[${node.elements
      .map(emitType,)
      .join(', ',)}]`;
  if (node.kind === 'set')
    return `${node.readonly ? 'ReadonlySet' : 'Set'}<${emitType(node.element,)}>`;
  if (node.kind === 'map')
    return `${node.readonly ? 'ReadonlyMap' : 'Map'}<${node.keyKind}, ${emitType(node.value,)}>`;
  if (node.kind === 'union') {
    return node.members
      .map(function member(child: TypeNode,) {
        return `(${emitType(child,)})`;
      },)
      .join(' | ',);
  }
  if (node.kind === 'intersection')
    return `(${emitType(node.left,)}) & (${emitType(node.right,)})`;
  if (node.kind === 'record') {
    return `Record<${node.keys
      .map(function quote(key: string,) {
      return JSON.stringify(key,);
    },)
      .join(' | ',)}, ${emitType(node.value,)}>`;
  }
  if (node.kind === 'box')
    return `Box<${emitType(node.inner,)}>`;
  // LeafNode's kind is itself a union, so the chain cannot narrow node to never; check the last kind explicitly.
  if (node.kind !== 'object')
    throw new Error(`emitType: unhandled node ${JSON.stringify(node,)}`,);
  return emitObject(node,);
}

//endregion Emitter
