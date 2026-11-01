/**
 Holds the family's source to the list of answer schemas the Claude route sends (ledger B1265: the case sat in the
 corpus-pass package's tests, which read the source of every package; a test of one package must not, so the check whose
 subject is the whole family's source lives here, where the input is the whole family and the gate runs this package's
 whole `test:unit` whatever changed).
 The list of producers is `claude-schema-producers.ts` in the corpus-pass package, which the pass reads at its start;
 the cases that hold each listed schema to what Anthropic's structured outputs accept stay in
 `corpus-pass/src/claude-schemas.unit.test.ts`.

 THE CASE THAT KEEPS THE LIST HONEST reads the source of the family for files that declare a `json_schema` and fails
 when one is neither a producer nor a named exception, so a new stage schema cannot skip the schema cases. THAT SEARCH
 READS THE SYNTAX TREE OF EACH FILE, and one case per form it reads or leaves unread holds it, each with imaginary
 files: a call that names a type argument, a builder written as any kind of function, a wrapper of a builder, a use
 through an import, an alias, a destructured member, a dynamic import or a module that exports the builder again, and
 the forms it does not follow. The fixtures `claude-schema-*-forms.test-fixture.ts` hold the files, and the header of
 `claude-schema-sources.test-fixture.ts` states, sentence by sentence, what the search reads and does not read. The
 files of the producer list are read off the source of `claude-schema-producers.ts` (`producerFilesOf`), and a case in
 the corpus-pass package holds that reading to the list the pass runs.

 NOTHING HERE REACHES A PROVIDER.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';
import { BINDING_KIND_CASES, BUILDER_MEMBER_CASES, } from './claude-schema-binding-kind-forms.test-fixture.ts';
import { BUILDER_FORM_CASES, } from './claude-schema-builder-forms.test-fixture.ts';
import {
  blindSpotsOf,
  spotsOf,
} from './claude-schema-blind-spots.test-fixture.ts';
import {
  SPOT_KEY_CASES,
  type SpotKeyCase,
} from './claude-schema-spot-key-forms.test-fixture.ts';
import {
  NAMED_PLACES,
  placesWithoutSpot,
  spotsNotNamed,
} from './claude-schema-named-places.test-fixture.ts';
import {
  FLOW_PLACE,
  NAMED_FLOW_CASES,
  type NamedFlowCase,
} from './claude-schema-named-flow-forms.test-fixture.ts';
import {
  BARE_NAME_IS_NO_KEY,
  NAMED_PLACE_CASES,
  type NamedPlaceCase,
} from './claude-schema-named-place-forms.test-fixture.ts';
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
import { CALLBACK_PIN_CASES, } from './claude-schema-callback-pin-forms.test-fixture.ts';
import { SCOPE_PIN_CASES, } from './claude-schema-scope-pin-forms.test-fixture.ts';
import { TEXT_PIN_CASES, } from './claude-schema-text-pin-forms.test-fixture.ts';
import { TEXT_CALL_CASES, } from './claude-schema-text-call-forms.test-fixture.ts';
import { TEXT_KEY_CASES, } from './claude-schema-text-key-forms.test-fixture.ts';
import { TEXT_STATIC_CASES, } from './claude-schema-text-static-forms.test-fixture.ts';
import { TEXT_ARGUMENT_CASES, } from './claude-schema-text-argument-forms.test-fixture.ts';
import { LOADER_OVERREAD_CASES, } from './claude-schema-loader-overread-forms.test-fixture.ts';
import { KEY_COLLISION_CASES, } from './claude-schema-key-collision-forms.test-fixture.ts';
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
import { FORWARDER_POSITION_CASES, } from './claude-schema-forwarder-position-forms.test-fixture.ts';
import { FORWARDER_NAMESPACE_CASES, } from './claude-schema-forwarder-namespace-forms.test-fixture.ts';
import { FORWARDER_BARREL_CASES, } from './claude-schema-forwarder-barrel-forms.test-fixture.ts';
import { FORWARDER_LOAD_CASES, } from './claude-schema-forwarder-load-forms.test-fixture.ts';
import { FORWARDER_OBJECT_CASES, } from './claude-schema-forwarder-object-forms.test-fixture.ts';
import { UNREAD_OBJECT_CASES, } from './claude-schema-forwarder-object-pin-forms.test-fixture.ts';
import { SENDER_SCOPE_CASES, } from './claude-schema-sender-scope-forms.test-fixture.ts';
import { METHOD_CALLBACK_CASES, } from './claude-schema-method-callback-forms.test-fixture.ts';
import { CLOSING_CASES, } from './claude-schema-closing-forms.test-fixture.ts';
import { IMPORT_CHAIN_CASES, } from './claude-schema-import-chain-forms.test-fixture.ts';
import { FORWARDER_VALUE_CASES, } from './claude-schema-forwarder-value-forms.test-fixture.ts';
import { DERIVED_NAME_CASES, } from './claude-schema-derived-forms.test-fixture.ts';
import { DERIVED_LOOP_CASES, } from './claude-schema-derived-loop-forms.test-fixture.ts';
import { DERIVED_CONTAINER_CASES, } from './claude-schema-derived-container-forms.test-fixture.ts';
import { DERIVED_DECLARATION_CASES, } from './claude-schema-derived-declaration-forms.test-fixture.ts';
import { DERIVED_TARGET_CASES, } from './claude-schema-derived-target-forms.test-fixture.ts';
import { DERIVED_CALLBACK_CASES, } from './claude-schema-derived-callback-forms.test-fixture.ts';
import { DERIVED_IIFE_CASES, } from './claude-schema-derived-iife-forms.test-fixture.ts';
import { DERIVED_LIMIT_CASES, } from './claude-schema-derived-limit-forms.test-fixture.ts';
import { DERIVED_STATIC_CASES, } from './claude-schema-derived-static-forms.test-fixture.ts';
import { CALLBACK_SCOPE_CASES, } from './claude-schema-callback-scope-forms.test-fixture.ts';
import { PARAMETER_PROPERTY_CASES, } from './claude-schema-parameter-property-forms.test-fixture.ts';
import { SENDER_NAMESPACE_CASES, } from './claude-schema-sender-namespace-forms.test-fixture.ts';
import { SENDER_PIN_CASES, } from './claude-schema-sender-pin-forms.test-fixture.ts';
import { SENDER_POSITION_CASES, } from './claude-schema-sender-position-forms.test-fixture.ts';
import { SENDER_SHADOW_CASES, } from './claude-schema-sender-shadow-forms.test-fixture.ts';
import { SPECIFIER_CASES, } from './claude-schema-specifier-forms.test-fixture.ts';
import { SURVIVOR_CASES, } from './claude-schema-survivor-forms.test-fixture.ts';
import { TAGGED_TEMPLATE_CASES, } from './claude-schema-tagged-template-forms.test-fixture.ts';
import { MEMBER_KEY_CASES, } from './claude-schema-member-key-forms.test-fixture.ts';
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
import { producerFilesOf, } from './claude-schema-producer-files.test-fixture.ts';
import { SHARED_BUILDERS, } from './claude-schema-shared-builders.test-fixture.ts';
import {
  expectFindingsAsListed,
  expectNoFindings,
  expectRecordAsListed,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/scan-findings.test-fixture.ts';

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
 The file of the family that declares the producer list, as `familySources` names it.
 */
const PRODUCER_LIST_FILE = 'corpus-pass/src/claude-schema-producers.ts';

/**
 The source of the file that declares the contest builder.
 */
const CONTEST_BUILDER_SOURCE: SourceFile = nonNullishOrThrow((await familySources()).find(
  function declaresContestBuilder({ file, },): boolean {
    return file === CONTEST_BUILDER_FILE;
  },
),);

await describe({
  name: 'the search of the family\'s source for answer schemas',
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
        const sources = await familySources();
        const declaring = declaringFilesOf({ sources, notStageSchemas: NOT_STAGE_SCHEMAS, },);
        /**
         The producer list's source, whose file column names the producers.
         */
        const producers = nonNullishOrThrow(sources.find(function isProducerList({ file, },): boolean {
          return file === PRODUCER_LIST_FILE;
        },),);
        expectFindingsAsListed({
          findings: declaring.toSorted(),
          listed: [
            ...producerFilesOf({ text: producers.text, },),
            ...NOT_STAGE_SCHEMAS,
            ...SHARED_BUILDERS.flatMap(function callerFilesOf({ callers, },): readonly string[] {
              return callers.map(function fileOfCaller({ file, },): string {
                return file;
              },);
            },),
            ...new Set(FORWARDERS.flatMap(function forwarderCallersOf({ callers, },): readonly string[] {
              return callers;
            },),),
          ].toSorted(),
        },);
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
        expectFindingsAsListed({
          findings: declaring.filter(function isImaginary(file,): boolean {
            return file.startsWith('imaginary/',);
          },).toSorted(),
          listed: [
            'imaginary/src/cat-gate-stage.ts',
            'imaginary/src/dog-gate-stage.ts',
            'imaginary/src/dog-wire.ts',
          ],
        },);
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
      ...CALLBACK_PIN_CASES,
      ...SCOPE_PIN_CASES,
      ...TEXT_PIN_CASES,
      ...TEXT_CALL_CASES,
      ...TEXT_KEY_CASES,
      ...TEXT_STATIC_CASES,
      ...TEXT_ARGUMENT_CASES,
      ...LOADER_OVERREAD_CASES,
      ...KEY_COLLISION_CASES,
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
      ...FORWARDER_POSITION_CASES,
      ...FORWARDER_NAMESPACE_CASES,
      ...FORWARDER_BARREL_CASES,
      ...FORWARDER_LOAD_CASES,
      ...FORWARDER_OBJECT_CASES,
      ...UNREAD_OBJECT_CASES,
      ...SENDER_SCOPE_CASES,
      ...METHOD_CALLBACK_CASES,
      ...CLOSING_CASES,
      ...IMPORT_CHAIN_CASES,
      ...FORWARDER_VALUE_CASES,
      ...DERIVED_NAME_CASES,
      ...DERIVED_LOOP_CASES,
      ...DERIVED_CONTAINER_CASES,
      ...DERIVED_DECLARATION_CASES,
      ...DERIVED_TARGET_CASES,
      ...DERIVED_CALLBACK_CASES,
      ...DERIVED_IIFE_CASES,
      ...DERIVED_LIMIT_CASES,
      ...DERIVED_STATIC_CASES,
      ...CALLBACK_SCOPE_CASES,
      ...PARAMETER_PROPERTY_CASES,
      ...MUTANT_PIN_CASES,
      ...SENDER_NAMESPACE_CASES,
      ...TAGGED_TEMPLATE_CASES,
      ...MEMBER_KEY_CASES,
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
      ...CALLBACK_PIN_CASES,
      ...SCOPE_PIN_CASES,
      ...TEXT_PIN_CASES,
      ...TEXT_CALL_CASES,
      ...TEXT_KEY_CASES,
      ...TEXT_STATIC_CASES,
      ...TEXT_ARGUMENT_CASES,
      ...LOADER_OVERREAD_CASES,
      ...KEY_COLLISION_CASES,
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
      ...FORWARDER_POSITION_CASES,
      ...FORWARDER_NAMESPACE_CASES,
      ...FORWARDER_BARREL_CASES,
      ...FORWARDER_LOAD_CASES,
      ...FORWARDER_OBJECT_CASES,
      ...UNREAD_OBJECT_CASES,
      ...SENDER_SCOPE_CASES,
      ...METHOD_CALLBACK_CASES,
      ...CLOSING_CASES,
      ...IMPORT_CHAIN_CASES,
      ...FORWARDER_VALUE_CASES,
      ...DERIVED_NAME_CASES,
      ...DERIVED_LOOP_CASES,
      ...DERIVED_CONTAINER_CASES,
      ...DERIVED_DECLARATION_CASES,
      ...DERIVED_TARGET_CASES,
      ...DERIVED_CALLBACK_CASES,
      ...DERIVED_IIFE_CASES,
      ...DERIVED_LIMIT_CASES,
      ...DERIVED_STATIC_CASES,
      ...CALLBACK_SCOPE_CASES,
      ...PARAMETER_PROPERTY_CASES,
      ...MUTANT_PIN_CASES,
      ...SENDER_NAMESPACE_CASES,
      ...TAGGED_TEMPLATE_CASES,
      ...MEMBER_KEY_CASES,
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
        + 'cannot hide in one: each place that holds such a shape is named by file and line, unless the test names it',
      fn: async () => {
        expectNoFindings({
          findings: spotsNotNamed({
            spots: blindSpotsOf({ sources: await familySources(), notStageSchemas: NOT_STAGE_SCHEMAS, },),
            places: NAMED_PLACES,
          },)
            .map(function named({ file, line, shape, held, },): string {
              return `${file}:${String(line,)} ${shape} ${held ?? ''}`;
            },),
        },);
      },
    },),

    it({
      name: 'REPORTS EVERY PLACE THE TEST NAMES IN THE FAMILY\'S SOURCE, so a name does not outlive the place it names',
      fn: async () => {
        /**
         The places the check reports over the family's source.
         */
        const spots = blindSpotsOf({ sources: await familySources(), notStageSchemas: NOT_STAGE_SCHEMAS, },);
        expectNoFindings({
          findings: placesWithoutSpot({
            spots,
            places: NAMED_PLACES,
          },).map(function fileOf({ file, },): string {
            return file;
          },),
        },);
      },
    },),

    ...NAMED_PLACE_CASES.map(function caseOfNames(namedCase: NamedPlaceCase,) {
      return it({
        name: `COMPARES THE REPORTS OF THE CHECK WITH THE PLACES THE TEST NAMES: ${namedCase.name}`,
        fn: async () => {
          expect({
            unnamed: spotsNotNamed({
              spots: namedCase.spots,
              places: namedCase.places,
            },),
            stale: placesWithoutSpot({
              spots: namedCase.spots,
              places: namedCase.places,
            },),
          },).toEqual({
            unnamed: namedCase.unnamed,
            stale: namedCase.stale,
          },);
        },
      },);
    },),

    ...SPOT_KEY_CASES.map(function caseOfKey(keyCase: SpotKeyCase,) {
      return it({
        name: `KEEPS THE REPORTS OF NODES AS PLACES BY THEIR TYPE AND THEIR RANGE: ${keyCase.name}`,
        fn: async () => {
          expect(spotsOf({
            text: 'a\n'.repeat(40,),
            found: keyCase.found,
            file: 'imaginary/src/key.ts',
          },).length,).toBe(keyCase.places,);
        },
      },);
    },),

    it({
      name: 'REFUSES A BARE NAME AS THE KEY OF A NAMED PLACE, by the type of the key, so a place stands for the report '
        + 'of a forwarder some module declares and for no free name',
      fn: async () => {
        expect(BARE_NAME_IS_NO_KEY,).toBe(true,);
      },
    },),

    ...NAMED_FLOW_CASES.map(function caseOfFlow(flowCase: NamedFlowCase,) {
      return it({
        name: `LEAVES THE REPORTS THE ONE NAME DOES NOT STAND FOR, over imaginary files: ${flowCase.name}`,
        fn: async () => {
          expect(spotsNotNamed({
            spots: blindSpotsOf({
              sources: [CONTEST_BUILDER_SOURCE, ...flowCase.files,],
              notStageSchemas: NOT_STAGE_SCHEMAS,
            },)
              .filter(function isImaginary({ file, },): boolean {
                return file.startsWith('imaginary/',);
              },),
            places: [FLOW_PLACE,],
          },),).toEqual(flowCase.unnamed,);
        },
      },);
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
        expectRecordAsListed({
          found: {
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
          },
          listed: {
            sameSources: true,
            copiedSources: true,
            namedFile: 0,
            unnamedFile: 1,
            editedFile: 0,
            inOrder: ['memoFormat', 'secondFormat',],
            reversed: ['secondFormat', 'memoFormat',],
          },
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
         Builders found, by the builder's name: the file that declares it, then the files that call it.
         */
        const found = Object.fromEntries(buildersOf({ sources, notStageSchemas: NOT_STAGE_SCHEMAS, },)
          .map(function listed(builder: SchemaBuilder,): readonly [string, readonly (string | readonly string[])[]] {
            return [
              builder.name,
              [builder.file, callersOf({ sources, builder, },).toSorted(),],
            ];
          },),);
        /**
         The builders the list names, in the same shape.
         */
        const listed = Object.fromEntries(SHARED_BUILDERS.map(function entryOf(shared,): readonly [
          string,
          readonly (string | readonly string[])[],
        ] {
          return [
            shared.builder,
            [
              shared.declaredIn,
              shared.callers.map(function fileOfCaller({ file, },): string {
                return file;
              },).toSorted(),
            ],
          ];
        },),);
        expectRecordAsListed({ found, listed, },);
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
         Forwarders found, by the function's name: the file that declares it, then the files that call it.
         */
        const found = Object.fromEntries(forwardersOf({ sources, notStageSchemas: NOT_STAGE_SCHEMAS, },)
          .map(function listed(forwarder,): readonly [string, readonly (string | readonly string[])[]] {
            return [
              forwarder.name,
              [forwarder.file, callersOf({ sources, builder: forwarder, },).toSorted(),],
            ];
          },),);
        /**
         The forwarders the list names, in the same shape.
         */
        const listed = Object.fromEntries(FORWARDERS.map(function entryOf(forwarder,): readonly [
          string,
          readonly (string | readonly string[])[],
        ] {
          return [forwarder.forwarder, [forwarder.declaredIn, forwarder.callers,],];
        },),);
        expectRecordAsListed({ found, listed, },);
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
      name: 'READS A PARAMETER NAMED LIKE A FORWARDER AS THAT FORWARDER, since a name the file neither declares nor '
        + 'imports stands for the function of that name: a helper that calls its parameter `choose` is listed beside '
        + 'the forwarder `choose`, and the same helper with the parameter named `decision` is not (B1264: the '
        + 'router\'s exchange mark was listed, and its callers after it, until the parameter was renamed)',
      fn: async () => {
        /**
         Forwarders of the family and of one imaginary helper whose parameter is called by the given name.
         */
        const imaginaryForwarders = async (parameter: string,): Promise<readonly string[]> =>
          forwardersOf({
            sources: [
              ...await familySources(),
              {
                file: 'imaginary/src/mark-after.ts',
                text: textOf([
                  `export async function markAfter({ ${parameter}, request, }) {`,
                  `  return await ${parameter}({ request, },);`,
                  '}',
                ],),
              },
            ],
            notStageSchemas: NOT_STAGE_SCHEMAS,
          },)
            .filter(function isImaginary({ file, },): boolean {
              return file.startsWith('imaginary/',);
            },)
            .map(function nameOf({ name, },): string {
              return name;
            },);
        expectFindingsAsListed({ findings: await imaginaryForwarders('choose',), listed: ['markAfter',], },);
        expectNoFindings({ findings: await imaginaryForwarders('decision',), },);
      },
    },),
  ],
},);
