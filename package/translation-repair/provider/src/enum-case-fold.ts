import { isJsonRecord, } from '@monochromatic-dev/translation-repair-document-structure/ts';

//region Enum case fold
// THE SPELLING A REPLY GIVES A LISTED VALUE, taken back to the spelling the schema lists. Anthropic's structured
// outputs page says (https://platform.claude.com/docs/en/build-with-claude/structured-outputs, read 2026-10-23):
// "Structured outputs don't guarantee the capitalization of string `enum` and `const` values: Claude might return a
// value that differs from your schema only in capitalization, typically in the first letter of a word following a
// space." Every stage guard compares a label exactly, so a reply of `Consistent` for a listed `consistent` read as no
// label at all and cost the voice (a live Claude call under a plain string once answered "Consistent", which the lane
// contest's guard refused, `lane-contest/src/contest-ballot-wire.ts`).
//
// THE RULE: when exactly one listed value matches the reply's value ignoring capitalization, the reply's value is
// replaced by the listed one and the replacement is reported to the caller to log. A value no listed value matches, or
// that several match (a schema listing `ab` and `aB` does not say which a reply of `AB` meant), is left as it came, and
// the guard decides. A value the schema lists exactly is left alone.
//
// WHAT IT READS: a node's `enum` and `const` strings, under `properties` and `items`, along the reply's own shape. It
// LEAVES SIX FORMS UNTOUCHED, and a value under any of them is read as it came: `anyOf`, `oneOf`, `allOf`, `$ref`, an
// `additionalProperties` that is a schema, and `items` as a list of schemas (tuple form). No schema the route or a
// stage sends carries one: `corpus-pass/src/claude-schemas.unit.test.ts` walks every producer's schema and fails when
// one does, or lists an `enum` or `const` where this does not read. `claude-schema.ts` accepts `anyOf`, `allOf` and
// `$ref`, so a stage that adds one would get the exact comparison it had before this file, and that test says so. A
// reply left as it came is the behaviour before this file, never a wrong value.
//
// WHAT THE CACHES DO WITH IT, read from their key files on 2026-10-23. No cache key hashes the reader, so a reader
// that accepts a reply it used to refuse is a change a key cannot see. The durable prompt payload store
// (`stage/src/prompt-payload-store.ts`) keeps a model's raw reply under the digest of the model and the messages, and
// `promptUniqueClient` reads a replayed reply through `readJsonOutcome`: a stored reply that failed the guard for a
// capital and was re-asked is now accepted on its first read, the re-ask's stored reply goes unused, and no outcome
// is stored, so the store stays valid and its format version (1) stays. Every slice cache keeps an outcome that
// stages settled from replies read before the fold, never a raw reply, so such an outcome would differ if the same
// replies were read now wherever a stage's schema lists an `enum` or `const`. A read of the built
// `CLAUDE_SCHEMA_PRODUCERS` found a listing in the schemas of the panel, critic, introduced-defect, resolution,
// coverage, archive-block, rendering-audit, consistency, contest-ballot and translate-repair stages (and of the
// derivability and restoration checks, whose output no slice record holds). So the repair slice, refine, consolidate,
// lane-contest, translate slice and consistency versions move for the fold. The block and section pairing schemas and
// the page title schema list no `enum` or `const`, so the fold reads nothing in them and the pairing and page title
// versions stay. The picture reading cache keeps text read from a picture under no schema and stays.
//
// THE ARTIFACT GENERATION STAYS WHERE IT IS, by the lead's ruling of 2026-10-23 (open to the owner's veto). The
// scheduler counts every `*.json` in a pass's artifacts directory as settled
// (`slice-cache/src/corpus-run/pass-schema-guard.ts:41-45`), so an entry settled before the fold is not run again
// after it and keeps the outcome the old reader gave it, a voice lost to a capital included. The fold only lets the
// reader accept replies it used to refuse, and a settled artifact holds an outcome that shipped. Moving the
// generation would run every settled entry of the corpus again, which that file names as money and therefore the
// operator's call (`pass-schema-guard.ts:58-60`). The way to run them again, if the owner wants it, is the one the
// quorum rule of B816 took: move the artifact generation, as `repair/src/repair-slice-key.ts:616` records for
// version 43 of `SLICE_CACHE_VERSION`.
//
// WHAT A RESUME MEETS AFTER THE FOLD, read from the guards on 2026-10-23, which bears on how much the ruling saves.
// Each claim is quoted from the code it names:
// - A resume into the same artifacts directory is refused by default, because the build differs. The guard
//   "Refuses to RESUME an accumulation into a directory whose settled entries were produced by a different build of
//   the pipeline" (`slice-cache/src/corpus-run/pass-generation-guard.ts:11-12`), and the build is identified by a
//   digest over the built output, "a directory" under `dist/final/node`
//   (`glossary/src/corpus-run/pipeline-digest.ts:37-39`), which holds this reader (`rg --text "function foldEnumCase"`
//   finds it once in `corpus-pass/dist/final/node/corpus-pass.mjs`, built 2026-10-23). `assertBuildGenerationResumable`
//   throws `GenerationDriftError` (`:377`) unless the launch carries `TRANSLATION_REPAIR_ALLOW_GENERATION_DRIFT=yes`
//   (`:32`, `:38`, `:360`, `:125`). So only an operator who opted in resumes into the old directory, and that
//   operator is the one for whom settled entries keep their outcome.
// - The runbook says the usual path is a new directory: "the rule is always kill and relaunch: ... relaunch the entry
//   into a fresh runs dir on the new build. The old runs dir holds what the old digest bought and is not resumed"
//   (`doc/runbook/translation-repair-corpus-pass.md:714-719`). A fresh directory has no settled entry, so every entry
//   runs on the new reader whatever the generation says; the slice caches answer by key, and the versions this header
//   names moved.
// - Moving the artifact generation would not re-run anything by itself. `SchemaGenerationError` is "Raised when a
//   resume would settle a second artifact generation into one pool" (`pass-schema-guard.ts:203`), its message says
//   "The pipeline drift opt-in does not cover this and is not asked about" (`:253`), and its first way forward is a
//   fresh directory and its third "Move the incompatible artifacts to an archive directory and resume here" (`:188`,
//   `:194`). The move would refuse the opted-in resume until an operator archives the old artifacts, and only the
//   artifacts archived are run again.

/**
 One value taken back to the schema's spelling.

 @example
 ```ts
 const fold: EnumCaseFold = { path: 'reply.findings[1].severity', received: 'Major', taken: 'major', };
 ```
 */
export type EnumCaseFold = {
  /**
   Where the value sits, from `reply`: property names joined by dots, a list position in brackets.
   */
  readonly path: string;

  /**
   The value as the reply gave it.
   */
  readonly received: string;

  /**
   The value as the schema lists it.
   */
  readonly taken: string;
};

/**
 A reply value read against its schema node, with what was taken back.

 @example
 ```ts
 const unchanged: EnumCaseFolded = { value: 'consistent', folds: [], };
 ```
 */
export type EnumCaseFolded = {
  /**
   The value to read onward: the same value where nothing was taken back.
   */
  readonly value: unknown;

  /**
   Every value taken back at the node and under it.
   */
  readonly folds: readonly EnumCaseFold[];
};

/**
 One property of a reply, read against its schema node.
 */
type NamedFolded = {
  /**
   The property's name.
   */
  readonly key: string;

  /**
   The reading of its value.
   */
  readonly folded: EnumCaseFolded;
};

/**
 Whether a value is a string.

 @param value - anything a reply holds

 @returns Whether it is text

 @example
 ```ts
 const text = isText('purr',);
 ```
 */
function isText(value: unknown,): value is string {
  return (typeof value) === 'string';
}

/**
 Every string a node lists as its value, from `const` and `enum`.

 @param schema - node read

 @returns Listed strings, in listed order, none where the node lists no string

 @example
 ```ts
 const listed = listedStringsOf({ schema: { enum: ['minor', 'major',], }, },);
 // => ['minor', 'major']
 ```
 */
function listedStringsOf({ schema, }: { readonly schema: Readonly<Record<string, unknown>>; },): readonly string[] {
  /**
   The node's single listed value and its list of them.
   */
  const {
    const: single,
    enum: members,
  } = schema;

  /**
   The members of the `enum`, none where the node has no list.
   */
  const listed: readonly unknown[] = Array.isArray(members,) ? members : [];
  return [
    single,
    ...listed,
  ].filter(isText,);
}

/**
 The listed spellings a value differs from only in capitalization.

 @param received - value as the reply gave it

 @param listed - strings the schema lists

 @returns Each distinct listed string that matches ignoring capitalization: none where the value is listed exactly,
 since it needs no change, and none where nothing matches

 @example
 ```ts
 const matches = matchingSpellingsOf({ received: 'not Consistent', listed: ['consistent', 'not consistent',], },);
 // => ['not consistent']
 ```
 */
function matchingSpellingsOf(
  {
    received,
    listed,
  }: {
    readonly received: string;
    readonly listed: readonly string[];
  },
): readonly string[] {
  if (listed.includes(received,))
    return [];

  /**
   The received value without capitals, which every spelling is compared by.
   */
  const lowered = received.toLowerCase();

  /**
   Listed strings that match it, one of them twice where the schema lists a string twice.
   */
  const matching = listed.filter(function sameLetters(spelling,): boolean {
    return spelling.toLowerCase() === lowered;
  },);
  return [...new Set(matching,),];
}

/**
 Joins a property name to its parent's path.

 @param path - where the parent sits, from `reply`

 @param key - property name

 @returns Dotted path of the property

 @example
 ```ts
 const path = childPathOf({ path: 'reply.findings[1]', key: 'severity', },);
 // => 'reply.findings[1].severity'
 ```
 */
function childPathOf(
  {
    path,
    key,
  }: {
    readonly path: string;
    readonly key: string;
  },
): string {
  return `${path}.${key}`;
}

/**
 Every value taken back under a set of members.

 @param members - each member of a list or object, read

 @returns Values taken back, in member order

 @example
 ```ts
 const folds = foldsOf({ members: [{ value: 'major', folds: [], },], },);
 // => []
 ```
 */
function foldsOf({ members, }: { readonly members: readonly EnumCaseFolded[]; },): readonly EnumCaseFold[] {
  return members.flatMap(function foldsBelow({ folds, },): readonly EnumCaseFold[] {
    return folds;
  },);
}

/**
 Reads one string against its schema node.

 @param schema - node the string answers to

 @param received - string as the reply gave it

 @param path - where the node sits in the reply

 @returns Listed spelling where exactly one differs only in capitalization, else the string as it came

 @example
 ```ts
 const { value, } = foldText({ schema: { enum: ['major',], }, received: 'Major', path: 'reply.severity', },);
 // => 'major'
 ```
 */
function foldText(
  {
    schema,
    received,
    path,
  }: {
    readonly schema: Readonly<Record<string, unknown>>;
    readonly received: string;
    readonly path: string;
  },
): EnumCaseFolded {
  /**
   The listed spellings this string differs from only in capitalization. It is read as one of them only where
   there is exactly one, since a schema listing `ab` and `aB` does not say which a reply of `AB` meant.
   */
  const [taken, ...others] = matchingSpellingsOf({
    received,
    listed: listedStringsOf({ schema, },),
  },);
  if ((taken === undefined) || (others.length > 0))
    return {
      value: received,
      folds: [],
    };
  return {
    value: taken,
    folds: [
      {
        path,
        received,
        taken,
      },
    ],
  };
}

/**
 Reads one reply value against its schema node, taking back what differs only in capitalization.

 @param schema - node the value answers to

 @param value - reply value at that node

 @param path - where the node sits in the reply

 @returns Value to read onward and every value taken back, the node's own value and those under it

 @example
 ```ts
 const { value, folds, } = foldAt({ schema, value: { verdict: 'Consistent', }, path: 'reply', },);
 ```
 */
function foldAt(
  {
    schema,
    value,
    path,
  }: {
    readonly schema: unknown;
    readonly value: unknown;
    readonly path: string;
  },
): EnumCaseFolded {
  if (!isJsonRecord(schema,))
    return {
      value,
      folds: [],
    };

  if (isText(value,)) {
    return foldText({
      schema,
      received: value,
      path,
    },);
  }

  if (Array.isArray(value,)) {
    /**
     Each member read against the node every member answers to.
     */
    const members = value.map(function memberFolded(
      member: unknown,
      index: number,
    ): EnumCaseFolded {
      return foldAt({
        schema: schema.items,
        value: member,
        path: `${path}[${String(index,)}]`,
      },);
    },);

    /**
     Everything taken back under the list.
     */
    const folds = foldsOf({ members, },);
    return {
      value: (folds.length === 0)
        ? value
        : members.map(function valueOf({ value: member, },): unknown {
          return member;
        },),
      folds,
    };
  }

  /**
   The nodes the schema describes the properties of the reply by.
   */
  const { properties, } = schema;
  if (isJsonRecord(value,) && isJsonRecord(properties,)) {
    /**
     The properties of the reply, as pairs.
     */
    const pairs = Object.entries(value,);

    /**
     Each property of the reply, read where the schema describes it and kept as it came where it does not, beside
     its name.
     */
    const named = pairs.map(function propertyFolded([key, member,],): NamedFolded {
      if (!Object.hasOwn(
        properties,
        key,
      ))
        return {
          key,
          folded: {
            value: member,
            folds: [],
          },
        };
      return {
        key,
        folded: foldAt({
          schema: properties[key],
          value: member,
          path: childPathOf({
            path,
            key,
          },),
        },),
      };
    },);

    /**
     The reading of each property, without its name.
     */
    const members = named.map(function foldedOf({ folded, },): EnumCaseFolded {
      return folded;
    },);

    /**
     Everything taken back under the object.
     */
    const folds = foldsOf({ members, },);
    return {
      value: (folds.length === 0)
        ? value
        : Object.fromEntries(named.map(function pairOf({
          key,
          folded,
        },) {
          return [
            key,
            folded.value,
          ] as const;
        },),),
      folds,
    };
  }

  return {
    value,
    folds: [],
  };
}

/**
 Takes every value of a parsed reply that differs from a listed `enum` or `const` value only in capitalization back to
 the schema's own spelling.

 NO VALUE IS EVER INVENTED: a replacement is a string the schema lists, taken only where it is the one listed string
 matching ignoring capitalization. The reply handed in is not changed; a reply with nothing to take back is returned
 as the same object.

 @param schema - JSON schema the caller sent, whose `properties` and `items` are followed

 @param value - reply parsed from JSON

 @returns Value to read onward, and each value taken back with where it sat, what the reply gave and what
 the schema lists

 @example
 ```ts
 const { value, folds, } = foldEnumCase({
   schema: { type: 'object', properties: { verdict: { enum: ['consistent',], }, }, },
   value: { verdict: 'Consistent', },
 },);
 // => { value: { verdict: 'consistent' },
 //      folds: [{ path: 'reply.verdict', received: 'Consistent', taken: 'consistent' }] }
 ```
 */
export function foldEnumCase(
  {
    schema,
    value,
  }: {
    readonly schema: Readonly<Record<string, unknown>>;
    readonly value: unknown;
  },
): EnumCaseFolded {
  return foldAt({
    schema,
    value,
    path: 'reply',
  },);
}

//endregion Enum case fold
