/**
 The exported deepmerge-ts types that `./type-export.unit.test.ts`
 references, and a reader for the type names a declaration file exports, so
 `./type-export-name.unit.test.ts` can fail when a release adds a type the
 tests never touch (the recall audit's `944b428` / `#304` class).

 @module
 */

/**
 Exported type names the type tests reference, sorted.
 */
export const EXPORTED_TYPE_NAMES: readonly string[] = [
  'DeepMergeArraysDefaultHKT',
  'DeepMergeBuiltInMetaData',
  'DeepMergeCircularReferencesDefaultHKT',
  'DeepMergeFastUnsafeOptions',
  'DeepMergeFastUnsafeUtils',
  'DeepMergeFilterValuesDefaultHKT',
  'DeepMergeFunctionURItoKind',
  'DeepMergeFunctionsDefaultURIs',
  'DeepMergeFunctionsDefaults',
  'DeepMergeFunctionsDefaultsFastUnsafe',
  'DeepMergeFunctionsURIs',
  'DeepMergeHKT',
  'DeepMergeIntoFastUnsafeOptions',
  'DeepMergeIntoFastUnsafeUtils',
  'DeepMergeIntoFunctionsDefaults',
  'DeepMergeIntoFunctionsDefaultsFastUnsafe',
  'DeepMergeIntoOptions',
  'DeepMergeIntoUtils',
  'DeepMergeLeaf',
  'DeepMergeLeafURI',
  'DeepMergeMapsDefaultHKT',
  'DeepMergeMergeInfo',
  'DeepMergeMetaData',
  'DeepMergeNoFilteringURI',
  'DeepMergeOptions',
  'DeepMergeRecordsDefaultHKT',
  'DeepMergeSetsDefaultHKT',
  'DeepMergeUtils',
  'DeepMergeValueReference',
  'FilterOut',
  'GetDeepMergeFunctionsURIs',
];

/**
 Line prefixes that declare an exported type in rolldown-style `.d.mts` output.
 */
const TYPE_PREFIXES = [
  'export type ',
  'export interface ',
  'export declare const enum ',
] as const;

/**
 Characters that end a declared name.
 */
const NAME_ENDS = [
  '<',
  ' ',
  '=',
  '{',
] as const;

/**
 Name at the start of `rest`: everything before the first `<`, space, `=`, or `{`.

 @param rest - Declaration text after its keyword.

 @returns Declared name.

 @example
 ```ts
 leadingName('Foo<T> = T;'); // 'Foo'
 ```
 */
function leadingName(rest: string,): string {
  /**
   Positions of each character that ends a name, where present.
   */
  const ends = NAME_ENDS.map(function positionOf(character,) {
    return rest.indexOf(character,);
  },)
    .filter(function present(position,) {
      return position !== (-1);
    },);
  return rest.slice(
    0,
    (ends.length === 0) ? rest.length : Math.min(...ends,),
  );
}

/**
 Type names a declaration file exports: `export type`, `export interface`,
 and `export declare const enum` lines, plus `X as Name` entries of a bare
 `export { ... }` block (rolldown's renamed exports).

 @param text - Declaration file contents.

 @returns Sorted, distinct names.

 @example
 ```ts
 declaredTypeNames('export type A = 1;\nexport {\n\tB$1 as B,\n};\n'); // ['A', 'B']
 ```
 */
export function declaredTypeNames(text: string,): readonly string[] {
  /**
   Lines with surrounding whitespace removed.
   */
  const lines = text.split('\n',)
    .map(function trim(line,) {
      return line.trim();
    },);
  /**
   Names from prefixed declaration lines.
   */
  const declared = lines.flatMap(function declaredName(line,) {
    /**
     Prefix this line starts with, if any.
     */
    const prefix = TYPE_PREFIXES.find(function starts(candidate,) {
      return line.startsWith(candidate,);
    },);
    return (prefix === undefined) ? [] : [leadingName(line.slice(prefix.length,),),];
  },);
  /**
   First line of the bare export block, or -1.
   */
  const blockStart = lines.indexOf('export {',);
  /**
   Lines inside that block; TSDoc prose elsewhere may also contain ` as `.
   */
  const block = (blockStart === (-1))
    ? []
    : lines.slice(blockStart + 1,)
      .slice(
      0,
      lines.slice(blockStart + 1,)
        .indexOf('};',),
      );
  /**
   Names renamed in the block: `Local as Name,`.
   */
  const renamed = block.flatMap(function renamedName(line,) {
    /**
     Position of the rename separator.
     */
    const separator = line.indexOf(' as ',);
    return (separator === (-1)) ? [] : [line.slice(separator + ' as '.length,)
      .replace(
      ',',
      '',
      ),];
  },);
  return [...new Set([
    ...declared,
    ...renamed,
  ],),].toSorted();
}
