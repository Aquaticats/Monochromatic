//region Marker keys -- Project-owned unique symbols hidden from the public surface

/**
 Project-owned marker key preserving transparent runtime representation.
 */
declare const FOREIGN_BORROWED_MARKER: unique symbol;

/**
 Project-owned marker key for explicitly audited opaque host capabilities.
 */
declare const FOREIGN_HOST_CAPABILITY_MARKER: unique symbol;

//endregion Marker keys

//region Exported marker types -- Ownership marks the readonly analyzer recognizes at a foreign boundary

/**
 Marks mutable handle whose ownership and parameter type are dictated by foreign interface.
 
 Type remains assignable to and from original foreign contract.
 Readonly parameter analysis recognizes exact alias as an ownership marker,
 not an immutability claim.
 
 @typeParam Value - Foreign handle type imposed by upstream interface.
 
 @example
 ```ts
 function inspectNode(node: ForeignBorrowed<ESTree.Node>): void {
   void node.type;
 }
 ```
 */
export type ForeignBorrowed<Value> = Value & {
  readonly [FOREIGN_BORROWED_MARKER]?: true;
};

/**
 Marks foreign host capability whose implementation cannot be inspected.
 
 Composes {@link ForeignBorrowed}, which owns the foreign-ownership property,
 and adds only the host-capability key.
 A further marker type composes one of these two rather than repeating a key.
 
 Use only at runtime boundaries after source or source-map inference fails.
 Readonly parameter analysis accepts unresolved effects reaching this marker
 only when callable documents corresponding `@mutates` contract.
 
 Type remains assignable to and from original foreign contract.
 
 @typeParam Value - Host capability type imposed by runtime interface.
 
 @example
 ```ts
 function register(api: ForeignHostCapability<ExtensionAPI>): void {
   api.registerCommand({ name: 'example', handler() {} });
 }
 ```
 */
export type ForeignHostCapability<Value> = ForeignBorrowed<Value> & {
  readonly [FOREIGN_HOST_CAPABILITY_MARKER]?: true;
};

//endregion Exported marker types
