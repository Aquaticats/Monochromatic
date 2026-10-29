import {
  mkdirSync,
  mkdtempSync,
  readFileSync,
  readdirSync,
  rmSync,
  writeFileSync,
} from 'node:fs';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import { join, } from 'node:path';
import { fileURLToPath, } from 'node:url';

import spawn from 'nano-spawn';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { isFunctionLikeDeclaration, } from 'typescript/ast/is';

import {
  asParameterIndex,
  buildEffectSummaryIndex,
  type CallableEffectSummary,
  clearEffectSummaryCache,
  clearFinalEffectIndexCache,
  closeSemanticBridge,
  effectSummaryCacheStats,
  finalEffectIndexCacheStats,
  NO_EFFECT_SUMMARY,
  openSemanticFile,
  SemanticBridgeError,
} from '../dist/final/node/index.mjs';

/** Effect summary semantic fixture. */
const FIXTURE_PATH = fileURLToPath(new URL(
  '../../../test-fixture/oxlint-no-restricted-syntax/src/valid/typescript-sync-adapter.ts',
  import.meta.url,
),);

/** Second configured source used to verify cross-file final-index reuse. */
const HELPER_PATH = fileURLToPath(new URL(
  '../../../test-fixture/oxlint-no-restricted-syntax/src/valid/semantic-effects-helper.ts',
  import.meta.url,
),);

/** Fixture whose functions reach caller state through member call results. */
const RESULT_PROVENANCE_PATH = fileURLToPath(new URL(
  '../../../test-fixture/oxlint-no-restricted-syntax/src/readonly-result-provenance-invalid.ts',
  import.meta.url,
),);

/** Current result-provenance fixture text. */
const RESULT_PROVENANCE_SOURCE = readFileSync(
  RESULT_PROVENANCE_PATH,
  'utf8',
);

/** Fixture storing a member result beyond the callable that produced it. */
const ASSIGNMENT_STORE_PATH = fileURLToPath(new URL(
  '../../../test-fixture/oxlint-no-restricted-syntax/src/readonly-assignment-store-invalid.ts',
  import.meta.url,
),);

/** Current assignment-store fixture text. */
const ASSIGNMENT_STORE_SOURCE = readFileSync(
  ASSIGNMENT_STORE_PATH,
  'utf8',
);

/** Fixture placing alias hops between a member result and its escaping position. */
const ALIAS_HOP_PATH = fileURLToPath(new URL(
  '../../../test-fixture/oxlint-no-restricted-syntax/src/readonly-alias-hop-invalid.ts',
  import.meta.url,
),);

/** Current alias-hop fixture text. */
const ALIAS_HOP_SOURCE = readFileSync(
  ALIAS_HOP_PATH,
  'utf8',
);

/** Fixture storing structural parameter state beyond the callable that read it. */
const STRUCTURAL_STORE_PATH = fileURLToPath(new URL(
  '../../../test-fixture/oxlint-no-restricted-syntax/src/readonly-structural-store-invalid.ts',
  import.meta.url,
),);

/** Current structural-store fixture text. */
const STRUCTURAL_STORE_SOURCE = readFileSync(
  STRUCTURAL_STORE_PATH,
  'utf8',
);

/** Fixture writing caller state through a callee's returned parameter. */
const RETURN_SUBSTITUTION_PATH = fileURLToPath(new URL(
  '../../../test-fixture/oxlint-no-restricted-syntax/src/readonly-return-substitution-invalid.ts',
  import.meta.url,
),);

/** Current return-substitution fixture text. */
const RETURN_SUBSTITUTION_SOURCE = readFileSync(
  RETURN_SUBSTITUTION_PATH,
  'utf8',
);

/** Statusline source proving audited object-property callback invocation. */
const STATUSLINE_USAGE_PATH = fileURLToPath(new URL(
  '../../../pi-plugin/statusline/src/usage-warning.ts',
  import.meta.url,
),);

/** Current statusline usage source text. */
const STATUSLINE_USAGE_SOURCE = readFileSync(
  STATUSLINE_USAGE_PATH,
  'utf8',
);

/** Built package entry exercised by independent process probe. */
const BUILT_ENTRY_URL = new URL(
  '../dist/final/node/index.mjs',
  import.meta.url,
).href;

/** Current effect fixture text. */
const SOURCE = readFileSync(
  FIXTURE_PATH,
  'utf8',
);

/** Disposable persistent cache directory. */
type DisposableCacheDirectory = {
  readonly path: string;
  readonly [Symbol.dispose]: () => void;
};

/**
 Creates disposable persistent cache root.
 
 @returns cache directory removed after test scope.
 */
function disposableCacheDirectory(): DisposableCacheDirectory {
  const path = mkdtempSync(join(scratchRoot(), 'readonly-effect-cache-',),);
  return {
    path,
    [Symbol.dispose](): void {
      rmSync(path, { recursive: true, force: true, },);
    },
  };
}

await describe({
  name: buildEffectSummaryIndex.name,
  concurrency: 1,
  children: [
    it({
      name: 'records audited object-property callback invocation as resolved while an unresolved reader stays reported',
      fn: async () => {
        const session = openSemanticFile({
          fileName: STATUSLINE_USAGE_PATH,
          sourceText: STATUSLINE_USAGE_SOURCE,
          hasBOM: false,
        },);
        const index = buildEffectSummaryIndex({
          project: session.project,
          activeSourceFile: session.sourceFile,
        },);
        const nameNode = session.nodeAtOffset(
          STATUSLINE_USAGE_SOURCE.indexOf('function formatUsageWarningStatus',)
            + 'function '.length,
        );
        const declaration = nameNode.parent;
        if (!isFunctionLikeDeclaration(declaration,))
          throw new Error('Expected statusline usage function declaration.',);
        const summary = index.get(declaration,);
        if (summary === NO_EFFECT_SUMMARY)
          throw new Error('Expected statusline usage effect summary.',);
        /** Invocation indexes retained before bridge cleanup. */
        const invoked = [...summary.invokedParameterIndexes,];
        /** Unresolved indexes retained before bridge cleanup. */
        const opaque = [...summary.opaqueParameterIndexes,];
        closeSemanticBridge();
        /* Style-callback invocation now proves inside the analyzed workspace
         * callee instead of an audited caller-side catalog marking. */
        expect(invoked,).toEqual([],);
        /* Unresolved reachability, and sound. `parseRateLimitSnapshots` reaches
         * `Object.entries` in `rate-limit-parse-helpers.ts`, which nothing derives, and
         * this callable packages its own parameter into that call. The claim used to
         * read `[]` because the owned call edge walked the argument literal with only
         * the property names the callee's `@mutates` blocks listed, so an authored
         * comment decided which caller-owned values inherited the callee's opacity.
         * Removing that filter is what makes this report appear, and the accompanying
         * offers it withdrew are the ones measured in
         * `doc/decision/prefer-readonly-contract-name-narrowing.md`. Deriving the
         * `Object` readers is what would return this to `[]` on proof rather than on
         * an omitted name. */
        expect(opaque,).toEqual([0,],);
      },
    },),
    it({
      name: 'attributes a mutation reached through a verified member result to its receiver',
      fn: async () => {
        /* The diagnostic-level tests cannot see this. A parameter mutated through a
         * call result also still carries that call's receiver opacity, and opacity
         * dominates the message, so the fixture emits the same count whether
         * attribution works or not. Only the summary distinguishes them. */
        const session = openSemanticFile({
          fileName: RESULT_PROVENANCE_PATH,
          sourceText: RESULT_PROVENANCE_SOURCE,
          hasBOM: false,
        },);
        const index = buildEffectSummaryIndex({
          project: session.project,
          activeSourceFile: session.sourceFile,
        },);
        /**
         Reads the written parameter indexes of one fixture function.
         
         Reads `referentMutatedParameterIndexes`, the set the readonly offer is gated
         on, rather than `mutatedParameterIndexes`, which is its union with the invoked
         set. Measured: the two agree for every function this case names, so the switch
         changed no expectation here. It matters because they do not always agree, and
         a case reading `referentMutated=[]` while the union reads `[0]` is a parameter
         that will be offered readonly the moment its opacity is discharged.
         
         @param functionName - Exported fixture function to inspect.
         
         @returns written parameter indexes in ascending order.
         */
        function mutatedIndexes(functionName: string,): readonly number[] {
          const nameNode = session.nodeAtOffset(
            RESULT_PROVENANCE_SOURCE.indexOf(`function ${functionName}`,)
              + 'function '.length,
          );
          const declaration = nameNode.parent;
          if (!isFunctionLikeDeclaration(declaration,))
            throw new Error(`Expected a declaration for ${functionName}.`,);
          const summary = index.get(declaration,);
          if (summary === NO_EFFECT_SUMMARY)
            throw new Error(`Expected an effect summary for ${functionName}.`,);
          /* Explicit numeric compare, since the default sort is lexicographic and
           * would order parameter 10 before parameter 2. */
          return [...summary.referentMutatedParameterIndexes,]
            .toSorted(function byIndex(left: number, right: number,): number {
              return left - right;
            },);
        }
        /**
         Reads the returned parameter origins of one fixture function.
         
         @param functionName - Exported fixture function to inspect.
         
         @returns returned parameter indexes in ascending order.
         */
        function returnedIndexes(functionName: string,): readonly number[] {
          const nameNode = session.nodeAtOffset(
            RESULT_PROVENANCE_SOURCE.indexOf(`function ${functionName}`,)
              + 'function '.length,
          );
          const declaration = nameNode.parent;
          if (!isFunctionLikeDeclaration(declaration,))
            throw new Error(`Expected a declaration for ${functionName}.`,);
          const summary = index.get(declaration,);
          if (summary === NO_EFFECT_SUMMARY)
            throw new Error(`Expected an effect summary for ${functionName}.`,);
          return [...summary.returnedParameterIndexes,]
            .toSorted(function byIndex(left: number, right: number,): number {
              return left - right;
            },);
        }
        /** Mutation through a bound lookup result. */
        const bound = mutatedIndexes('boundLookupMutationEffect',);
        /** Mutation through a lookup result with no binding at all. */
        const chained = mutatedIndexes('chainedLookupMutationEffect',);
        /** Property write through an element obtained by `at`. */
        const element = mutatedIndexes('chainedElementWriteEffect',);
        /** Property write through an element obtained by an observer member. */
        const observerValue = mutatedIndexes('observerValueResultMutationEffect',);
        /** Write through an object this callable built around the parameter. */
        const heldObject = mutatedIndexes('heldObjectMutationEffect',);
        /** Write through an array this callable built around the parameter. */
        const heldArray = mutatedIndexes('heldArrayMutationEffect',);
        /** Restructuring of a container this callable built, writing nothing. */
        const heldContainer = mutatedIndexes('heldContainerRestructureEffect',);
        /** Restructuring of a container the caller owns, reached through one hop. */
        const borrowedContainer = mutatedIndexes('borrowedContainerRestructureEffect',);
        /** Returned origins of a callable handing back the receiver's own elements. */
        const carriedContainer = returnedIndexes('returnsReceiverElements',);
        /** Write through a container another callable returned. */
        const wroteThroughCarried = mutatedIndexes('writesThroughReturnedContainer',);
        /** Read of nothing but the returned container's length. */
        const readCarriedLength = mutatedIndexes('readsReturnedContainerLength',);
        /** Returned origins of a callable composing two container members. */
        const composedContainer = returnedIndexes('returnsComposedReceiverElements',);
        /** Write through a composed container another callable returned. */
        const wroteThroughComposed = mutatedIndexes('writesThroughComposedContainer',);
        /** Returned origins of a callable whose container sits inside a selector. */
        const selectedContainer = returnedIndexes('returnsSelectedReceiverElements',);
        /** Write through a container another callable returned past a selector. */
        const wroteThroughSelected = mutatedIndexes('writesThroughSelectedContainer',);
        /** Mutation through a lookup narrowed by a runtime-erased assertion. */
        const asserted = mutatedIndexes('assertedLookupMutationEffect',);
        /** Mutation through a lookup whose value type is a union of object types. */
        const unionValued = mutatedIndexes('unionValueLookupEffect',);
        /** Restructuring reached through a computed member call. */
        const computedStructure = mutatedIndexes('computedStructureEffect',);
        /** Mutation through a computed lookup, invisible before one receiver rule. */
        const computedLookup = mutatedIndexes('computedLookupMutationEffect',);
        /** Function that only reads what it looks up. */
        const readOnly = mutatedIndexes('readOnlyLookupEffect',);
        closeSemanticBridge();
        /* Each names parameter 0, the collection the result came from. Dropping the
         * call branch from `provenanceSuccessors` empties all three while leaving
         * every diagnostic count in the sibling fixture test unchanged, which is why
         * this assertion exists rather than a message-level one. */
        expect(bound,).toEqual([0,],);
        expect(chained,).toEqual([0,],);
        expect(element,).toEqual([0,],);
        /* The observer sibling of the `at` case above, and the reason it is asserted here
         * rather than as a message: the value arm of `viewResultUnaccounted` discharges the
         * receiver, so `find` stopped reporting opacity and only the summary shows the write
         * that replaced it. Removing that arm reports opacity again; removing the element
         * attribution beneath it empties this and grants a read-only offer for an array whose
         * element the body rewrites. */
        expect(observerValue,).toEqual([0,],);
        /* The two programs `doc/planning/prefer-readonly-container-value-provenance.md`
         * names as the ones a careless container fix would wrongly offer read-only. Both
         * write the caller's value *through* a container this callable built, one by
         * property and one by element, and both must stay attributed. Emptying a fresh
         * literal's value origins, which is the redesign that document costs and rejects,
         * empties these two and nothing else here would catch it. */
        expect(heldObject,).toEqual([0,],);
        expect(heldArray,).toEqual([0,],);
        /* And the direction the record exists for. Nothing writes the parameter: `pop`
         * restructures the fresh array, and the parameter is what the array holds rather
         * than what holds the array. This read `[0]` before the record existed. */
        expect(heldContainer,).toEqual([],);
        /* The control that keeps the record from covering every local. `inner` names the
         * caller's own array, so its binding arrives through a property step, the record
         * is not set, and the charge stands. Keying the record on locality rather than on
         * how the value was built discharges this and loses a real write. */
        expect(borrowedContainer,).toEqual([0,],);
        /* A returned container of receiver elements is a fact callers propagate, and it was
         * not recorded at all until the return asked its element sibling as well as its
         * value one. The value question is right to answer nothing here: the array handed
         * back is not `rows`. What a caller reaches through it is every element `rows` held. */
        expect(carriedContainer,).toEqual([0,],);
        /* Which is what makes the caller's write attributable. Dropping the element origins
         * from the return empties both of these while every diagnostic count stays put,
         * since the callables involved are opaque either way. */
        expect(wroteThroughCarried,).toEqual([0,],);
        /* And the control that keeps it from being blanket attribution: a returned origin
         * says a caller can reach the parameter through the result, not that this one did. */
        expect(readCarriedLength,).toEqual([],);
        /* Container members compose, and the element walk resolved one relation and
         * stopped, so `rows.slice(0,).toReversed()` reported no origin though every step in
         * it holds. Its one-member sibling above is the control: without the composition the
         * two disagree about identical state reached through one extra member. */
        expect(composedContainer,).toEqual([0,],);
        expect(wroteThroughComposed,).toEqual([0,],);
        /* The same disagreement reached through a selector instead of an extra member. The
         * element walk asked the container question only where the selector stood, so
         * `cond ? rows.slice(0,) : []` reported no origin while the bare `cond ? rows : []`
         * reported one, and value provenance and the element walk answered differently about
         * identical state. Ten further spellings shared it, parentheses and `as` among them.
         *
         * The write below is the half that makes it a soundness fact rather than a precision
         * one. With no returned origin to substitute, it landed on no parameter at all and
         * `rows` became offerable while the callable rewrites a row it holds. */
        expect(selectedContainer,).toEqual([0,],);
        expect(wroteThroughSelected,).toEqual([0,],);
        /* `as` erases at runtime, so the asserted value is the lookup's own. Dropping
         * assertion expressions from `transparentOperand` empties this one. */
        expect(asserted,).toEqual([0,],);
        /* The union-valued receiver. Its held position is `Labelled | Tagged` as one
         * type object while the result flattens to both plus absence, so comparing
         * only flattened result constituents against the unflattened held type found
         * nothing. Measured: flattening one side only empties this while leaving every
         * other case here passing. */
        expect(unionValued,).toEqual([0,],);
        /* The destructured extraction, asserted here because its diagnostic is gone:
           the lookup discharged once every use of the row was attributed, so only the
           summary shows the mutation that replaced the report. */
        expect(mutatedIndexes('destructuredLookupMutationEffect',),).toEqual([0,],);
        /* Computed member access records the same facts as property access, because
         * which of the two the author wrote has no runtime bearing on what receives
         * the call. Both were entirely unrecorded before `memberCallReceiver`, and
         * `computedStructureEffect` was offered `readonly` while pushing to its
         * parameter. */
        expect(computedStructure,).toEqual([0,],);
        expect(computedLookup,).toEqual([0,],);
        /* And the control stays empty, so the resolver is not crediting every lookup
         * with a mutation it never performed. */
        expect(readOnly,).toEqual([],);
        /* Returning parameter-reachable state is recorded as a returned origin rather
         * than as an effect, per the accepted policy: the caller already holds the
         * parameter, so handing back a piece of it grants nothing new. The fact exists
         * so a caller can keep tracking the value, which is what makes the policy
         * sound; no discharge may rest on it until callers substitute through it. */
        expect(returnedIndexes('returnedLookupEffect',),).toEqual([0,],);
        /* A function returning nothing parameter-derived records nothing, so the fact
         * is not simply "every callable returns something". */
        expect(returnedIndexes('readOnlyLookupEffect',),).toEqual([],);
        /* How the callee spells its returned literal must not decide what its callers
         * can see. A shorthand property's name resolves to the property rather than to
         * the local it reads, so the provenance walk asked for the wrong symbol and
         * `packageRowShorthand` recorded no returned origin while its longhand sibling
         * recorded one. Reverting the shorthand value-symbol lookup in
         * `effect-expression-provenance.ts` empties the first of each pair below and
         * leaves the second passing, which is the asymmetry these pin. */
        expect(returnedIndexes('packageRowShorthand',),).toEqual([0,],);
        expect(returnedIndexes('packageRowExplicit',),).toEqual([0,],);
        /* And the consequence that made it a defect rather than a precision gap: the
         * caller's write through the returned holder was attributed to nothing, so the
         * row it mutates kept its read-only offer while the identical longhand write
         * reported the mutation. */
        expect(mutatedIndexes('shorthandPackagedWriteEffect',),).toEqual([0,],);
        expect(mutatedIndexes('explicitPackagedWriteEffect',),).toEqual([0,],);
        /* And the other direction of the same question. `packageCountFresh` returns an
         * object holding one string, so a caller reaches nothing through it, while the walk
         * credited the parameter because `expressionRoot` strips a property access back to
         * its receiver. Dropping the successor pruning in `effect-expression-provenance.ts`
         * turns this into `[0]` and makes it indistinguishable from the pair above, which
         * is the distinction result provenance is being built on. */
        expect(returnedIndexes('packageCountFresh',),).toEqual([],);
        /* What carries the parameter through a fresh container, and what does not. Both
         * element writes reach the caller's own row through a copy that holds it, and the
         * push reaches a container whose identity is fresh. One set of origins cannot answer
         * both, which is why the element step is resolved before the access layers are
         * stripped rather than after.
         *
         * All three were empty before the element step was answered, and the growth staying
         * empty is what makes the other two evidence: dropping the element-access branch in
         * `effect-expression-provenance.ts` empties the writes, and crediting the container
         * itself would fill the growth. */
        expect(mutatedIndexes('containerElementWriteEffect',),).toEqual([0,],);
        expect(mutatedIndexes('filteredElementWriteEffect',),).toEqual([0,],);
        expect(mutatedIndexes('containerGrowthEffect',),).toEqual([],);
        /* The three spellings that take an element step without writing an element access.
         * Each was empty until the element question was asked of the pattern's initializer,
         * the iterated expression and the spread source, and each reaches the caller's row
         * exactly as the access spelling does. Syntax decides how the step is spelled and
         * nothing about what it reaches. */
        expect(mutatedIndexes('destructuredContainerWriteEffect',),).toEqual([0,],);
        expect(mutatedIndexes('iteratedContainerWriteEffect',),).toEqual([0,],);
        expect(mutatedIndexes('spreadContainerWriteEffect',),).toEqual([0,],);
      },
    },),
    it({
      name: 'keeps receiver opacity for a member result stored beyond the callable, and discharges a local transfer',
      fn: async () => {
        /* The store cases were offered `readonly` before `assignmentStoreEscapes`. Every
         * caller of `useEscapes` hands it `valueConsumer` of the node, that ascends the
         * right operand of an assignment, and so the branch written to classify a store
         * could never receive one: `sink.value = rows.at(0,)` arrived as the assignment
         * expression, whose parent is an `ExpressionStatement`, and was discarded as
         * consumed in place. Measured by rebuilding at the prior commit, where all four
         * cases below are offered.
         *
         * The controls carry the other half. Classifying every assignment as a store
         * would report `assignToLocal` too, and a local transfer keeps the value inside
         * the body where the holder set is responsible for it. */
        const session = openSemanticFile({
          fileName: ASSIGNMENT_STORE_PATH,
          sourceText: ASSIGNMENT_STORE_SOURCE,
          hasBOM: false,
        },);
        const index = buildEffectSummaryIndex({
          project: session.project,
          activeSourceFile: session.sourceFile,
        },);
        /**
         Reads the opaque parameter indexes of one fixture function.
         
         @param functionName - Exported fixture function to inspect.
         
         @returns opaque parameter indexes in ascending order.
         */
        function opaqueIndexes(functionName: string,): readonly number[] {
          /**
           Name node of the requested fixture declaration.
           */
          const nameNode = session.nodeAtOffset(
            ASSIGNMENT_STORE_SOURCE.indexOf(`function ${functionName}`,)
              + 'function '.length,
          );
          /**
           Declaration owning that name.
           */
          const declaration = nameNode.parent;
          if (!isFunctionLikeDeclaration(declaration,))
            throw new Error(`Expected a declaration for ${functionName}.`,);
          /**
           Effect summary for that declaration.
           */
          const summary = index.get(declaration,);
          if (summary === NO_EFFECT_SUMMARY)
            throw new Error(`Expected an effect summary for ${functionName}.`,);
          return [...summary.opaqueParameterIndexes,]
            .toSorted(function byIndex(left: number, right: number,): number {
              return left - right;
            },);
        }
        /** Store into a caller-owned property. */
        const property = opaqueIndexes('storeIntoProperty',);
        /** Store into a caller-owned element position. */
        const element = opaqueIndexes('storeIntoElement',);
        /** Assignment to a plain local, which stays inside the callable. */
        const local = opaqueIndexes('assignToLocal',);
        /** Result consumed in place, never bound. */
        const inPlace = opaqueIndexes('readInPlace',);
        /** Store into a binding declared outside every callable body. */
        const moduleBinding = opaqueIndexes('storeIntoModuleBinding',);
        /** Copy into another parameter of the same callable. */
        const ownParameter = opaqueIndexes('storeIntoParameter',);
        /** Store of an object rest holding only primitives. */
        const primitiveRest = opaqueIndexes('storeRestOverPrimitiveState',);
        /** Store of an object rest holding a caller-owned reference. */
        const carriedRest = opaqueIndexes('storeRestOverCarriedState',);
        /** Store of an object rest over a constrained type parameter. */
        const constrainedRest = opaqueIndexes('storeRestOverGenericState',);
        /** Store of an object rest over an unconstrained type parameter. */
        const unconstrainedRest = opaqueIndexes('storeRestOverUnconstrainedState',);
        /** Store of an object rest whose index values carry a reference. */
        const indexedRest = opaqueIndexes('storeRestOverIndexedState',);
        /** Store of an object rest whose index values are primitive. */
        const primitiveIndexRest = opaqueIndexes('storeRestOverPrimitiveIndex',);
        /** Store of an object rest whose type unions a carrying and a primitive shape. */
        const unionRest = opaqueIndexes('storeRestOverUnionState',);
        /** Store of an object rest whose type unions two primitive shapes. */
        const primitiveUnionRest = opaqueIndexes('storeRestOverPrimitiveUnion',);
        /** Store of an object rest that enumerates no member at all. */
        const emptyRest = opaqueIndexes('storeRestOverEmptyState',);
        closeSemanticBridge();
        /* The receiver keeps its opacity, because nothing follows the stored element. */
        expect(property,).toEqual([0,],);
        expect(element,).toEqual([0,],);
        /* The module target is the one this classification most needed to cover, and it
         * threw instead. `targetIsCallableLocal` walks the target's declaration toward the
         * body, a module binding's ascent runs past the source file, and that file's
         * `parent` is `undefined` while `Node` types it as present. The demand index caught
         * the throw and omitted the callable, so this read `NO_EFFECT_SUMMARY` rather than
         * any verdict. Restoring the self-reference-only guard reproduces that. */
        expect(moduleBinding,).toEqual([0,],);
        /* And the controls stay discharged. Removing the locality test from
         * `targetIsCallableLocal`, so that every assignment target counts as a store,
         * turns `local` into `[0]` while leaving both cases above passing. */
        expect(local,).toEqual([],);
        expect(inPlace,).toEqual([],);
        /* One shape the classification used to take with it, measured at `[0,]` before
         * its fix and `[]` after. A parameter's declaration sits beside the body rather
         * than inside it, so a containment test called rebinding one an escape. */
        expect(ownParameter,).toEqual([],);
        /* Every object rest keeps its opacity, including the ones whose declared members
         * are all primitive. A narrowing that discharged those shipped and was reverted
         * rather than repaired: a TypeScript object type states which members a value
         * must have and never which it may have besides, so a value assignable to
         * `{ label: string; count: number; }` can carry a reference the type never
         * mentions, and object rest copies what the value has rather than what its type
         * declares. `leakExcessRestMember` measures the offer that produced.
         *
         * These cases are kept rather than deleted because they are the record of what a
         * type-member reading can and cannot establish. Each pair was built to
         * discriminate one branch of that reading, and all of them now agree, which is
         * the point: the reading itself was the mistake, not any particular branch. */
        expect(primitiveRest,).toEqual([0,],);
        expect(carriedRest,).toEqual([0,],);
        expect(constrainedRest,).toEqual([0, 1,],);
        expect(unconstrainedRest,).toEqual([0, 1,],);
        expect(indexedRest,).toEqual([0,],);
        expect(primitiveIndexRest,).toEqual([0,],);
        expect(unionRest,).toEqual([0, 1,],);
        expect(primitiveUnionRest,).toEqual([0, 1,],);
        expect(emptyRest,).toEqual([0,],);
      },
    },),
    it({
      name: 'records what the escape test currently sees for a structural store, before that changes',
      fn: async () => {
        /* A characterization, not a specification. Seven of these callables hand a piece
         * of a caller-owned structure to a binding that outlives the call, and one of them
         * is caught. That one is caught incidentally: only a verified member call carries
         * receiver opacity, and discharging receiver opacity is the sole trigger for the
         * escape test, so an index read, a property read, an alias hop, a logical
         * assignment, an iteration binding and a nested store never reach it.
         *
         * On the structural path this is a false offer rather than an imprecision. The
         * rule's own `--fix-suggestions --fix` writes `ReadonlyDeep<Config>` onto them,
         * the annotated file type-checks clean under TypeScript 7.0.2, and driving it
         * changes the caller's own `config.row.label` through the escaped reference.
         * Measured in `doc/planning/prefer-readonly-return-substitution.md`, section
         * "A third false offer, on a store into a module binding".
         *
         * The five reads carry the other half, and they are why this is pinned before the
         * classification widens. Reporting an assignment because its target is not
         * declared inside the body would take `assignIntoParameter` with it, since a
         * parameter's declaration sits beside the body rather than inside it, and would
         * take `countIntoModuleBinding` too, since a primitive stored beyond the callable
         * grants nothing. */
        const session = openSemanticFile({
          fileName: STRUCTURAL_STORE_PATH,
          sourceText: STRUCTURAL_STORE_SOURCE,
          hasBOM: false,
        },);
        const index = buildEffectSummaryIndex({
          project: session.project,
          activeSourceFile: session.sourceFile,
        },);
        /**
         Reads one index set of one structural-store fixture function.
         
         @param functionName - Exported fixture function to inspect.
         
         @param read - Which index set to take off the summary.
         
         @returns those parameter indexes in ascending order.
         */
        function structuralIndexes({
          functionName,
          read,
        }: {
          readonly functionName: string;
          readonly read: (summary: CallableEffectSummary,) => Iterable<number>;
        },): readonly number[] {
          /**
           Name node of the requested fixture declaration.
           */
          const nameNode = session.nodeAtOffset(
            STRUCTURAL_STORE_SOURCE.indexOf(`function ${functionName}`,)
              + 'function '.length,
          );
          /**
           Declaration owning that name.
           */
          const declaration = nameNode.parent;
          if (!isFunctionLikeDeclaration(declaration,))
            throw new Error(`Expected a declaration for ${functionName}.`,);
          /**
           Effect summary for that declaration.
           */
          const summary = index.get(declaration,);
          if (summary === NO_EFFECT_SUMMARY)
            throw new Error(`Expected an effect summary for ${functionName}.`,);
          return [...read(summary,),]
            .toSorted(function byIndex(left: number, right: number,): number {
              return left - right;
            },);
        }
        /**
         Reads the opaque parameter indexes of one structural-store fixture function.
         
         @param functionName - Exported fixture function to inspect.
         
         @returns opaque parameter indexes in ascending order.
         */
        function structuralOpaque(functionName: string,): readonly number[] {
          return structuralIndexes({
            functionName,
            read: function opaqueOf(summary,): Iterable<number> {
              return summary.opaqueParameterIndexes;
            },
          },);
        }
        /**
         Reads the mutated parameter indexes of one structural-store fixture function.
         
         @param functionName - Exported fixture function to inspect.
         
         @returns mutated parameter indexes in ascending order.
         */
        function structuralMutated(functionName: string,): readonly number[] {
          return structuralIndexes({
            functionName,
            read: function mutatedOf(summary,): Iterable<number> {
              return summary.mutatedParameterIndexes;
            },
          },);
        }
        /**
         Reads the returned parameter indexes of one structural-store fixture function.
         
         @param functionName - Exported fixture function to inspect.
         
         @returns returned parameter indexes in ascending order.
         */
        function structuralReturned(functionName: string,): readonly number[] {
          return structuralIndexes({
            functionName,
            read: function returnedOf(summary,): Iterable<number> {
              return summary.returnedParameterIndexes;
            },
          },);
        }
        /** Indexed element stored into a module binding. */
        const storedElement = structuralOpaque('storeElementIntoModuleBinding',);
        /** Member result stored into a module binding. */
        const storedMember = structuralOpaque('storeMemberIntoModuleBinding',);
        /** Plain property stored into a module binding. */
        const storedProperty = structuralOpaque('storePropertyIntoModuleBinding',);
        /** Property stored after an alias hop. */
        const storedAlias = structuralOpaque('storeAliasedIntoModuleBinding',);
        /** Property stored through a logical assignment. */
        const storedLogical = structuralOpaque('storeThroughLogicalAssignment',);
        /** Property stored through a nullish assignment. */
        const storedNullish = structuralOpaque('storeThroughNullishAssignment',);
        /** Property stored through a conjunction assignment. */
        const storedConjunction = structuralOpaque('storeThroughAndAssignment',);
        /** Property stored after an owned call laundered it. */
        const storedThroughCall = structuralOpaque('storeThroughOwnedCall',);
        /** Iteration binding stored into a module binding. */
        const storedIteration = structuralOpaque('storeIterationBinding',);
        /** Property stored into a local of the enclosing callable. */
        const storedEnclosing = structuralOpaque('storeIntoEnclosingLocal',);
        /** Row destructured into a binding outside the callable. */
        const destructuredOutward = structuralOpaque('destructureIntoModuleBinding',);
        /** First row destructured into a binding outside the callable. */
        const destructuredElement = structuralOpaque('destructureElementIntoModuleBinding',);
        /** Row destructured into a binding the callable declares. */
        const destructuredLocal = structuralOpaque('destructureIntoOwnLocal',);
        /** Rows retained one at a time through an iteration target. */
        const storedIterationTarget = structuralOpaque('storeIterationTarget',);
        /** Primitives read one at a time into a binding outside the callable. */
        const iterationPrimitiveTarget = structuralOpaque('storeIterationPrimitiveTarget',);
        /** Rows read one at a time into a binding the iteration declares. */
        const declaredIterationBinding = structuralOpaque('declareIterationBinding',);
        /** Property stored past the enclosing callable from a nested one. */
        const storedFromNested = structuralOpaque('storeFromNestedIntoModuleBinding',);
        /** Store written inside a nested callable nothing ever runs. */
        const storedFromInert = structuralOpaque('storeFromInertNested',);
        /** Property assigned to another parameter. */
        const intoParameter = structuralOpaque('assignIntoParameter',);
        /** Property assigned to a local the callable declares. */
        const intoOwnLocal = structuralOpaque('assignIntoOwnLocal',);
        /** Primitive accumulated into a module binding. */
        const intoCount = structuralOpaque('countIntoModuleBinding',);
        /** Fresh object built from a parameter's primitive, then stored. */
        const freshAggregate = structuralOpaque('storeFreshAggregate',);
        /** Object rest escaping with members its declared type never mentions. */
        const excessRest = structuralOpaque('leakExcessRestMember',);
        /** Structure read in place without binding. */
        const readInPlaceOnly = structuralOpaque('readStructureInPlace',);
        /** Rows iterated while reading only primitives. */
        const iterated = structuralOpaque('iterateStructureRows',);
        closeSemanticBridge();
        /* Every store reaches a report now, where only the member form did before and
         * only because a verified member call carries receiver opacity whose discharge
         * happened to run the escape test. `recordAssignmentStore` asks the question
         * directly at the assignment instead, so an index read, a property read, an alias
         * hop and the three retaining operators all arrive. */
        expect(storedMember,).toEqual([0,],);
        expect(storedElement,).toEqual([0,],);
        expect(storedProperty,).toEqual([0,],);
        expect(storedAlias,).toEqual([0,],);
        expect(storedLogical,).toEqual([0,],);
        expect(storedNullish,).toEqual([0,],);
        expect(storedConjunction,).toEqual([0,],);
        expect(storedIteration,).toEqual([0,],);
        /* Silent, and correctly so, which took a refuted explanation to establish. This
         * was recorded as a hole caused by nesting: the store leaves the nested body, and
         * the origins were said to belong to the enclosing callable where the
         * classification never sees them. `storeFromNestedIntoModuleBinding` refutes that
         * by holding the nesting and the invocation fixed and moving only the target. The
         * real reason is that the callable being summarised is the enclosing one and
         * `captured` is its own per-invocation local, which dies when the call returns. */
        expect(storedEnclosing,).toEqual([],);
        expect(storedFromNested,).toEqual([0,],);
        /* An iteration target retains exactly what an assignment target retains, and no
         * assignment expression appears anywhere in it, which is what made it invisible.
         * The two controls beside it are the halves that would take ordinary loops with
         * them: an element decides, so iterating primitives into an outside binding
         * retains nothing a caller can write through, and a declaration binds afresh each
         * time rather than storing. */
        expect(storedIterationTarget,).toEqual([0,],);
        expect(iterationPrimitiveTarget,).toEqual([],);
        expect(declaredIterationBinding,).toEqual([],);
        /* Both destructuring forms reach a binding outside the callable and are reported,
         * which the target policy gets right by answering no for every non-identifier. */
        expect(destructuredOutward,).toEqual([0,],);
        expect(destructuredElement,).toEqual([0,],);
        /* The same answer where it is wrong, pinned so a fix has something to flip. Every
         * leaf of this pattern is a local, which makes it no more an escape than a
         * declaration is, and the policy cannot see leaves. Withholding costs precision
         * only, so it is recorded rather than fixed inside work about soundness. */
        expect(destructuredLocal,).toEqual([0,],);
        /* The activation half of the same pair. Escaping syntax alone must not report, or
         * the shape above would be satisfied by a scan that never asked whether the nested
         * callable runs. */
        expect(storedFromInert,).toEqual([],);
        /* The one store no assignment-site test can reach on its own, and the deferred
         * result relation is what supplies its origins: a callee's summary does not exist
         * while its callers are scanned, so the right side has none of its own.
         *
         * This line read `[]` while the boundary stood, and the comment beside it said a
         * fix would have to flip it. It flipped. The offer it was withholding was false,
         * measured end to end: `ReadonlyDeep` applied to this parameter and to
         * `firstRow`'s type-checked together, and a write through the stored value then
         * changed the caller's row. */
        expect(storedThroughCall,).toEqual([0,],);
        /* And the control that keeps it an attribution rather than a rule against storing
         * any call result. `freshRow` returns nothing the caller owns, so its returned set
         * is empty and substitution hands over nothing. */
        expect(structuralOpaque('storeFreshThroughOwnedCall',),).toEqual([],);
        /* The iteration form of the same store, with its own control. The retention is
         * recorded from two sites and a fixture covering one of them proves nothing about
         * the other: an iterable that came back from a call has no origins of its own,
         * exactly as an assigned call result has none. */
        expect(structuralOpaque('storeIterationThroughCall',),).toEqual([0,],);
        expect(structuralOpaque('storeIterationThroughFreshCall',),).toEqual([],);
        /* The store equivalents of the local shapes, sharing the binding record with the
         * write side rather than repeating its reasoning. */
        expect(structuralOpaque('storeHeldResult',),).toEqual([0,],);
        expect(structuralOpaque('storeHeldFresh',),).toEqual([],);
        /* Retaining a primitive read off a returned value keeps nothing a caller can write
         * through. The deferred retention does not travel through `parameterIndexes`, so it
         * does not inherit that resolver's leaf test and needed its own: measured without
         * one, this recorded `[0,]` with store provenance for a `string`. The fresh control
         * above cannot stand in, since it stays empty for a different reason. */
        expect(structuralOpaque('storePrimitiveProjection',),).toEqual([],);
        /* The controls, which must still read empty after that flip. */
        expect(intoParameter,).toEqual([],);
        expect(intoOwnLocal,).toEqual([],);
        expect(intoCount,).toEqual([],);
        expect(readInPlaceOnly,).toEqual([],);
        expect(iterated,).toEqual([],);
        /* The control that catches the widest wrong shape of the fix. Gating on whether
         * the whole right side can carry state reports here, because an object literal is
         * a reference and the origin walk reaches the parameter through the property read
         * filling it, while the only thing stored is a string in an object this callable
         * allocated. */
        expect(freshAggregate,).toEqual([],);
        /* And the case that decided an object rest may not be discharged from its
         * declared members at all. Nothing here writes, so the direct-write attribution
         * that catches the mutating form never fires, and the reading of the rest's type
         * was the only thing between this and an offer. It measured `[]` while that
         * reading was in place.
         *
         * It reports for a better reason now. The store classification sees a value
         * leaving the callable and does not need to know what the rest copied, which is
         * the difference between asking where a value went and asking what its type
         * claims to hold. */
        expect(excessRest,).toEqual([0, 1,],);
        /* A closure stored outside the callable hands over everything it captured, and the
         * store site alone cannot see that: the assignment names no parameter, so the
         * origin walk over the right side comes back empty, and the closure body goes
         * unscanned because a stored closure counts as inactive.
         *
         * False rather than imprecise. `ReadonlyDeep<Config>` applied to
         * `storeCapturingClosure` type-checked clean, and a holder invoking the stored
         * closure changed the caller's row. Measured in
         * `doc/planning/prefer-readonly-return-substitution.md`, section "The escaping
         * closure is a false offer, falsified". */
        expect(structuralOpaque('storeCapturingClosure',),).toEqual([0,],);
        /* The same capture in a closure that writes rather than returns. Self-limiting once
         * an annotation is applied, so it cannot falsify anything, and it is pinned because
         * what a store hands over does not depend on what the closure does with it. An
         * implementation recording only writes would separate these two. */
        expect(structuralOpaque('storeCapturingClosureWriting',),).toEqual([0,],);
        /* Withheld where it need not be, pinned so a narrowing fix has something to flip.
         * The stored closure reads a `string` and hands back its length, so the holder can
         * reach nothing, but `packagedCallableOrigins` names every binding a packaged body
         * mentions whatever position it appears in. Task #64 holds the question. */
        expect(structuralOpaque('storeReadingClosure',),).toEqual([0,],);
        /* The normalization control, with the same closure behind parentheses and an
         * assertion. Two wrappers, because one is satisfied by unwrapping once while the
         * code loops until nothing more comes off. Measured with the normalization removed:
         * this line read `[]` while every bare shape above stayed `[0,]`. */
        expect(structuralOpaque('storeWrappedCapturingClosure',),).toEqual([0,],);
        /* The caller, which settles what the store record has to cover. It writes nothing
         * and stores nothing, so the only thing that can withhold its offer is the callee's
         * slot arriving through the call edge. That it reads `[0,]` is why the unrecorded
         * write inside the stored closure needs no channel of its own. */
        expect(structuralOpaque('passToCapturingStore',),).toEqual([0,],);
        /* The control that must not move. A closure assigned to a binding the callable owns
         * is not a store, `targetIsCallableLocal` answers that for both forms below, and the
         * declaration form's write reaches `config` through the ordinary active-body scan
         * instead. Store provenance appearing on either would mean the target policy had
         * stopped telling an owned binding from an outside one. */
        expect(structuralOpaque('invokeLocalClosureWriting',),).toEqual([],);
        /* The same closure filled by assignment rather than by an initializer. It recorded no
         * effect at all while the declaration form recorded `mutated=[0]`, because overload
         * resolution answers with the declared type's signature rather than with the arrow
         * assigned into the binding, so nothing activated it and the write was filtered out.
         *
         * Activation now also resolves assignments to the called binding. Flow-insensitive, so a
         * binding holding different closures at different points activates all of them, which
         * attributes more and therefore withholds more. */
        expect(structuralMutated('invokeAssignedLocalClosureWriting',),).toEqual([0,],);
        expect(structuralMutated('invokeLocalClosureWriting',),).toEqual([0,],);
        /* And the control that must not move with it: a closure assigned to a property outside the
         * callable is stored, not invoked here, so it stays a retention rather than becoming an
         * activation. */
        expect(structuralOpaque('storeCapturingClosure',),).toEqual([0,],);
        /* A closure written straight into a call reaches caller state through its body, and
         * the argument walk reads values rather than bodies, so this attributed nothing while
         * the same capture inside an object literal attributed correctly. Falsified with every
         * offer in the file applied and type-checking clean. */
        expect(structuralOpaque('handCaptureToRetainer',),).toEqual([0,],);
        /* The alias, and the reason the channel is driven by the declarations the edge already
         * resolves rather than by argument syntax. A syntax test sees an identifier here. */
        expect(structuralOpaque('handNamedCaptureToRetainer',),).toEqual([0,],);
        /* Two controls, and they fail in opposite directions. The first hands the same
         * retaining callee a closure that allocates its own row, so nothing captured travels
         * and the parameter keeps its offer: without it, this would be a rule against handing
         * callables to retaining callees rather than an attribution of what they captured. */
        expect(structuralOpaque('handFreshCaptureToRetainer',),).toEqual([],);
        /* The second hands a capturing closure to a callee that merely invokes it. That callee
         * is certain about its formal, so the gate never opens. Without it, the channel could
         * withhold on every callable ever handed to an owned callee and still look correct. */
        expect(structuralOpaque('handCaptureToReader',),).toEqual([],);
        /* The shape that decides what the admission gate reads, and the only one here that
         * does. `relayCallable` stores nothing and forwards its callable to an unresolved
         * boundary, so it carries `queueMicrotask` rather than any retention. Gating on
         * retention provenance was the first design and it passes every other assertion in
         * this file: measured with that gate in place, this line alone read `[]` while the
         * whole suite stayed green. Absent retention means call-caused or unknown, never
         * proven non-retaining. */
        expect(structuralOpaque('handCaptureToRelay',),).toEqual([0,],);
        /* The store side of the same resolution. `callbackHolder.produce = producer` is what
         * ordinary source writes where the fixtures above write the closure inline, and the
         * syntax gate saw only an identifier. Its control stores a named closure that
         * allocates its own row and keeps its offer, so this attributes what a stored
         * callable captured rather than reporting every callable ever stored. */
        expect(structuralOpaque('storeNamedCapturingClosure',),).toEqual([0,],);
        expect(structuralOpaque('storeNamedFreshClosure',),).toEqual([],);
        /* The returned closure, withheld through opacity because the accepted decision's
         * precondition fails: a function expression records no returned origin, so no caller
         * can substitute through it, which is exactly the condition the decision names.
         * Falsified with the annotation applied and type-checking clean. */
        expect(structuralOpaque('returnCapturingClosure',),).toEqual([0,],);
        /* Two controls in opposite directions. A returned closure naming nothing the caller
         * owns keeps its offer, so this attributes captures rather than refusing returns. */
        expect(structuralOpaque('returnFreshClosure',),).toEqual([],);
        /* And a direct return of caller state keeps its offer, which is the accepted policy
         * working: that return is tracked, callers substitute through it, and the condition
         * holds. Without this line the change would read as a rule against returning caller
         * state, which is precisely what the decision permits. */
        expect(structuralOpaque('returnRowDirectly',),).toEqual([],);
        /* The async twin of the permitted return, which stays permitted. The precondition the
         * decision names was failing at the caller rather than here: this recorded its returned
         * origin all along, and no caller could substitute through an await. */
        expect(structuralOpaque('returnRowAsync',),).toEqual([],);
        /* A result bound through a pattern, a logical assignment and a parameter default. The
         * registration refused every non-identifier name, the binding scan collected plain
         * assignment alone, and it collected local declarations and not this callable's own
         * parameters. Three separate omissions in one record. */
        expect(structuralMutated('writeThroughPatternBinding',),).toEqual([0,],);
        expect(structuralMutated('writeThroughLogicalBinding',),).toEqual([0,],);
        expect(structuralMutated('writeThroughDefaultBinding',),).toEqual([0, 1,],);
        /* A conditional write target, which the normalisation walk could not see through
         * because a conditional is where a value came from rather than a layer over it. */
        expect(structuralMutated('writeThroughConditionalTarget',),).toEqual([0,],);
        /* And a return of an element of an authored literal, which the return branch missed by
         * asking the expression alone where every write and store site consults the record. It
         * is a tracked returned origin now, which is what the accepted decision requires. */
        expect(structuralReturned('projectResultOutward',),).toEqual([0,],);
        /* Activation discovery is gated on ancestry now. It visited every node in the body, so a
         * call written inside a closure nothing runs activated its target, and the target's body
         * was read as though the enclosing callable had run it. Measured before the gate:
         * `mutated=[0]` for a write this callable never reaches, and a returned origin it never
         * returns.
         *
         * Both assertions are pairs on purpose. The false fact is gone and the offer is still
         * withheld, because the stored closure genuinely captures the configuration and the
         * capture walk answers for that. A fix that lost the withholding along with the false
         * fact would be a regression dressed as a correction, and asserting only the emptiness
         * would not notice. */
        expect(structuralMutated('storeClosureReachingWriter',),).toEqual([],);
        expect(structuralOpaque('storeClosureReachingWriter',),).toEqual([0,],);
        expect(structuralReturned('storeClosureReachingReturner',),).toEqual([],);
        expect(structuralOpaque('storeClosureReachingReturner',),).toEqual([0,],);
        /* The control. A sibling the callable actually calls must still activate, or the gate
         * would silence every ordinary nested helper. */
        expect(structuralMutated('invokeWritingSibling',),).toEqual([0,],);
        /* The construction channel asks the classifier rather than the leaf test, because the
         * leaf test answers yes for any array and cost the one offer this channel moved across
         * the workspace. `deep-readonly` means every reachable position is readonly, so no
         * write can travel through the value, whatever the constructor keeps. */
        expect(structuralOpaque('constructFromReadonlyKeys',),).toEqual([],);
        /* Its control, since a collection of writable rows retains writable rows. Without it the
         * gate would read as a rule against constructing from any array at all. */
        expect(structuralOpaque('constructFromMutableRows',),).toEqual([0,],);
        /* A tag is a call, and a tagged template is not a call expression, so the call branch
         * skipped it and every interpolated value reached the tag unrecorded. Falsified. */
        expect(structuralOpaque('handRowToTag',),).toEqual([0,],);
        /* Its leaf control, since a tag records only what its interpolated values can carry. */
        expect(structuralOpaque('handLabelToTag',),).toEqual([],);
        /* A callable held inside a returned literal, which the returned-callable capture missed by
         * resolving the returned expression itself, an object literal being no callable.
         * Falsified. */
        expect(structuralOpaque('handBackIteratorObject',),).toEqual([0,],);
        /* Its control, so descending a returned literal attributes what its callables captured
         * rather than reporting every returned literal that holds one. */
        expect(structuralOpaque('handBackFreshIterator',),).toEqual([],);
        /* A callable written inside the one being summarised has no summary of its own, since its
         * body is scanned inline, so the deferred result relation had nothing to substitute
         * against its call site and a store of what it handed back attributed nothing. Falsified.
         *
         * Answered with what the callable can reach rather than with what it returns, which
         * over-approximates in the direction that withholds. A precise answer needs nested
         * callables to carry summaries, which is larger than the falsification requires. */
        expect(structuralOpaque('storeLocalFunctionResult',),).toEqual([0,],);
        /* And it no longer claims a returned origin it never returns. A nested callable's body is
         * scanned inline, so its return reached the same branch as the enclosing callable's own
         * returns, and this callable returns nothing at all. A returned origin is a positive
         * capability claim, so claiming one where there is no result is the same kind of wrong fact
         * the activation gate removed.
         *
         * Asserted beside the opacity on purpose: the origins the nested body supplies must
         * survive, and they come from the capture walk at the call site rather than from the
         * return branch. */
        expect(structuralReturned('storeLocalFunctionResult',),).toEqual([],);
        /* The controls that must keep their returned origins, since a callable's own return is
         * what the accepted decision permits and tracks. */
        expect(structuralReturned('returnRowDirectly',),).toEqual([0,],);
        /* The member-call form, where the resolver answers about a value and a property is not
         * one, so the receiver's authored literal is what answers. */
        expect(structuralOpaque('storeArrowPropertyResult',),).toEqual([0,],);
        /* The control, so following a local callee attributes what it reaches rather than
         * reporting every store of a locally computed value. */
        expect(structuralOpaque('storeFreshLocalFunctionResult',),).toEqual([],);
        /* A throw is a handoff in exactly the sense a yield is, and nothing modelled one anywhere.
         * Task #64 recorded that absence as the reason no body summary here can be complete enough
         * to grant an offer, and this closes the escape while that stays true. Falsified. */
        expect(structuralOpaque('throwRowOutward',),).toEqual([0,],);
        /* Its leaf control, since a throw records only what its expression can carry. */
        expect(structuralOpaque('throwLabelOutward',),).toEqual([],);
        /* A destructuring default, which the declaration scan missed by reading the declaration's
         * own initializer while the parameter is named inside a binding element. Falsified. */
        expect(structuralOpaque('storeDestructuringDefault',),).toEqual([0,],);
        /* Its control, so a default naming nothing the caller owns keeps the offer. */
        expect(structuralOpaque('storeFreshDestructuringDefault',),).toEqual([],);
        /* The last known false offer. A method reading `this.row` names no binding, because `this`
         * is a keyword, so scanning the method body answers empty while the capture sits in the
         * literal the method was written in. Resolving the callee succeeds for such a method, so
         * returning on that success scanned exactly the body that cannot see it. The receiver is
         * asked as well as the callee now, never instead of it.
         *
         * Three sibling shapes hid this by passing: a method naming the parameter directly, an
         * arrow property naming it, and a plain property read. */
        expect(structuralOpaque('storeMethodThisResult',),).toEqual([0,],);
        /* Its control, so asking the receiver attributes what the literal mentions rather than
         * reporting every method call on a local holder. */
        expect(structuralOpaque('storeFreshMethodThisResult',),).toEqual([],);
        /* The capture walk follows calls now, because a lexical scan was answering a call-graph
         * question. A stored closure naming only `read` reached caller state through it, and a
         * local bound to a function expression carries no parameter origin, so the scan came
         * back empty and the parameter was offered. Falsified. */
        expect(structuralOpaque('storeClosureCallingSibling',),).toEqual([0,],);
        /* The same capture on the other two paths, because fixing one site and leaving the
         * others would look correct while the identical shape stayed invisible next door. */
        expect(structuralOpaque('handSiblingCaptureToRetainer',),).toEqual([0,],);
        expect(structuralOpaque('returnClosureCallingSibling',),).toEqual([0,],);
        /* The control. Following calls must attribute what a callee reaches rather than report
         * every closure that calls anything, so a sibling allocating its own row keeps the
         * offer. */
        expect(structuralOpaque('storeClosureCallingFreshSibling',),).toEqual([],);
        /* The termination control. A mutually recursive pair is folded in once each rather than
         * chased forever, and the capture is real so the offer stays withheld. Nothing else
         * here would notice a walk that failed to terminate, because it would hang rather than
         * answer wrongly, which no assertion can catch. */
        expect(structuralOpaque('storeMutuallyRecursiveClosures',),).toEqual([0,],);
        /* The store path asks what a value can be rather than how it is written, which is what
         * a conditional and a container held in a local needed. Both were falsified while the
         * syntax test saw a conditional and an identifier and stopped. */
        expect(structuralOpaque('storeConditionalClosure',),).toEqual([0,],);
        expect(structuralOpaque('storeAliasedContainer',),).toEqual([0,],);
        /* Both operands of `??` can be the value, unlike `&&`, whose left operand is discarded
         * whenever the right is produced, and that table is easy to get backwards. Two slots
         * here rather than one, because storing the caller's own callable outward retains it
         * exactly as storing a closure over the caller's row does. */
        expect(structuralOpaque('storeCoalescedClosure',),).toEqual([0, 1,],);
        /* The control. Following both branches must attribute what they capture rather than
         * report every conditional store, so a pair of branches naming nothing the caller owns
         * keeps its offer. */
        expect(structuralOpaque('storeConditionalFresh',),).toEqual([],);
        /* The line that decides the operand table, and the only one that can. The coalescence
         * above puts its capture on the right, so treating nullish coalescence as
         * right-operand-only passes it and every other assertion here: measured with that
         * mutation in place, the whole suite stayed green. This one puts the capture on the
         * left, and the origin walk cannot reach it either, since the binding is bound to a
         * conditional holding an arrow and an arrow has no provenance successors. */
        expect(structuralOpaque('storeLeftBiasedClosure',),).toEqual([0,],);
        /* Three channels found by walking escape shapes rather than by working a queue, all
         * three falsified, none of them a call edge or a store or a return. A construction hands
         * its argument to an object that keeps it, and `NewExpression` appeared nowhere in this
         * analysis at all. */
        expect(structuralOpaque('handRowToConstructor',),).toEqual([0,],);
        /* A yield hands its value to a driver that outlives it, and nothing about a yielded
         * value reaches the enclosing callable's returned set, so the tracking that makes a
         * return benign is unavailable. */
        expect(structuralOpaque('yieldRowOutward',),).toEqual([0,],);
        /* Both leaf controls, which keep their offers because a construction and a yield record
         * only what their operands can carry, and a count carries nothing. */
        expect(structuralOpaque('handLabelToConstructor',),).toEqual([],);
        expect(structuralOpaque('yieldCountOutward',),).toEqual([],);
        /* One cluster of queue shapes, one cause: a call result reaching a use site the deferred
         * relation did not cover. All eight falsified, each fixed by asking where a value can
         * have come from rather than what layer sits over it.
         *
         * An argument that is a call result, through an unresolved receiver and an owned one,
         * since the two fail for the same reason and one proves nothing about the other. */
        expect(structuralOpaque('retainResultThroughPush',),).toEqual([0,],);
        expect(structuralOpaque('handResultToRetainer',),).toEqual([0,],);
        /* Its leaf control, which is what keeps this from withholding on every call handed a
         * projection off a parameter. */
        expect(structuralOpaque('handCountToCollection',),).toEqual([],);
      },
    },),
    it({
      name: 'attributes a write reaching caller state through a callee returned parameter, at any depth',
      fn: async () => {
        /* `directReturned` recorded which parameters a result can carry from the day the
         * accepted decision asked for it, and nothing consumed the fact, so every case
         * here was offered `readonly Row[]`. The offer was false rather than imprecise:
         * applying it to `growThroughReturn` type-checked under TypeScript 7.0.2 and
         * running it grew the caller's array. Measured in
         * `doc/planning/prefer-readonly-return-substitution.md`, section "A second false
         * offer, on the array path".
         *
         * The fresh control is the half that keeps this sound. A callee allocating its
         * own array shares no identity with its argument, so attributing a write through
         * that result would withhold an offer that is true. It is written without `map`
         * deliberately: a caller-supplied callback opens an opaque boundary of its own,
         * and a control withheld for the wrong reason discriminates nothing. */
        const session = openSemanticFile({
          fileName: RETURN_SUBSTITUTION_PATH,
          sourceText: RETURN_SUBSTITUTION_SOURCE,
          hasBOM: false,
        },);
        const index = buildEffectSummaryIndex({
          project: session.project,
          activeSourceFile: session.sourceFile,
        },);
        /**
         Reads one fixture function's summary.
         
         @param functionName - Exported or local fixture function to inspect.
         
         @returns effect summary for that declaration.
         */
        function summaryOf(functionName: string,) {
          /**
           Name node of the requested fixture declaration.
           */
          const nameNode = session.nodeAtOffset(
            RETURN_SUBSTITUTION_SOURCE.indexOf(`function ${functionName}`,)
              + 'function '.length,
          );
          /**
           Declaration owning that name.
           */
          const declaration = nameNode.parent;
          if (!isFunctionLikeDeclaration(declaration,))
            throw new Error(`Expected a declaration for ${functionName}.`,);
          /**
           Effect summary for that declaration.
           */
          const summary = index.get(declaration,);
          if (summary === NO_EFFECT_SUMMARY)
            throw new Error(`Expected an effect summary for ${functionName}.`,);
          return summary;
        }
        /**
         Reads the written parameter indexes of one fixture function.
         
         Reads `referentMutatedParameterIndexes` rather than the union with the invoked
         set, because the readonly offer is gated on that set alone.
         
         @param functionName - Fixture function to inspect.
         
         @returns written parameter indexes in ascending order.
         */
        function writtenIndexes(functionName: string,): readonly number[] {
          return [...summaryOf(functionName,)
            .referentMutatedParameterIndexes,]
            .toSorted(function byIndex(left: number, right: number,): number {
              return left - right;
            },);
        }
        /** One returning callable between the write and the caller's array. */
        const oneHop = writtenIndexes('growThroughReturn',);
        /** Two returning callables, which only the fixed point can follow. */
        const twoHops = writtenIndexes('growThroughTwoReturns',);
        /** A freshly allocated array the caller does not own. */
        const fresh = writtenIndexes('growFresh',);
        /** The same write with no returning callable between. */
        const direct = writtenIndexes('growDirectly',);
        /** A read through a returning callable, which changes nothing. */
        const read = writtenIndexes('measureThroughReturn',);
        /** A property write through a returned element, which substitution misses today. */
        const propertyWrite = writtenIndexes('writePropertyThroughReturn',);
        /** Returned parameters of the single-hop callable. */
        const handedBack = [...summaryOf('handBack',)
          .returnedParameterIndexes,];
        /** Returned parameters of the callable that returns another's result. */
        const handedBackTwice = [...summaryOf('handBackTwice',)
          .returnedParameterIndexes,];
        /** Returned parameters of the callable allocating its own array. */
        const allocated = [...summaryOf('buildFresh',)
          .returnedParameterIndexes,];
        /** Whether the fresh control carries unresolved reachability instead. */
        const freshOpaque = [...summaryOf('growFresh',)
          .opaqueParameterIndexes,];
        closeSemanticBridge();
        /* The write attributes to the caller's parameter however many returning
         * callables sit between, and the direct case still behaves as it always did. */
        expect(oneHop,).toEqual([0,],);
        expect(twoHops,).toEqual([0,],);
        expect(direct,).toEqual([0,],);
        /* And nothing is attributed through a result the callee allocated.
         *
         * Three mutations, each measured rather than predicted, because the first
         * prediction written here was wrong. Removing the deferred recording in
         * `effect-collection-member-effect.ts` turns `oneHop` and `twoHops` into `[]`
         * while `direct` keeps passing. Removing the deferred recording in the return
         * branch of `direct-effect-summary.ts` turns `twoHops` and `handedBackTwice` into
         * `[]` while `oneHop` keeps passing, which is the transitivity discriminator.
         * Projecting `directReturned` instead of `returned` in `effect-public-summary.ts`
         * turns `handedBackTwice` into `[]` and moves NOTHING else: the substitution reads
         * the propagated set directly, so the projection only decides what callers of the
         * summary API see. */
        expect(fresh,).toEqual([],);
        expect(read,).toEqual([],);
        /* The fresh control must be silent rather than withheld some other way, or it
         * cannot show that substitution declined to attribute. */
        expect(freshOpaque,).toEqual([],);
        expect(handedBack,).toEqual([0,],);
        expect(handedBackTwice,).toEqual([0,],);
        expect(allocated,).toEqual([],);
        /* The boundary that was pinned here has flipped, which is the whole point of
         * having pinned it. A local between the call and the write no longer hides it: the
         * binding records which call filled it, and the write consults that. This case
         * needs the access strip as well as the binding hop, since the local holds
         * `handBack(rows,)[0]` rather than the call's own result. */
        expect(propertyWrite,).toEqual([0,],);
        /* The local shapes, and the two controls that keep the binding record from
         * withholding wherever any call feeds a local. The alias case is here to prove the
         * hop rather than assume it: a fix reading only declarations directly initialized
         * by a call passes `writeThroughHeldResult` and fails this one. */
        expect(writtenIndexes('writeThroughHeldResult',),).toEqual([0,],);
        expect(writtenIndexes('writeThroughAliasedResult',),).toEqual([0,],);
        expect(writtenIndexes('writeThroughHeldFresh',),).toEqual([],);
        expect(writtenIndexes('readHeldResult',),).toEqual([],);
        /* The wrapper pair. Access layers and identity-keeping wrappers are removed in one
         * loop rather than once each, because an alias with no access layer left the
         * wrapper in place and the identifier test then ran against the wrapper. Measured
         * before the loop: both of these recorded nothing while the bare alias recorded a
         * write. Two node kinds, so a fix handling one by accident fails the other. */
        expect(writtenIndexes('writeThroughAssertedAlias',),).toEqual([0,],);
        expect(writtenIndexes('writeThroughParenAlias',),).toEqual([0,],);
        /* The structural half of the same defect, and the one that was falsified end to
         * end. `firstRow` and `writeThroughOwnedCall` were both offered `ReadonlyDeep`,
         * applying BOTH type-checked under TypeScript 7.0.2, and running the pair printed
         * the caller's row carrying the written label.
         *
         * Checking one annotation at a time is the wrong experiment and produced the wrong
         * answer first: annotating the caller alone fails with `TS2345`, which reads as a
         * self-limiting offer. The rule reports per parameter and a reader applies every
         * suggestion in the file, so the pair is what has to be tested. It compiles because
         * assignability ignores `readonly` property modifiers, so `firstRow` declaring
         * `Row` hands a deeply readonly value back as a mutable one silently.
         *
         * The delete form is here because `inspectDirectWrite` serves three syntactic
         * shapes and a fix reaching only assignment would be a fix for one of them.
         *
         * Mutated to check these two lines discriminate: removing the deferred recording
         * from `inspectDirectWrite` turns both into `[]` and moves nothing else, leaving
         * `oneHop` at `[0,]` because that write travels the collection-member path. */
        expect(writtenIndexes('writeThroughOwnedCall',),).toEqual([0,],);
        expect(writtenIndexes('deleteThroughOwnedCall',),).toEqual([0,],);
        /* The structural control, which must stay silent or the fix is a blanket
         * withholding rather than an attribution.
         *
         * Its first draft was `return { label: config.row.label, };` and it reported. That
         * looked like the fix over-reaching and was not: an isolation probe showed a fresh
         * object literal whose only property is a copied primitive is recorded as returning
         * parameter state, indistinguishably from one that genuinely aliases. The control
         * had been carrying an origin all along. It is written in the `buildFresh` shape
         * now, and the separate over-approximation is tracked rather than silently
         * absorbed into this assertion. */
        expect(writtenIndexes('writeThroughFreshCall',),).toEqual([],);
        expect([...summaryOf('freshRow',)
          .returnedParameterIndexes,],).toEqual([],);
        expect([...summaryOf('firstRow',)
          .returnedParameterIndexes,],).toEqual([0,],);
      },
    },),
    it({
      name: 'keeps receiver opacity across alias hops, and still discharges an alias that only reads',
      fn: async () => {
        /* Every escaping case here was offered `readonly` before the holder set followed
         * aliases: `resultHolderSymbolIds` collected only the identifier the call directly
         * initializes, so `alias` never joined the set and its escaping use was never
         * scanned. The consequence was not confined to precision. Over a structural
         * parameter the rule projects with `ReadonlyDeep`, which does constrain elements,
         * and the offer that followed type-checks clean under TypeScript 7.0.2 while the
         * annotated callable rewrites the caller's state at runtime. Measured end to end
         * in `doc/planning/prefer-readonly-return-substitution.md`, section "A false offer
         * on the structural path".
         *
         * The controls carry the other half, and they are the reason this is not simply a
         * wider net. Enlarging the holder set without skipping declaration and
         * assignment-target occurrences classifies a destructured binding's own name as an
         * escape, because its parent is a `BindingElement` that no attributed position
         * matches, and classifies an assignment target the same way. Either mistake
         * reports every alias in the workspace. */
        const session = openSemanticFile({
          fileName: ALIAS_HOP_PATH,
          sourceText: ALIAS_HOP_SOURCE,
          hasBOM: false,
        },);
        const index = buildEffectSummaryIndex({
          project: session.project,
          activeSourceFile: session.sourceFile,
        },);
        /**
         Reads the opaque parameter indexes of one fixture function.
         
         @param functionName - Exported fixture function to inspect.
         
         @returns opaque parameter indexes in ascending order.
         */
        function opaqueIndexes(functionName: string,): readonly number[] {
          /**
           Name node of the requested fixture declaration.
           */
          const nameNode = session.nodeAtOffset(
            ALIAS_HOP_SOURCE.indexOf(`function ${functionName}`,)
              + 'function '.length,
          );
          /**
           Declaration owning that name.
           */
          const declaration = nameNode.parent;
          if (!isFunctionLikeDeclaration(declaration,))
            throw new Error(`Expected a declaration for ${functionName}.`,);
          /**
           Effect summary for that declaration.
           */
          const summary = index.get(declaration,);
          if (summary === NO_EFFECT_SUMMARY)
            throw new Error(`Expected an effect summary for ${functionName}.`,);
          return [...summary.opaqueParameterIndexes,]
            .toSorted(function byIndex(left: number, right: number,): number {
              return left - right;
            },);
        }
        /** Return reached through one alias hop. */
        const returned = opaqueIndexes('returnAfterAliasHop',);
        /** Module-binding store reached through one alias hop. */
        const stored = opaqueIndexes('storeAfterAliasHop',);
        /** Nested state extracted by destructuring, then returned. */
        const destructured = opaqueIndexes('destructureThenReturn',);
        /** Primitive extracted by destructuring, which can hold no caller state. */
        const primitive = opaqueIndexes('destructurePrimitiveThenReturn',);
        /** Nested state extracted by destructuring, then read in place. */
        const destructuredRead = opaqueIndexes('destructureReadInPlace',);
        /** Alias read in place, never leaving the callable. */
        const readAlias = opaqueIndexes('aliasReadInPlace',);
        /** Alias established by assignment, then read in place. */
        const assignedAlias = opaqueIndexes('assignAliasReadInPlace',);
        /**
         Reads the boundary names recorded against one fixture function's first parameter.
         
         Names only, with the source location dropped, because the location is an absolute
         path and what this asks about is which boundaries appear rather than where.
         
         @param functionName - Exported fixture function to inspect.
         
         @returns boundary names in ascending order.
         */
        function boundaryNames(functionName: string,): readonly string[] {
          /**
           Name node of the requested fixture declaration.
           */
          const nameNode = session.nodeAtOffset(
            ALIAS_HOP_SOURCE.indexOf(`function ${functionName}`,)
              + 'function '.length,
          );
          /**
           Declaration owning that name.
           */
          const declaration = nameNode.parent;
          if (!isFunctionLikeDeclaration(declaration,))
            throw new Error(`Expected a declaration for ${functionName}.`,);
          /**
           Effect summary for that declaration.
           */
          const summary = index.get(declaration,);
          if (summary === NO_EFFECT_SUMMARY)
            throw new Error(`Expected an effect summary for ${functionName}.`,);
          return [...summary.opaqueProvenanceByParameter
            .get(asParameterIndex(0,),) ?? [],]
            .map(function withoutLocation(fact: string,): string {
              return fact.split(' [',)[0] ?? fact;
            },)
            .toSorted();
        }
        /** Boundaries named for the spread form of the work-stack drain. */
        const spreadBoundaries = boundaryNames('drainWithSpread',);
        /** Boundaries named for the direct form of the same drain. */
        const directBoundaries = boundaryNames('drainWithDirectPush',);
        /** Receiver opacity for a spread into a literal that is stored rather than called. */
        const spreadIntoStored = opaqueIndexes('spreadIntoStoredLiteral',);
        closeSemanticBridge();
        /* The receiver keeps its opacity wherever the aliased result can leave. */
        expect(returned,).toEqual([0,],);
        expect(stored,).toEqual([0,],);
        expect(destructured,).toEqual([0,],);
        /* And the controls stay discharged, each one measured against the mutation it
         * exists to catch rather than assumed to catch one. Dropping the mutable-state
         * filter from `recordLeaf` turns `primitive` into `[0]` and moves nothing else.
         * Dropping `occurrenceEstablishesBinding` turns `assignedAlias` and
         * `destructuredRead` into `[0]` and moves nothing else, because the two escaping
         * destructuring and assignment cases already keep opacity and cannot go higher. */
        expect(primitive,).toEqual([],);
        expect(destructuredRead,).toEqual([],);
        expect(readAlias,).toEqual([],);
        expect(assignedAlias,).toEqual([],);
        /* A spread argument names the same boundaries a direct argument names, which it
         * did not before the consumer walk ascended a `SpreadElement`. The parent of a
         * spread operand is the spread rather than the call, so the walk stopped there and
         * skipped every attributed position, and the escape test then disagreed with the
         * argument analysis it defers to about identical state.
         *
         * Measured before the ascent: the spread form named `pending.pop` beside
         * `pending.push` while the direct form named `pending.push` alone. Asserting they
         * are EQUAL rather than asserting a literal list, because what was wrong was the
         * disagreement and any future change should move both or neither. */
        expect(spreadBoundaries,).toEqual(directBoundaries,);
        expect(spreadBoundaries,).toEqual(['pending.push',],);
        /* And the control that stops the ascent from being a blanket discharge. A spread
         * element now sits where a direct element sits, and a direct element in a literal
         * escapes whenever the literal is stored rather than handed to a call, so the
         * receiver keeps its opacity here. */
        expect(spreadIntoStored,).toEqual([0,],);
      },
    },),
    it({
      name: 'records every write through one destructured object parameter, whatever its contract names or siblings do',
      fn: async () => {
        /* Invisible at the diagnostic level in the direction that matters. The defect
         * produced an *offer*, so its signature is a missing message, and a fixture
         * nothing linted would look identical. The summary states the fact directly. */
        const session = openSemanticFile({
          fileName: RESULT_PROVENANCE_PATH,
          sourceText: RESULT_PROVENANCE_SOURCE,
          hasBOM: false,
        },);
        const index = buildEffectSummaryIndex({
          project: session.project,
          activeSourceFile: session.sourceFile,
        },);
        /**
         Reads the written parameter indexes of one fixture function.
         
         Reads `referentMutatedParameterIndexes` rather than
         `mutatedParameterIndexes`, because the second is the union with the invoked
         set while the readonly offer is gated on the first alone. Measuring the union
         here would have reported a write for `methodReturnPackagedEffect`, which is
         offered readonly.
         
         @param functionName - Fixture function to inspect.
         
         @returns written parameter indexes in ascending order.
         */
        function writtenIndexes(functionName: string,): readonly number[] {
          const nameNode = session.nodeAtOffset(
            RESULT_PROVENANCE_SOURCE.indexOf(`function ${functionName}`,)
              + 'function '.length,
          );
          const declaration = nameNode.parent;
          if (!isFunctionLikeDeclaration(declaration,))
            throw new Error(`Expected a declaration for ${functionName}.`,);
          const summary = index.get(declaration,);
          if (summary === NO_EFFECT_SUMMARY)
            throw new Error(`Expected an effect summary for ${functionName}.`,);
          /* Explicit numeric compare, since the default sort is lexicographic and
           * would order parameter 10 before parameter 2. */
          return [...summary.referentMutatedParameterIndexes,]
            .toSorted(function byIndex(left: number, right: number,): number {
              return left - right;
            },);
        }
        /** Parameter placed in a literal property the callee contract omits. */
        const omittedProperty = writtenIndexes('directRestrictedRowEffect',);
        /** Element obtained by `at` and placed in an omitted property. */
        const omittedLookup = writtenIndexes('contractRestrictedRowEffect',);
        /** Stored set obtained by `get` and placed in an omitted property. */
        const omittedStored = writtenIndexes('contractRestrictedLiteralEffect',);
        /** Whole container handed over as a direct argument. */
        const directArgument = writtenIndexes('directArgumentRestrictedEffect',);
        /** Literal whose every mutated property the contract names. */
        const fullyNamed = writtenIndexes('fullContractLiteralEffect',);
        /** Literal handed to a callee taking an identifier parameter. */
        const identifierParameter = writtenIndexes('identifierParameterLiteralEffect',);
        /** Two parameters where the callee writes only one. */
        const precisionCost = writtenIndexes('narrowingPrecisionCostEffect',);
        /** Callee invoking one property and writing another. */
        const invokedBeside = writtenIndexes('invokedExclusionDirectEffect',);
        /** Direct write beside a parameter forwarded next to a callback. */
        const invokedMiddle = writtenIndexes('middleInvokedExclusionEffect',);
        /** The same write one call further out. */
        const invokedOuter = writtenIndexes('outerInvokedExclusionEffect',);
        /** Parameter packaged behind an object-literal getter. */
        const accessorPackaged = writtenIndexes('accessorPackagedEffect',);
        /** Parameter packaged through a spread of a local object. */
        const spreadPackaged = writtenIndexes('spreadPackagedEffect',);
        /** Parameter reached by accessors nested one literal deeper. */
        const nestedAccessor = writtenIndexes('nestedAccessorPackagedEffect',);
        /** Parameter behind a method the callee calls for a row it writes. */
        const methodReturn = writtenIndexes('methodReturnPackagedEffect',);
        /** The same shape with an arrow held in an ordinary property. */
        const arrowReturn = writtenIndexes('arrowReturnPackagedEffect',);
        /** Function that only reads what it looks up, as a negative control. */
        const readOnly = writtenIndexes('readOnlyLookupEffect',);
        closeSemanticBridge();
        /* The defect, and the three shapes it took. Restoring the contract-name filter
         * in `effect-owned-call-edge.ts` empties all three, and the first is the one
         * that needs no collection member call at all: a parameter placed straight into
         * an object literal, mutated by the callee through a property its `@mutates`
         * blocks omit. Measured before the fix, `directRestrictedRowEffect` reported
         * `mutated=[]` and the rule emitted `Parameter "row" can be deeply readonly:
         * property label is writable` while the callee wrote `row.label`. */
        expect(omittedProperty,).toEqual([0,],);
        expect(omittedLookup,).toEqual([0,],);
        expect(omittedStored,).toEqual([0,],);
        /* The three neighbours that were already correct, kept so the assertions above
         * are known to isolate the contract-name filter rather than callee routing,
         * literal arguments, or destructuring in general. Each measured `mutated=[0]`
         * with the filter still in place. */
        expect(directArgument,).toEqual([0,],);
        expect(fullyNamed,).toEqual([0,],);
        expect(identifierParameter,).toEqual([0,],);
        /* The precision the slot model exists to recover, now measured rather than
         * predicted. The callee writes `named` and only reads `unnamed`, and the caller
         * hands `first` to the one and `second` to the other, so only `first` is written.
         *
         * This read `[0, 1]` for as long as the call edge repeated an argument's whole
         * origin set on every property slot the callee reads. It reads `[0]` because
         * `effect-argument-properties.ts` now resolves each property slot against the
         * caller's authored literal, and the sibling assertion in
         * `prefer-readonly-parameter-type.unit.test.ts` records the offer for `second`
         * that follows from it. Reverting that module restores `[0, 1]` and withdraws the
         * offer, which is the safe direction and was the state before. */
        expect(precisionCost,).toEqual([0,],);
        /* The same collapse seen from the other side, and the reason widening the walk
         * above was not conservative on its own. Mutation propagation used to subtract
         * the callee's invoked set by index, so a callee taking `{ run, target }` that
         * called `run` and wrote `target` cancelled its own write.
         *
         * These three state the facts; they do not detect that defect, and measuring said
         * so. Restoring `excludedIndexes: calleeSummary.invoked` in
         * `effect-fixed-point-propagation.ts` leaves all three reading `[0]` here while
         * the fixture gains two diagnostics, so the sibling case in
         * `prefer-readonly-parameter-type.unit.test.ts` is what catches it, through its
         * count and its assertion that the fixture offers nothing. The reason is the
         * defect's own order sensitivity: whichever propagation pass ran first decided
         * the answer, and this index is built over one file in an order that happens to
         * keep the write. That sensitivity is itself the argument for having removed the
         * subtraction rather than resequencing the passes. */
        expect(invokedBeside,).toEqual([0,],);
        expect(invokedMiddle,).toEqual([0,],);
        expect(invokedOuter,).toEqual([0,],);
        /* The third form the argument walk could not see. A property assignment, a
         * shorthand and a spread all expose a value the walk reads; an accessor computes
         * one by running its body when the callee reads the property, so a parameter it
         * returns reached the callee while contributing no origin. Deleting the accessor
         * branch from `parameterIndexes` empties the first of these and offers `row`
         * readonly again, and the spread beside it is the neighbour proving the walk was
         * not simply blind to everything that is not a plain property. */
        expect(accessorPackaged,).toEqual([0,],);
        expect(spreadPackaged,).toEqual([0,],);
        expect(nestedAccessor,).toEqual([0,],);
        /* Closed, and it took both halves. The callee side had to attribute the write:
         * `callThroughMethodResult` writes `get().label`, and calling a callable the
         * caller supplied may return caller-owned state, so `provenanceSuccessors` now
         * contributes an identifier callee. The caller side had to carry the origin to
         * the edge, which is the method and function-valued-property routing in
         * `parameterIndexes`. Reverting either one alone puts both of these back to `[]`
         * and offers `row` readonly again, which is how they were measured: with only the
         * caller half the callee still recorded `referentMutated=[]`, and with only the
         * callee half the origin never reached the edge. */
        expect(methodReturn,).toEqual([0,],);
        expect(arrowReturn,).toEqual([0,],);
        /* The control that keeps every assertion here from passing vacuously: none of
         * these changes may credit a parameter that is only read. */
        expect(readOnly,).toEqual([],);
      },
    },),
    it({
      name: 'propagates direct, cross-file, and immediate callback mutation',
      fn: async () => {
        const session = openSemanticFile({
          fileName: FIXTURE_PATH,
          sourceText: SOURCE,
          hasBOM: false,
        },);
        const index = buildEffectSummaryIndex({
          project: session.project,
          activeSourceFile: session.sourceFile,
        },);
        const effects = [
          'directSemanticEffect',
          'mutatePackagedState',
          'packagedSemanticEffect',
          'crossFileSemanticEffect',
          'callbackSemanticEffect',
          'directCallbackEffect',
          'asyncIteratorEffect',
          'wholeParameterContractEffect',
          'arrayCallbackSemanticEffect',
          'reduceElementParameterEffect',
          'forEachWholeArrayEffect',
          'readonlyMapCallbackEffect',
          'referencedObserverEffect',
          'mutableArrayObservationEffect',
          'mutableArrayStructureEffect',
          'mutableSetStructureEffect',
          'mutableSortObserverEffect',
          'mutableDefaultSortEffect',
          'aliasedCallbackSemanticEffect',
          'noSemanticEffect',
          'observationalIntrinsicEffect',
          'arrayBrandObservationEffect',
          'primitiveArraySortObservationEffect',
          'textEncoderObservationEffect',
          'objectArraySortCallbackEffect',
          'plainArrayDefaultSortObservationEffect',
          'hookedArrayDefaultSortOpaqueEffect',
          'objectArrayUndefinedSortOpaqueEffect',
          'plainArrayOptionalSortObservationEffect',
          'observationalValueEffects',
          'pathObservationEffect',
          'dateObservationEffect',
          'fileUrlObservationEffect',
          'direntObservationEffect',
          'aliasSemanticEffect',
          'assignedAliasSemanticEffect',
          'reboundParameterSemanticEffect',
          'destructuredAliasSemanticEffect',
          'destructuredParameterSemanticEffect',
          'opaqueSemanticEffect',
          'primitiveOpaqueArgumentEffect',
          'packagedPrimitiveOpaqueArgumentEffect',
          'transitiveOpaqueSemanticEffect',
          'unusedClosureSemanticEffect',
          'calledClosureSemanticEffect',
          'returnedClosureSemanticEffect',
          'passedClosureSemanticEffect',
          'aliasedPassedClosureSemanticEffect',
          'unusedFunctionExpressionSemanticEffect',
          'returnedContainerClosureSemanticEffect',
          'passedContainerClosureSemanticEffect',
          'deadParentClosureSemanticEffect',
          'storedClosureSemanticEffect',
        ].map(function summaryFor(functionName,) {
          const nameNode = session.nodeAtOffset(SOURCE.indexOf(functionName,),);
          const declaration = nameNode.parent;
          if (!isFunctionLikeDeclaration(declaration,))
            throw new Error(`Expected function declaration for ${functionName}.`,);
          const summary = index.get(declaration,);
          if (summary === NO_EFFECT_SUMMARY)
            throw new Error(`Expected effect summary for ${functionName}.`,);
          return {
            functionName,
            mutated: [...summary.mutatedParameterIndexes,],
            opaque: [...summary.opaqueParameterIndexes,],
          };
        },);
        /** Transitive opaque callable declaration. */
        const transitiveNameNode = session.nodeAtOffset(
          SOURCE.indexOf('transitiveOpaqueSemanticEffect',),
        );
        /** Parent function declaration for transitive opaque callable. */
        const transitiveDeclaration = transitiveNameNode.parent;
        if (!isFunctionLikeDeclaration(transitiveDeclaration,))
          throw new Error('Expected transitive opaque function declaration.',);
        /** Transitive opaque callable summary retaining originating boundary. */
        const transitiveSummary = index.get(transitiveDeclaration,);
        if (transitiveSummary === NO_EFFECT_SUMMARY)
          throw new Error('Expected transitive opaque summary.',);
        /** Opaque boundary names propagated to wrapper parameter. */
        const transitiveProvenance = [
          ...transitiveSummary.opaqueProvenanceByParameter.get(asParameterIndex(0,),) ?? [],
        ];
        /** Mutation contracts leave unresolved implementations opaque. */
        const contractedOpaqueEffects = [
          'documentedUncertainSemanticEffect',
          'transitiveDocumentedUncertainSemanticEffect',
        ].map(function documentedSummary(functionName,) {
          const nameNode = session.nodeAtOffset(SOURCE.indexOf(functionName,),);
          const declaration = nameNode.parent;
          if (!isFunctionLikeDeclaration(declaration,))
            throw new Error(`Expected function declaration for ${functionName}.`,);
          const summary = index.get(declaration,);
          if (summary === NO_EFFECT_SUMMARY)
            throw new Error(`Expected effect summary for ${functionName}.`,);
          return {
            functionName,
            affected: [...summary.mutatedParameterIndexes,],
            referentMutated: [...summary.referentMutatedParameterIndexes,],
            opaque: [...summary.opaqueParameterIndexes,],
            provenance: [...summary.opaqueProvenanceByParameter.get(asParameterIndex(0,),) ?? [],],
          };
        },);
        closeSemanticBridge();
        expect(effects,).toEqual([
          {
            functionName: 'directSemanticEffect',
            mutated: [0,],
            opaque: [],
          },
          {
            functionName: 'mutatePackagedState',
            mutated: [0,],
            opaque: [],
          },
          {
            functionName: 'packagedSemanticEffect',
            mutated: [0,],
            opaque: [],
          },
          {
            functionName: 'crossFileSemanticEffect',
            mutated: [0,],
            opaque: [],
          },
          {
            functionName: 'callbackSemanticEffect',
            mutated: [0,],
            opaque: [],
          },
          {
            functionName: 'directCallbackEffect',
            mutated: [0,],
            opaque: [],
          },
          {
            functionName: 'asyncIteratorEffect',
            mutated: [0,],
            opaque: [],
          },
          {
            functionName: 'wholeParameterContractEffect',
            mutated: [0,],
            opaque: [],
          },
          {
            functionName: 'arrayCallbackSemanticEffect',
            mutated: [0,],
            opaque: [],
          },
          {
            functionName: 'reduceElementParameterEffect',
            mutated: [0,],
            opaque: [],
          },
          {
            functionName: 'forEachWholeArrayEffect',
            mutated: [0,],
            opaque: [],
          },
          {
            functionName: 'readonlyMapCallbackEffect',
            mutated: [0,],
            opaque: [],
          },
          {
            functionName: 'referencedObserverEffect',
            mutated: [0,],
            opaque: [],
          },
          {
            functionName: 'mutableArrayObservationEffect',
            mutated: [],
            opaque: [],
          },
          {
            functionName: 'mutableArrayStructureEffect',
            mutated: [0,],
            opaque: [],
          },
          {
            functionName: 'mutableSetStructureEffect',
            mutated: [0,],
            opaque: [],
          },
          {
            functionName: 'mutableSortObserverEffect',
            mutated: [0,],
            opaque: [],
          },
          {
            functionName: 'mutableDefaultSortEffect',
            mutated: [0,],
            opaque: [0,],
          },
          {
            functionName: 'aliasedCallbackSemanticEffect',
            mutated: [0,],
            opaque: [],
          },
          {
            functionName: 'noSemanticEffect',
            mutated: [],
            opaque: [],
          },
          {
            functionName: 'observationalIntrinsicEffect',
            mutated: [],
            opaque: [0,],
          },
          {
            functionName: 'arrayBrandObservationEffect',
            mutated: [],
            opaque: [],
          },
          {
            functionName: 'primitiveArraySortObservationEffect',
            mutated: [],
            opaque: [],
          },
          {
            functionName: 'textEncoderObservationEffect',
            mutated: [],
            opaque: [0,],
          },
          {
            functionName: 'objectArraySortCallbackEffect',
            mutated: [],
            opaque: [0,],
          },
          {
            functionName: 'plainArrayDefaultSortObservationEffect',
            mutated: [],
            opaque: [0,],
          },
          {
            functionName: 'hookedArrayDefaultSortOpaqueEffect',
            mutated: [],
            opaque: [0,],
          },
          {
            functionName: 'objectArrayUndefinedSortOpaqueEffect',
            mutated: [],
            opaque: [0,],
          },
          {
            functionName: 'plainArrayOptionalSortObservationEffect',
            mutated: [],
            opaque: [0,],
          },
          {
            functionName: 'observationalValueEffects',
            mutated: [],
            opaque: [0,],
          },
          {
            functionName: 'pathObservationEffect',
            mutated: [],
            opaque: [],
          },
          {
            functionName: 'dateObservationEffect',
            mutated: [],
            opaque: [0,],
          },
          {
            functionName: 'fileUrlObservationEffect',
            mutated: [],
            opaque: [0,],
          },
          {
            functionName: 'direntObservationEffect',
            mutated: [],
            opaque: [0,],
          },
          {
            functionName: 'aliasSemanticEffect',
            mutated: [0,],
            opaque: [],
          },
          {
            functionName: 'assignedAliasSemanticEffect',
            mutated: [0,],
            opaque: [],
          },
          {
            functionName: 'reboundParameterSemanticEffect',
            mutated: [],
            opaque: [],
          },
          {
            functionName: 'destructuredAliasSemanticEffect',
            mutated: [0,],
            opaque: [],
          },
          {
            functionName: 'destructuredParameterSemanticEffect',
            mutated: [0,],
            opaque: [],
          },
          {
            functionName: 'opaqueSemanticEffect',
            mutated: [],
            opaque: [0,],
          },
          {
            functionName: 'primitiveOpaqueArgumentEffect',
            mutated: [],
            opaque: [],
          },
          {
            functionName: 'packagedPrimitiveOpaqueArgumentEffect',
            mutated: [],
            opaque: [],
          },
          {
            functionName: 'transitiveOpaqueSemanticEffect',
            mutated: [],
            opaque: [0,],
          },
          {
            functionName: 'unusedClosureSemanticEffect',
            mutated: [],
            opaque: [],
          },
          {
            functionName: 'calledClosureSemanticEffect',
            mutated: [0,],
            opaque: [],
          },
          {
            functionName: 'returnedClosureSemanticEffect',
            mutated: [0,],
            /* Opaque as well as mutated once returned callables started recording what they
             * captured. Redundant here and load-bearing next door: this closure writes
             * through its capture, so the active-body scan already attributed the mutation,
             * while a returned closure that only hands the capture back records no mutation
             * and nothing else would withhold it.
             *
             * Nothing a reader sees changes. A mutation withholds silently and a retention
             * withholds silently, and this file emits no diagnostic at this callable before
             * or after. Checked at the oxlint boundary rather than reasoned about. */
            opaque: [0,],
          },
          {
            functionName: 'passedClosureSemanticEffect',
            mutated: [0,],
            opaque: [],
          },
          {
            functionName: 'aliasedPassedClosureSemanticEffect',
            mutated: [0,],
            opaque: [],
          },
          {
            functionName: 'unusedFunctionExpressionSemanticEffect',
            mutated: [],
            opaque: [],
          },
          {
            functionName: 'returnedContainerClosureSemanticEffect',
            mutated: [0,],
            opaque: [],
          },
          {
            /* Opaque as well as mutated, and sound. The literal handed to
             * `Promise.resolve` holds a method that writes `closureState`, so an
             * unresolved callee receives a capability over caller-owned state. The claim
             * read `[]` until `parameterIndexes` began scanning packaged callables for
             * the origins they carry, which is what the identifier-callee attribution
             * needs to reach an edge at all. */
            functionName: 'passedContainerClosureSemanticEffect',
            mutated: [0,],
            opaque: [0,],
          },
          {
            functionName: 'deadParentClosureSemanticEffect',
            mutated: [],
            opaque: [],
          },
          {
            functionName: 'storedClosureSemanticEffect',
            mutated: [0,],
            opaque: [0,],
          },
        ],);
        /* Provenance facts carry origin-call locations so surfaced
         * diagnostics name where each remediation applies. */
        expect(transitiveProvenance.length,).toBe(1,);
        expect(transitiveProvenance[0]?.startsWith('JSON.stringify [',),).toBe(true,);
        expect(contractedOpaqueEffects.map(function withoutProvenance({
          provenance,
          ...rest
        },) {
          return {
            ...rest,
            provenanceShape: provenance.map(function factShape(fact,): boolean {
              return fact.startsWith('JSON.stringify [',) && fact.endsWith(']',);
            },),
          };
        },),).toEqual([
          {
            functionName: 'documentedUncertainSemanticEffect',
            affected: [],
            referentMutated: [],
            opaque: [0,],
            provenanceShape: [true,],
          },
          {
            functionName: 'transitiveDocumentedUncertainSemanticEffect',
            affected: [],
            referentMutated: [],
            opaque: [0,],
            provenanceShape: [true,],
          },
        ],);
      },
    },),
    it({
      name: 'keeps deferred callback invocation separate from forwarded argument uncertainty',
      fn: async () => {
        using projectRoot = disposableCacheDirectory();
        /** Disposable configured source path. */
        const inputPath = join(projectRoot.path, 'input.ts',);
        /** Source forwarding payload to deferred callback. */
        const inputSource = [
          'export function schedule(callback: (payload: { value: string }) => void, payload: { value: string }): void {',
          '  setTimeout(callback, 0, payload);',
          '}',
          '',
        ].join('\n',);
        writeFileSync(
          join(projectRoot.path, 'tsconfig.json',),
          `${JSON.stringify({ compilerOptions: { strict: true, lib: ['ESNext', 'DOM',], }, files: ['input.ts',], },)}\n`,
        );
        writeFileSync(inputPath, inputSource,);
        const session = openSemanticFile({
          fileName: inputPath,
          sourceText: inputSource,
          hasBOM: false,
        },);
        const index = buildEffectSummaryIndex({
          project: session.project,
          activeSourceFile: session.sourceFile,
        },);
        /** Scheduled function declaration. */
        const declaration = session.nodeAtOffset(inputSource.indexOf('schedule',),)
          .parent;
        if (!isFunctionLikeDeclaration(declaration,))
          throw new Error('Expected scheduled function declaration.',);
        /** Deferred callback effect summary. */
        const summary = index.get(declaration,);
        if (summary === NO_EFFECT_SUMMARY)
          throw new Error('Expected deferred callback effect summary.',);
        expect([...summary.invokedParameterIndexes,],).toEqual([],);
        expect([...summary.mutatedParameterIndexes,],).toEqual([],);
        expect([...summary.referentMutatedParameterIndexes,],).toEqual([],);
        expect([...summary.opaqueParameterIndexes,],).toEqual([0, 1,],);
        expect(summary.callbackRelations,).toEqual([],);
        closeSemanticBridge();
      },
    },),
    it({
      name: 'rejects shallow frozen copy retaining caller-owned nested identity',
      fn: async () => {
        using projectRoot = disposableCacheDirectory();
        /** Disposable configured source path. */
        const inputPath = join(projectRoot.path, 'input.ts',);
        /** Source freezing a fresh shallow copy instead of caller-owned input. */
        const inputSource = [
          'export function freezeCopy(options: { nested: { value: string } }): void {',
          '  Object.freeze({ nested: options.nested });',
          '}',
          '',
        ].join('\n',);
        writeFileSync(
          join(projectRoot.path, 'tsconfig.json',),
          `${JSON.stringify({ compilerOptions: { strict: true, lib: ['ESNext',], }, files: ['input.ts',], },)}\n`,
        );
        writeFileSync(inputPath, inputSource,);
        const session = openSemanticFile({
          fileName: inputPath,
          sourceText: inputSource,
          hasBOM: false,
        },);
        const index = buildEffectSummaryIndex({
          project: session.project,
          activeSourceFile: session.sourceFile,
        },);
        /** Fresh-copy function declaration. */
        const declaration = session.nodeAtOffset(inputSource.indexOf('freezeCopy',),)
          .parent;
        if (!isFunctionLikeDeclaration(declaration,))
          throw new Error('Expected fresh-copy function declaration.',);
        /** Fresh-copy effect summary. */
        const summary = index.get(declaration,);
        if (summary === NO_EFFECT_SUMMARY)
          throw new Error('Expected fresh-copy effect summary.',);
        expect({
          mutated: [...summary.mutatedParameterIndexes,],
          referentMutated: [...summary.referentMutatedParameterIndexes,],
          invoked: [...summary.invokedParameterIndexes,],
          opaque: [...summary.opaqueParameterIndexes,],
          callbackRelations: summary.callbackRelations,
        },).toEqual({
          mutated: [],
          referentMutated: [],
          invoked: [],
          opaque: [0,],
          callbackRelations: [],
        },);
        closeSemanticBridge();
      },
    },),
    it({
      name: 'infers direct imported package effects from shipped implementation',
      fn: async () => {
        using projectRoot = disposableCacheDirectory();
        /** Disposable package root under ordinary Node resolution boundary. */
        const packageRoot = join(projectRoot.path, 'node_modules', 'effect-probe',);
        mkdirSync(packageRoot, { recursive: true, },);
        writeFileSync(
          join(projectRoot.path, 'package.json',),
          '{"name":"effect-consumer","private":true,"type":"module"}\n',
        );
        writeFileSync(
          join(projectRoot.path, 'tsconfig.json',),
          '{"compilerOptions":{"strict":true,"module":"NodeNext","moduleResolution":"NodeNext"},"include":["input.ts"]}\n',
        );
        writeFileSync(
          join(projectRoot.path, 'pnpm-lock.yaml',),
          "lockfileVersion: '9.0'\npackages:\n  effect-probe@1.2.3: {}\n",
        );
        writeFileSync(
          join(packageRoot, 'package.json',),
          '{"name":"effect-probe","version":"1.2.3","type":"module","types":"./index.d.ts","exports":{".":{"node":"./node.js","import":"./index.js"},"./barrel":{"types":"./barrel.d.ts","node":"./barrel.js"},"./typed":{"types":"./typed.d.ts","import":"./typed.ts"},"./missing":{"types":"./missing.d.ts","import":"./missing.js"}}}\n',
        );
        writeFileSync(
          join(packageRoot, 'index.d.ts',),
          'export declare function observe(value: { text: string; }): string;\nexport declare function observe(value: { text: string; }, fallback: string): string;\nexport declare function mutate(value: { text: string; }): string;\nexport declare const toolkit: { mutate(value: { text: string; }): string; };\nexport declare class Toolkit { static mutate(value: { text: string; }): string; mutate(value: { text: string; }): string; }\nexport declare function visit(value: { text: string; }, callback: (value: { text: string; }) => void): void;\n',
        );
        writeFileSync(
          join(packageRoot, 'node.js',),
          "export { mutate, observe, Toolkit, toolkit, visit, } from './internal.js';\n//# sourceMappingURL=node.js.map\n",
        );
        writeFileSync(
          join(packageRoot, 'node.js.map',),
          '{"version":3,"file":"node.js","sources":["source.js"],"names":[],"mappings":""}\n',
        );
        writeFileSync(
          join(packageRoot, 'index.js',),
          "export { observe, Toolkit, toolkit, visit, } from './internal.js';\nexport function mutate(value) { return value.text; }\n",
        );
        writeFileSync(
          join(packageRoot, 'source.js',),
          "export { mutate, observe, Toolkit, toolkit, visit, } from './internal.js';\n",
        );
        writeFileSync(
          join(packageRoot, 'internal.js',),
          "export function observe(value) { return value.text; }\nexport function mutate(value) { value.text = 'changed'; return value.text; }\nexport const toolkit = { mutate(value) { value.text = 'object'; return value.text; } };\nexport class Toolkit { static mutate(value) { value.text = 'static'; return value.text; } mutate(value) { value.text = 'instance'; return value.text; } }\nexport function visit(value, callback) { callback(value); }\n",
        );
        writeFileSync(
          join(packageRoot, 'barrel.d.ts',),
          'export declare function barrelMutate(value: { text: string; }): string;\n',
        );
        writeFileSync(
          join(packageRoot, 'barrel.js',),
          "import { barrelMutate, } from './barrel-internal.js';\nexport { barrelMutate, };\n",
        );
        writeFileSync(
          join(packageRoot, 'barrel-internal.d.ts',),
          'export declare function barrelMutate(value: { text: string; }): string;\n',
        );
        writeFileSync(
          join(packageRoot, 'barrel-internal.js',),
          "export function barrelMutate(value) { value.text = 'barrel'; return value.text; }\n",
        );
        writeFileSync(
          join(packageRoot, 'typed.d.ts',),
          'export declare function typedMutate(value: { text: string; }): string;\n',
        );
        writeFileSync(
          join(packageRoot, 'typed.ts',),
          "export function typedMutate(value: { text: string; }): string { value.text = 'typed'; return value.text; }\n",
        );
        writeFileSync(
          join(packageRoot, 'missing.d.ts',),
          'export declare function missing(value: { text: string; }): string;\n',
        );
        /** Consumer wrappers covering re-exported JS, overloads, shipped TS, and missing implementation. */
        const inputSource = "import { mutate, observe, Toolkit, toolkit, visit, } from 'effect-probe';\nimport { barrelMutate, } from 'effect-probe/barrel';\nimport { missing, } from 'effect-probe/missing';\nimport { typedMutate, } from 'effect-probe/typed';\nexport function observed(value: { text: string; }): string { return observe(value); }\nexport function mutated(value: { text: string; }): string { return mutate(value); }\nexport function barrelMutated(value: { text: string; }): string { return barrelMutate(value); }\nexport function objectMutated(value: { text: string; }): string { return toolkit.mutate(value); }\nconst toolkitInstance = new Toolkit();\nexport function staticMutated(value: { text: string; }): string { return Toolkit.mutate(value); }\nexport function instanceMutated(value: { text: string; }): string { return toolkitInstance.mutate(value); }\nexport function typedMutated(value: { text: string; }): string { return typedMutate(value); }\nexport function visited(value: { text: string; }, callback: (value: { text: string; }) => void): void { visit(value, callback); }\nexport function unresolved(value: { text: string; }): string { return missing(value); }\n";
        /** Consumer source path. */
        const inputPath = join(projectRoot.path, 'input.ts',);
        writeFileSync(inputPath, inputSource,);
        clearEffectSummaryCache();
        clearFinalEffectIndexCache();
        const session = openSemanticFile({
          fileName: inputPath,
          sourceText: inputSource,
          hasBOM: false,
        },);
        const index = buildEffectSummaryIndex({
          project: session.project,
          activeSourceFile: session.sourceFile,
        },);
        const effects = [
          'observed',
          'mutated',
          'barrelMutated',
          'objectMutated',
          'staticMutated',
          'instanceMutated',
          'typedMutated',
          'unresolved',
        ].map(function wrapperEffect(functionName,) {
          const nameNode = session.nodeAtOffset(inputSource.indexOf(functionName,),);
          const declaration = nameNode.parent;
          if (!isFunctionLikeDeclaration(declaration,))
            throw new Error(`Expected package wrapper ${functionName}.`,);
          const summary = index.get(declaration,);
          if (summary === NO_EFFECT_SUMMARY)
            throw new Error(`Expected package wrapper summary ${functionName}.`,);
          return {
            functionName,
            mutated: [...summary.referentMutatedParameterIndexes,],
            opaque: [...summary.opaqueParameterIndexes,],
          };
        },);
        expect(effects,).toEqual([
          {
            functionName: 'observed',
            mutated: [],
            opaque: [],
          },
          {
            functionName: 'mutated',
            mutated: [0,],
            opaque: [],
          },
          {
            functionName: 'barrelMutated',
            mutated: [0,],
            opaque: [],
          },
          {
            functionName: 'objectMutated',
            mutated: [0,],
            opaque: [],
          },
          {
            functionName: 'staticMutated',
            mutated: [0,],
            opaque: [],
          },
          {
            functionName: 'instanceMutated',
            mutated: [0,],
            opaque: [],
          },
          {
            functionName: 'typedMutated',
            mutated: [0,],
            opaque: [],
          },
          {
            functionName: 'unresolved',
            mutated: [],
            opaque: [0,],
          },
        ],);
        /** Wrapper declaration for unresolved external callback relation. */
        const visitedName = session.nodeAtOffset(inputSource.indexOf('visited',),);
        /** Function declaration receiving callback and forwarded value. */
        const visitedDeclaration = visitedName.parent;
        if (!isFunctionLikeDeclaration(visitedDeclaration,))
          throw new Error('Expected visited package wrapper.',);
        /** Summary retaining callback invocation and fail-closed source relation. */
        const visitedSummary = index.get(visitedDeclaration,);
        if (visitedSummary === NO_EFFECT_SUMMARY)
          throw new Error('Expected visited package wrapper summary.',);
        expect({
          referentMutated: [...visitedSummary.referentMutatedParameterIndexes,],
          invoked: [...visitedSummary.invokedParameterIndexes,],
          opaque: [...visitedSummary.opaqueParameterIndexes,],
        },).toEqual({
          referentMutated: [],
          invoked: [1,],
          opaque: [0,],
        },);
        closeSemanticBridge();
        clearEffectSummaryCache();
        clearFinalEffectIndexCache();
      },
    },),
    it({
      name: 'fails closed when effect analysis budget is exhausted',
      fn: async () => {
        using projectRoot = disposableCacheDirectory();
        /** Single-source project used to exhaust budget before analysis. */
        const inputPath = join(projectRoot.path, 'input.ts',);
        /** Stable source whose summary must never be assumed after exhaustion. */
        const inputSource = 'export function inspect(value: { text: string; },): string { return value.text; }\n';
        writeFileSync(
          join(projectRoot.path, 'tsconfig.json',),
          '{"compilerOptions":{"strict":true},"include":["input.ts"]}\n',
        );
        writeFileSync(inputPath, inputSource,);
        clearEffectSummaryCache();
        clearFinalEffectIndexCache();
        const session = openSemanticFile({
          fileName: inputPath,
          sourceText: inputSource,
          hasBOM: false,
        },);
        let caught: unknown;
        try {
          buildEffectSummaryIndex({
            project: session.project,
            activeSourceFile: session.sourceFile,
            cacheRootOverride: join(projectRoot.path, '.effect-cache',),
            analysisBudgetMilliseconds: 0,
          },);
        }
        catch (error) {
          caught = error;
        }
        expect(caught,).toBeInstanceOf(SemanticBridgeError,);
        expect((caught as SemanticBridgeError).reason,).toBe('analysis-incomplete',);
        closeSemanticBridge();
        clearEffectSummaryCache();
        clearFinalEffectIndexCache();
      },
    },),
    it({
      name: 'scans only sources reached through owned calls and callbacks',
      fn: async () => {
        using projectRoot = disposableCacheDirectory();
        /** Disposable persistent cache separated from fixture source files. */
        const cacheRoot = join(projectRoot.path, '.effect-cache',);
        /** Active caller source whose effect closure is demanded. */
        const inputPath = join(projectRoot.path, 'input.ts',);
        /** Higher-order helper reached by active caller. */
        const helperPath = join(projectRoot.path, 'helper.ts',);
        /** Owned callback reached only through call-edge callback identity. */
        const callbackPath = join(projectRoot.path, 'callback.ts',);
        /** Configured source outside demanded effect closure. */
        const unrelatedPath = join(projectRoot.path, 'unrelated.ts',);
        /** Active source forwarding caller-owned state to imported callback. */
        const inputSource = "import { mutate, } from './callback.js';\nimport { invoke, mutateDirect, } from './helper.js';\nexport function demanded(value: { text: string; },): void { mutateDirect(value); invoke(mutate, value); }\n";
        writeFileSync(
          join(projectRoot.path, 'tsconfig.json',),
          '{"compilerOptions":{"strict":true},"include":["*.ts"]}\n',
        );
        writeFileSync(inputPath, inputSource,);
        writeFileSync(
          helperPath,
          "export function invoke(callback: (value: { text: string; }) => void, value: { text: string; },): void { callback(value); }\nexport function mutateDirect(value: { text: string; },): void { value.text = 'direct'; }\n",
        );
        writeFileSync(
          callbackPath,
          "export function mutate(value: { text: string; },): void { value.text = 'changed'; }\n",
        );
        writeFileSync(
          unrelatedPath,
          'export function unrelated(value: { text: string; },): string { return value.text; }\n',
        );
        clearEffectSummaryCache();
        clearFinalEffectIndexCache();
        const session = openSemanticFile({
          fileName: inputPath,
          sourceText: inputSource,
          hasBOM: false,
        },);
        const index = buildEffectSummaryIndex({
          project: session.project,
          activeSourceFile: session.sourceFile,
          cacheRootOverride: cacheRoot,
        },);
        /** Active caller declaration receiving transitive callback effect. */
        const declaration = session.nodeAtOffset(inputSource.indexOf('demanded',),)
          .parent;
        if (!isFunctionLikeDeclaration(declaration,))
          throw new Error('Expected demanded caller declaration.',);
        /** Summary proving callback source participated in fixed point. */
        const summary = index.get(declaration,);
        if (summary === NO_EFFECT_SUMMARY)
          throw new Error('Expected demanded caller effect summary.',);
        /** Cold counters exposing exact demanded callable set. */
        const stats = effectSummaryCacheStats();
        expect([...summary.referentMutatedParameterIndexes,],).toEqual([0,],);
        expect(stats.directSummaryBuildCount,).toBe(5,);
        expect(stats.persistentCacheWriteCount,).toBe(3,);
        closeSemanticBridge();
        clearEffectSummaryCache();
        clearFinalEffectIndexCache();
      },
    },),
    it({
      name: 'does not infer foreign ownership while an unscanned owned inbound call may exist',
      fn: async () => {
        using projectRoot = disposableCacheDirectory();
        /** Marker source directory preserving exact semantic identity suffix. */
        const markerRoot = join(
          projectRoot.path,
          'ownership-marker',
          'foreign-borrowed',
          'src',
        );
        /** Active foreign-boundary source. */
        const foreignPath = join(projectRoot.path, 'foreign.ts',);
        /** Shared helper receiving both foreign and owned values. */
        const helperPath = join(projectRoot.path, 'helper.ts',);
        /** Unreached ordinary caller disproving helper-wide foreign ownership. */
        const ownedPath = join(projectRoot.path, 'owned.ts',);
        /** Active source carrying explicit marker into shared helper. */
        const foreignSource = "import type { ForeignBorrowed, } from './ownership-marker/foreign-borrowed/src/index.js';\nimport { read, } from './helper.js';\nexport function readForeign(value: ForeignBorrowed<{ text: string; }>,): string { return read(value); }\n";
        mkdirSync(markerRoot, { recursive: true, },);
        writeFileSync(
          join(projectRoot.path, 'tsconfig.json',),
          '{"compilerOptions":{"strict":true},"include":["**/*.ts"]}\n',
        );
        writeFileSync(
          join(markerRoot, 'index.ts',),
          'declare const FOREIGN_BORROWED_MARKER: unique symbol;\nexport type ForeignBorrowed<Value> = Value & { readonly [FOREIGN_BORROWED_MARKER]?: true; };\n',
        );
        writeFileSync(foreignPath, foreignSource,);
        writeFileSync(
          helperPath,
          'export function read(value: { text: string; },): string { return value.text; }\n',
        );
        writeFileSync(
          ownedPath,
          "import { read, } from './helper.js';\nexport function readOwned(value: { text: string; },): string { return read(value); }\n",
        );
        clearEffectSummaryCache();
        clearFinalEffectIndexCache();
        const session = openSemanticFile({
          fileName: foreignPath,
          sourceText: foreignSource,
          hasBOM: false,
        },);
        const index = buildEffectSummaryIndex({
          project: session.project,
          activeSourceFile: session.sourceFile,
          cacheRootOverride: join(projectRoot.path, '.effect-cache',),
        },);
        /** Active boundary declaration retaining explicit foreign provenance. */
        const foreignDeclaration = session.nodeAtOffset(foreignSource.indexOf('readForeign',),)
          .parent;
        if (!isFunctionLikeDeclaration(foreignDeclaration,))
          throw new Error('Expected active foreign boundary declaration.',);
        /** Active boundary summary after complete inbound fallback. */
        const foreignSummary = index.get(foreignDeclaration,);
        if (foreignSummary === NO_EFFECT_SUMMARY)
          throw new Error('Expected active foreign boundary summary.',);
        expect([...index.proveForeignBorrowed(foreignDeclaration,),],).toEqual([0,],);
        /** Shared helper source decoded by active semantic project. */
        const helperSource = session.project.program.getSourceFile(helperPath,);
        if (helperSource === undefined)
          throw new Error('Expected shared helper source.',);
        /** Shared helper declaration whose every inbound must be considered. */
        const [helperDeclaration,] = helperSource.statements;
        if ((helperDeclaration === undefined)
          || (!isFunctionLikeDeclaration(helperDeclaration,)))
          throw new Error('Expected shared helper declaration.',);
        /** Helper summary must retain ordinary ownership from unreached caller. */
        const helperSummary = index.get(helperDeclaration,);
        if (helperSummary === NO_EFFECT_SUMMARY)
          throw new Error('Expected shared helper summary.',);
        expect([...index.proveForeignBorrowed(helperDeclaration,),],).toEqual([],);
        closeSemanticBridge();
        clearEffectSummaryCache();
        clearFinalEffectIndexCache();
      },
    },),
    it({
      name: 'refuses a marker declared in a declaration file',
      fn: async () => {
        using projectRoot = disposableCacheDirectory();
        /** Marker directory matching the identity check's expected shape. */
        const markerRoot = join(
          projectRoot.path,
          'ownership-marker',
          'foreign-borrowed',
          'src',
        );
        /** Boundary source naming the marker it imports. */
        const foreignPath = join(projectRoot.path, 'foreign.ts',);
        /** Boundary whose parameter carries the marker. */
        const foreignSource = "import type { ForeignBorrowed, } from './ownership-marker/foreign-borrowed/src/index.js';\nexport function readForeign(value: ForeignBorrowed<{ text: string; }>,): string { return value.text; }\n";
        mkdirSync(markerRoot, { recursive: true, },);
        writeFileSync(
          join(projectRoot.path, 'tsconfig.json',),
          '{"compilerOptions":{"strict":true},"include":["**/*.ts"]}\n',
        );
        /* Declaration file rather than `index.ts`, which is the whole point: the identity
         * check names one exact `.ts` path, and a marker anywhere else is not this marker.
         * `foreign-borrowed-demand.ts` depends on that. Its scope pre-scan reads source text,
         * so it could only miss a marker whose declaring file is absent from the indexed
         * scope, and the only file that can declare one has no `node_modules` segment and is
         * therefore always indexed. Accepting a declaration file here would break that
         * reasoning and make the pre-scan skip a scope whose parameters are genuinely
         * foreign, which withholds nothing and emits an offer. */
        writeFileSync(
          join(markerRoot, 'index.d.ts',),
          'declare const FOREIGN_BORROWED_MARKER: unique symbol;\nexport type ForeignBorrowed<Value> = Value & { readonly [FOREIGN_BORROWED_MARKER]?: true; };\n',
        );
        writeFileSync(foreignPath, foreignSource,);
        clearEffectSummaryCache();
        clearFinalEffectIndexCache();
        const session = openSemanticFile({
          fileName: foreignPath,
          sourceText: foreignSource,
          hasBOM: false,
        },);
        const index = buildEffectSummaryIndex({
          project: session.project,
          activeSourceFile: session.sourceFile,
          cacheRootOverride: join(projectRoot.path, '.effect-cache',),
        },);
        /** Boundary declaration whose parameter names the marker alias. */
        const foreignDeclaration = session.nodeAtOffset(foreignSource.indexOf('readForeign',),)
          .parent;
        if (!isFunctionLikeDeclaration(foreignDeclaration,))
          throw new Error('Expected active foreign boundary declaration.',);
        expect([...index.proveForeignBorrowed(foreignDeclaration,),],).toEqual([],);
        closeSemanticBridge();
        clearEffectSummaryCache();
        clearFinalEffectIndexCache();
      },
    },),
    it({
      name: 'proves the same parameters whichever callable was asked about first',
      fn: async () => {
        using projectRoot = disposableCacheDirectory();
        /** Marker source directory preserving exact semantic identity suffix. */
        const markerRoot = join(
          projectRoot.path,
          'ownership-marker',
          'foreign-borrowed',
          'src',
        );
        /** Active foreign-boundary source. */
        const foreignPath = join(projectRoot.path, 'foreign.ts',);
        /** Shared helper reached from the boundary and from an ordinary caller. */
        const helperPath = join(projectRoot.path, 'helper.ts',);
        /** Ordinary caller keeping the shared helper out of foreign ownership. */
        const ownedPath = join(projectRoot.path, 'owned.ts',);
        /** Active source carrying explicit marker into shared helper. */
        const foreignSource = "import type { ForeignBorrowed, } from './ownership-marker/foreign-borrowed/src/index.js';\nimport { read, } from './helper.js';\nexport function readForeign(value: ForeignBorrowed<{ text: string; }>,): string { return read(value); }\n";
        mkdirSync(markerRoot, { recursive: true, },);
        writeFileSync(
          join(projectRoot.path, 'tsconfig.json',),
          '{"compilerOptions":{"strict":true},"include":["**/*.ts"]}\n',
        );
        writeFileSync(
          join(markerRoot, 'index.ts',),
          'declare const FOREIGN_BORROWED_MARKER: unique symbol;\nexport type ForeignBorrowed<Value> = Value & { readonly [FOREIGN_BORROWED_MARKER]?: true; };\n',
        );
        writeFileSync(foreignPath, foreignSource,);
        writeFileSync(
          helperPath,
          'export function read(value: { text: string; },): string { return value.text; }\n',
        );
        writeFileSync(
          ownedPath,
          "import { read, } from './helper.js';\nexport function readOwned(value: { text: string; },): string { return read(value); }\n",
        );

        /**
         Proves both callables in one order, from a cold index each time.
         
         The helper's backwards closure walks through the boundary and reaches every callable
         the boundary's own closure would, so asking the helper first is what leaves an answer
         for the boundary behind. Retaining that answer instead of proving the boundary in its
         own right is what attempt two did, and the closure's caller summaries carry only the
         edges the helper's walk discovered, so the retained answer is not the same answer.
         
         @param helperFirst - Whether to demand the shared helper before the marked boundary.
         
         @returns proven parameter positions for the boundary and for the helper.
         */
        const proveInOrder = (helperFirst: boolean,): {
          readonly foreign: readonly number[];
          readonly helper: readonly number[];
        } => {
          clearEffectSummaryCache();
          clearFinalEffectIndexCache();
          /** Semantic session for one complete cold run. */
          const session = openSemanticFile({
            fileName: foreignPath,
            sourceText: foreignSource,
            hasBOM: false,
          },);
          /** Cold index sharing nothing with the other order. */
          const index = buildEffectSummaryIndex({
            project: session.project,
            activeSourceFile: session.sourceFile,
            cacheRootOverride: join(projectRoot.path, `.effect-cache-${String(helperFirst,)}`,),
          },);
          /** Marked boundary declaration. */
          const foreignDeclaration = session.nodeAtOffset(foreignSource.indexOf('readForeign',),)
            .parent;
          if (!isFunctionLikeDeclaration(foreignDeclaration,))
            throw new Error('Expected active foreign boundary declaration.',);
          /** Shared helper source decoded by active semantic project. */
          const helperSource = session.project.program.getSourceFile(helperPath,);
          if (helperSource === undefined)
            throw new Error('Expected shared helper source.',);
          /** Shared helper declaration reached from both callers. */
          const [helperDeclaration,] = helperSource.statements;
          if ((helperDeclaration === undefined)
            || (!isFunctionLikeDeclaration(helperDeclaration,)))
            throw new Error('Expected shared helper declaration.',);
          /** Answers in the order this run demands them. */
          const answers = helperFirst
            ? {
              helper: [...index.proveForeignBorrowed(helperDeclaration,),],
              foreign: [...index.proveForeignBorrowed(foreignDeclaration,),],
            }
            : {
              foreign: [...index.proveForeignBorrowed(foreignDeclaration,),],
              helper: [...index.proveForeignBorrowed(helperDeclaration,),],
            };
          return answers;
        };

        expect(proveInOrder(true,),).toEqual({
          foreign: [0,],
          helper: [],
        },);
        expect(proveInOrder(false,),).toEqual({
          foreign: [0,],
          helper: [],
        },);
        closeSemanticBridge();
        clearEffectSummaryCache();
        clearFinalEffectIndexCache();
      },
    },),
    it({
      name: 'rejects overload provenance when callable alias escapes beside matching call',
      fn: async () => {
        using projectRoot = disposableCacheDirectory();
        /** Marker source directory preserving exact semantic identity suffix. */
        const markerRoot = join(
          projectRoot.path,
          'ownership-marker',
          'foreign-borrowed',
          'src',
        );
        /** Active foreign-boundary source. */
        const foreignPath = join(projectRoot.path, 'foreign.ts',);
        /** Overloaded helper source. */
        const helperPath = join(projectRoot.path, 'helper.ts',);
        /** Separate source escaping helper through value alias. */
        const escapedPath = join(projectRoot.path, 'escaped.ts',);
        /** Active source carrying explicit marker into overloaded helper. */
        const foreignSource = "import type { ForeignBorrowed, } from './ownership-marker/foreign-borrowed/src/index.js';\nimport { read, } from './helper.js';\nexport function readForeign(value: ForeignBorrowed<{ text: string; }>,): string { return read(value); }\n";
        mkdirSync(markerRoot, { recursive: true, },);
        writeFileSync(
          join(projectRoot.path, 'tsconfig.json',),
          '{"compilerOptions":{"strict":true},"include":["**/*.ts"]}\n',
        );
        writeFileSync(
          join(markerRoot, 'index.ts',),
          'declare const FOREIGN_BORROWED_MARKER: unique symbol;\nexport type ForeignBorrowed<Value> = Value & { readonly [FOREIGN_BORROWED_MARKER]?: true; };\n',
        );
        writeFileSync(foreignPath, foreignSource,);
        writeFileSync(
          helperPath,
          "export function read(value: { text: string; },): string;\nexport function read(value: { text: string; readonly extra?: true; },): string;\nexport function read(value: { text: string; readonly extra?: true; },): string { return value.text; }\n",
        );
        writeFileSync(
          escapedPath,
          "import { read, } from './helper.js';\nexport const escapedRead = read;\n",
        );
        clearEffectSummaryCache();
        clearFinalEffectIndexCache();
        const session = openSemanticFile({
          fileName: foreignPath,
          sourceText: foreignSource,
          hasBOM: false,
        },);
        const index = buildEffectSummaryIndex({
          project: session.project,
          activeSourceFile: session.sourceFile,
          cacheRootOverride: join(projectRoot.path, '.effect-cache',),
        },);
        /** Overloaded helper source decoded by active semantic project. */
        const helperSource = session.project.program.getSourceFile(helperPath,);
        if (helperSource === undefined)
          throw new Error('Expected overloaded helper source.',);
        /** Callable implementation sharing overload-family references. */
        const helperImplementation = helperSource.statements.find(function hasBody(statement,): boolean {
          return isFunctionLikeDeclaration(statement,)
            && ('body' in statement)
            && (statement.body !== undefined);
        },);
        if ((helperImplementation === undefined)
          || (!isFunctionLikeDeclaration(helperImplementation,)))
          throw new Error('Expected overloaded helper implementation.',);
        /** Escaped value usage must prevent call-only provenance inference. */
        const helperSummary = index.get(helperImplementation,);
        if (helperSummary === NO_EFFECT_SUMMARY)
          throw new Error('Expected overloaded helper summary.',);
        expect([...index.proveForeignBorrowed(helperImplementation,),],).toEqual([],);
        closeSemanticBridge();
        clearEffectSummaryCache();
        clearFinalEffectIndexCache();
      },
    },),
    it({
      name: 'resolves a container receiver through its elements, in both fold spellings',
      fn: async () => {
        const session = openSemanticFile({
          fileName: FIXTURE_PATH,
          sourceText: SOURCE,
          hasBOM: false,
        },);
        const index = buildEffectSummaryIndex({
          project: session.project,
          activeSourceFile: session.sourceFile,
        },);
        /**
         Reads the opaque parameter indexes of one fixture function.
         
         @param functionName - Exported fixture function to inspect.
         
         @returns opaque parameter indexes in ascending order.
         */
        function opaqueIndexes(functionName: string,): readonly number[] {
          /**
           Name node of the requested fixture declaration.
           */
          const nameNode = session.nodeAtOffset(
            SOURCE.indexOf(`function ${functionName}`,)
              + 'function '.length,
          );
          /**
           Declaration owning that name.
           */
          const declaration = nameNode.parent;
          if (!isFunctionLikeDeclaration(declaration,))
            throw new Error(`Expected a declaration for ${functionName}.`,);
          /**
           Effect summary for that declaration.
           */
          const summary = index.get(declaration,);
          if (summary === NO_EFFECT_SUMMARY)
            throw new Error(`Expected an effect summary for ${functionName}.`,);
          return [...summary.opaqueParameterIndexes,]
            .toSorted(function byIndex(left: number, right: number,): number {
              return left - right;
            },);
        }
        /* Both spellings fold a container `filter` built, and no parameter is the value
         * that container holds, so asking where the receiver's value came from answers
         * nothing and the call falls to the receiver claim, which cannot answer for a
         * member carrying an observer. Asking where its elements came from answers the
         * parameter, and the observer derivation then discharges on its merits: the fold
         * reads a length and returns a number, so nothing receiver-reachable leaves.
         *
         * Measured by reverting `recordReadonlyViewApplications` to resolve the receiver
         * through `rootParameterOrigins`, where both read `[0]` with `reduce` named as
         * the cause. The pair matters as much as either case: binding the container
         * changes only the spelling of the cause, so both clear together or the fix is
         * keyed to syntax rather than to provenance. */
        expect(opaqueIndexes('chainedContainerFoldEffect',),).toEqual([],);
        expect(opaqueIndexes('boundContainerFoldEffect',),).toEqual([],);
        /* A fold whose receiver is the parameter itself, which resolved before this and
         * must keep resolving: the element question has to subsume the value question,
         * never replace it. */
        expect(opaqueIndexes('reduceElementParameterEffect',),).toEqual([],);
        /* The controls that keep the three above from passing vacuously. Widening how a
         * receiver resolves must not discharge a parameter whose opacity is real, and
         * these two are opaque for reasons the receiver walk never touches: an
         * unanalyzable callee, and a default sort on an array carrying an own hook. */
        expect(opaqueIndexes('opaqueSemanticEffect',),).toEqual([0,],);
        expect(opaqueIndexes('hookedArrayDefaultSortOpaqueEffect',),).toEqual([0,],);
        /**
         Reads the written parameter indexes of one fixture function.
         
         Reads `referentMutatedParameterIndexes`, the set the read-only offer is gated
         on, rather than its union with the invoked set.
         
         @param functionName - Exported fixture function to inspect.
         
         @returns written parameter indexes in ascending order.
         */
        function writtenIndexes(functionName: string,): readonly number[] {
          /**
           Name node of the requested fixture declaration.
           */
          const nameNode = session.nodeAtOffset(
            SOURCE.indexOf(`function ${functionName}`,)
              + 'function '.length,
          );
          /**
           Declaration owning that name.
           */
          const declaration = nameNode.parent;
          if (!isFunctionLikeDeclaration(declaration,))
            throw new Error(`Expected a declaration for ${functionName}.`,);
          /**
           Effect summary for that declaration.
           */
          const summary = index.get(declaration,);
          if (summary === NO_EFFECT_SUMMARY)
            throw new Error(`Expected an effect summary for ${functionName}.`,);
          return [...summary.referentMutatedParameterIndexes,]
            .toSorted(function byIndex(left: number, right: number,): number {
              return left - right;
            },);
        }
        /* The other half of the same question, and the half a discharge can quietly lose.
         * Clearing the opacity is only correct if the derivation that cleared it also
         * reports what the observer does, so the same two spellings with a writing fold
         * must attribute the write to the parameter: `filter` hands back the receiver's
         * own rows, and the fold writes one. A discharge that recorded nothing here would
         * have traded a report for silence rather than for a proof, which is the failure
         * the effect-model split already caught once when an empty observer match dropped
         * a real mutation. */
        expect(writtenIndexes('chainedContainerFoldWriteEffect',),).toEqual([0,],);
        expect(writtenIndexes('boundContainerFoldWriteEffect',),).toEqual([0,],);
        /* And the reading pair stays unwritten, so the assertions above are discriminating
         * rather than true of every fold. */
        expect(writtenIndexes('chainedContainerFoldEffect',),).toEqual([],);
        expect(writtenIndexes('boundContainerFoldEffect',),).toEqual([],);
        closeSemanticBridge();
        clearEffectSummaryCache();
        clearFinalEffectIndexCache();
      },
    },),
    it({
      name: 'keeps a write through an iterated tuple attributed to its parameter',
      fn: async () => {
        /** Fixture separating a held pair from a freshly built one. */
        const tuplePath = fileURLToPath(new URL(
          '../../../test-fixture/oxlint-no-restricted-syntax/src/readonly-tuple-exposure-invalid.ts',
          import.meta.url,
        ),);
        /** Current tuple-exposure fixture text. */
        const tupleSource = readFileSync(tuplePath, 'utf8',);
        const session = openSemanticFile({
          fileName: tuplePath,
          sourceText: tupleSource,
          hasBOM: false,
        },);
        const index = buildEffectSummaryIndex({
          project: session.project,
          activeSourceFile: session.sourceFile,
        },);
        /**
         Reads the written parameter indexes of one fixture function.
         
         @param functionName - Exported fixture function to inspect.
         
         @returns written parameter indexes.
         */
        function writtenIndexes(functionName: string,): readonly number[] {
          /**
           Name node of the requested fixture declaration.
           */
          const nameNode = session.nodeAtOffset(
            tupleSource.indexOf(`function ${functionName}`,)
              + 'function '.length,
          );
          /**
           Declaration owning that name.
           */
          const declaration = nameNode.parent;
          if (!isFunctionLikeDeclaration(declaration,))
            throw new Error(`Expected a declaration for ${functionName}.`,);
          /**
           Effect summary for that declaration.
           */
          const summary = index.get(declaration,);
          if (summary === NO_EFFECT_SUMMARY)
            throw new Error(`Expected an effect summary for ${functionName}.`,);
          return [...summary.referentMutatedParameterIndexes,];
        }
        /* This is the guarantee `prefer-readonly-parameter-type.unit.test.ts` holds its
         * writer-silence assertions to. A tuple the receiver holds is caller-owned state
         * whatever its positions are, because the tuple itself is writable, so
         * `pair[0] = 'rewritten'` reached through `rows.values()` writes `rows`. While
         * these two read `[0]` no read-only offer can pass the gate for a parameter
         * either of them rewrites, so the one offer the fixture permits (the read-only
         * control) stays the only one, rather than merely quiet. */
        expect(writtenIndexes('rewriteMutableStoredPair',),).toEqual([0,],);
        expect(writtenIndexes('rewriteStoredPair',),).toEqual([0,],);
        /* The control, and the reason a change making all three silent has answered the
         * wrong question: the reader iterates the identical receiver through the identical
         * member and writes nothing. */
        expect(writtenIndexes('readStoredPair',),).toEqual([],);
        /* The key side of the same question. A map holds caller-owned state on both sides
         * of an entry, and `keys` carries the element relation for position 0, so a write
         * through an iterated key lands on the parameter exactly as a write through an
         * iterated value does. Measured before the entry existed: this read `[]` while the
         * parameter was reported opaque, which is the shape that produces a false offer the
         * moment anything discharges that report. */
        expect(writtenIndexes('rewriteMapKey',),).toEqual([0,],);
        expect(writtenIndexes('readMapKey',),).toEqual([],);
        closeSemanticBridge();
        clearEffectSummaryCache();
        clearFinalEffectIndexCache();
      },
    },),
    it({
      name: 'charges draining an iterator this repository declares, and not the trusted one',
      fn: async () => {
        /** Fixture separating a trusted iterator from a repository-declared one. */
        const drainPath = fileURLToPath(new URL(
          '../../../test-fixture/oxlint-no-restricted-syntax/src/readonly-iteration-channel-invalid.ts',
          import.meta.url,
        ),);
        /** Current iteration-channel fixture text. */
        const drainSource = readFileSync(drainPath, 'utf8',);
        const session = openSemanticFile({
          fileName: drainPath,
          sourceText: drainSource,
          hasBOM: false,
        },);
        const index = buildEffectSummaryIndex({
          project: session.project,
          activeSourceFile: session.sourceFile,
        },);
        /**
         Reads the opaque parameter indexes of one fixture function.
         
         @param functionName - Exported fixture function to inspect.
         
         @returns parameter indexes left opaque.
         */
        function opaqueIndexes(functionName: string,): readonly number[] {
          /**
           Name node of the requested fixture declaration.
           */
          const nameNode = session.nodeAtOffset(
            drainSource.indexOf(`function ${functionName}`,)
              + 'function '.length,
          );
          /**
           Declaration owning that name.
           */
          const declaration = nameNode.parent;
          if (!isFunctionLikeDeclaration(declaration,))
            throw new Error(`Expected a declaration for ${functionName}.`,);
          /**
           Effect summary for that declaration.
           */
          const summary = index.get(declaration,);
          if (summary === NO_EFFECT_SUMMARY)
            throw new Error(`Expected an effect summary for ${functionName}.`,);
          return [...summary.opaqueParameterIndexes,];
        }
        /* Draining is a call, and the baseline trusts only an iterator the default library
         * declares. `CountingIterable` declares its own and writes `count` from it, so both
         * spellings run repository code that mutates what the caller passed. Measured before
         * `effect-iteration-channel.ts`: both read `[]`, so the parameter was offerable while
         * the loop rewrote it. The pair matters because the two reach the walk through
         * different branches, `for...of` and a spread element. */
        expect(opaqueIndexes('iterateCountingValue',),).toEqual([0,],);
        expect(opaqueIndexes('spreadCountingValue',),).toEqual([0,],);
        /* The control, and the reason this is not simply "iteration is opaque now": a plain
         * array drains an iterator too, and that one the library declares. */
        expect(opaqueIndexes('iterateTrustedRows',),).toEqual([],);
        closeSemanticBridge();
        clearEffectSummaryCache();
        clearFinalEffectIndexCache();
      },
    },),
    it({
      name: 'reuses direct scans in process and through persistent cache',
      fn: async () => {
        using cache = disposableCacheDirectory();
        clearEffectSummaryCache();
        clearFinalEffectIndexCache();
        const firstSession = openSemanticFile({
          fileName: FIXTURE_PATH,
          sourceText: SOURCE,
          hasBOM: false,
        },);
        buildEffectSummaryIndex({
          project: firstSession.project,
          activeSourceFile: firstSession.sourceFile,
          cacheRootOverride: cache.path,
        },);
        /** Counters after uncached whole-project scan and persistent write. */
        const firstStats = effectSummaryCacheStats();
        expect(firstStats.directSummaryBuildCount > 0,).toBe(true,);
        expect(firstStats.persistentCacheWriteCount > 0,).toBe(true,);
        clearEffectSummaryCache();
        clearFinalEffectIndexCache();
        const secondSession = openSemanticFile({
          fileName: FIXTURE_PATH,
          sourceText: SOURCE,
          hasBOM: false,
        },);
        buildEffectSummaryIndex({
          project: secondSession.project,
          activeSourceFile: secondSession.sourceFile,
          cacheRootOverride: cache.path,
        },);
        /** Counters after process cache reset and persistent reuse. */
        const persistentStats = effectSummaryCacheStats();
        expect(persistentStats.directSummaryBuildCount,).toBe(0,);
        expect(persistentStats.persistentSourceCacheHitCount > 0,).toBe(true,);
        const thirdSession = openSemanticFile({
          fileName: FIXTURE_PATH,
          sourceText: SOURCE,
          hasBOM: false,
        },);
        buildEffectSummaryIndex({
          project: thirdSession.project,
          activeSourceFile: thirdSession.sourceFile,
          cacheRootOverride: cache.path,
        },);
        /** Counters after same-process fixed-point index reuse. */
        const processStats = effectSummaryCacheStats();
        /** Fixed-point cache counters after unchanged project query. */
        const finalStats = finalEffectIndexCacheStats();
        expect(processStats.directSummaryBuildCount,).toBe(0,);
        expect(processStats.sourceCacheHitCount,).toBe(persistentStats.sourceCacheHitCount,);
        expect(finalStats.hitCount > 0,).toBe(true,);
        /** Different active source from same unchanged configured project. */
        const helperSession = openSemanticFile({
          fileName: HELPER_PATH,
          sourceText: readFileSync(HELPER_PATH, 'utf8',),
          hasBOM: false,
        },);
        buildEffectSummaryIndex({
          project: helperSession.project,
          activeSourceFile: helperSession.sourceFile,
          cacheRootOverride: cache.path,
        },);
        /** Fixed-point cache counters after changing only active source path. */
        const crossFileStats = finalEffectIndexCacheStats();
        expect(crossFileStats.hitCount > finalStats.hitCount,).toBe(true,);
        closeSemanticBridge();
        clearEffectSummaryCache();
        clearFinalEffectIndexCache();
      },
    },),
    it({
      name: 'restores deliberate callable omissions from persistent cache',
      fn: async () => {
        using projectRoot = disposableCacheDirectory();
        /** Disposable persistent cache root separated from TypeScript input. */
        const cacheRoot = join(projectRoot.path, '.effect-cache',);
        /** Source whose complete summaries let the test seed one persisted omission record. */
        const inputPath = join(projectRoot.path, 'input.ts',);
        /** Single-source input with a generic tuple parameter and one call of it. */
        const inputSource = `export function take<Fn extends (...args: never[]) => unknown,>(
  fn: Fn,
  args: Parameters<Fn>,
): void {
  void fn;
  void args;
}

export function use(): void {
  take(
    function render(): string {
      return '';
    },
    [],
  );
}
`;
        writeFileSync(
          join(projectRoot.path, 'tsconfig.json',),
          '{"compilerOptions":{"strict":true},"include":["input.ts"]}\n',
        );
        writeFileSync(inputPath, inputSource,);
        clearEffectSummaryCache();
        clearFinalEffectIndexCache();
        const coldSession = openSemanticFile({
          fileName: inputPath,
          sourceText: inputSource,
          hasBOM: false,
        },);
        buildEffectSummaryIndex({
          project: coldSession.project,
          activeSourceFile: coldSession.sourceFile,
          cacheRootOverride: cacheRoot,
        },);
        /** Relative persistent entries written by the cold scan. */
        const cacheEntries = readdirSync(cacheRoot, {
          recursive: true,
          encoding: 'utf8',
        },)
          .filter(function jsonEntry(entry,): boolean {
            return entry.endsWith('.json',);
          },);
        const [cacheEntry,] = cacheEntries;
        if (cacheEntry === undefined)
          throw new Error('Expected persistent summary cache entry.',);
        /** Exact persistent cache entry path for schema controls. */
        const cacheEntryPath = join(cacheRoot, cacheEntry,);
        /** Persisted JSON whose omission metadata the test seeds. */
        const cached = JSON.parse(readFileSync(
          cacheEntryPath,
          'utf8',
        ),) as {
          readonly fileName: string;
          readonly omittedCallableKeys?: readonly string[];
          readonly omissionReasons?: readonly string[];
        };
        /* The compiler summarizes this fixture completely, so the cold entry carries no
         * omission. The record a failing scan persists is seeded instead: restoring a
         * persisted omission is a cache contract, and seeding is how a suite whose scan
         * always completes still pins that contract. */
        expect(cached.omittedCallableKeys ?? [],).toEqual([],);
        expect(cached.omissionReasons ?? [],).toEqual([],);
        /** Entry rewritten to carry the omission record a failing scan leaves behind. */
        const seededText = JSON.stringify({
          ...cached,
          omittedCallableKeys: [`${cached.fileName}:seeded-omitted-callable`,],
          omissionReasons: ['direct-summary-construction-failed',],
        },);
        writeFileSync(cacheEntryPath, seededText,);
        closeSemanticBridge();
        clearEffectSummaryCache();
        clearFinalEffectIndexCache();
        const warmSession = openSemanticFile({
          fileName: inputPath,
          sourceText: inputSource,
          hasBOM: false,
        },);
        const warmIndex = buildEffectSummaryIndex({
          project: warmSession.project,
          activeSourceFile: warmSession.sourceFile,
          cacheRootOverride: cacheRoot,
        },);
        /** Warm counters proving source restored rather than rescanned. */
        const warmStats = effectSummaryCacheStats();
        expect(warmStats.directSummaryBuildCount,).toBe(0,);
        expect(warmStats.persistentSourceCacheHitCount,).toBeGreaterThan(0,);
        /* The restored record surfaces through the same ledger a consumer reads, naming the
         * seeded key's source with its count and reason. */
        for (const statement of warmSession.sourceFile.statements) {
          if (!isFunctionLikeDeclaration(statement,))
            continue;
          expect(warmIndex.reachedOmissions(statement,)).toEqual([
            {
              sourceFileName: warmSession.sourceFile.fileName,
              omittedCount: 1,
              reasons: ['direct-summary-construction-failed',],
            },
          ],);
        }
        closeSemanticBridge();
        clearEffectSummaryCache();
        clearFinalEffectIndexCache();
        /** Parsed current entry before simulating pre-omission schema. */
        const currentEnvelope = JSON.parse(readFileSync(
          cacheEntryPath,
          'utf8',
        ),) as Readonly<Record<string, unknown>>;
        /** Legacy envelope without omission metadata and with prior schema identity. */
        const legacyEnvelope = Object.fromEntries(Object.entries(currentEnvelope,)
          .filter(function nonOmissionField([field,],): boolean {
            return (field !== 'omittedCallableKeys') && (field !== 'omissionReasons');
          },),);
        writeFileSync(
          cacheEntryPath,
          JSON.stringify({
            ...legacyEnvelope,
            schema: 5,
          },),
        );
        const legacySession = openSemanticFile({
          fileName: inputPath,
          sourceText: inputSource,
          hasBOM: false,
        },);
        buildEffectSummaryIndex({
          project: legacySession.project,
          activeSourceFile: legacySession.sourceFile,
          cacheRootOverride: cacheRoot,
        },);
        /** Counters proving legacy entry became miss rather than inferred omission. */
        const legacyStats = effectSummaryCacheStats();
        expect(legacyStats.directSummaryBuildCount,).toBeGreaterThan(0,);
        expect(legacyStats.persistentSourceCacheHitCount,).toBe(0,);
        closeSemanticBridge();
        clearEffectSummaryCache();
        clearFinalEffectIndexCache();
      },
    },),
    it({
      name: 'does not reuse an index that excluded the next active external-classified source',
      fn: async () => {
        using projectRoot = disposableCacheDirectory();
        /** Disposable persistent cache root separated from TypeScript inputs. */
        const cacheRoot = join(projectRoot.path, '.effect-cache',);
        /** Root-owned source used to build first fixed-point index. */
        const rootInputPath = join(projectRoot.path, 'input.ts',);
        /** Installed package directory classified as external by root project. */
        const nestedRoot = join(
          projectRoot.path,
          'node_modules',
          'effect-cache-probe',
        );
        /** Package callable omitted from first root-project effect index. */
        const nestedInputPath = join(nestedRoot, 'index.ts',);
        mkdirSync(nestedRoot, { recursive: true, },);
        writeFileSync(
          join(projectRoot.path, 'tsconfig.json',),
          '{"compilerOptions":{"module":"NodeNext","moduleResolution":"NodeNext","strict":true},"include":["input.ts"]}\n',
        );
        writeFileSync(
          join(nestedRoot, 'package.json',),
          '{"name":"effect-cache-probe","type":"module","exports":"./index.ts"}\n',
        );
        writeFileSync(
          rootInputPath,
          "import { nestedValue, } from 'effect-cache-probe';\nexport const rootValue: string = nestedValue('root');\n",
        );
        writeFileSync(
          nestedInputPath,
          'export function nestedValue(value: string): string { return value; }\n',
        );
        clearEffectSummaryCache();
        clearFinalEffectIndexCache();
        const session = openSemanticFile({
          fileName: rootInputPath,
          sourceText: readFileSync(rootInputPath, 'utf8',),
          hasBOM: false,
        },);
        /** Nested source decoded in root program but classified as external. */
        const nestedSource = session.project.program.getSourceFile(nestedInputPath,);
        if (nestedSource === undefined)
          throw new Error('Expected root project to decode nested source.',);
        expect(session.project.program.isSourceFileFromExternalLibrary(nestedSource,),).toBe(true,);
        buildEffectSummaryIndex({
          project: session.project,
          activeSourceFile: session.sourceFile,
          cacheRootOverride: cacheRoot,
        },);
        /** Index rebuilt with nested external-classified source as active target. */
        const nestedIndex = buildEffectSummaryIndex({
          project: session.project,
          activeSourceFile: nestedSource,
          cacheRootOverride: cacheRoot,
        },);
        /** Nested function declaration requiring exact summary lookup. */
        const [nestedDeclaration,] = nestedSource.statements;
        if ((nestedDeclaration === undefined)
          || (!isFunctionLikeDeclaration(nestedDeclaration,)))
          throw new Error('Expected nested function declaration.',);
        expect(nestedIndex.get(nestedDeclaration,),).not.toBe(NO_EFFECT_SUMMARY,);
        closeSemanticBridge();
        clearEffectSummaryCache();
        clearFinalEffectIndexCache();
      },
    },),
    it({
      name: 'invalidates final index when active overlay creates a new semantic snapshot',
      fn: async () => {
        using projectRoot = disposableCacheDirectory();
        /** Disposable persistent cache root. */
        const cacheRoot = join(projectRoot.path, '.effect-cache',);
        /** Active source path shared by both semantic snapshots. */
        const inputPath = join(projectRoot.path, 'input.ts',);
        /** Initial observational implementation. */
        const initialSource = 'export function inspect(value: { text: string; },): string { return value.text; }\n';
        /** Changed overlay implementation mutating caller-owned state. */
        const changedSource = 'export function inspect(value: { text: string; },): string { value.text = value.text.trim(); return value.text; }\n';
        writeFileSync(
          join(projectRoot.path, 'tsconfig.json',),
          '{"compilerOptions":{"strict":true},"include":["input.ts"]}\n',
        );
        writeFileSync(inputPath, initialSource,);
        clearEffectSummaryCache();
        clearFinalEffectIndexCache();
        const initialSession = openSemanticFile({
          fileName: inputPath,
          sourceText: initialSource,
          hasBOM: false,
        },);
        buildEffectSummaryIndex({
          project: initialSession.project,
          activeSourceFile: initialSession.sourceFile,
          cacheRootOverride: cacheRoot,
        },);
        const changedSession = openSemanticFile({
          fileName: inputPath,
          sourceText: changedSource,
          hasBOM: false,
        },);
        const changedIndex = buildEffectSummaryIndex({
          project: changedSession.project,
          activeSourceFile: changedSession.sourceFile,
          cacheRootOverride: cacheRoot,
        },);
        /** Changed declaration decoded from new immutable snapshot. */
        const [changedDeclaration,] = changedSession.sourceFile.statements;
        if ((changedDeclaration === undefined)
          || (!isFunctionLikeDeclaration(changedDeclaration,)))
          throw new Error('Expected changed overlay declaration.',);
        /** Changed summary must not reuse observational final index. */
        const changedSummary = changedIndex.get(changedDeclaration,);
        if (changedSummary === NO_EFFECT_SUMMARY)
          throw new Error('Expected changed overlay summary.',);
        expect([...changedSummary.mutatedParameterIndexes,],).toEqual([0,],);
        expect(finalEffectIndexCacheStats().writeCount,).toBe(2,);
        closeSemanticBridge();
        clearEffectSummaryCache();
        clearFinalEffectIndexCache();
      },
    },),
    it({
      name: 'invalidates unchanged caller summaries when dependency changes across bridge lifecycle',
      fn: async () => {
        using projectRoot = disposableCacheDirectory();
        /** Disposable persistent cache root separated from TypeScript inputs. */
        const cacheRoot = join(projectRoot.path, '.effect-cache',);
        /** Unchanged caller source whose call target changes on disk. */
        const inputPath = join(projectRoot.path, 'input.ts',);
        /** Imported implementation participating in caller effect resolution. */
        const helperPath = join(projectRoot.path, 'helper.ts',);
        writeFileSync(
          join(projectRoot.path, 'tsconfig.json',),
          '{"compilerOptions":{"strict":true},"include":["*.ts"]}\n',
        );
        writeFileSync(
          inputPath,
          "import { inspect, } from './helper.js';\nexport function caller(value: { text: string; },): string { return inspect(value); }\n",
        );
        writeFileSync(
          helperPath,
          'export function inspect(value: { text: string; },): string { return value.text; }\n',
        );
        clearEffectSummaryCache();
        clearFinalEffectIndexCache();
        const firstSession = openSemanticFile({
          fileName: inputPath,
          sourceText: readFileSync(inputPath, 'utf8',),
          hasBOM: false,
        },);
        buildEffectSummaryIndex({
          project: firstSession.project,
          activeSourceFile: firstSession.sourceFile,
          cacheRootOverride: cacheRoot,
        },);
        closeSemanticBridge();
        writeFileSync(
          helperPath,
          'export function inspect(value: { text: string; },): string { value.text = value.text.trim(); return value.text; }\n',
        );
        clearEffectSummaryCache();
        const changedSession = openSemanticFile({
          fileName: inputPath,
          sourceText: readFileSync(inputPath, 'utf8',),
          hasBOM: false,
        },);
        buildEffectSummaryIndex({
          project: changedSession.project,
          activeSourceFile: changedSession.sourceFile,
          cacheRootOverride: cacheRoot,
        },);
        /** Counters proving project fingerprint rejected stale caller entry. */
        const changedStats = effectSummaryCacheStats();
        /** Final-index writes proving new semantic lifecycle recomputed project. */
        const changedFinalStats = finalEffectIndexCacheStats();
        expect(changedStats.directSummaryBuildCount > 0,).toBe(true,);
        expect(changedFinalStats.writeCount,).toBe(1,);
        expect(changedStats.persistentSourceCacheHitCount,).toBe(0,);
        closeSemanticBridge();
        clearEffectSummaryCache();
        clearFinalEffectIndexCache();
      },
    },),
    it({
      name: 'rejects corrupt nested persistent payloads',
      fn: async () => {
        using projectRoot = disposableCacheDirectory();
        /** Disposable persistent cache root. */
        const cacheRoot = join(projectRoot.path, '.effect-cache',);
        /** Single-source configured project input. */
        const inputPath = join(projectRoot.path, 'input.ts',);
        /** Stable single-source input text. */
        const inputSource = 'export function inspect(value: { text: string; },): string { return value.text; }\n';
        writeFileSync(
          join(projectRoot.path, 'tsconfig.json',),
          '{"compilerOptions":{"strict":true},"include":["input.ts"]}\n',
        );
        writeFileSync(inputPath, inputSource,);
        clearEffectSummaryCache();
        clearFinalEffectIndexCache();
        const firstSession = openSemanticFile({
          fileName: inputPath,
          sourceText: inputSource,
          hasBOM: false,
        },);
        buildEffectSummaryIndex({
          project: firstSession.project,
          activeSourceFile: firstSession.sourceFile,
          cacheRootOverride: cacheRoot,
        },);
        closeSemanticBridge();
        /** Relative persistent entry paths discovered after cold write. */
        const cacheEntries: string[] = [];
        for (const entry of readdirSync(cacheRoot, {
          recursive: true,
          encoding: 'utf8',
        },)) {
          if (entry.endsWith('.json',))
            cacheEntries.push(entry,);
        }
        const [cacheEntry,] = cacheEntries;
        if (cacheEntry === undefined)
          throw new Error('Expected persistent summary cache entry.',);
        /** Exact persistent JSON before nested corruption. */
        const cacheText = readFileSync(join(cacheRoot, cacheEntry,), 'utf8',);
        /** Cache JSON with valid envelope and invalid nested parameter count. */
        const corruptText = cacheText.replace(
          '"parameterCount":1',
          '"parameterCount":"invalid"',
        );
        if (corruptText === cacheText)
          throw new Error('Expected serialized parameter count to corrupt.',);
        writeFileSync(
          join(cacheRoot, cacheEntry,),
          corruptText,
        );
        clearEffectSummaryCache();
        clearFinalEffectIndexCache();
        const secondSession = openSemanticFile({
          fileName: inputPath,
          sourceText: inputSource,
          hasBOM: false,
        },);
        buildEffectSummaryIndex({
          project: secondSession.project,
          activeSourceFile: secondSession.sourceFile,
          cacheRootOverride: cacheRoot,
        },);
        /** Counters proving malformed nested payload became a miss. */
        const recoveredStats = effectSummaryCacheStats();
        expect(recoveredStats.directSummaryBuildCount > 0,).toBe(true,);
        closeSemanticBridge();
        clearEffectSummaryCache();
        clearFinalEffectIndexCache();
      },
    },),
    it({
      name: 'keeps omission fingerprints equal across cold and warm processes',
      fn: async () => {
        using project = disposableCacheDirectory();
        /** Persistent cache isolated from every other omission test. */
        const cacheRoot = join(project.path, '.effect-cache',);
        /** Single-source input with a generic tuple parameter and one call of it. */
        const inputPath = join(project.path, 'input.ts',);
        /** Independent process probe importing built package interface. */
        const probePath = join(project.path, 'omission-probe.mjs',);
        writeFileSync(
          join(project.path, 'tsconfig.json',),
          '{"compilerOptions":{"strict":true},"include":["input.ts"]}\n',
        );
        writeFileSync(
          inputPath,
          `export function take<Fn extends (...args: never[]) => unknown,>(
  fn: Fn,
  args: Parameters<Fn>,
): void {
  void fn;
  void args;
}

export function use(): void {
  take(
    function render(): string {
      return '';
    },
    [],
  );
}
`,
        );
        /** Probe source printing cache activity and every callable verdict. */
        const probeSource = `import { readFileSync } from 'node:fs';
import { buildEffectSummaryIndex, closeSemanticBridge, effectSummaryCacheStats, openSemanticFile, NO_EFFECT_SUMMARY } from ${JSON.stringify(BUILT_ENTRY_URL)};
const [fileName, cacheRoot] = process.argv.slice(2);
const sourceText = readFileSync(fileName, 'utf8');
const session = openSemanticFile({ fileName, sourceText, hasBOM: false });
const index = buildEffectSummaryIndex({ project: session.project, activeSourceFile: session.sourceFile, cacheRootOverride: cacheRoot });
const fingerprint = session.sourceFile.statements.flatMap((declaration) => {
  if (!('parameters' in declaration)) return [];
  const summary = index.get(declaration);
  if (summary === NO_EFFECT_SUMMARY) return [[declaration.name?.text ?? 'anonymous', 'NO_SUMMARY']];
  return [[
    declaration.name?.text ?? 'anonymous',
    [...summary.referentMutatedParameterIndexes].sort().join(','),
    [...summary.opaqueParameterIndexes].sort().join(','),
    [...summary.returnedParameterIndexes].sort().join(','),
  ]];
});
const omissions = index.reachedOmissions(session.sourceFile.statements[0]);
console.log(JSON.stringify({ ...effectSummaryCacheStats(), fingerprint, omissions }));
closeSemanticBridge();
`;
        writeFileSync(probePath, probeSource,);
        const cold = await spawn(
          'node',
          [probePath, inputPath, cacheRoot,],
        );
        /* The compiler summarizes this fixture completely, so the cold process's entry
         * carries no omission. The record a failing scan persists is seeded between the
         * processes: the warm process must restore it and report exactly its fingerprint
         * while its verdicts stay equal to the cold process's. */
        /** Relative persistent entries the cold process wrote. */
        const writtenEntries = readdirSync(cacheRoot, {
          recursive: true,
          encoding: 'utf8',
        },)
          .filter(function jsonEntry(entry,): boolean {
            return entry.endsWith('.json',);
          },);
        const [writtenEntry,] = writtenEntries;
        if (writtenEntry === undefined)
          throw new Error('Expected persistent summary cache entry.',);
        /** Exact entry path rewritten with the seeded omission record. */
        const writtenEntryPath = join(cacheRoot, writtenEntry,);
        /** Entry JSON whose omission fields the test seeds. */
        const writtenEnvelope = JSON.parse(readFileSync(
          writtenEntryPath,
          'utf8',
        ),) as {
          readonly fileName: string;
        };
        writeFileSync(
          writtenEntryPath,
          JSON.stringify({
            ...writtenEnvelope,
            omittedCallableKeys: [`${writtenEnvelope.fileName}:seeded-omitted-callable`,],
            omissionReasons: ['direct-summary-construction-failed',],
          },),
        );
        const warm = await spawn(
          'node',
          [probePath, inputPath, cacheRoot,],
        );
        /** Cold process result built from syntax. */
        const coldResult = JSON.parse(cold.stdout.trim(),) as {
          readonly directSummaryBuildCount: number;
          readonly persistentSourceCacheHitCount: number;
          readonly fingerprint: readonly (readonly string[])[];
          readonly omissions: readonly unknown[];
        };
        /** Warm process result restored from persisted summaries and omissions. */
        const warmResult = JSON.parse(warm.stdout.trim(),) as {
          readonly directSummaryBuildCount: number;
          readonly persistentSourceCacheHitCount: number;
          readonly fingerprint: readonly (readonly string[])[];
          readonly omissions: readonly unknown[];
        };
        expect(coldResult.directSummaryBuildCount,).toBeGreaterThan(0,);
        expect(coldResult.persistentSourceCacheHitCount,).toBe(0,);
        expect(warmResult.directSummaryBuildCount,).toBe(0,);
        expect(warmResult.persistentSourceCacheHitCount,).toBeGreaterThan(0,);
        expect(warmResult.fingerprint,).toEqual(coldResult.fingerprint,);
        /** Omission fingerprint the seeded record carries. */
        const seededFingerprint = [{
          sourceFileName: inputPath,
          omittedCount: 1,
          reasons: ['direct-summary-construction-failed',],
        },];
        expect(coldResult.omissions,).toEqual([],);
        expect(warmResult.omissions,).toEqual(seededFingerprint,);
        expect(warm.stderr,).toContain(
          'restored 1 omitted callable summaries for',
        );
        expect(warm.stderr,).toContain(
          'from effect cache: direct-summary-construction-failed',
        );
      },
    },),
    it({
      name: 'reuses persistent summaries across independent Node processes',
      fn: async () => {
        using project = disposableCacheDirectory();
        /** Persistent cache isolated from concurrently changing fixture projects. */
        const cacheRoot = join(project.path, '.effect-cache',);
        /** Single-source project analyzed identically by both processes. */
        const inputPath = join(project.path, 'input.ts',);
        /** Independent process probe importing only built package API. */
        const probePath = join(project.path, 'persistent-probe.mjs',);
        writeFileSync(
          join(project.path, 'tsconfig.json',),
          '{"compilerOptions":{"strict":true},"include":["input.ts"]}\n',
        );
        /* The source carries a deferred RETAINED result use, and that is the whole reason
         * it is not the one-liner it used to be.
         *
         * This test read `inspect(value) { return value.text; }`, which produces no
         * deferred result use of any kind, so it asserted that the counters move and
         * nothing about what survives the round trip. When the retaining kind arrived it
         * would have stayed green while serialization dropped the new field, while the
         * validator rejected the payload, or while the restored application reached the
         * propagation throw, because none of those paths were entered. A cache test over a
         * fixture that exercises none of the cached shapes measures the counters only.
         *
         * `storeItem` stores what `firstItem` handed back, which is a piece of its own
         * parameter, so the retention is real and the warm process must arrive at the same
         * verdict as the cold one. */
        writeFileSync(
          inputPath,
          'type Item = { text: string; };\n'
            + 'type Box = { item: Item; };\n'
            + 'let held: Item | undefined;\n'
            + 'export function firstItem(box: Box): Item { return box.item; }\n'
            + 'export function storeItem(box: Box): void { held = firstItem(box); }\n'
            + 'export function readHeld(): string { return held === undefined ? \'\' : held.text; }\n',
        );
        /** Probe source printing cache counters and the retained verdict itself. */
        const probeSource = `import { readFileSync } from 'node:fs';\nimport { buildEffectSummaryIndex, closeSemanticBridge, effectSummaryCacheStats, openSemanticFile, NO_EFFECT_SUMMARY } from ${JSON.stringify(BUILT_ENTRY_URL)};\nconst [fileName, cacheRoot] = process.argv.slice(2);\nconst sourceText = readFileSync(fileName, 'utf8');\nconst session = openSemanticFile({ fileName, sourceText, hasBOM: false });\nconst index = buildEffectSummaryIndex({ project: session.project, activeSourceFile: session.sourceFile, cacheRootOverride: cacheRoot });\nconst declaration = session.nodeAtOffset(sourceText.indexOf('function storeItem') + 'function '.length).parent;\nconst summary = index.get(declaration);\nconst opaque = summary === NO_EFFECT_SUMMARY ? 'NO_SUMMARY' : [...summary.opaqueParameterIndexes].sort().join(',');\nconsole.log(JSON.stringify({ ...effectSummaryCacheStats(), opaque }));\ncloseSemanticBridge();\n`;
        writeFileSync(probePath, probeSource,);
        const first = await spawn(
          'node',
          [probePath, inputPath, cacheRoot,],
        );
        const second = await spawn(
          'node',
          [probePath, inputPath, cacheRoot,],
        );
        /** Cold-process counters and verdict from direct analysis. */
        const coldStats = JSON.parse(first.stdout.trim(),) as {
          readonly directSummaryBuildCount: number;
          readonly persistentSourceCacheHitCount: number;
          readonly opaque: string;
        };
        /** Warm-process counters and verdict rebuilt from the persisted payload. */
        const warmStats = JSON.parse(second.stdout.trim(),) as {
          readonly directSummaryBuildCount: number;
          readonly persistentSourceCacheHitCount: number;
          readonly opaque: string;
        };
        expect(coldStats.directSummaryBuildCount > 0,).toBe(true,);
        expect(coldStats.persistentSourceCacheHitCount,).toBe(0,);
        expect(warmStats.directSummaryBuildCount,).toBe(0,);
        expect(warmStats.persistentSourceCacheHitCount > 0,).toBe(true,);
        /* The verdict, not just the counters. The cold process computes the retention from
         * syntax and the warm one restores it from disk, so agreement here is the only
         * thing proving the retained application survives serialization, validation and
         * rehydration. Asserting the exact value rather than equality of the two, because
         * two processes that both lost it would agree on `''` and pass. */
        expect(coldStats.opaque,).toBe('0',);
        expect(warmStats.opaque,).toBe('0',);
      },
    },),
    it({
      name: 'completes the foreign-borrowed graph when a usage sits at module top level',
      fn: async () => {
        using projectRoot = disposableCacheDirectory();
        /** Marker source directory preserving exact semantic identity suffix. */
        const markerRoot = join(
          projectRoot.path,
          'ownership-marker',
          'foreign-borrowed',
          'src',
        );
        /** Active foreign-boundary source. */
        const foreignPath = join(projectRoot.path, 'foreign.ts',);
        /** Shared helper reached through the marked boundary. */
        const helperPath = join(projectRoot.path, 'helper.ts',);
        /**
         Active source whose boundary is itself invoked at module top level.
         
         The inbound walk for `readForeign` starts at that trailing call and
         passes no callable before the source file, which is the shape that
         previously stepped off the root.
         */
        const foreignSource = "import type { ForeignBorrowed, } from './ownership-marker/foreign-borrowed/src/index.js';\nimport { read, } from './helper.js';\nexport function readForeign(value: ForeignBorrowed<{ text: string; }>,): string { return read(value); }\nexport const eager: string = readForeign({ text: 'top level', },);\n";
        mkdirSync(markerRoot, { recursive: true, },);
        writeFileSync(
          join(projectRoot.path, 'tsconfig.json',),
          '{"compilerOptions":{"strict":true},"include":["**/*.ts"]}\n',
        );
        writeFileSync(
          join(markerRoot, 'index.ts',),
          'declare const FOREIGN_BORROWED_MARKER: unique symbol;\nexport type ForeignBorrowed<Value> = Value & { readonly [FOREIGN_BORROWED_MARKER]?: true; };\n',
        );
        writeFileSync(foreignPath, foreignSource,);
        writeFileSync(
          helperPath,
          'export function read(value: { text: string; },): string { return value.text; }\n',
        );
        clearEffectSummaryCache();
        clearFinalEffectIndexCache();
        const session = openSemanticFile({
          fileName: foreignPath,
          sourceText: foreignSource,
          hasBOM: false,
        },);
        /** Index build threw before the parent walk guarded the root. */
        const index = buildEffectSummaryIndex({
          project: session.project,
          activeSourceFile: session.sourceFile,
          cacheRootOverride: join(projectRoot.path, '.effect-cache',),
        },);
        /** Active boundary declaration retaining explicit foreign provenance. */
        const foreignDeclaration = session.nodeAtOffset(foreignSource.indexOf('readForeign',),)
          .parent;
        if (!isFunctionLikeDeclaration(foreignDeclaration,))
          throw new Error('Expected active foreign boundary declaration.',);
        /** Active boundary summary proving graph completion survived the walk. */
        const foreignSummary = index.get(foreignDeclaration,);
        if (foreignSummary === NO_EFFECT_SUMMARY)
          throw new Error('Expected active foreign boundary summary.',);
        expect([...index.proveForeignBorrowed(foreignDeclaration,),],).toEqual([0,],);
        closeSemanticBridge();
        clearEffectSummaryCache();
        clearFinalEffectIndexCache();
      },
    },),
  ],
},);
