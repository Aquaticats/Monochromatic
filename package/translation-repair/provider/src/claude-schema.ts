import { isJsonRecord, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import {
  NAMED_SCHEMA_MAPS,
  SCHEMA_BEARING,
  structuralRefusalOf,
} from './claude-schema-refusals.ts';

//region Claude schema
// THE SCHEMA ONE CLAUDE CALL SENDS, narrowed to what Anthropic's structured outputs accept. The bundled API skill
// (`shared/tool-use-concepts.md`, "JSON Schema Limitations", lines 541 to 558) says `additionalProperties: false` is
// required on every object, that the numeric bounds (`minimum`, `maximum`, `multipleOf`), the string bounds
// (`minLength`, `maxLength`) and the complex array bounds are not supported, and that the Python and TypeScript SDKs
// strip the bounds from what they send and check them on the reply (line 558). This client is raw HTTP, so no SDK does
// either for it; this file does.
//
// A STRIPPED BOUND IS TOLD TO THE MODEL, as the SDKs do it: step 2 of "How SDK transformation works" on the live
// Structured outputs page (https://platform.claude.com/docs/en/build-with-claude/structured-outputs, fetched
// 2026-10-22; the bundled skill has no such section) adds each unsupported constraint to the field's description, its
// example being `{minimum: 100}`. The form here is the page's: the dropped constraints of one schema node as one
// `{keyword: value, ...}` text, appended after a blank line to a description the node already has.
//
// THE STRUCTURED OUTPUTS PAGE IS THE AUTHORITY FOR WHAT THE FEATURE TAKES
// (https://platform.claude.com/docs/en/build-with-claude/structured-outputs, "JSON Schema limitations", read
// 2026-10-22). Its supported list, quoted: "String formats: `date-time`, `time`, `date`, `duration`, `email`,
// `hostname`, `uri`, `ipv4`, `ipv6`, `uuid`" and "Array `minItems` (only values 0 and 1 supported)". Its unsupported
// list, quoted: "Array constraints beyond `minItems` of 0 or 1". So `minItems` of 0 or 1 goes to the wire as written,
// any other `minItems` is stripped and told like the other bounds, and a `format` on the list of ten goes as written
// while any other is stripped and told, which is step 4 of "How SDK transformation works", "Filter string formats to
// the supported list only".
//
// `pattern` IS SUPPORTED. The page's "Pattern support (regex)" accordion, quoted: "Supported regex features: Full
// matching (`^...$`) and partial matching; Quantifiers: `*`, `+`, `?`, simple `{n,m}` cases; Character classes: `[]`,
// `.`, `\d`, `\w`, `\s`; Groups: `(...)`" and "NOT supported: Backreferences to groups (for example, `\1`, `\2`);
// Lookahead/lookbehind assertions (for example, `(?=...)`, `(?!...)`); Word boundaries: `\b`, `\B`; Complex `{n,m}`
// quantifiers with large ranges". The keyed probes (three runs of `probe3.ts`, of 26, 30 and 34 calls, the first on
// 2026-10-22 and the others on 2026-10-23, all claude-haiku-5-5) answered HTTP 200 to a `pattern` control, which
// agrees with the page. The reply obeyed the pattern where the same ask without the keyword did not (one reply for
// each arm), which shows the keyword changes the reply in that sample; it does not show whether the compiled grammar
// or the model reading the schema text did it
// (`doc/decision/translation-repair-direct-providers.md`, "Addendum: the keyed Claude probes"). So a `pattern`
// passes, except one that spells a construct the page names as unsupported, which fails here.
// A LARGE `{n,m}` QUANTIFIER IS NOT CHECKED: the page says only "Complex `{n,m}` quantifiers with large ranges" and
// names no bound, so no figure is invented here, and a pattern with a range the API finds too large may still
// answer HTTP 400.
//
// WHAT THE ALLOWLIST CANNOT SEE IS REFUSED IN `claude-schema-refusals.ts`: a named backreference (`\k` outside a class,
// refused here with the numbered ones), an `allOf` member that holds a `$ref`, an `enum` member that is not a string,
// a number, a Boolean or null, a `$ref` into another document, a `$ref` that names no schema, a recursive `$ref`, and a
// request past the page's limits of 24 optional and 16 union-typed parameters. That file's header quotes the
// page for each.
//
// A KEYWORD THE FEATURE REFUSES THAT NO SDK STRIPS STOPS THE BODY: `oneOf`, `not`, `if`, `then`, `else`,
// `patternProperties`, `propertyNames`, `contains`, `prefixItems` and any other keyword outside `ALLOWED_KEYWORDS`
// would answer HTTP 400 on every call of the stage, which the retry ladder returns unretried and the router counts as a
// lost voice. It fails here, at build time of the body, naming the keyword and where it sits.
//
// WHAT THE STRIPPED BOUNDS LEAVE TO THE READER: every stage reads its reply with its own guard, and a bound removed
// here is still enforced there. The only bound any stage schema declares is `minimum: 1` on a finding's paragraph in
// the absolute naturalness review, which `isAbsoluteNaturalnessReviewWire` rejects below 1; `claude-schemas.unit.test.ts`
// in `corpus-pass` lists what is stripped from every stage schema, so a bound added later shows there.

/**
 Refusal raised when a schema carries a keyword the structured-output feature does not support and no SDK strips.

 A THROW RATHER THAN A DATA OUTCOME, because it is a stage schema written wrong in our own code and not a thing a
 model did: the API would answer HTTP 400 to it on every call of the stage.

 @example
 ```ts
 throw new ClaudeSchemaKeywordError({ refusedKeyword: 'oneOf', schemaPath: 'root.pick', },);
 ```
 */
export class ClaudeSchemaKeywordError extends Error {
  /**
   Declares this message safe to forward: it names a schema keyword and a path of member names, both written in this
   package's own source.
   */
  readonly messageNamesOnly: true = true;

  /**
   Keyword the schema carries that the feature does not take, read by a check that lists every refusal at once.
   */
  readonly refusedKeyword: string;

  /**
   Dotted path of the schema node that carries it, `root` for the top.
   */
  readonly schemaPath: string;

  /**
   Builds the failure.

   @param refusedKeyword - keyword the schema carries

   @param schemaPath - dotted path of the schema node that carries it, member names included, `root` for the top

   @example
   ```ts
   new ClaudeSchemaKeywordError({ refusedKeyword: 'oneOf', schemaPath: 'root.pick', },);
   ```
   */
  public constructor(
    {
      refusedKeyword,
      schemaPath,
    }: {
      readonly refusedKeyword: string;
      readonly schemaPath: string;
    },
  ) {
    super(
      `The Claude route cannot send a schema that carries "${refusedKeyword}" at ${schemaPath}: structured outputs `
        + 'do not support it and no SDK strips it, so the API would answer HTTP 400 on every call of the stage; '
        + 'write the stage schema with the keywords the bundled API skill lists as supported, and check the reply in '
        + 'the stage\'s own guard',
    );
    this.name = 'ClaudeSchemaKeywordError';
    this.refusedKeyword = refusedKeyword;
    this.schemaPath = schemaPath;
  }
}

/**
 Keywords the structured-output feature does not support, which the SDKs strip and this file strips: the numeric
 bounds, the string bounds and the array bounds, the object-size bounds and `uniqueItems`. `minItems` of 0 or 1 is
 the one array bound the page lists as supported, so it is stripped only for another value.
 */
const STRIPPED_KEYWORDS: ReadonlySet<string> = new Set([
  'minimum',
  'maximum',
  'exclusiveMinimum',
  'exclusiveMaximum',
  'multipleOf',
  'minLength',
  'maxLength',
  'minItems',
  'maxItems',
  'uniqueItems',
  'minProperties',
  'maxProperties',
],);

/**
 Keywords a schema may carry into the request: the bundled API skill's supported list (basic types, `enum`, `const`,
 `anyOf`, `allOf`, `$ref` and `$defs`, string `format`, `additionalProperties`), the structure that list assumes
 (`properties`, `required`, `items`, `definitions`), `default`, `pattern` and `minItems` (the live Structured
 outputs page lists each as supported, `minItems` for 0 and 1 only and `pattern` without four constructs, see the
 header), and the annotations `description` and `title`, which the model reads and the feature does not constrain.
 */
const ALLOWED_KEYWORDS: ReadonlySet<string> = new Set([
  'type',
  'properties',
  'required',
  'items',
  'enum',
  'const',
  'anyOf',
  'allOf',
  '$ref',
  '$defs',
  'definitions',
  'default',
  'format',
  'pattern',
  'minItems',
  'additionalProperties',
  'description',
  'title',
],);

/**
 String formats the live Structured outputs page lists as supported, in its words: `date-time`, `time`, `date`,
 `duration`, `email`, `hostname`, `uri`, `ipv4`, `ipv6`, `uuid`.
 */
const SUPPORTED_FORMATS: ReadonlySet<unknown> = new Set([
  'date-time',
  'time',
  'date',
  'duration',
  'email',
  'hostname',
  'uri',
  'ipv4',
  'ipv6',
  'uuid',
],);

/**
 Whether the feature does not take a keyword as written, so the node drops it and tells the model in the description.

 @param keyword - keyword of the node

 @param value - value the node gives it

 @returns True for a bound the feature lists as unsupported, a `minItems` other than 0 or 1, and a
 `format` off the page's list

 @example
 ```ts
 const dropped = isDroppedKeyword({ keyword: 'minItems', value: 2, },);
 // => true
 ```
 */
function isDroppedKeyword(
  {
    keyword,
    value,
  }: {
    readonly keyword: string;
    readonly value: unknown;
  },
): boolean {
  if (keyword === 'minItems')
    return (value !== 0) && (value !== 1);
  if (keyword === 'format')
    return !SUPPORTED_FORMATS.has(value,);
  return STRIPPED_KEYWORDS.has(keyword,);
}

/**
 Openers of the lookahead assertions the page lists as not supported.
 */
const LOOKAHEAD_OPENERS: readonly string[] = [
  '(?=',
  '(?!',
];

/**
 Openers of the lookbehind assertions the page lists as not supported.
 */
const LOOKBEHIND_OPENERS: readonly string[] = [
  '(?<=',
  '(?<!',
];

/**
 Whether a character is a digit a backreference starts with.

 @param character - one character, or the empty text past the end of a pattern

 @returns Whether it is one of 1 to 9

 @example
 ```ts
 const digit = isBackreferenceDigit({ character: '1', },);
 // => true
 ```
 */
function isBackreferenceDigit({ character, }: { readonly character: string; },): boolean {
  return (character >= '1') && (character <= '9');
}

/**
 Whether one of the openers stands at a position of a pattern.

 @param pattern - regular expression source

 @param index - position read

 @param openers - texts that open the construct looked for

 @returns Whether any of them starts at the position

 @example
 ```ts
 const lookahead = opensAny({ pattern: 'a(?=b)', index: 1, openers: LOOKAHEAD_OPENERS, },);
 // => true
 ```
 */
function opensAny(
  {
    pattern,
    index,
    openers,
  }: {
    readonly pattern: string;
    readonly index: number;
    readonly openers: readonly string[];
  },
): boolean {
  return openers.some(function opens(opener,): boolean {
    return pattern.startsWith(
      opener,
      index,
    );
  },);
}

/**
 Names the first construct of a regular expression that the live Structured outputs page lists as not supported.

 ONE LINEAR PASS over the text with the escape and the character class tracked, so an escaped backslash followed by a
 digit and a `\b` inside a class (a backspace there) are data and not constructs.

 @param pattern - regular expression source a schema carries

 @returns First construct's name alone in a list, or no name where the pattern spells none of the four

 @example
 ```ts
 const construct = unsupportedPatternConstruct({ pattern: '^(a)\\1$', },);
 // => ['backreference',]
 ```
 */
function unsupportedPatternConstruct({ pattern, }: { readonly pattern: string; },): readonly string[] {
  /**
   Whether the scan stands inside a character class, set and cleared as the brackets pass.
   */
  const scan = { inClass: false, };
  for (let index = 0; index < pattern.length; index += 1) {
    /**
     Character read at this position.
     */
    const character = pattern.charAt(index,);
    if (character === '\\') {
      /**
       The character the backslash escapes.
       */
      const escaped = pattern.charAt(index + 1,);
      if ((!scan.inClass) && (isBackreferenceDigit({ character: escaped, },) || (escaped === 'k')))
        return ['backreference',];
      if ((!scan.inClass) && ((escaped === 'b') || (escaped === 'B')))
        return ['word boundary',];
      index += 1;
    }
    else if (character === '[')
      scan.inClass = true;
    else if (character === ']')
      scan.inClass = false;
    else if ((!scan.inClass) && opensAny({
      pattern,
      index,
      openers: LOOKAHEAD_OPENERS,
    },))
      return ['lookahead',];
    else if ((!scan.inClass) && opensAny({
      pattern,
      index,
      openers: LOOKBEHIND_OPENERS,
    },))
      return ['lookbehind',];
  }
  return [];
}

/**
 Whether a schema describes an object: it says so by `type`, or it declares members.

 @param schema - schema read, a record

 @returns Whether the feature requires `additionalProperties: false` on it

 @example
 ```ts
 const closed = describesObject({ schema: { type: 'object', }, },);
 ```
 */
function describesObject({ schema, }: { readonly schema: Readonly<Record<string, unknown>>; },): boolean {
  /**
   The `type` member, one name or a list of them.
   */
  const { type, } = schema;
  if (type === 'object')
    return true;
  if (Array.isArray(type,) && type.includes('object',))
    return true;
  return isJsonRecord(schema.properties,);
}

/**
 Narrows one value that is a schema, or passes it through where it is not a record (a boolean schema).

 @param value - value in a schema position

 @param path - where the value sits

 @returns Narrowed copy

 @throws {@link ClaudeSchemaKeywordError} when a schema under the value carries a keyword the feature refuses

 @example
 ```ts
 const narrowed = narrowedValue({ value: { type: 'object', }, path: 'root', },);
 ```
 */
function narrowedValue(
  {
    value,
    path,
  }: {
    readonly value: unknown;
    readonly path: string;
  },
): unknown {
  if (Array.isArray(value,)) {
    return value.map(function narrowItem(
      item: unknown,
      index,
    ): unknown {
      return narrowedValue({
        value: item,
        path: `${path}[${String(index,)}]`,
      },);
    },);
  }
  if (!isJsonRecord(value,))
    return value;
  return narrowedSchema({
    schema: value,
    path,
  },);
}

/**
 Narrows each schema of a name-to-schema map, keeping the names.

 @param map - map, as the schema wrote it

 @param path - where the map's node sits

 @param keyword - keyword the map is the value of, which names the path of its members when it is not `properties`

 @returns Copy whose values are narrowed

 @throws {@link ClaudeSchemaKeywordError} when a schema under the map carries a keyword the feature refuses

 @example
 ```ts
 const members = narrowedMap({ map: { age: { type: 'integer', minimum: 1, }, }, path: 'root', keyword: 'properties', },);
 ```
 */
function narrowedMap(
  {
    map,
    path,
    keyword,
  }: {
    readonly map: unknown;
    readonly path: string;
    readonly keyword: string;
  },
): unknown {
  if (!isJsonRecord(map,))
    return map;
  return Object.fromEntries(
    Object.entries(map,)
      .map(function narrowMember([name, member,],): readonly [
        string,
        unknown
      ] {
        return [
          name,
          narrowedValue({
            value: member,
            path: (keyword === 'properties') ? `${path}.${name}` : `${path}.${keyword}.${name}`,
          },),
        ];
      },),
  );
}

/**
 Text that tells the model which constraints were dropped from a schema node, in the form the live Structured outputs
 page gives (`{minimum: 100}`).

 @param dropped - dropped keywords and their values, in the order the schema wrote them

 @returns Text, braces included

 @example
 ```ts
 const told = droppedConstraintsText({ dropped: [['minimum', 1,],], },);
 // => '{minimum: 1}'
 ```
 */
function droppedConstraintsText(
  {
    dropped,
  }: {
    readonly dropped: readonly (readonly [
      string,
      unknown
    ])[];
  },
): string {
  return `{${
    dropped
      .map(function pairText([keyword, value,],): string {
        return `${keyword}: ${JSON.stringify(value,)}`;
      },)
      .join(', ',)
  }}`;
}

/**
 Narrows one schema node and everything under it.

 @param schema - node, as the stage wrote it, never edited

 @param path - where the node sits, `root` for the top

 @returns New node that shares no object with the one given

 @throws {@link ClaudeSchemaKeywordError} when the node carries a keyword the feature refuses

 @example
 ```ts
 const node = narrowedSchema({ schema: { type: 'integer', minimum: 1, }, path: 'root', },);
 ```
 */
function narrowedSchema(
  {
    schema,
    path,
  }: {
    readonly schema: Readonly<Record<string, unknown>>;
    readonly path: string;
  },
): Record<string, unknown> {
  /**
   The node's keywords and values in the order the schema wrote them.
   */
  const entries = Object.entries(schema,);
  for (const [keyword, value,] of entries) {
    if (!(STRIPPED_KEYWORDS.has(keyword,) || ALLOWED_KEYWORDS.has(keyword,))) {
      throw new ClaudeSchemaKeywordError({
        refusedKeyword: keyword,
        schemaPath: path,
      },);
    }
    if ((keyword === 'pattern') && ((typeof value) === 'string')) {
      /**
       Construct of the pattern that the feature does not take, if it spells one.
       */
      const [construct,] = unsupportedPatternConstruct({ pattern: value as string, },);
      if (construct !== undefined) {
        throw new ClaudeSchemaKeywordError({
          refusedKeyword: `pattern with a ${construct}`,
          schemaPath: path,
        },);
      }
    }
  }

  /**
   The constraints this node drops, which the model is told in the description.
   */
  const dropped = entries.filter(function isStripped([keyword, value,],): boolean {
    return isDroppedKeyword({
      keyword,
      value,
    },);
  },);

  /**
   The members that survive, each narrowed by what it holds.
   */
  const kept = Object.fromEntries(
    entries
      .filter(function isSupported([keyword, value,],): boolean {
        return !isDroppedKeyword({
          keyword,
          value,
        },);
      },)
      .map(function narrowEntry([keyword, value,],): readonly [
        string,
        unknown
      ] {
        if (NAMED_SCHEMA_MAPS.has(keyword,))
          return [
            keyword,
            narrowedMap({
              map: value,
              path,
              keyword,
            },),
          ];
        if (SCHEMA_BEARING.has(keyword,))
          return [
            keyword,
            narrowedValue({
              value,
              path: `${path}.${keyword}`,
            },),
          ];
        return [
          keyword,
          structuredClone(value,),
        ];
      },),
  );

  /**
   The description the node carries to the wire: its own, with the dropped constraints told after it.
   */
  const told = (dropped.length === 0)
    ? {}
    : {
      description: ((typeof kept.description) === 'string')
        ? `${kept.description}\n\n${droppedConstraintsText({ dropped, },)}`
        : droppedConstraintsText({ dropped, },),
    };
  return {
    ...kept,
    ...told,
    ...(describesObject({ schema, },) ? { additionalProperties: false, } : {}),
  };
}

/**
 Narrows a JSON Schema to what Anthropic's structured outputs accept: every object gets `additionalProperties: false`,
 the unsupported bounds are dropped and told to the model in the field's description, and a keyword the feature does
 not support that no SDK strips is refused. Nothing else changes.

 A BOUNDED WALK OVER THE SCHEMA'S OWN NESTING, which a stage writes by hand and which is a few levels deep; the
 narrowing never follows a `$ref` (a string here), and the check for a recursive one is in
 `claude-schema-refusals.ts`.

 @param schema - schema as the stage wrote it, never edited

 @returns New schema that shares no object with the one given

 @throws {@link ClaudeSchemaKeywordError} when a node of the schema carries a keyword outside the allowlist that is not
 one of the stripped bounds, or anything `structuralRefusalOf` refuses

 @example
 ```ts
 const sent = claudeSchemaOf({ schema: { type: 'object', properties: { age: { type: 'integer', minimum: 1, }, }, }, },);
 // => { type: 'object', properties: { age: { type: 'integer', description: '{minimum: 1}', }, }, additionalProperties: false, }
 ```
 */
export function claudeSchemaOf(
  { schema, }: { readonly schema: Readonly<Record<string, unknown>>; },
): Record<string, unknown> {
  /**
   The schema with its bounds dropped and told, or the refusal of a keyword outside the allowlist.
   */
  const narrowed = narrowedSchema({
    schema,
    path: 'root',
  },);

  /**
   What the schema carries that the page lists as not supported and the allowlist cannot see.
   */
  const [refusal,] = structuralRefusalOf({ schema, },);
  if (refusal !== undefined)
    throw new ClaudeSchemaKeywordError(refusal,);
  return narrowed;
}

//endregion Claude schema
