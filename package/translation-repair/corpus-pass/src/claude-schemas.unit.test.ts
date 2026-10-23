/**
 Reads every answer schema a stage can send and holds it to what Anthropic's structured outputs accept. The bundled
 API skill (`shared/tool-use-concepts.md`, "JSON Schema Limitations") says
 `additionalProperties: false` is required on every object and that the numeric, string and array bounds are not
 supported. EVERY KEYWORD A SENT SCHEMA CARRIES MUST BE ON AN ALLOWLIST of what the feature supports (R10 of the second
 review), so a keyword nobody thought of fails here and not with HTTP 400 at the first live call. A schema Anthropic
 refuses answers HTTP 400 on every call, which the ladder returns unretried and the router counts as a lost voice, so
 a seat would be silently absent from a stage. The list of producers is `claude-schema-producers.ts`, which the pass
 reads at its start.

 THREE QUESTIONS, ASKED OF EACH SCHEMA PRODUCER: is every object of the schema as the stage writes it closed, does the
 body the Claude client builds send a schema that meets every rule, and which bounds does the body drop (only the one
 the reader of its stage enforces). A FOURTH CASE KEEPS THE LIST HONEST: it reads the source of the family for files
 that declare a `json_schema` and fails when one is missing here, so a new stage schema cannot skip the check. THAT
 SEARCH IS HELD IN TURN BY ONE CASE PER FORM IT FOLLOWS, each with imaginary files: a call that names a type argument,
 a builder handed over by reference, a spread of its result, a call through a namespace import or an import alias, a
 builder written as an arrow function, a function expression, the default export, a method of an object, an argument
 of a call, an async arrow with one bare parameter, a generic arrow or an async default export, and a wrapper of a
 function, of a method of an object or of a default export (returned at once, awaited, in parentheses, with a type
 argument, as an arrow function body, or through another wrapper). Wrappers of the other builder forms have no case
 of their own. The fixtures `claude-schema-call-forms.test-fixture.ts`,
 `claude-schema-builder-forms.test-fixture.ts`, `claude-schema-wrapper-forms.test-fixture.ts` and
 `claude-schema-wrapped-builder-forms.test-fixture.ts` hold the files, and the header of
 `claude-schema-code.test-fixture.ts` lists the shapes the search does not read.

 A FIFTH CASE HOLDS THE LIMIT OF THE ENUM CASE FOLD (`provider/src/enum-case-fold.ts`): the fold takes a reply value
 back to the spelling its schema lists only along `properties` and `items`, and leaves `anyOf`, `oneOf`, `allOf`,
 `$ref`, a schema-valued `additionalProperties` and tuple-form `items` untouched. It fails when a schema a stage sends
 carries one of those six forms, or lists an `enum` or `const` where the fold does not read, so a stage that adds one
 cannot lose a voice to a capital without this file saying so. A SIXTH CASE IS ITS POSITIVE CONTROL: it feeds the walk a
 schema with each form and a schema the fold reads, so a walk that stopped finding them could not pass.

 NOTHING HERE REACHES A PROVIDER: the body is built and read, never sent.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import { nonNullishOrThrow, } from '@monochromatic-dev/module-or-throw/ts';
import { ABSOLUTE_NATURALNESS_REVIEW_RESPONSE_FORMAT, } from '@monochromatic-dev/translation-repair-archive';
import { isJsonRecord, } from '@monochromatic-dev/translation-repair-document-structure';
import {
  buildClaudeBody,
  CLAUDE_MODELS,
  type JsonSchemaResponseFormat,
} from '@monochromatic-dev/translation-repair-provider';
import { CLAUDE_SCHEMA_PRODUCERS, } from '../dist/final/node/index.mjs';
import { BUILDER_FORM_CASES, } from './claude-schema-builder-forms.test-fixture.ts';
import { CALL_FORM_CASES, type FormCase, } from './claude-schema-call-forms.test-fixture.ts';
import {
  buildersOf,
  callersOf,
  declaringFilesOf,
  familySources,
  type SchemaBuilder,
  type SourceFile,
} from './claude-schema-sources.test-fixture.ts';
import {
  WRAPPED_BUILDER_CASES,
  WRAPPER_OPENING_CASES,
} from './claude-schema-wrapped-builder-forms.test-fixture.ts';
import { NO_BUILDER_CASES, WRAPPER_FORM_CASES, } from './claude-schema-wrapper-forms.test-fixture.ts';

//region Not stage schemas
// Files that are listed beside the producers without being one.

/**
 Files that mention a `json_schema` response format without declaring a stage's answer schema: the provider's own
 plumbing (the format's type, the tool projection, the Claude body) and the health probe, whose schema closes its one
 object and drops no bound.
 */
const NOT_STAGE_SCHEMAS: ReadonlySet<string> = new Set([
  'bench/src/corpus-run/model-health-probe.ts',
  'provider/src/anthropic-tool.ts',
  'provider/src/chat-contract.ts',
  'provider/src/claude-body.ts',
]);

//endregion Not stage schemas

//region Shared builders
// Functions that build an answer schema when called, so one file holds the schema text and several files each make a
// schema of their own. The search for the text `json_schema` finds the file that holds the builder and no caller.

/**
 One shared builder, the file that declares it and the files that call it.
 */
type SharedBuilder = {
  /**
   The function's name.
   */
  readonly builder: string;

  /**
   Path of the file that declares it, which is a producer in the list.
   */
  readonly declaredIn: string;

  /**
   Each file that calls it, with the name of the schema it makes where the list holds that schema by name.
   */
  readonly callers: readonly {
    readonly file: string;
    readonly schemaName?: string;
  }[];
};

/**
 A builder as the case that reads the source finds it, keyed by the builder's name: the callers are paths.
 */
type BuilderCallerFiles = {
  readonly declaredIn: string;
  readonly callers: readonly string[];
};

/**
 Every builder of an answer schema that more than one file can call, and every file that calls it. A stage that adds a
 call, or a builder of its own, fails the case that reads these until it is listed here, and a caller that names its
 schema fails the case on named callers until the producer list holds a schema of that name. The consistency
 builders make their schemas per ask from the labels shown, so the producer list holds one sample of each, and their
 callers carry no schema name here.
 */
const SHARED_BUILDERS: readonly SharedBuilder[] = [
  {
    builder: 'consistencyChoiceResponseFormat',
    declaredIn: 'consistency/src/consistency-choice.ts',
    callers: [
      { file: 'corpus-pass/src/corpus-run/pass-consistency-carry-ask.ts', },
      { file: 'corpus-pass/src/corpus-run/pass-consistency-run-off.ts', },
    ],
  },
  {
    builder: 'consistencyResponseFormat',
    declaredIn: 'consistency/src/consistency-wire.ts',
    callers: [
      { file: 'corpus-pass/src/corpus-run/pass-consistency-ask.ts', },
    ],
  },
  {
    builder: 'contestResponseFormat',
    declaredIn: 'lane-contest/src/contest-ballot-wire.ts',
    callers: [
      { file: 'consolidate/src/consolidate-gate-stage.ts', schemaName: 'consolidate_gate', },
      { file: 'consolidate/src/consolidation-polish-gate-stage.ts', schemaName: 'consolidation_polish_gate', },
      { file: 'lane-contest/src/lane-contest-stage.ts', schemaName: 'lane_contest', },
    ],
  },
];

//endregion Shared builders

//region Rules
// What the feature accepts, written out here on its own so the case does not lean on the sanitiser it checks.

/**
 The keywords the structured-output feature supports, from the bundled API skill (`shared/tool-use-concepts.md`, "JSON
 Schema Limitations", lines 541 to 558): the basic types, `enum`, `const`, `anyOf`, `allOf`, `$ref` and `$def`, the
 string formats and `additionalProperties`, with the structure that list assumes (`properties`, `required`, `items`,
 `definitions`, `$defs`), `default` (the live Structured outputs page lists it) and the annotations `description` and
 `title`. A keyword outside this list is refused by the API or stripped by the SDKs, and the Claude route sends neither.
 EVERY KEYWORD A SENT SCHEMA CARRIES MUST BE ON IT: an allowlist, so a keyword nobody thought of fails here and not with
 HTTP 400 at the first live call (R10 of the second review of the Claude route).
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
 Keywords whose value maps a name to a schema.
 */
const NAME_MAPS: ReadonlySet<string> = new Set(['properties', '$defs', 'definitions',],);

/**
 Keywords whose value is one schema.
 */
const SCHEMA_VALUED: ReadonlySet<string> = new Set(['items', 'not', 'if', 'then', 'else',],);

/**
 Keywords whose value is a list of schemas.
 */
const SCHEMA_LISTS: ReadonlySet<string> = new Set(['anyOf', 'allOf', 'oneOf', 'prefixItems',],);

/**
 One schema node and where it sits.
 */
type SchemaNode = {
  /**
   Dotted path from the root, names under `properties` included.
   */
  readonly path: string;

  /**
   The node.
   */
  readonly node: Readonly<Record<string, unknown>>;
};

/**
 Every schema node under a schema and the schema itself, a name under `properties` read as a name and not as a keyword.

 @param schema - schema read
 @param path - where it sits

 @returns Nodes, the root first

 @example
 ```ts
 const nodes = schemaNodesOf({ schema: { type: 'object', }, path: 'root', },);
 ```
 */
function schemaNodesOf(
  {
    schema,
    path,
  }: {
    readonly schema: Readonly<Record<string, unknown>>;
    readonly path: string;
  },
): readonly SchemaNode[] {
  return [
    { path, node: schema, },
    ...Object.entries(schema,)
      .flatMap(function nodesBelow([keyword, value,],): readonly SchemaNode[] {
        if (NAME_MAPS.has(keyword,) && isJsonRecord(value,)) {
          return Object.entries(value,)
            .flatMap(function nodesOfMember([name, member,],): readonly SchemaNode[] {
              return isJsonRecord(member,)
                ? schemaNodesOf({ schema: member, path: `${path}.${name}`, },)
                : [];
            },);
        }
        if (SCHEMA_VALUED.has(keyword,) && isJsonRecord(value,))
          return schemaNodesOf({ schema: value, path: `${path}.${keyword}`, },);
        if (SCHEMA_LISTS.has(keyword,) && Array.isArray(value,)) {
          return value.flatMap(function nodesOfItem(item: unknown, index,): readonly SchemaNode[] {
            return isJsonRecord(item,)
              ? schemaNodesOf({ schema: item, path: `${path}.${keyword}[${String(index,)}]`, },)
              : [];
          },);
        }
        return [];
      },),
  ];
}

/**
 The objects of a schema that are not closed with `additionalProperties: false`.

 @param schema - schema read

 @returns Paths of the open objects

 @example
 ```ts
 const open = openObjectsOf({ schema, },);
 ```
 */
function openObjectsOf({ schema, }: { readonly schema: Readonly<Record<string, unknown>>; },): readonly string[] {
  return schemaNodesOf({ schema, path: 'root', },)
    .filter(function isObject({ node, },): boolean {
      return (node.type === 'object') || isJsonRecord(node.properties,);
    },)
    .filter(function isOpen({ node, },): boolean {
      return node.additionalProperties !== false;
    },)
    .map(function pathOf({ path, },): string {
      return path;
    },);
}

/**
 The keywords a schema carries that are not on the allowlist, each as the path it sits at and its keyword.

 @param schema - schema read

 @returns One entry per keyword, `path: keyword`

 @example
 ```ts
 const outside = keywordsOutsideAllowlistOf({ schema, },);
 ```
 */
function keywordsOutsideAllowlistOf(
  { schema, }: { readonly schema: Readonly<Record<string, unknown>>; },
): readonly string[] {
  return schemaNodesOf({ schema, path: 'root', },)
    .flatMap(function outsideOf({ path, node, },): readonly string[] {
      return Object.keys(node,)
        .filter(function isOutside(keyword,): boolean {
          return !ALLOWED_KEYWORDS.has(keyword,);
        },)
        .map(function named(keyword,): string {
          return `${path}: ${keyword}`;
        },);
    },);
}

//endregion Rules

//region Forms the enum case fold does not read
// The six forms `foldEnumCase` leaves untouched, and the places it does read. Written here on their own, so the case
// does not lean on the fold it guards.

/**
 Keywords whose presence puts schemas where the enum case fold does not read.
 */
const UNREAD_KEYWORDS: readonly string[] = ['anyOf', 'oneOf', 'allOf', '$ref',];

/**
 The forms the enum case fold does not read that a schema carries, at any depth.

 @param schema - schema read, or a node of it
 @param path - where it sits

 @returns One entry per form, `path: form`

 @example
 ```ts
 const forms = unreadFormsOf({ schema: { oneOf: [], }, path: 'root', },);
 // => ['root: oneOf']
 ```
 */
function unreadFormsOf(
  {
    schema,
    path,
  }: {
    readonly schema: Readonly<Record<string, unknown>>;
    readonly path: string;
  },
): readonly string[] {
  /**
   The forms this node itself carries.
   */
  const own = [
    ...UNREAD_KEYWORDS.filter(function carried(keyword,): boolean {
      return Object.hasOwn(
        schema,
        keyword,
      );
    },),
    ...(isJsonRecord(schema.additionalProperties,) ? ['additionalProperties as a schema',] : []),
    ...(Array.isArray(schema.items,) ? ['items as a list of schemas',] : []),
  ].map(function named(form,): string {
    return `${path}: ${form}`;
  },);
  return [
    ...own,
    ...Object.entries(schema,)
      .flatMap(function formsBelow([keyword, value,],): readonly string[] {
        if ((keyword === 'properties') && isJsonRecord(value,)) {
          return Object.entries(value,)
            .flatMap(function formsOfMember([name, member,],): readonly string[] {
              return isJsonRecord(member,)
                ? unreadFormsOf({ schema: member, path: `${path}.${name}`, },)
                : [];
            },);
        }
        if (isJsonRecord(value,))
          return unreadFormsOf({ schema: value, path: `${path}.${keyword}`, },);
        if (Array.isArray(value,)) {
          return value.flatMap(function formsOfItem(item: unknown, index,): readonly string[] {
            return isJsonRecord(item,)
              ? unreadFormsOf({ schema: item, path: `${path}.${keyword}[${String(index,)}]`, },)
              : [];
          },);
        }
        return [];
      },),
  ];
}

/**
 The `enum` and `const` keywords a schema lists where the enum case fold does not read: anywhere but the root, a
 member of `properties` and a single `items` schema, reached through those alone.

 @param schema - schema read, or a node of it
 @param path - where it sits
 @param read - whether the fold reads this node

 @returns One entry per keyword, `path: keyword`

 @example
 ```ts
 const unread = choicesOutsideReadOf({ schema: { anyOf: [{ enum: ['a',], },], }, path: 'root', read: true, },);
 // => ['root.anyOf[0]: enum']
 ```
 */
function choicesOutsideReadOf(
  {
    schema,
    path,
    read,
  }: {
    readonly schema: Readonly<Record<string, unknown>>;
    readonly path: string;
    readonly read: boolean;
  },
): readonly string[] {
  return Object.entries(schema,)
    .flatMap(function choicesBelow([keyword, value,],): readonly string[] {
      if ((keyword === 'enum') || (keyword === 'const'))
        return read ? [] : [`${path}: ${keyword}`,];
      if ((keyword === 'properties') && isJsonRecord(value,)) {
        return Object.entries(value,)
          .flatMap(function choicesOfMember([name, member,],): readonly string[] {
            return isJsonRecord(member,)
              ? choicesOutsideReadOf({ schema: member, path: `${path}.${name}`, read, },)
              : [];
          },);
      }
      if ((keyword === 'items') && isJsonRecord(value,))
        return choicesOutsideReadOf({ schema: value, path: `${path}.items`, read, },);
      if (isJsonRecord(value,))
        return choicesOutsideReadOf({ schema: value, path: `${path}.${keyword}`, read: false, },);
      if (Array.isArray(value,)) {
        return value.flatMap(function choicesOfItem(item: unknown, index,): readonly string[] {
          return isJsonRecord(item,)
            ? choicesOutsideReadOf({ schema: item, path: `${path}.${keyword}[${String(index,)}]`, read: false, },)
            : [];
        },);
      }
      return [];
    },);
}

//endregion Forms the enum case fold does not read

/**
 Every schema of every producer, named by the file and the schema's own name.

 @returns One entry per schema

 @example
 ```ts
 const named = schemasOf();
 ```
 */
function schemasOf(): readonly {
  readonly label: string;
  readonly format: JsonSchemaResponseFormat;
}[] {
  return CLAUDE_SCHEMA_PRODUCERS.flatMap(function schemasOfProducer({ file, formats, },) {
    return formats.map(function labelled(format,) {
      return { label: `${file} (${format.json_schema.name})`, format, };
    },);
  },);
}

/**
 Files no family source has: a stage that builds its answer schema through a builder another stage shares, a builder
 of its own that a second file calls, and a file that only mentions a builder in its documentation.
 */
const IMAGINARY_SOURCES: readonly SourceFile[] = [
  {
    file: 'imaginary/src/cat-gate-stage.ts',
    text: [
      'export const CAT_GATE_RESPONSE_FORMAT = contestResponseFormat({',
      '  schemaName: \'cat_gate\',',
      '  verdicts: [\'purrs\', \'hisses\',],',
      '  asksArchive: false,',
      '},);',
    ].join('\n',),
  },
  {
    file: 'imaginary/src/dog-wire.ts',
    text: [
      'export function dogResponseFormat({ moods, }: { readonly moods: readonly string[]; },) {',
      '  return {',
      '    type: \'json_schema\',',
      '    json_schema: {',
      '      name: \'dog_mood\',',
      '      schema: { type: \'object\', properties: { mood: { enum: moods, }, }, },',
      '    },',
      '  };',
      '}',
    ].join('\n',),
  },
  {
    file: 'imaginary/src/dog-gate-stage.ts',
    text: 'export const DOG_GATE_RESPONSE_FORMAT = dogResponseFormat({ moods: [\'wags\',], },);\n',
  },
  {
    file: 'imaginary/src/mention.ts',
    text: [
      '/**',
      ' Builds the question. The schema comes from the contest builder:',
      '',
      ' const format = contestResponseFormat({ schemaName: \'x\', verdicts: [], asksArchive: false, },);',
      ' */',
      'export const NOTHING = 1;',
    ].join('\n',),
  },
];

/**
 The file of the family that declares the contest builder, which every form case is given beside its imaginary
 files.
 */
const CONTEST_BUILDER_FILE = 'lane-contest/src/contest-ballot-wire.ts';

/**
 The source of the file that declares the contest builder.
 */
const CONTEST_BUILDER_SOURCE: SourceFile = nonNullishOrThrow((await familySources()).find(
  function declaresContestBuilder({ file, },): boolean {
    return file === CONTEST_BUILDER_FILE;
  },
),);

await describe({
  name: 'the answer schemas the Claude route sends',
  concurrency: 1,
  children: [
    it({
      name: 'LISTS EVERY FILE OF THE FAMILY THAT DECLARES AN ANSWER SCHEMA, so a stage schema added later cannot skip '
        + 'the schema cases of this file: the family\'s source is searched for a json_schema format, and each '
        + 'file found is a producer here or a named exception',
      fn: async () => {
        /**
         Files of the family's source that write a `json_schema` response format.
         */
        const declaring = declaringFilesOf({ sources: await familySources(), notStageSchemas: NOT_STAGE_SCHEMAS, },);
        expect(declaring.toSorted(),).toEqual([
          ...CLAUDE_SCHEMA_PRODUCERS.map(function fileOf({ file, },): string {
            return file;
          },),
          ...NOT_STAGE_SCHEMAS,
          ...SHARED_BUILDERS.flatMap(function callerFilesOf({ callers, },): readonly string[] {
            return callers.map(function fileOfCaller({ file, },): string {
              return file;
            },);
          },),
        ].toSorted(),);
      },
    },),

    it({
      name: 'FINDS A STAGE THAT BUILDS ITS ANSWER SCHEMA THROUGH A BUILDER OTHER STAGES SHARE, which the search '
        + 'for the text json_schema alone misses: a file that calls the contest builder, a file that calls a '
        + 'builder of its own kind that a second file calls, and not a file that only shows the builder in '
        + 'its documentation',
      fn: async () => {
        /**
         Files found over the family's source with the imaginary files beside it.
         */
        const declaring = declaringFilesOf({
          sources: [...await familySources(), ...IMAGINARY_SOURCES,],
          notStageSchemas: NOT_STAGE_SCHEMAS,
        },);
        expect(declaring.filter(function isImaginary(file,): boolean {
          return file.startsWith('imaginary/',);
        },).toSorted(),).toEqual([
          'imaginary/src/cat-gate-stage.ts',
          'imaginary/src/dog-gate-stage.ts',
          'imaginary/src/dog-wire.ts',
        ],);
      },
    },),

    ...[
      ...CALL_FORM_CASES,
      ...BUILDER_FORM_CASES,
      ...WRAPPER_FORM_CASES,
      ...WRAPPER_OPENING_CASES,
      ...WRAPPED_BUILDER_CASES,
      ...NO_BUILDER_CASES,
    ].map(function caseOfForm(formCase: FormCase,) {
      return it({
        name: formCase.name,
        fn: async () => {
          /**
           The sources searched: the contest builder's file and the form's imaginary files.
           */
          const sources = [CONTEST_BUILDER_SOURCE, ...formCase.files,];
          expect({
            found: declaringFilesOf({ sources, notStageSchemas: NOT_STAGE_SCHEMAS, },)
              .filter(function isImaginary(file,): boolean {
                return file.startsWith('imaginary/',);
              },)
              .toSorted(),
            builders: buildersOf({ sources, notStageSchemas: NOT_STAGE_SCHEMAS, },)
              .filter(function isImaginary({ file, },): boolean {
                return file.startsWith('imaginary/',);
              },)
              .map(function nameOf({ name, },): string {
                return name;
              },)
              .toSorted(),
          },).toEqual({ found: formCase.found, builders: formCase.builders, },);
        },
      },);
    },),

    it({
      name: 'LISTS EVERY SHARED BUILDER OF AN ANSWER SCHEMA AND EVERY FILE THAT CALLS IT, so a fourth stage that calls '
        + 'the contest builder, or a builder added beside it, is named here before its schema goes unread',
      fn: async () => {
        /**
         The family's source.
         */
        const sources = await familySources();
        /**
         Builders found, and the callers of each, by the builder's name.
         */
        const found = Object.fromEntries(buildersOf({ sources, notStageSchemas: NOT_STAGE_SCHEMAS, },)
          .map(function listed(builder: SchemaBuilder,): readonly [string, BuilderCallerFiles,] {
            return [
              builder.name,
              {
                declaredIn: builder.file,
                callers: callersOf({ sources, builder, },).toSorted(),
              },
            ];
          },),);
        /**
         The builders the list names, in the same shape.
         */
        const listed = Object.fromEntries(SHARED_BUILDERS.map(function entryOf(shared,) {
          return [
            shared.builder,
            {
              declaredIn: shared.declaredIn,
              callers: shared.callers.map(function fileOfCaller({ file, },): string {
                return file;
              },).toSorted(),
            },
          ];
        },),);
        expect(found,).toEqual(listed,);
      },
    },),

    it({
      name: 'HOLDS THE SCHEMA OF EVERY NAMED CALLER OF A SHARED BUILDER IN THE PRODUCER LIST, under the builder\'s own '
        + 'file, so the walks of this file read what each stage sends',
      fn: async () => {
        /**
         Listed schemas of each producer file, by the schema's name.
         */
        const namesByFile = new Map(CLAUDE_SCHEMA_PRODUCERS.map(function namesOf({ file, formats, },) {
          return [file, formats.map(function nameOf(format,): string {
            return format.json_schema.name;
          },),] as const;
        },),);
        expect(SHARED_BUILDERS.flatMap(function missingFor({ declaredIn, callers, },): readonly string[] {
          return callers.flatMap(function missingOf({ file, schemaName, },): readonly string[] {
            if (!namesByFile.has(declaredIn,))
              return [`${declaredIn} is no producer`,];
            return (schemaName === undefined) || (namesByFile.get(declaredIn,) ?? []).includes(schemaName,)
              ? []
              : [`${file} makes ${schemaName}, which ${declaredIn} does not list`,];
          },);
        },),).toEqual([],);
      },
    },),

    it({
      name: 'CLOSES EVERY OBJECT OF EVERY SCHEMA AS THE STAGE WRITES IT, so the schema is acceptable to Anthropic '
        + 'without the client narrowing it and the omission is never relied on (the block pairing protocol and '
        + 'the contest ballot once left objects open)',
      fn: async () => {
        expect(schemasOf()
          .flatMap(function openIn({ label, format, },): readonly string[] {
            return openObjectsOf({ schema: format.json_schema.schema, },)
              .map(function named(path,): string {
                return `${label} ${path}`;
              },);
          },),).toEqual([],);
      },
    },),

    it({
      name: 'SENDS A SCHEMA THAT MEETS EVERY RULE, for every schema, through the Claude body on both models: every '
        + 'object closed, every keyword on the allowlist of what the feature supports, and the name and members of the '
        + 'schema intact',
      fn: async () => {
        for (const modelId of ['claude-sonnet-5-5', 'claude-haiku-5-5',] as const) {
          /**
           Faults found in the schemas the body carries for this model.
           */
          const faults = schemasOf()
            .flatMap(function faultsOf({ label, format, },): readonly string[] {
              const body = buildClaudeBody({
                card: CLAUDE_MODELS[modelId],
                messages: [{ role: 'user', content: 'Where does the cat sleep?', },],
                responseFormat: format,
                maxTokens: 100,
              },);
              /**
               The schema the body sends.
               */
              const sent = ((body.output_config as { readonly format: { readonly schema: unknown; }; }).format
                .schema) as Readonly<Record<string, unknown>>;
              return [
                ...openObjectsOf({ schema: sent, },)
                  .map(function openNamed(path,): string {
                    return `${label} sends an open object at ${path}`;
                  },),
                ...keywordsOutsideAllowlistOf({ schema: sent, },)
                  .map(function outsideNamed(outside,): string {
                    return `${label} sends ${outside}`;
                  },),
                ...(JSON.stringify(Object.keys((sent.properties ?? {}) as object,),)
                  === JSON.stringify(Object.keys((format.json_schema.schema.properties ?? {}) as object,),)
                  ? []
                  : [`${label} changed its members`,]),
              ];
            },);
          expect(faults,).toEqual([],);
        }
      },
    },),

    it({
      name: 'DROPS ONE BOUND IN THE WHOLE FAMILY, minimum 1 on the paragraph of a naturalness finding, which '
        + '`isAbsoluteNaturalnessReviewWire` still enforces (its case "REFUSES A FINDING WITH paragraph BELOW '
        + 'ONE"), and tells the model it in the field\'s description (R6); a keyword outside the allowlist added '
        + 'to any other schema lists here, which is also the positive control that the allowlist finds what it is '
        + 'there to find',
      fn: async () => {
        /**
         The schema the body sends for the naturalness review, whose one bound is stripped and told.
         */
        const sentReview = buildClaudeBody({
          card: CLAUDE_MODELS['claude-sonnet-5-5'],
          messages: [{ role: 'user', content: 'Where does the cat sleep?', },],
          responseFormat: ABSOLUTE_NATURALNESS_REVIEW_RESPONSE_FORMAT,
          maxTokens: 100,
        },);
        expect(JSON.stringify(sentReview.output_config,),).toContain('"description":"{minimum: 1}"',);
        expect(schemasOf()
          .flatMap(function boundsIn({ label, format, },): readonly string[] {
            return keywordsOutsideAllowlistOf({ schema: format.json_schema.schema, },)
              .map(function named(bound,): string {
                return `${label} ${bound}`;
              },);
          },),).toEqual([
            'archive/src/absolute-naturalness-review-wire.ts (absolute_naturalness_review) '
              + 'root.findings.items.paragraph: minimum',
          ],);
      },
    },),

    it({
      name: 'CARRIES NO anyOf, oneOf, allOf, $ref, SCHEMA-VALUED additionalProperties OR LIST-FORM items IN ANY '
        + 'SCHEMA A STAGE SENDS, and lists no enum or const where the enum case fold does not read, so a capital '
        + 'that differs from a listed value cannot cost a voice in a place the fold never looks',
      fn: async () => {
        expect(schemasOf()
          .flatMap(function unreadIn({ label, format, },): readonly string[] {
            return [
              ...unreadFormsOf({ schema: format.json_schema.schema, path: 'root', },),
              ...choicesOutsideReadOf({ schema: format.json_schema.schema, path: 'root', read: true, },),
            ].map(function named(unread,): string {
              return `${label} ${unread}`;
            },);
          },),).toEqual([],);
      },
    },),

    it({
      name: 'FINDS EACH OF THE SIX FORMS AND EACH enum OR const OUTSIDE THE FOLD\'S READING in a schema made for the '
        + 'purpose, and nothing in a schema whose enums sit under properties and items, which is the positive '
        + 'control that the walk of every stage schema can fail',
      fn: async () => {
        /**
         A schema carrying every form the fold does not read, with an enum under each of the keywords that hold one.
         */
        const carrying = {
          type: 'object',
          properties: {
            a: { anyOf: [{ enum: ['x',], },], },
            b: { oneOf: [{ const: 'x', },], },
            c: { allOf: [{ enum: ['x',], },], },
            d: { $ref: '#/$defs/d', },
            e: { type: 'object', additionalProperties: { enum: ['x',], }, },
            f: { type: 'array', items: [{ enum: ['x',], },], },
          },
          $defs: { d: { enum: ['x',], }, },
        };
        expect(unreadFormsOf({ schema: carrying, path: 'root', },),).toEqual([
          'root.a: anyOf',
          'root.b: oneOf',
          'root.c: allOf',
          'root.d: $ref',
          'root.e: additionalProperties as a schema',
          'root.f: items as a list of schemas',
        ],);
        expect(choicesOutsideReadOf({ schema: carrying, path: 'root', read: true, },),).toEqual([
          'root.a.anyOf[0]: enum',
          'root.b.oneOf[0]: const',
          'root.c.allOf[0]: enum',
          'root.e.additionalProperties: enum',
          'root.f.items[0]: enum',
          'root.$defs.d: enum',
        ],);

        /**
         A schema whose every enum and const sit where the fold reads: the root, a property, and a list's one schema.
         */
        const read = {
          type: 'object',
          additionalProperties: false,
          properties: {
            mood: { type: 'string', enum: ['calm',], },
            kind: { const: 'ballot', },
            findings: {
              type: 'array',
              items: { type: 'object', additionalProperties: false, properties: { severity: { enum: ['minor',], }, }, },
            },
          },
        };
        expect(unreadFormsOf({ schema: read, path: 'root', },),).toEqual([],);
        expect(choicesOutsideReadOf({ schema: read, path: 'root', read: true, },),).toEqual([],);
      },
    },),
  ],
},);
