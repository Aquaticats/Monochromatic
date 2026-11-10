/**
 Shared runtime primitives for oxlint plugin packages.
 
 These helpers live in a shipped package because plugin rules call them while
 linting user source. Test-only helpers belong in
 `@monochromatic-dev/oxlint-plugin-test-support` instead.
 
 @module
 */

/**
 Readonly view of an untyped object's string-keyed fields.
 */
export type ReadonlyRecord = Readonly<Record<string, unknown>>;

export {
  type ParsedMutationContractBlock,
  parseMutationContractBlocks,
} from './mutation-contract.ts';

/**
 Narrows an unknown value to a readonly record-like object.
 
 @param value - candidate runtime value
 
 @returns whether `value` is non-null object data with inspectable fields
 
 @example
 ```ts
 if (isRecord(value)) {
   value.type;
 }
 ```
 */
export function isRecord(value: unknown,): value is ReadonlyRecord {
  return ((typeof value) === 'object') && (value !== null);
}
