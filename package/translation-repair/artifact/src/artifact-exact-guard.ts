import {
  ArtifactParseError,
  requireArray,
  requireCount,
  requireRecord,
  requireString,
} from '@monochromatic-dev/translation-repair-artifact-pool/ts';
import type { IndexPair, } from '@monochromatic-dev/translation-repair-stage/ts';
import type { ArtifactJsonValue, } from './corpus-run/artifact-two-lane-contract.ts';
import {
  isJsonArray,
  isJsonRecord,
} from '@monochromatic-dev/translation-repair-document-structure/ts';

//region Artifact exact guards
// The two checks a versioned artifact reader needs and a lenient one does not:
// refusing keys the schema does not name, and reading a value whose shape the
// schema deliberately leaves open.
//
// APART FROM `artifact-guard.ts` because that file holds the checks every
// reader shares, these belong to readers that enforce a SCHEMA, and it is
// already at its own size. Nothing here overlaps it: those answer "is this a
// string", these answer "does this record say anything the version does not
// describe".
//
// WHY EXACTNESS MATTERS AT ALL, since a tolerant reader is the usual advice: a
// version number is a promise about what a file means, and an unknown key is
// either a field from a generation this reader does not understand or a typo
// that silently dropped a field it does. Both are cases where reading on
// produces a confident answer about a file nobody wrote. The tolerance this
// schema does grant is NAMED and bounded: each lane's raw `result`, which
// `requireOpenRecord` reads and says why.

/**
 Refuses a record carrying any key the schema does not name.

 ONE DIRECTION ONLY. Missing keys are not this function's business, because
 every field a version 2 reader needs is read by a guard that throws on its
 absence with a path pointing at the field rather than at the record.

 @param record - record to check

 @param allowed - every key this version describes here

 @param path - dotted path for error message

 @throws {@link ArtifactParseError} naming the first unknown key, in the order
 the record lists them

 @example
 ```ts
 requireExactKeys({ record: lane, allowed: ['result', 'delivery',], path: 'lanes.repair', },);
 ```
 */
export function requireExactKeys(
  {
    record,
    allowed,
    path,
  }: {
    readonly record: Readonly<Record<string, unknown>>;
    readonly allowed: readonly string[];
    readonly path: string;
  },
): void {
  /**
   Names this version describes, as a set, so a wide record is not a quadratic
   scan over a list.
   */
  const known = new Set(allowed,);

  /**
   First key the schema does not name, or nothing.
   */
  const unknownKey = Object.keys(record,)
    .find(function isUnknown(key,): boolean {
      return !known.has(key,);
    },);
  if (unknownKey !== undefined) {
    throw new ArtifactParseError({
      path: `${path}.${unknownKey}`,
      reason: `no key here beyond ${allowed.join(', ',)}`,
    },);
  }
}

/**
 Reads a string the schema allows only a named few of.

 Returns the member FOUND IN `allowed` rather than the value read, so the
 result is narrowed by the list rather than by an assertion: a caller passing
 a literal tuple gets that tuple's union back, and no cast stands between the
 check and the type.

 @param value - value to check

 @param allowed - every member this version describes here

 @param path - dotted path for error message

 @returns Whichever member the value matched

 @throws {@link ArtifactParseError} when the value is not a string, or is one
 this version does not name

 @example
 ```ts
 const kind = requireOneOf({ value: row.incumbentKind, allowed: ['present', 'absent',], path, },);
 ```
 */
export function requireOneOf<const TAllowed extends string,>(
  {
    value,
    allowed,
    path,
  }: {
    readonly value: unknown;
    readonly allowed: readonly TAllowed[];
    readonly path: string;
  },
): TAllowed {
  /**
   String the artifact carries, before it is known to be one of these.
   */
  const held = requireString({
    value,
    path,
  },);

  /**
   Member it matched, or nothing.
   */
  const member = allowed.find(function isHeld(one,): boolean {
    return one === held;
  },);
  if (member === undefined) {
    throw new ArtifactParseError({
      path,
      reason: `one of ${allowed.join(', ',)}`,
    },);
  }
  return member;
}

/**
 Reads a string the artifact carries as one of the keys of a record the
 schema keys by its vocabulary.

 THE SAME CHECK AS {@link requireOneOf}, with the vocabulary held as a record
 keyed by the type it describes, so the compiler refuses a member of that type
 the record lacks, where a list typed by the same union proves membership
 only. The refusal names the keys in the order the record spells them.

 @param value - value to check

 @param record - every member this version describes here, as the keys

 @param path - dotted path for error message

 @returns Whichever key the value matched

 @throws {@link ArtifactParseError} when the value is not a string, or is one
 this version does not name

 @example
 ```ts
 const kind = requireKeyOf({ value: row.incumbentKind, record: { present: true, absent: true, }, path, },);
 ```
 */
export function requireKeyOf<KeyT extends string,>(
  {
    value,
    record,
    path,
  }: {
    readonly value: unknown;
    readonly record: Readonly<Record<KeyT, unknown>>;
    readonly path: string;
  },
): KeyT {
  /**
   String the artifact carries, before it is known to be one of these.
   */
  const held = requireString({
    value,
    path,
  },);

  /**
   Whether the string is a key the record holds, by own key so an inherited
   property never answers for a name the artifact carries.

   @param candidate - string read

   @returns Whether the record holds it

   @example
   ```ts
   const known = isKey('present',);
   ```
   */
  function isKey(candidate: string,): candidate is KeyT {
    return Object.hasOwn(
      record,
      candidate,
    );
  }
  if (!isKey(held,)) {
    throw new ArtifactParseError({
      path,
      reason: `one of ${
        Object.keys(record,)
          .join(', ',)
      }`,
    },);
  }
  return held;
}

/**
 Reads a value the schema leaves OPEN but not arbitrary.

 `null` is refused at every depth. It is absence spelled as a value, which is
 the thing this generation exists to stop recording, and the writer controls
 every byte reaching the field this guards: a configuration with nothing to
 say about a setting leaves the key out. A reader meeting a null here has met
 a file this schema did not write.

 @param value - value to check

 @param path - dotted path for error message

 @returns Value as the open JSON this schema allows

 @throws {@link ArtifactParseError} at the first null, function, or undefined,
 naming the path that reached it

 @example
 ```ts
 const callConfig = requireArtifactJsonValue({ value: artifact.callConfig, path: 'callConfig', },);
 ```
 */
export function requireArtifactJsonValue(
  {
    value,
    path,
  }: {
    readonly value: unknown;
    readonly path: string;
  },
): ArtifactJsonValue {
  // One test per kind rather than one chained condition, because each of these
  // narrows the returned value on its own and a combined test narrows to their
  // union at best.
  if ((typeof value) === 'boolean')
    return value;
  if ((typeof value) === 'number')
    return value;
  if ((typeof value) === 'string')
    return value;
  if (isJsonArray(value,)) {
    return value.map(function readElement(
      element,
      position,
    ): ArtifactJsonValue {
      return requireArtifactJsonValue({
        value: element,
        path: `${path}[${String(position,)}]`,
      },);
    },);
  }

  // An array took its own case, and `isJsonRecord` refuses one in any order
  // (ledger B92), so what reaches this case is an object.
  if (isJsonRecord(value,)) {
    return Object.fromEntries(Object.entries(value,)
      .map(function readEntry([
        key,
        held,
      ],): readonly [
        string,
        ArtifactJsonValue,
      ] {
        return [
          key,
          requireArtifactJsonValue({
            value: held,
            path: `${path}.${key}`,
          },),
        ];
      },),);
  }
  throw new ArtifactParseError({
    path,
    reason: 'a boolean, number, string, array, or object, and never null',
  },);
}

/**
 Reads a RECORD of values the schema leaves open.

 Beside {@link requireArtifactJsonValue} rather than folded into it, because a
 caller reading a field declared as a record needs that back: the value guard
 answers with a union, and narrowing it at the call site would put a cast
 between the check and the type it proves.

 @param value - value to check

 @param path - dotted path for error message

 @returns Record whose values are open JSON, with no null at any depth

 @throws {@link ArtifactParseError} when the value is not an object, is an
 array, or holds anything this schema does not allow

 @example
 ```ts
 const callConfig = requireArtifactJsonRecord({ value: artifact.callConfig, path, },);
 ```
 */
export function requireArtifactJsonRecord(
  {
    value,
    path,
  }: {
    readonly value: unknown;
    readonly path: string;
  },
): Readonly<Record<string, ArtifactJsonValue>> {
  return Object.fromEntries(Object.entries(requireRecord({
    value,
    path,
  },),)
    .map(function readEntry([
      key,
      held,
    ],): readonly [
      string,
      ArtifactJsonValue,
    ] {
      return [
        key,
        requireArtifactJsonValue({
          value: held,
          path: `${path}.${key}`,
        },),
      ];
    },),);
}

/**
 Reads a record the schema deliberately does not describe.

 TOLERANT WHERE THE OTHER GUARD IS NOT, and the difference is the point. This
 reads the raw lane results, which are EVIDENCE: they are typed by the live
 pipeline shapes, they grow by addition, and a reader takes the fields it
 knows. `null` is accepted here because nothing in this file controls what a
 lane result may hold, so refusing it would refuse artifacts a later pipeline
 legitimately wrote. What version 2 requires OF that evidence is parsed
 separately, out of this same record, by the frozen evidence core.

 @param value - value to check

 @param path - dotted path for error message

 @returns Value as a record whose contents stay unknown

 @throws {@link ArtifactParseError} when the value is not an object, or is an
 array

 @example
 ```ts
 const raw = requireOpenRecord({ value: lane.result, path: 'lanes.repair.result', },);
 ```
 */
export function requireOpenRecord(
  {
    value,
    path,
  }: {
    readonly value: unknown;
    readonly path: string;
  },
): Readonly<Record<string, unknown>> {
  return requireRecord({
    value,
    path,
  },);
}

/**
 Reads a required list of index pairs, each an exact `{ source, target }`
 record of two counts.

 The two section-pairing readers of artifact version 2
 (`artifact-two-lane-read-pairing.ts` for the block pairing within each
 section, `artifact-two-lane-read-section-pairing.ts` for the sections
 themselves) each kept their own copy (audit area six, 2026-09-28).

 @param value - value to check

 @param path - dotted path for error message; an entry's path is this with
 its position in brackets

 @returns Pairs in stored order

 @throws {@link ArtifactParseError} when the value is not an array, an entry
 is not a record of exactly those two keys, or either is not a count

 @example
 ```ts
 const pairs = requireIndexPairList({ value: record.pairs, path: 'pairing.pairs', },);
 ```
 */
export function requireIndexPairList(
  {
    value,
    path,
  }: {
    readonly value: unknown;
    readonly path: string;
  },
): readonly IndexPair[] {
  return requireArray({
    value,
    path,
  },)
    .map(function readPair(
      entry,
      at,
    ): IndexPair {
      /**
       Where this pair is reported from.
       */
      const entryPath = `${path}[${String(at,)}]`;

      /**
       Pair as a record.
       */
      const record = requireRecord({
        value: entry,
        path: entryPath,
      },);
      requireExactKeys({
        record,
        allowed: [
          'source',
          'target',
        ],
        path: entryPath,
      },);
      return {
        source: requireCount({
          value: record.source,
          path: `${entryPath}.source`,
        },),
        target: requireCount({
          value: record.target,
          path: `${entryPath}.target`,
        },),
      };
    },);
}

//endregion Artifact exact guards
