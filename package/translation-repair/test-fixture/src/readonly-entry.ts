//region Readonly entry
// The type a callback gives its parameter when it destructures one entry of `Object.entries`, a map, or a count table.
// The entry those calls hand out is a mutable tuple; naming it here lets the parameter be read as deeply readonly.

/**
 One key and its value as a deeply readonly pair, which is what a callback reading an entry of `Object.entries` or of a map needs for its destructured parameter.

 Kept as this package's own copy of the type `@monochromatic-dev/translation-repair-shared-contract` exports, because that package lists this one as a development dependency and importing it back would close a dependency cycle.
 */
export type ReadonlyEntry<Key, Value,> = readonly [
  Key,
  Value,
];

//endregion Readonly entry
