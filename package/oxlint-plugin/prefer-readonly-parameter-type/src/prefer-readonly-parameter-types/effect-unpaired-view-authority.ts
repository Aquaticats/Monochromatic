/**
 Read-only view membership for default-library interfaces the library never paired.
 
 @module
 */

/**
 Members a read-only `DataView` would carry, by name.
 
 `collectionStructureClaim` derives "this member restructures its receiver" by diffing an
 interface against its `Readonly` counterpart: `Array` has `push` and `ReadonlyArray` does
 not, so `push` mutates. The derivation needs a pair, and the default library declares no
 `ReadonlyDataView`, so every `DataView` member answered "unrecognized" and every buffer
 write landed on the opaque boundary. Measured before this existed: 21 findings across the
 workspace named a buffer member, 14 named nothing else, and `writeEndOfCentralDirectory`
 in `package/zip-writer/core/src/serialize.ts` carried a `ForeignBorrowed` marker and a
 hand-written `@mutates view` for what is a specification-defined store.
 
 Declaring the missing view rather than listing the mutators is deliberate. It reuses the
 existing derivation unchanged instead of adding a second one that could disagree with it,
 and it states the fact in the same shape the library states it for every paired interface.
 
 The reading members only, including the `Big` pair a later library file adds. Every `set`
 member is absent, which is what makes them mutate.
 `buffer`, `byteLength` and `byteOffset` are present because reading them changes nothing;
 `buffer` hands out the underlying `ArrayBuffer`, which is an exposure question the result
 relations answer and not a restructuring of the view.
 
 Enforced by `effect-unpaired-view.unit.test.ts`, which drives a real `DataView` and proves
 each absent member changes what a present one returns, and that each present member does
 not. An entry added without that probe is a defect by the same standard the other
 authorities are held to.
 */
const DATA_VIEW_READONLY_MEMBERS: ReadonlySet<string> = new Set([
  'buffer',
  'byteLength',
  'byteOffset',
  'getBigInt64',
  'getBigUint64',
  'getFloat16',
  'getFloat32',
  'getFloat64',
  'getInt8',
  'getInt16',
  'getInt32',
  'getUint8',
  'getUint16',
  'getUint32',
],);

/**
 Members a read-only `Date` would carry, by name.
 
 `Date` is unpaired for the same reason `DataView` is: the default library declares no
 `ReadonlyDate`, so the diff had nothing to compare and every member answered
 `COLLECTION_UNRECOGNIZED`. Measured before this existed, on the real
 `package/toml-edit/core/src/values.ts`: `encodeValue` and the three callables around
 it all read `opaque=[0]`, and 48 workspace findings named `input.toISOString` as the
 sole remaining cause once `Object.getPrototypeOf` was discharged.
 
 A `Date` instance has one mutable specification slot, `[[DateValue]]`, and the fifteen
 declared setters write it. Everything here leaves it alone, which is what makes the
 membership derivable rather than asserted.
 
 Three names the engine provides are deliberately absent, and their absence claims
 nothing: `getYear`, `setYear` and `toGMTString` are Annex B members TypeScript never
 declares, so no lookup can reach this table for them. `getVarDate` is declared, but
 only by `lib.scripthost.d.ts`, which `package/config/typescript/tsconfig.options.json`
 never includes and no engine here provides; an entry for it could not be probed, and an
 unprobed entry is the one thing this design refuses. `[Symbol.toPrimitive]` is declared
 and preserving, and is still absent, because `collectionStructureClaim` rejects a
 computed member name before consulting this table: it stays unrecognized rather than
 being claimed to mutate.
 
 `toJSON`, `toLocaleString`, `toLocaleDateString` and `toLocaleTimeString` belong here.
 Each preserves its receiver, which is the only question this table answers. That they
 dispatch to receiver-selected code is a channel question, and the channel authority
 answers it separately by listing none of them.
 
 Enforced by `effect-unpaired-view.unit.test.ts`, which drives a real `Date` and proves
 every declared member leaves both `[[DateValue]]` and the receiver's own properties
 unchanged, and every absent one changes the timestamp under at least one argument.
 */
const DATE_READONLY_MEMBERS: ReadonlySet<string> = new Set([
  'getDate',
  'getDay',
  'getFullYear',
  'getHours',
  'getMilliseconds',
  'getMinutes',
  'getMonth',
  'getSeconds',
  'getTime',
  'getTimezoneOffset',
  'getUTCDate',
  'getUTCDay',
  'getUTCFullYear',
  'getUTCHours',
  'getUTCMilliseconds',
  'getUTCMinutes',
  'getUTCMonth',
  'getUTCSeconds',
  'toDateString',
  'toISOString',
  'toJSON',
  'toLocaleDateString',
  'toLocaleString',
  'toLocaleTimeString',
  'toString',
  'toTemporalInstant',
  'toTimeString',
  'toUTCString',
  'valueOf',
],);

/**
 Read-only view membership for every interface the default library leaves unpaired.
 */
const UNPAIRED_VIEW_MEMBERS: ReadonlyMap<string, ReadonlySet<string>> = new Map([
  [
    'DataView',
    DATA_VIEW_READONLY_MEMBERS,
  ],
  [
    'Date',
    DATE_READONLY_MEMBERS,
  ],
],);

/**
 Total declared members across every unpaired interface, pinned against silent growth.
 
 A literal in the architecture guard must match this, so adding a member means changing a
 number in a second file, which is where the probe becomes unavoidable.
 */
export const VERIFIED_UNPAIRED_VIEW_COUNT = 43;

/**
 Sentinel for an interface this authority says nothing about.
 */
export const UNPAIRED_VIEW_UNKNOWN: unique symbol = Symbol(
  'default-library interface has no declared read-only view membership here',
);

/**
 Reads the declared read-only membership for one unpaired interface.
 
 @param ownerName - Default-library interface owning the member.
 
 @returns member names a read-only view would carry, or sentinel when undeclared.
 
 @example
 ```ts
 unpairedViewMembers({ ownerName: 'DataView' });
 ```
 */
export function unpairedViewMembers({
  ownerName,
}: {
  readonly ownerName: string;
},): ReadonlySet<string> | typeof UNPAIRED_VIEW_UNKNOWN {
  return UNPAIRED_VIEW_MEMBERS.get(ownerName,)
    ?? UNPAIRED_VIEW_UNKNOWN;
}

/**
 Every interface this authority declares, for the architecture guard.
 */
export const UNPAIRED_VIEW_INTERFACES: ReadonlyMap<
  string,
  ReadonlySet<string>
  > = UNPAIRED_VIEW_MEMBERS;
