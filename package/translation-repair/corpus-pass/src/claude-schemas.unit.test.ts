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
 SEARCH READS THE SYNTAX TREE OF EACH FILE, and one case per form it reads or leaves unread holds it, each with
 imaginary files: a call that names a type argument, a builder written as any kind of function, a wrapper of a builder,
 a use through an import, an alias, a destructured member, a dynamic import or a module that exports the builder
 again, and the forms it does not follow. The fixtures `claude-schema-*-forms.test-fixture.ts` hold the files, and the
 header of `claude-schema-sources.test-fixture.ts` states, sentence by sentence, what the search reads and does not
 read.

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
import { BINDING_KIND_CASES, BUILDER_MEMBER_CASES, } from './claude-schema-binding-kind-forms.test-fixture.ts';
import { BUILDER_FORM_CASES, } from './claude-schema-builder-forms.test-fixture.ts';
import { blindSpotsOf, } from './claude-schema-blind-spots.test-fixture.ts';
import { CALLER_CASES, } from './claude-schema-caller-forms.test-fixture.ts';
import {
  BINDING_IN_FUNCTION_CASES,
  CALL_METHOD_CASES,
  LOAD_IN_PLACE_CASES,
} from './claude-schema-followed-forms.test-fixture.ts';
import { CYCLE_CASES, } from './claude-schema-cycle-forms.test-fixture.ts';
import { DESTINATION_CALL_CASES, } from './claude-schema-destination-forms.test-fixture.ts';
import { DESTINATION_OTHER_CASES, } from './claude-schema-destination-other-forms.test-fixture.ts';
import { DESTINATION_SENDER_CASES, } from './claude-schema-destination-sender-forms.test-fixture.ts';
import { DESTINATION_STORE_CASES, } from './claude-schema-destination-store-forms.test-fixture.ts';
import { DESTINATION_TOP_CASES, } from './claude-schema-destination-top-forms.test-fixture.ts';
import { CALL_FORM_CASES, type FormCase, textOf, } from './claude-schema-call-forms.test-fixture.ts';
import { WHITELIST_BEARING_CASES, } from './claude-schema-whitelist-bearing-forms.test-fixture.ts';
import { WHITELIST_CALL_CASES, } from './claude-schema-whitelist-call-forms.test-fixture.ts';
import { WHITELIST_CASES, } from './claude-schema-whitelist-forms.test-fixture.ts';
import { WHITELIST_MARK_CASES, } from './claude-schema-whitelist-mark-forms.test-fixture.ts';
import { WHITELIST_MODULE_CASES, } from './claude-schema-whitelist-module-forms.test-fixture.ts';
import { WHITELIST_PATH_CASES, } from './claude-schema-whitelist-path-forms.test-fixture.ts';
import { WHITELIST_READ_CASES, } from './claude-schema-whitelist-read-forms.test-fixture.ts';
import { BINDING_CASES, NAME_RULE_CASES, } from './claude-schema-binding-forms.test-fixture.ts';
import { DECLARATION_SHAPE_CASES, } from './claude-schema-declaration-forms.test-fixture.ts';
import { FORMAT_LIMIT_CASES, SPELLING_CASES, } from './claude-schema-format-forms.test-fixture.ts';
import {
  FOLLOWED_LOAD_CASES,
  UNAWAITED_LOAD_CASES,
  UNREAD_BUILDER_CASES,
} from './claude-schema-limit-forms.test-fixture.ts';
import { OUTSIDE_MODULE_CASES, } from './claude-schema-outside-module-forms.test-fixture.ts';
import { OVER_READ_CASES, } from './claude-schema-over-read-forms.test-fixture.ts';
import { NOT_A_REFERENCE_CASES, REFERENCE_POSITION_CASES, } from './claude-schema-position-forms.test-fixture.ts';
import { DEFAULT_EXPORT_CASES, RE_EXPORT_CASES, } from './claude-schema-re-export-forms.test-fixture.ts';
import { CALLBACK_CASES, } from './claude-schema-callback-forms.test-fixture.ts';
import { UNREAD_CALLBACK_CASES, } from './claude-schema-callback-unread-forms.test-fixture.ts';
import { CONSTANT_END_CASES, } from './claude-schema-constant-end-forms.test-fixture.ts';
import { CONSTANT_PIN_CASES, } from './claude-schema-constant-pin-forms.test-fixture.ts';
import { CONSTANT_READER_CASES, } from './claude-schema-constant-reader-forms.test-fixture.ts';
import { EXPORT_ASSIGNMENT_CASES, } from './claude-schema-export-assignment-forms.test-fixture.ts';
import { IMPORT_EQUALS_CASES, } from './claude-schema-import-equals-forms.test-fixture.ts';
import { FORWARDER_CHAIN_CASES, } from './claude-schema-forwarder-chain-forms.test-fixture.ts';
import {
  FORWARDER_CASES,
  WRAPPER_THAT_SENDS_CASE,
} from './claude-schema-forwarder-forms.test-fixture.ts';
import { UNDECLARED_FORWARDER_CASES, } from './claude-schema-forwarder-shape-forms.test-fixture.ts';
import { forwardersOf, } from './claude-schema-forwarders-found.test-fixture.ts';
import { PINNED_CASES, } from './claude-schema-pinned-forms.test-fixture.ts';
import { MUTANT_PIN_CASES, } from './claude-schema-mutant-pin-forms.test-fixture.ts';
import { SENDER_ALIAS_CASES, } from './claude-schema-sender-alias-forms.test-fixture.ts';
import { SENDER_NAMESPACE_CASES, } from './claude-schema-sender-namespace-forms.test-fixture.ts';
import { SENDER_PIN_CASES, } from './claude-schema-sender-pin-forms.test-fixture.ts';
import { SENDER_POSITION_CASES, } from './claude-schema-sender-position-forms.test-fixture.ts';
import { SENDER_SHADOW_CASES, } from './claude-schema-sender-shadow-forms.test-fixture.ts';
import { SPECIFIER_CASES, } from './claude-schema-specifier-forms.test-fixture.ts';
import { SURVIVOR_CASES, } from './claude-schema-survivor-forms.test-fixture.ts';
import { TAGGED_TEMPLATE_CASES, } from './claude-schema-tagged-template-forms.test-fixture.ts';
import { type ShapeCase, UNREAD_SHAPE_CASES, } from './claude-schema-unread-shape-forms.test-fixture.ts';
import { UNREADABLE_CODE_CASES, } from './claude-schema-unreadable-forms.test-fixture.ts';
import { CODE_FROM_TEXT_CASES, } from './claude-schema-code-from-text-forms.test-fixture.ts';
import { GLOBAL_ALIAS_CASES, } from './claude-schema-global-alias-forms.test-fixture.ts';
import { CALL_SHAPE_CASES, NOT_A_USE_CASES, } from './claude-schema-use-forms.test-fixture.ts';
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
import { WRAPPER_NESTING_CASES, } from './claude-schema-wrapper-nesting-forms.test-fixture.ts';
import { NOT_A_WRAPPER_CASES, TYPE_MARK_WRAPPER_CASES, } from './claude-schema-wrapper-limit-forms.test-fixture.ts';

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
// schema of their own. A search for the files that hold the schema text finds the file that holds the builder and no
// caller.

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

//region Forwarders
// Functions that make an answer schema from what their caller passes and hand it to a sender, and the functions that
// pass what they are given on to one of those, however long the chain, so the files that call any of them write the
// arguments of the schema and are listed beside the stages.

/**
 One function that hands a sender a builder call made from its own parameters, or passes a name made from them to one
 that does.
 */
type Forwarder = {
  /**
   The function's name.
   */
  readonly forwarder: string;

  /**
   Path of the file that declares it.
   */
  readonly declaredIn: string;

  /**
   Each file that calls it, which writes the arguments of the schema.
   */
  readonly callers: readonly string[];
};

/**
 Every function of the family that hands a sender a builder call made from its own parameters, or passes a name made
 from them to one that does, to the end of the chain, and every file that calls it. The first three each make the
 schema of a consistency ask from the labels shown and give back the bench's answer; the others pass what they are
 given on until the entry pipeline, so the files that call any of them are listed as stages and not reported for the
 answer they use. A function added or removed fails the case that reads these until it is listed here.
 */
const FORWARDERS: readonly Forwarder[] = [
  {
    forwarder: 'askCarryChoice',
    declaredIn: 'corpus-pass/src/corpus-run/pass-consistency-carry-ask.ts',
    callers: ['corpus-pass/src/corpus-run/pass-consistency-carry-choose.ts',],
  },
  {
    forwarder: 'askConsistencyBench',
    declaredIn: 'corpus-pass/src/corpus-run/pass-consistency-ask.ts',
    callers: ['corpus-pass/src/corpus-run/pass-consistency-bench-decide.ts',],
  },
  {
    forwarder: 'askRunOff',
    declaredIn: 'corpus-pass/src/corpus-run/pass-consistency-run-off.ts',
    callers: ['corpus-pass/src/corpus-run/pass-consistency-bench-decide.ts',],
  },
  {
    forwarder: 'decideAskedGroup',
    declaredIn: 'corpus-pass/src/corpus-run/pass-consistency-bench-decide.ts',
    callers: ['corpus-pass/src/corpus-run/pass-consistency-bench.ts',],
  },
  {
    forwarder: 'choose',
    declaredIn: 'corpus-pass/src/corpus-run/pass-consistency-carry-choose.ts',
    callers: ['corpus-pass/src/corpus-run/pass-consistency-carry.ts',],
  },
  {
    forwarder: 'askAll',
    declaredIn: 'corpus-pass/src/corpus-run/pass-consistency-bench.ts',
    callers: ['corpus-pass/src/corpus-run/pass-consistency.ts',],
  },
  {
    forwarder: 'carryUncontained',
    declaredIn: 'corpus-pass/src/corpus-run/pass-consistency-carry.ts',
    callers: [],
  },
  {
    forwarder: 'carryToUnit',
    declaredIn: 'corpus-pass/src/corpus-run/pass-consistency-carry.ts',
    callers: [],
  },
  {
    forwarder: 'settleAll',
    declaredIn: 'corpus-pass/src/corpus-run/pass-consistency.ts',
    callers: [],
  },
  {
    forwarder: 'carryAll',
    declaredIn: 'corpus-pass/src/corpus-run/pass-consistency-carry.ts',
    callers: ['corpus-pass/src/corpus-run/pass-consistency-bench.ts',],
  },
  {
    forwarder: 'runPassConsistency',
    declaredIn: 'corpus-pass/src/corpus-run/pass-consistency.ts',
    callers: ['corpus-pass/src/corpus-run/pass-settlement.ts',],
  },
  {
    forwarder: 'settleAskedGroup',
    declaredIn: 'corpus-pass/src/corpus-run/pass-consistency-bench.ts',
    callers: [],
  },
  {
    forwarder: 'runPassSettlement',
    declaredIn: 'corpus-pass/src/corpus-run/pass-settlement.ts',
    callers: ['corpus-pass/src/corpus-run/pass-entry.ts',],
  },
  {
    forwarder: 'runEntryPipeline',
    declaredIn: 'corpus-pass/src/corpus-run/pass-entry.ts',
    callers: ['corpus-pass/src/corpus-run/pass-entry-settle.ts',],
  },
  {
    forwarder: 'settleEntry',
    declaredIn: 'corpus-pass/src/corpus-run/pass-entry-settle.ts',
    callers: ['corpus-pass/src/corpus-run/corpus-pass.ts',],
  },
];

//endregion Forwarders

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
          ...new Set(FORWARDERS.flatMap(function forwarderCallersOf({ callers, },): readonly string[] {
            return callers;
          },),),
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

    it({
      name: 'NAMES WHAT THE PARSER FOUND WRONG IN A FILE THAT DOES NOT PARSE, so a file the search cannot read stops '
        + 'it with a message that says what is wrong and not with an object printed as text',
      fn: async () => {
        expect(function searchBrokenFile(): readonly string[] {
          return declaringFilesOf({
            sources: [{ file: 'imaginary/src/broken.ts', text: 'export const A = ;\n', },],
            notStageSchemas: NOT_STAGE_SCHEMAS,
          },);
        },).toThrow('imaginary/src/broken.ts does not parse: Unexpected token',);
      },
    },),

    ...[
      ...CALL_FORM_CASES,
      ...BUILDER_FORM_CASES,
      ...WRAPPER_FORM_CASES,
      ...WRAPPER_NESTING_CASES,
      ...WRAPPER_OPENING_CASES,
      ...WRAPPED_BUILDER_CASES,
      ...NO_BUILDER_CASES,
      ...DECLARATION_SHAPE_CASES,
      ...NOT_A_USE_CASES,
      ...CALL_SHAPE_CASES,
      ...NOT_A_WRAPPER_CASES,
      ...TYPE_MARK_WRAPPER_CASES,
      ...SPELLING_CASES,
      ...FORMAT_LIMIT_CASES,
      ...RE_EXPORT_CASES,
      ...DEFAULT_EXPORT_CASES,
      ...BINDING_CASES,
      ...NAME_RULE_CASES,
      ...UNREAD_BUILDER_CASES,
      ...UNREAD_SHAPE_CASES,
      ...FOLLOWED_LOAD_CASES,
      ...REFERENCE_POSITION_CASES,
      ...NOT_A_REFERENCE_CASES,
      ...OVER_READ_CASES,
      ...CYCLE_CASES,
      ...BINDING_IN_FUNCTION_CASES,
      ...LOAD_IN_PLACE_CASES,
      ...UNAWAITED_LOAD_CASES,
      ...CALL_METHOD_CASES,
      ...BINDING_KIND_CASES,
      ...BUILDER_MEMBER_CASES,
      ...DESTINATION_CALL_CASES,
      ...DESTINATION_OTHER_CASES,
      ...DESTINATION_SENDER_CASES,
      ...DESTINATION_STORE_CASES,
      ...DESTINATION_TOP_CASES,
      ...OUTSIDE_MODULE_CASES,
      ...UNREADABLE_CODE_CASES,
      ...GLOBAL_ALIAS_CASES,
      ...CODE_FROM_TEXT_CASES,
      ...WHITELIST_BEARING_CASES,
      ...WHITELIST_CALL_CASES,
      ...WHITELIST_CASES,
      ...WHITELIST_MARK_CASES,
      ...WHITELIST_MODULE_CASES,
      ...WHITELIST_PATH_CASES,
      ...WHITELIST_READ_CASES,
      ...SPECIFIER_CASES,
      ...SURVIVOR_CASES,
      ...CALLBACK_CASES,
      ...UNREAD_CALLBACK_CASES,
      ...CONSTANT_END_CASES,
      ...CONSTANT_READER_CASES,
      ...EXPORT_ASSIGNMENT_CASES,
      ...IMPORT_EQUALS_CASES,
      ...FORWARDER_CASES,
      ...FORWARDER_CHAIN_CASES,
      ...UNDECLARED_FORWARDER_CASES,
      WRAPPER_THAT_SENDS_CASE,
      ...CONSTANT_PIN_CASES,
      ...SENDER_PIN_CASES,
      ...PINNED_CASES,
      ...SENDER_POSITION_CASES,
      ...SENDER_SHADOW_CASES,
      ...SENDER_ALIAS_CASES,
      ...MUTANT_PIN_CASES,
      ...SENDER_NAMESPACE_CASES,
      ...TAGGED_TEMPLATE_CASES,
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

    ...[
      ...UNREAD_BUILDER_CASES,
      ...UNREAD_SHAPE_CASES,
      ...FOLLOWED_LOAD_CASES,
      ...DESTINATION_CALL_CASES,
      ...DESTINATION_OTHER_CASES,
      ...DESTINATION_SENDER_CASES,
      ...DESTINATION_STORE_CASES,
      ...DESTINATION_TOP_CASES,
      ...OUTSIDE_MODULE_CASES,
      ...UNREADABLE_CODE_CASES,
      ...GLOBAL_ALIAS_CASES,
      ...CODE_FROM_TEXT_CASES,
      ...WHITELIST_BEARING_CASES,
      ...WHITELIST_CALL_CASES,
      ...WHITELIST_CASES,
      ...WHITELIST_MARK_CASES,
      ...WHITELIST_MODULE_CASES,
      ...WHITELIST_PATH_CASES,
      ...WHITELIST_READ_CASES,
      ...SPECIFIER_CASES,
      ...SURVIVOR_CASES,
      ...CALLBACK_CASES,
      ...UNREAD_CALLBACK_CASES,
      ...CONSTANT_END_CASES,
      ...CONSTANT_READER_CASES,
      ...EXPORT_ASSIGNMENT_CASES,
      ...IMPORT_EQUALS_CASES,
      ...FORWARDER_CASES,
      ...FORWARDER_CHAIN_CASES,
      ...UNDECLARED_FORWARDER_CASES,
      WRAPPER_THAT_SENDS_CASE,
      ...CONSTANT_PIN_CASES,
      ...SENDER_PIN_CASES,
      ...PINNED_CASES,
      ...SENDER_POSITION_CASES,
      ...SENDER_SHADOW_CASES,
      ...SENDER_ALIAS_CASES,
      ...MUTANT_PIN_CASES,
      ...SENDER_NAMESPACE_CASES,
      ...TAGGED_TEMPLATE_CASES,
    ].map(function caseOfSpots(shapeCase: ShapeCase,) {
      return it({
        name: `REPORTS WITH FILE AND LINE THE SHAPE OF THIS CASE: ${shapeCase.name}`,
        fn: async () => {
          /**
           The sources checked: the contest builder's file and the shape's imaginary files.
           */
          const sources = [CONTEST_BUILDER_SOURCE, ...shapeCase.files,];
          expect(blindSpotsOf({ sources, notStageSchemas: NOT_STAGE_SCHEMAS, },)
            .filter(function isImaginary({ file, },): boolean {
              return file.startsWith('imaginary/',);
            },),).toEqual(shapeCase.spots,);
        },
      },);
    },),

    it({
      name: 'FINDS NO SHAPE THE SEARCH LEAVES UNREAD IN THE FAMILY\'S SOURCE, so a file that sends an answer schema '
        + 'cannot hide in one: each place that holds such a shape is named by file and line',
      fn: async () => {
        expect(blindSpotsOf({ sources: await familySources(), notStageSchemas: NOT_STAGE_SCHEMAS, },)
          .map(function named({ file, line, shape, },): string {
            return `${file}:${String(line,)} ${shape}`;
          },),).toEqual([],);
      },
    },),

    it({
      name: 'SKIPS A FILE NAMED AS NO STAGE SCHEMA WHEN IT LOOKS FOR A FUNCTION THAT HOLDS A FORMAT AND NO DECLARED '
        + 'FUNCTION HOLDS, and reports the same file when it is not named',
      fn: async () => {
        /**
         A file whose static method holds a format, which no declared function holds.
         */
        const sources = [
          {
            file: 'imaginary/src/not-stage-wire.ts',
            text: textOf([
              'export class NotStageWire {',
              '  static format() {',
              '    return { type: \'json_schema\', };',
              '  }',
              '}',
            ],),
          },
        ];
        expect({
          named: blindSpotsOf({ sources, notStageSchemas: new Set(['imaginary/src/not-stage-wire.ts',],), },),
          unnamed: blindSpotsOf({ sources, notStageSchemas: new Set<string>(), },),
        },).toEqual({
          named: [],
          unnamed: [
            {
              file: 'imaginary/src/not-stage-wire.ts',
              line: 2,
              shape: 'format-in-undeclared-function',
              position: 'FunctionExpression',
            },
          ],
        },);
      },
    },),

    it({
      name: 'SEARCHES THE BUILDERS OF THE SAME MODULES ONCE, whichever entry point asks and however the sources are '
        + 'listed, and again when the files named as no stage schema change, so the suite pays for the family once',
      fn: async () => {
        /**
         The family's sources.
         */
        const sources = await familySources();
        /**
         A file that writes a format, which the names decide whether the search reads.
         */
        const memoSources = [
          {
            file: 'imaginary/src/memo-wire.ts',
            text: textOf([
              'export function memoFormat() {',
              '  return { type: \'json_schema\', };',
              '}',
            ],),
          },
        ];
        /**
         A second file that writes a format, to list the files in either order.
         */
        const secondSource = {
          file: 'imaginary/src/memo-second-wire.ts',
          text: textOf([
            'export function secondFormat() {',
            '  return { type: \'json_schema\', };',
            '}',
          ],),
        };
        /**
         The builders found for the family, and again.
         */
        const first = buildersOf({ sources, notStageSchemas: NOT_STAGE_SCHEMAS, },);
        expect({
          sameSources: buildersOf({ sources, notStageSchemas: NOT_STAGE_SCHEMAS, },) === first,
          copiedSources: buildersOf({
            sources: [...sources,],
            notStageSchemas: new Set(NOT_STAGE_SCHEMAS,),
          },) === first,
          namedFile: buildersOf({
            sources: memoSources,
            notStageSchemas: new Set(['imaginary/src/memo-wire.ts',],),
          },).length,
          unnamedFile: buildersOf({ sources: memoSources, notStageSchemas: new Set<string>(), },).length,
          editedFile: buildersOf({
            sources: [{ file: 'imaginary/src/memo-wire.ts', text: 'export function memoFormat() {}\n', },],
            notStageSchemas: new Set<string>(),
          },).length,
          inOrder: buildersOf({ sources: [...memoSources, secondSource,], notStageSchemas: new Set<string>(), },)
            .map(function nameOf({ name, },): string {
              return name;
            },),
          reversed: buildersOf({ sources: [secondSource, ...memoSources,], notStageSchemas: new Set<string>(), },)
            .map(function nameOf({ name, },): string {
              return name;
            },),
        },).toEqual({
          sameSources: true,
          copiedSources: true,
          namedFile: 0,
          unnamedFile: 1,
          editedFile: 0,
          inOrder: ['memoFormat', 'secondFormat',],
          reversed: ['secondFormat', 'memoFormat',],
        },);
      },
    },),

    ...CALLER_CASES.map(function caseOfCallers(callerCase,) {
      return it({
        name: callerCase.name,
        fn: async () => {
          /**
           The sources searched: the contest builder's file and the case's imaginary files.
           */
          const sources = [CONTEST_BUILDER_SOURCE, ...callerCase.files,];
          /**
           The builder whose callers are listed.
           */
          const builder = nonNullishOrThrow(buildersOf({ sources, notStageSchemas: NOT_STAGE_SCHEMAS, },)
            .find(function isNamed({ name, },): boolean {
              return name === callerCase.builder;
            },),);
          expect(callersOf({ sources, builder, },).toSorted(),).toEqual(callerCase.callers,);
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
      name: 'LISTS EVERY FUNCTION THAT HANDS A SENDER A BUILDER CALL MADE FROM ITS OWN PARAMETERS AND EVERY FILE THAT '
        + 'CALLS IT, so a forwarder added later is named here before the files that write its arguments go unlisted',
      fn: async () => {
        /**
         The family's source.
         */
        const sources = await familySources();
        /**
         Forwarders found, and the callers of each, in the shape the list is written.
         */
        const found = Object.fromEntries(forwardersOf({ sources, notStageSchemas: NOT_STAGE_SCHEMAS, },)
          .map(function listed(forwarder,): readonly [string, Forwarder,] {
            return [
              forwarder.name,
              {
                forwarder: forwarder.name,
                declaredIn: forwarder.file,
                callers: callersOf({ sources, builder: forwarder, },).toSorted(),
              },
            ];
          },),);
        /**
         The forwarders the list names, in the same shape.
         */
        const listed = Object.fromEntries(FORWARDERS.map(function entryOf(forwarder,): readonly [string, Forwarder,] {
          return [forwarder.forwarder, forwarder,];
        },),);
        expect(found,).toEqual(listed,);
      },
    },),

    it({
      name: 'LISTS NO WRAPPER AS A FORWARDER, since the callers of a wrapper are listed already as the users of a '
        + 'builder',
      fn: async () => {
        expect(forwardersOf({
          sources: [CONTEST_BUILDER_SOURCE, ...WRAPPER_THAT_SENDS_CASE.files,],
          notStageSchemas: NOT_STAGE_SCHEMAS,
        },),).toEqual([],);
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
