import {
  mkdtempSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from 'node:fs';
import { resolve, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  fixtureConfigPath,
  fixtureSourceRoot,
  type OxlintRuleDiagnostic as OxlintDiagnostic,
  resolveFixtureTarget,
  runOxlintFixture,
} from '@monochromatic-dev/oxlint-plugin-test-support/ts';

/** Fixture source root. */
const FIXTURES = fixtureSourceRoot({
  fixturePackageName: 'oxlint-no-restricted-syntax',
},);

/** Fixture config enabling only semantic readonly-effect rule. */
const READONLY_FIXTURE_CONFIG = fixtureConfigPath({
  fixturePackageName: 'oxlint-no-restricted-syntax',
  fileName: '.oxlintrc.readonly.fixture.json',
},);

/** Disposable in-project TypeScript source. */
type GeneratedSource = {
  readonly filePath: string;
  readonly [Symbol.dispose]: () => void;
};

/**
 Runs semantic readonly-effect rule against dedicated fixture source.
 
 @param fixturePath - Relative fixture path under source root.
 
 @returns readonly-rule diagnostics.
 */
async function lintReadonly(fixturePath: string,): Promise<readonly OxlintDiagnostic[]> {
  return runOxlintFixture({
    codePrefix: 'prefer-readonly-parameter-type(',
    configFlag: '-c',
    fixtureConfig: READONLY_FIXTURE_CONFIG,
    target: resolveFixtureTarget({
      fixtureSourceRoot: FIXTURES,
      fixturePath,
    },),
    threads: 1,
  },);
}

/**
 Removes action guidance when a legacy assertion isolates diagnostic claim.
 
 @param message - Complete public diagnostic message.
 
 @returns preference claim through classification reason, or unchanged non-preference message.
 
 @example
 ```ts
 preferenceClaim('Parameter "x" can be deeply readonly: `x` is writable. Guidance.');
 ```
 */
function preferenceClaim(message: string,): string {
  /**
   Start of preference predicate in diagnostic message.
   */
  const preferenceAt = message.indexOf(' can be deeply readonly:',);
  if (preferenceAt === (-1))
    return message;
  /**
   Separator after classification reason and before action guidance.
   */
  const guidanceAt = message.indexOf('. ', preferenceAt,);
  return guidanceAt === (-1) ? message : message.slice(0, guidanceAt + 1,);
}

/**
 Runs semantic rule after Oxlint creates high-count fixed allocator pool.
 
 @param fixturePath - Relative fixture path under source root.
 
 @returns readonly-rule diagnostics.
 */
async function lintReadonlyWithHighWorkerCount(
  fixturePath: string,
): Promise<readonly OxlintDiagnostic[]> {
  return runOxlintFixture({
    codePrefix: 'prefer-readonly-parameter-type(',
    configFlag: '-c',
    fixtureConfig: READONLY_FIXTURE_CONFIG,
    target: resolveFixtureTarget({
      fixtureSourceRoot: FIXTURES,
      fixturePath,
    },),
    threads: 16,
  },);
}

/**
 Applies readonly semantic fixes to disposable source inside fixture project.
 
 @param source - Source text to lint and optionally suggest-fix.
 
 @param fixSuggestions - Whether suggestion channel is enabled.
 
 @returns resulting source text.
 */
async function fixReadonlyGeneratedSource({
  source,
  fixSuggestions,
}: {
  readonly source: string;
  readonly fixSuggestions: boolean;
},): Promise<string> {
  /** Disposable in-project directory discoverable by fixture tsconfig. */
  const dirPath = mkdtempSync(resolve(FIXTURES, 'readonly-fix-',),);
  /** Disposable source path inside configured project. */
  const filePath = resolve(dirPath, 'stale-contract.ts',);
  writeFileSync(filePath, source,);
  using fixture: GeneratedSource = {
    filePath,
    [Symbol.dispose]: function cleanup(): void {
      rmSync(dirPath, { recursive: true, force: true, },);
    },
  };
  await runOxlintFixture({
    codePrefix: 'prefer-readonly-parameter-type(',
    configFlag: '-c',
    fixtureConfig: READONLY_FIXTURE_CONFIG,
    target: fixture.filePath,
    threads: 1,
    // Suggestions stay opt-in so a case can show what the ordinary fix leaves alone.
    fix: fixSuggestions ? 'fixes-and-suggestions' : 'fixes',
  },);
  return readFileSync(fixture.filePath, 'utf8',);
}

await describe({
  name: 'prefer-readonly-parameter-types diagnostics',
  concurrency: 1,
  children: [
    it({
      name: 'accepts primitive reads from deeply readonly input without callable boundaries',
      fn: async () => {
        expect(await lintReadonly('readonly-valid.ts',),).toEqual([],);
      },
    },),
    it({
      name: 'starts semantic child before high-worker fixed allocator reservations',
      fn: async () => {
        expect(await lintReadonlyWithHighWorkerCount('readonly-valid.ts',),).toEqual([],);
      },
    },),
    it({
      name: 'rejects former catalog and contract exemptions',
      fn: async () => {
        const diagnostics = await lintReadonly('readonly-catalog-free-invalid.ts',);
      /* Eleven after position-aware foreign observer inbounds removed five preference
       * findings on callback formals receiving elements or receiver collections from marked
       * boundaries. The remaining two preferences have mixed owned and foreign inbounds,
       * while every unresolved implementation finding remains.
       *
       * Four programs were previously added here for returned-result discharge ownership
       * guards and removed because each was charged through the foreign-borrowed opaque
       * boundary without reaching that discharge. */
        expect(diagnostics.length,).toBe(11,);
        const messages = diagnostics.map(function diagnosticMessage(diagnostic,): string {
          return preferenceClaim(diagnostic.message,);
        },);
        expect(messages.some(function catalogRemediationRemoved(message,): boolean {
          return message.includes('audited-call catalogue',);
        },),).toBe(false,);
      /* Nine findings still say a contract cannot discharge an unresolved implementation, and
       * two of them say it in the collection message's words instead. Both sentences are
       * counted, because the claim this pins is that no finding offers a contract as a way
       * out, not which of the two texts carries it. Splitting the count would let a finding
       * lose the claim entirely by moving between messages.
       *
       * Nine to eight when the already-readonly message was added. One finding here names an
       * input readonly at every level, and that message mentions no contract at all, so the
       * claim it pins is kept rather than lost: what is counted is texts that refuse a
       * contract as a way out, and a text that never raises one refuses it by omission. */
        expect(messages.filter(function contractsCannotDischarge(message,): boolean {
          return message.includes(
            'An @mutates block alone documents known effects but cannot make an unresolved implementation safe.',
          )
            || message.includes(
              'ForeignHostCapability does not apply here.',
            );
        },).length,).toBe(9,);
      },
    },),
    it({
      name: 'tells a collection finding what would resolve it, and nothing that would not',
      fn: async () => {
      /* The message issue #414 asked for, pinned by content rather than by count. Its
       * complaint was that the four printed remediations named no change resolving a finding
       * about an array method: an engine intrinsic has no repository-owned implementation to
       * add to a tsconfig, and ordinary rows are not a runtime-owned host capability. A
       * finding whose every cause is a collection member now gets a message whose every
       * remediation is a measured behaviour of this rule. */
        const messages = (await lintReadonly('readonly-result-provenance-invalid.ts',))
          .map(function diagnosticMessage(diagnostic,): string {
            return preferenceClaim(diagnostic.message,);
          },);
      /**
       Findings routed to the collection message.
       */
        const collectionMessages = messages.filter(function isCollection(message,): boolean {
          return message.includes('is exposed through these unresolved collection calls',);
        },);
        expect(collectionMessages.length > 0,).toBe(true,);
        expect(collectionMessages.every(function namesEveryRemediation(message,): boolean {
          return message.includes('an observer this repository owns',)
            && message.includes('Keep the result inside this function.',)
            && message.includes('Fold to a primitive',)
            && message.includes('Iterate directly with for...of',);
        },),).toBe(true,);
      /* And none of the remediations that fit nothing here. The tsconfig one names a
       * repository-owned implementation, which no intrinsic has, and the marker one names a
       * host capability, which ordinary collection data is not. */
        expect(collectionMessages.some(function offersTsconfig(message,): boolean {
          return message.includes('nearest tsconfig.json',);
        },),).toBe(false,);
        expect(collectionMessages.every(function refusesTheMarker(message,): boolean {
          return message.includes('ForeignHostCapability does not apply here.',);
        },),).toBe(true,);
      },
    },),
    it({
      name: 'admits exact contracted host capability only after ordinary inference fails',
      fn: async () => {
        expect(await lintReadonly('readonly-host-capability-valid.ts',),).toEqual([],);
      },
    },),
    it({
      name: 'requires host contract and rejects same-named marker lookalike',
      fn: async () => {
        const diagnostics = await lintReadonly('readonly-host-capability-invalid.ts',);
      /* Exact uncontracted host capability receives both its missing-contract and unresolved-effect
       * findings; the same-named marker lookalike receives only unresolved effect. */
        expect(diagnostics.length,).toBe(3,);
        expect(diagnostics.filter(function opaqueRule(diagnostic,): boolean {
          return diagnostic.code
            === 'prefer-readonly-parameter-type(no-opaque-parameter-effects)';
        },).length,).toBe(2,);
        expect(diagnostics.filter(function contractRule(diagnostic,): boolean {
          return diagnostic.code
            === 'prefer-readonly-parameter-type(no-invalid-parameter-effect-contracts)';
        },).length,).toBe(1,);
        expect(diagnostics.some(function missingContract(diagnostic,): boolean {
          return diagnostic.message.includes(
            'uses ForeignHostCapability for unresolved runtime behavior but lacks corresponding @mutates contract',
          );
        },),).toBe(true,);
        expect(diagnostics.some(function shadowedAliasRemainsOpaque(diagnostic,): boolean {
          return diagnostic.message.startsWith(
            'The function input named "controller" is the receiver of these unresolved method calls: controller.abort [',
          );
        },),).toBe(true,);
      },
    },),
    it({
      name: 'reports inert ForeignBorrowed markers over deeply readonly types',
      fn: async () => {
        const diagnostics = await lintReadonly('readonly-redundant-marker-invalid.ts',);
        expect(diagnostics.length,).toBe(1,);
        expect(diagnostics[0]?.message
          .startsWith(
          'Parameter "state" carries a ForeignBorrowed marker that no longer affects classification',
          ),).toBe(true,);
      },
    },),
    it({
      name: 'rejects static plain-data claims without runtime isolation',
      fn: async () => {
        const diagnostics = await lintReadonly('readonly-static-plain-data-invalid.ts',);
      /* Three, down from four, because `enumeratePlainRecord` returns
       * `Object.entries(record).length` and that reader is now derived. Its own doc calls
       * it an enumeration "without a hook-class effect", so recognizing it is the fixture
       * author's stated intent rather than a weakening. No offer appeared with the report
       * removed, which is the check that matters: this file emits none. */
      /* Two, since `joinPlainElements` stopped reporting when `join` gained the coercion
       * channel. Its elements are `number`, so the coercion the member performs provably
       * runs nothing and the receiver claim is answered. `labels.toSorted` and the `String`
       * argument still report, which is what keeps this from reading as a blanket discharge
       * of the fixture: the first invokes a comparator and the second hands a value to a
       * host call that can reach a getter. */
        expect(diagnostics.length,).toBe(2,);
      /* Every one still says the input reaches something unproven, and the collection findings
       * among them now say it in the collection message's words: `join` and a bare `toSorted()`
       * supply no observer to analyze, so they keep reporting and get the text that names what
       * would resolve them. Accepting either sentence keeps this about the claim rather than
       * about which message carries it. */
        expect(diagnostics.every(function unresolvedBoundary(diagnostic,): boolean {
          return diagnostic.message.includes('cannot inspect enough of those calls',)
            || diagnostic.message.includes('code this rule cannot follow',);
        },),).toBe(true,);
      },
    },),
    it({
      name: 'keeps hook-capable inputs and descriptor mutation fail-closed',
      fn: async () => {
        const diagnostics = await lintReadonly('readonly-plain-data-invalid.ts',);
      /* Still three, and that is the point of this case rather than an absence of change.
       * Deriving the Object readers did not touch it, because both survivors are outside
       * what that derivation admits: `Object.freeze` is not a reader at all, and
       * `enumerateReadonlyMapEntries` enumerates a `ReadonlyMap`, whose properties are
       * methods rather than data. The audit that removed the plain-data catalog left an
       * operand like that fail-closed, and the reader authority's structural data-only
       * gate keeps it that way. */
        expect(diagnostics.length,).toBe(3,);
        const messages = diagnostics.map(function diagnosticMessage(diagnostic,): string {
          return preferenceClaim(diagnostic.message,);
        },);
        expect(messages.some(function unknownCoercionStaysClosed(message,): boolean {
          return message.startsWith(
            'The function input named "value" is exposed to these unresolved calls: String [',
          );
        },),).toBe(true,);
        expect(messages.some(function frozenPlainDataStaysMutation(message,): boolean {
          return message.startsWith(
            'The function input named "value" is exposed to these unresolved calls: Object.freeze [',
          );
        },),).toBe(true,);
        expect(messages.some(function nonPlainEnumerationStaysClosed(message,): boolean {
          return message.startsWith(
            'The function input named "entriesSource" is exposed to these unresolved calls: Object.entries [',
          );
        },),).toBe(true,);
      },
    },),
    it({
      name: 'keeps owned call paths separate from propagated foreign provenance',
      fn: async () => {
        const diagnostics = await lintReadonly('readonly-foreign-provenance-invalid.ts',);
        expect(diagnostics.length,).toBe(5,);
        const messages = diagnostics.map(function diagnosticMessage(diagnostic,): string {
          return preferenceClaim(diagnostic.message,);
        },);
        expect(messages.filter(function ownedChildDiagnostic(message,): boolean {
          return message.startsWith('Parameter "child" can be deeply readonly',);
        },).length,).toBe(2,);
        expect(messages.some(function boundaryRemainsOwned(message,): boolean {
          return message.startsWith('Parameter "tree" can be deeply readonly',);
        },),).toBe(true,);
        expect(messages.some(function replacementRemainsOpaque(message,): boolean {
          return message.startsWith(
            'The function input named "replacement" is exposed to these unresolved calls: values.with [',
          );
        },),).toBe(true,);
        expect(messages.some(function mixedResultRemainsOwned(message,): boolean {
          return message.startsWith('Parameter "child" can be deeply readonly',);
        },),).toBe(true,);
      },
    },),
    it({
      name: 'propagates foreign ownership through exact collection observer positions',
      fn: async () => {
        const diagnostics = await lintReadonly('readonly-foreign-observer-invalid.ts',);
      /**
       Preference findings isolated from separately tested effect categories.
       */
        const preferences = diagnostics.filter(function preference(diagnostic,): boolean {
          return diagnostic.code
            === 'prefer-readonly-parameter-type(prefer-readonly-parameter-types)';
        },);
      /**
       Preference messages for ordinary fold seed and mixed-inbound observer controls.
       */
        const messages = preferences.map(function diagnosticMessage(diagnostic,): string {
          return preferenceClaim(diagnostic.message,);
        },);
        expect(messages.filter(function ordinarySeed(message,): boolean {
          return message.startsWith('Parameter "seededAccumulator" can be deeply readonly:',);
        },).length,).toBe(1,);
        expect(messages.filter(function mixedObserver(message,): boolean {
          return message.startsWith('Parameter "child" can be deeply readonly:',);
        },).length,).toBe(1,);
        [
          'mapChild',
          'mapValues',
          'referencedChild',
          'thisChild',
          'thisValues',
          'wrappedChild',
          'forEachChild',
          'forEachValues',
          'filterChild',
          'filterValues',
          'findChild',
          'findValues',
          'findLastChild',
          'findLastValues',
          'everyChild',
          'everyValues',
          'someChild',
          'someValues',
          'flatMapChild',
          'flatMapValues',
          'seededChild',
          'seededValues',
          'noSeedAccumulator',
          'noSeedChild',
          'noSeedValues',
        ].forEach(function receiverDerivedParameter(parameterName,): void {
          expect(messages.some(function matchingParameter(message,): boolean {
            return message.startsWith(`Parameter "${parameterName}" can be deeply readonly:`,);
          },),).toBe(false,);
        },);
      },
    },),
    it({
      name: 'keeps state reachable through a member result out of every discharge',
      fn: async () => {
        const diagnostics = await lintReadonly('readonly-member-channel-invalid.ts',);
        const messages = diagnostics.map(function diagnosticMessage(diagnostic,): string {
          return preferenceClaim(diagnostic.message,);
        },);
        expect(messages.length,).toBe(4,);
      /* A channel entry for an iterator member discharges exactly when the iterator
       * yields primitives, and these two are the whole of what it buys on its own.
       * Both loops iterate something whose element type carries no receiver state, so
       * the verified channel is the only outstanding claim and nothing is reported. */
        expect(messages.some(function primitiveKeyIterationStaysSilent(
          message,
        ): boolean {
          return message.includes('entries.keys',);
        },),).toBe(false,);
        expect(messages.some(function indexIterationStaysSilent(message,): boolean {
          return message.includes('values.keys',);
        },),).toBe(false,);
      /* And the two the entries did not buy on their own, which now clear under exactly
       * the condition this case predicted: a relation describing a container whose
       * elements are receiver state. `values` carries the element relation and `entries`
       * the paired one, and admitting the `for...of` position stops the iterator reading
       * as a value that leaves.
       *
       * Both loops only read, `held.label.length` and `key.length + held.length`, so no
       * attribution replaces these two: there is nothing beneath them to attribute. That
       * is what separates them from the container cases in the sibling fixture, where a
       * report is traded for a recorded write rather than for a proof that no write
       * exists. */
        expect(messages.some(function heldValueIterationDischarges(
          message,
        ): boolean {
          return message.includes('entries.values',);
        },),).toBe(false,);
        expect(messages.some(function primitivePairIterationDischarges(
          message,
        ): boolean {
          return message.includes('entries.entries',);
        },),).toBe(false,);
      /* A higher-order member is not opaque by virtue of being higher-order.
       * `[...records,].reduce(owned, 0)` is answered by
       * `recordReadonlyViewApplications`, which runs before the channel check and
       * requires exactly what this call supplies: a primitive result, an argument
       * carrying nothing, and a callback resolving to owned source. Its sibling
       * `rows.reduce` above reports, because its result aliases the receiver's own
       * element type, and the pair is what keeps either result from being read as a
       * fact about `reduce` itself.
       *
       * This is a control rather than a case. It measures what a container result
       * relation for iterator members would be worth, since it puts a parameter
       * origin on an array literal with no such relation needed. */
        expect(messages.some(function spreadAccumulatorStaysSilent(
          message,
        ): boolean {
          return message.includes('records.reduce',);
        },),).toBe(false,);
      /* A generic instantiation is not evidence the call built the value. `reduce`
       * returning the accumulator it was handed has result type `string[]` over
       * `string[][]`, a type reference whose only argument is primitive, which the
       * exposure test alone reads as a fresh container of primitives. */
      /* Named as a collection call now, since every cause of this finding is one. The claim is
       * unchanged and so is the finding: `reduce` has no result relation, the aliasing fallback
       * still refuses it, and the parameter stays opaque. Only the sentence naming the calls
       * moved, which is what issue #414 asked for. */
        expect(messages.some(function accumulatorStaysOpaque(message,): boolean {
          return message.startsWith(
            'The function input named "rows" is exposed through these unresolved collection calls: rows.reduce [',
          );
        },),).toBe(true,);
      /* An answered receiver claim must not carry the argument analysis with it.
       * `push` answers its receiver completely, a mutation and nothing unresolved,
       * while `replacement` stays reachable through the array afterwards. Widening
       * the discharge to return early puts this diagnostic out. */
        expect(messages.some(function retainedArgumentStaysOpaque(message,): boolean {
          return message.startsWith(
            'The function input named "replacement" is exposed to these unresolved calls: values.push [',
          );
        },),).toBe(true,);
      /* `join` returns a `string`, so the result condition alone would discharge it
       * while it coerces every element. Dropping the verified-channel condition puts
       * this diagnostic out.
       *
       * Named as a collection call since `join` joined the channel table under the
       * coercion channel. The finding is unchanged and so is the reason for it: its
       * elements are `SealedLabel`, an object, so the coercion reaches that value's own
       * `toString` and the channel is withheld. Only the sentence naming the calls moved,
       * which is what `COLLECTION_MEMBER_NAMES` deriving from the tables does. */
        expect(messages.some(function coercionStaysOpaque(message,): boolean {
          return message.startsWith(
            'The function input named "values" is exposed through these unresolved collection calls: values.join [',
          );
        },),).toBe(true,);
      /* `ReadonlyMap.has` reaches no user code and returns a boolean, so the
       * receiver claim is answered and nothing is left to report. Removing that
       * discharge puts a third diagnostic back. */
        expect(messages.some(function statelessResultStaysSilent(message,): boolean {
          return message.includes('entries.has',);
        },),).toBe(false,);
      /* `find` no longer reports either, and it closed the same hole `at` did rather
       * than widening a discharge. Both carry the verified relation saying the result is
       * one of the receiver's own elements; `at` answered from the channel table while
       * `find`, being observer-bearing, reached the result gate, which had an observer arm
       * and a container arm and no arm for a bare value. So the identical write through the
       * identical kind of element was attributed for one member and merely reported for the
       * other. With the value arm present, `observerResultEscapeEffect` reads
       * `mutated=[0]` with `opaque=[]`, exactly as its `at` sibling does, and
       * `effect-summaries.unit.test.ts` asserts the mutation that replaced this message
       * through `observerValueResultMutationEffect`. Removing the value arm from
       * `viewResultUnaccounted` puts this message back. */
        expect(messages.some(function observerResultStaysOpaque(message,): boolean {
          return message.startsWith(
            'The function input named "values" is exposed through these unresolved collection calls: values.find [',
          );
        },),).toBe(false,);
      /* `at` no longer reports, and that is the hole this rule was carrying rather than
       * a discharge granted on trust. `interiorEscapeEffect` writes through the element
       * `values.at(0)` hands back, and result provenance now tracks that element as
       * receiver state, so the write is recorded against `values` directly. The opacity
       * report existed precisely because nothing tracked the alias; with the alias
       * tracked it is redundant, and `effect-summaries.unit.test.ts` asserts the
       * mutation that replaced it. Removing the escape check in
       * `receiverClaimAnswerable` puts this message back. */
        expect(messages.some(function accessorResultReported(message,): boolean {
          return message.includes('values.at [',);
        },),).toBe(false,);
      /* No collection parameter may be offered as read-only while its elements are
       * rewritten through a result, which is what `values[0].label` mutation already
       * suppresses. The one offer here is on `kept`, an observer parameter whose own
       * elements nothing rewrites, and which cannot be annotated read-only anyway
       * because `reduce` requires the accumulator's own type back. */
        expect(messages.filter(function readonlyOffers(message,): boolean {
          return message.includes('can be deeply readonly',);
        },),).toEqual([
          'Parameter "kept" can be deeply readonly: parameter type uses mutable `Array`.',
        ],);
      /* The indexed control must stay silent: the alias records a mutation of the
       * parameter, so there is nothing to offer and nothing unresolved. Both
       * functions rewrite the same caller-owned row, and for one build they
       * disagreed. */
        expect(messages.some(function indexedControlStaysSilent(message,): boolean {
          return message.includes('indexedElementControlEffect',);
        },),).toBe(false,);
      },
    },),
    it({
      name: 'attributes a member result to its receiver while every sink stays unmodelled',
      fn: async () => {
      /* Where result provenance has reached, measured rather than intended.
       *
       * The resolver now follows a verified member result to its receiver, so an
       * escaping result is attributed: `escapingLookupEffect` reports the
       * `JSON.stringify` its looked-up value reaches, where before it reported the
       * lookup and never named the escape at all.
       *
       * The count has not moved, and that is the design rather than a shortfall.
       * Discharging a receiver's opacity is sound only once every sink a tracked
       * result can reach is attributed or reported, and returns, property stores,
       * container insertions and closure captures are not yet.
       * `doc/decision/prefer-readonly-result-provenance.md` records that ordering. */
        const diagnostics = await lintReadonly('readonly-result-provenance-invalid.ts',);
        const messages = diagnostics.map(function diagnosticMessage(diagnostic,): string {
          return preferenceClaim(diagnostic.message,);
        },);
      /* Five reports and three offers. The count went to seven while the packaged-callable
       * gap was open, since that gap produced offers rather than reports, and back to
       * five once both halves of it landed. Caller-side property matching then added the
       * sixth, which is an offer: `narrowingPrecisionCostEffect` hands `second` to a
       * property its callee only reads, so nothing writes it.
       *
       * The shorthand provenance pair added two, and both are correct: neither
       * `packageRowShorthand` nor `packageRowExplicit` writes the row it packages, so each
       * earns the offer its sibling assertion below names. Their callers, which do write
       * through the returned holder, are contracted and report nothing.
       *
       * The container cases added seven reports, and discharging `slice` removed two of them
       * and added one offer, which is that increment's whole visible effect here. What went
       * silent went silent because the write beneath it is attributed instead: every
       * container parameter is recorded as mutated in `effect-summaries.unit.test.ts`, and
       * `containerGrowthEffect` writes nothing at all, which is why it earns the projection
       * offer the enumeration below names.
       *
       * Replacing the type-shape gate took the last one: `filteredElementWriteEffect` writes
       * through a filtered copy, that write is attributed to `rows`, and the report it used
       * to carry is now that attribution. Every container case in this fixture has made the
       * same trade, which is why the count fell three times and no offer appeared for a
       * parameter any of them writes. */
      /* Admitting the iterated element step took the twelfth: `iteratedContainerWriteEffect`
       * reached its copy through `for...of`, a spelling the escape walk did not attribute,
       * so a container that never leaves reported anyway. Its write is recorded against
       * `rows` in `effect-summaries.unit.test.ts`, so this is the same trade every other
       * container case in this fixture made. `spreadContainerWriteEffect` went the same way
       * once the spread step was ascended to the literal carrying it: `referentMutated`
       * reads `[0]` for both, so both traded a report for a write already recorded.
       *
       * `containerGrowthEffect` is what keeps that from being a blanket discharge. It
       * pushes a caller-owned row into a fresh container and is still reported at its row
       * parameter, because nothing attributes where that row ends up. */
      /* The selection pair takes it to eighteen, and reports for the same reason as the
       * composed pair through a selector instead of an extra member. The element walk asked
       * the container question only where the selector stood, so
       * `return cond ? rows.slice(0,) : [];` carried no origin while the bare
       * `return cond ? rows : [];` carried one, and value provenance and the element walk
       * disagreed about identical state. Ten further spellings were in the same state,
       * including parentheses and `as`, which is why the fix shares one definition of the
       * family rather than naming the conditional.
       *
       * Both are reports and neither is an offer, which is the load-bearing part. The offer
       * count in this fixture is unchanged, and `writesThroughSelectedContainer` is why it
       * had to be: with the returned origin missing it had nothing to substitute, so its
       * write landed on no parameter and `rows` was offerable while the callable rewrites a
       * row it holds. */
      /* Eighteen to sixteen when the returned-result discharge landed, and the two that
       * went quiet are the ones this fixture carried for it.
       * `writesThroughReturnedContainer` and `writesThroughComposedContainer` are two thirds
       * of the returned-container trio that
       * `doc/planning/prefer-readonly-return-substitution.md` named as the shape a discharge
       * would clear, on the stated condition that their write attribution survive. It does:
       * both still record `referentMutated=[0]` in `effect-summaries.unit.test.ts`, so each
       * traded a report for an attribution rather than for silence.
       *
       * Three controls keep it from being a blanket discharge, each measured rather than
       * assumed. `returnedLookupEffect` keeps its report, having no caller in the program at
       * all, and a discharge resting on callers that do not exist rests on nothing. The
       * three `ForeignBorrowed` cases in the catalog-free fixture are untouched, since a
       * container returned out of foreign-owned state is refused however completely its
       * callers enumerate. And the pinned effect list in `effect-summaries.unit.test.ts` is
       * unchanged. */
      /* Sixteen to seventeen when `returnsFromNestedCallable` was added, and it is a control
       * on which callable the discharge reasons about rather than another instance of the
       * shape. Its `rows.slice(0,)` is returned outright exactly as in
       * `returnsReceiverElements`, and it keeps its report because the `return` belongs to a
       * nested declaration while the callers being enumerated are the outer function's.
       * Those callers substitute for a result that is a callable rather than a container, so
       * none of them accounts for a write through what the inner one returns.
       *
       * Measured 2026-08-07, correcting what this comment first claimed. Removing the
       * containment check leaves this program's diagnostics byte-identical: `rows` is charged
       * either way, by a path that never consults the discharge. So it pins the shape and
       * does not isolate the check, and no program was found that does. The check stays as
       * defence in depth, and this count records a report rather than a prevented offer. */
      /* Seventeen to nineteen when the shared completeness predicate began requiring
       * closed-world callers, and the two that came back are the two the discharge had
       * cleared. Every callable here is exported, so `callersAreEnumerable` refuses each one:
       * a file's module surface is how a caller outside this program reaches it, and an
       * enumeration that cannot see those callers cannot license removing a charge on the
       * strength of what they substitute.
       *
       * The offer count falls with it, which is the sound price of the change rather than a
       * side effect. The discharge now fires only for a callable no other file can import, so
       * on this fixture it fires for none of them.
       *
       * Measured through this harness rather than through the `lint-fixture-readonly-*` mise
       * tasks. Those lint one file on its own, so `getSignatureUsage` finds no callers at all
       * and the discharge is refused before any of its own conditions are reached. A probe run
       * there reports no difference for any change to this feature, which is a property of the
       * probe and not of the code. */
      /* Nineteen to twenty-one for the positive control and its caller, then to twenty-nine
       * for the two guard programs and theirs, then to thirty-three for a third. Each of those
       * six callables charges both its parameters, which is the whole of the difference: they
       * are the positive control with one statement added, and that statement is what the
       * guard refuses to reason past. */
      /* Thirty-three to thirty-nine when the position condition widened from "returned
       * outright" to "returning is the only escape", then back to thirty-three when the six
       * two-parameter programs took a destructured parameter, which merges each pair of
       * charges into one message naming both inputs.
       *
       * Destructuring them was measured rather than assumed. The first version kept positional
       * parameters on the reasoning that a destructured binding declares a `BindingElement`
       * rather than a `VariableDeclaration`, which is what `bindingIsReassignable` keys on, so
       * destructuring might void the very tests these programs exist for. It does not: only
       * this total moved, and every discriminating assertion below is unchanged, because the
       * reassignable binding under test is a `let` inside the body and the repointed one is
       * reached by symbol rather than by declaration kind.
       *
       * Thirty-three to thirty-five for the already-readonly message pair. */
        expect(messages.length,).toBe(35,);
      /* Both declarations retain the unresolved charge under its dedicated rule.
       * `handsReadonlyNamesOnward` proves that a deeply readonly declaration receives no
       * preference diagnostic, while `handsMutableNamesOnward` proves the same charge still
       * propagates to its mutable caller. The shared message is intentional because neither
       * declaration has a proved readonly replacement at this unresolved boundary. */
        const handedNameDiagnostics = diagnostics.filter(
          function namesHandedParameter(diagnostic,): boolean {
            return diagnostic.message.includes(
              '"handedNames" is exposed to these unresolved calls',
            );
          },
        );
        expect(handedNameDiagnostics.length,).toBe(2,);
        expect(handedNameDiagnostics.every(function ownedByOpaqueRule(diagnostic,): boolean {
          return diagnostic.code
            === 'prefer-readonly-parameter-type(no-opaque-parameter-effects)';
        },),).toBe(true,);
        expect(handedNameDiagnostics.some(function ownedByPreferenceRule(diagnostic,): boolean {
          return diagnostic.code
            === 'prefer-readonly-parameter-type(prefer-readonly-parameter-types)';
        },),).toBe(false,);
      /* The condition that makes widening safe, pinned by the charge it keeps rather than by
       * an offer, which is what makes this assertion the discriminating one.
       *
       * `localBoundAndStoredElements` writes its copy into a module-level holder and then
       * returns it. Returning is the one escape whose destination this analysis follows; the
       * store is not, and no caller substitution accounts for a write made through it. So the
       * charge has to stand, and with the condition removed it silently does not: both this
       * callable and its caller lose their diagnostic entirely.
       *
       * Ten with the condition and eight without it, measured both ways. Counted rather than
       * named because the message carries no callable name, and the two that vanish are this
       * program and its caller.
       *
       * The store is a property assignment rather than a `push` on purpose. A collection call
       * charges the parameter by itself, which hid this condition completely when the first
       * version of the program used one, and made a probe of it report no difference. The
       * probe that finally showed it had to count charges rather than offers, since neither
       * callable is offered either way. */
        expect(messages.filter(function keepsStoredCharge(message,): boolean {
          return message.includes('is exposed through these unresolved collection calls: rows.slice',);
        },).length,).toBe(10,);
      /* The two guards that are now known to have a failing case, pinned by the programs that
       * fail without them rather than by a count that would not move.
       *
       * `localReassignedElements` holds its rows in a `let` pointed at the other parameter
       * before the member runs, and `localRepointedElements` does the same to a parameter
       * directly. Both are `localReceiverElements` with one statement added, and both are
       * offered read-only if their guard is removed: measured by neutralising each condition
       * in turn, four offers appear in the first case and four in the second.
       *
       * `localAssertedRepointedElements` is the third, and it works by making another guard
       * unreachable rather than by being refused itself. `bindingAssignedWithin` can only
       * answer about an `Identifier`, so an assertion around the base hides the name from it
       * and the written-endpoint check passes on a parameter that was pointed elsewhere.
       * Removing the wrapper unwrap offers four parameters that must not be offered.
       *
       * That is what the earlier guard programs could not show. They were exported, so
       * `callersAreEnumerable` refused them before any guard was consulted, and they were run
       * through a per-file task where the discharge is refused before that. Being unexported
       * with an in-file caller is what makes these three reach the code they test. */
        expect(messages.filter(function offersGuardedProgram(message,): boolean {
          return message.includes('"other" can be deeply readonly',);
        },).length,).toBe(0,);
      /* Three of the fourteen are the returned-container trio, and all three are correct as
       * things stand rather than tolerated. `returnsReceiverElements` hands back a container
       * of the caller's own rows, which the escape condition refuses to discharge because
       * nothing attributes a use that leaves the callable; its two callers inherit that.
       *
       * They are here as the shape the deferred discharge in
       * `doc/planning/prefer-readonly-return-substitution.md` would clear. When it lands,
       * these three go quiet and the write attribution asserted for them in
       * `effect-summaries.unit.test.ts` has to survive: a discharge that empties both is the
       * failure that document records being refuted once already.
       *
       * The composed pair beside them takes the count to sixteen and reports for the same
       * reason through one more member. They are here because the element walk resolved a
       * single relation before composing them, so `rows.slice(0,).toReversed()` carried no
       * origin at all while its one-member sibling carried one, and the two disagreed about
       * identical state reached through one extra call. */
      /* The eleventh is the container record's visible consequence, and it is an offer
       * rather than a report. `heldContainerRestructureEffect` builds an array around its
       * parameter and pops it, so it restructures a container this callable made and writes
       * nothing the caller owns. It was reported as mutating before the record existed, so
       * no offer could be made; now the parameter is correctly offered read-only.
       *
       * Its three siblings in the same fixture are the controls and stay silent here:
       * `heldObjectMutationEffect` and `heldArrayMutationEffect` write the caller's value
       * through the container they built, and `borrowedContainerRestructureEffect`
       * restructures a container the caller owns. `effect-summaries.unit.test.ts` pins all
       * four at summary level, which is where the distinction actually lives. */
        expect(messages.some(function offersContainerHolder(message,): boolean {
          return message.startsWith('Parameter "box" can be deeply readonly',);
        },),).toBe(true,);
      /* Both spellings of the packaging pair are offered, which is what makes the pair a
       * control for each other rather than two unrelated cases. Before the shorthand value
       * symbol reached the provenance walk the offers were also two, and the difference sat
       * where no message could show it: in whether the callers' writes were attributed.
       * `effect-summaries.unit.test.ts` pins that half. */
        expect(messages.filter(function offersPackagedRow(message,): boolean {
          return message.startsWith('Parameter "held" can be deeply readonly',);
        },).length,).toBe(2,);
      /* No offer on a computed-access receiver, which was an unsound suggestion until
       * `memberCallReceiver` gave every consumer one definition of "the receiver".
       * `computedStructureEffect` calls `values['push']('appended')`, and the
       * collection handling, the opaque boundary and the result relation each tested
       * for a property-access callee, so the call fell through all three and nothing
       * recorded the mutation. Applying the resulting offer failed with
       * `error TS7015: Element implicitly has an 'any' type because index expression
       * is not of type 'number'`. Reverting that module restores the offer here. */
        expect(messages.filter(function offersComputedReceiver(message,): boolean {
          return message.startsWith('Parameter "values" can be deeply readonly',);
        },),).toEqual([],);
      /* The computed lookup is visible, and now discharged: its report has moved to the
       * computed `set` that retains a caller-owned value, which is the argument claim
       * rather than the receiver claim. It reported nothing whatsoever before
       * `memberCallReceiver`, summary measured `mutated=[] opaque=[]`. */
        expect(messages.filter(function reportsComputedStore(message,): boolean {
          return message.includes(`facts['set']`,);
        },).length,).toBe(1,);
      /**
       Counts messages naming one call as the unresolved receiver operation.
       
       @param call - Authored member call text.
       
       @returns how many diagnostics name it.
       */
        function namingCall(call: string,): number {
          return messages.filter(function namesCall(message,): boolean {
          /* Either phrasing, since a finding whose causes are all collection members now says
           * so, and which of the two texts carries a named call is not what these counts are
           * about. */
            return message.includes(`method calls: ${call} [`,)
              || message.includes(`collection calls: ${call} [`,);
          },).length;
        }
      /* Two `Map.get` receivers left, down from five, and which two is the whole point.
       * A lookup whose result is used only in attributed positions is discharged: the
       * mutation is recorded against the receiver's parameter and the opacity report
       * that stood in for the untracked alias is gone.
       *
       * What remains reports for a reason the tracking does not remove.
       * `returnedLookupEffect` hands its result out of the callable, and until callers
       * substitute through `returnedParameterIndexes` that is a use nothing follows.
       * `boundLookupMutationEffect` keeps a report on its `facts.set`, an argument claim
       * about storing a caller-owned value, not a receiver claim. */
        expect(namingCall('facts.get',),).toBe(1,);
      /* The chained mutation keeps its report for a different reason worth separating:
       * `facts.get(key)?.add('recorded')` discharges the lookup, and the reported call
       * is the `add`, whose own result is the receiver it was called on. That is a
       * distinct relation from "a value the receiver held", nothing in the result
       * authority proves it, and it fails closed rather than being assumed. */
        expect(messages.filter(function reportsChainedAdd(message,): boolean {
          return message.includes('facts.get(key,).add',);
        },).length,).toBe(1,);
      /* Discharged, each verified by a mutation assertion in
       * `effect-summaries.unit.test.ts` rather than by the absence of a message here:
       * the destructured row, the union-valued lookup, and the array element. */
        expect(namingCall('rows.get',),).toBe(0,);
        expect(namingCall('records.get',),).toBe(0,);
        expect(namingCall('values.at',),).toBe(0,);
      /* The one that moved. `escapingLookupEffect` hands its looked-up value to
       * `JSON.stringify`, and the report now names that call rather than the lookup,
       * which is only possible because the value carries `facts` as an origin for the
       * argument analysis to find. Removing the call case from
       * `provenanceSuccessors` puts this back to naming `facts.get`. */
      /* One to three with the already-readonly message pair, which reaches an unresolved call
       * the same way. `JSON.stringify` is this fixture's stand-in for a call the rule cannot
       * inspect, so a program needing one reuses it rather than inventing a second stand-in. */
        expect(messages.filter(function namesEscape(message,): boolean {
          return message.includes('JSON.stringify',);
        },).length,).toBe(3,);
      /* No lookup receiver is offered read-only yet, not even `readOnlyLookupEffect`'s,
       * which only reads: that awaits the discharge, not the attribution. `rows` belongs
       * here too, since a discharged `at` result was the second route to the
       * contract-name defect. `row` is deliberately absent from this filter: the offers
       * this fixture still emits name a parameter called `row`, and they are pinned as a
       * set immediately below rather than folded into a claim about lookup receivers. */
      /* `rows` is matched on the writable-property form alone, which is the shape the
       * `at`-result defect produced. The projection form is a different claim and a correct
       * one: `containerGrowthEffect` copies its parameter and writes only the copy, so a
       * `readonly` projection applies and still type-checks, since `ReadonlyArray.slice`
       * returns a mutable array. Matching the name alone would have made this assertion
       * reject the first true offer the container discharge produced. */
        expect(messages.filter(function offersLookupReceiver(message,): boolean {
          return message.includes('"facts" can be deeply readonly',)
            || message.includes('"records" can be deeply readonly',);
        },),).toEqual([],);
      /* `rows` moved out of the filter above rather than being dropped from it, because the
       * two claims stopped being the same one. That filter guards the `at`-result defect,
       * which offered a receiver whose element a callable had written through, and it still
       * guards it for every lookup receiver named there.
       *
       * These three are a different cause and each is true of its own callable.
       * `returnsReceiverElements` and `returnsComposedReceiverElements` hand back a fresh
       * container of the caller's rows, and `readsReturnedContainerLength` reads a length;
       * none of the three writes anything, so a read-only parameter is an accurate
       * description of what each does.
       *
       * The judgement they rest on is the repository's own, recorded for the packaging pair
       * in this same fixture: neither `packageRowShorthand` nor `packageRowExplicit` writes
       * the row it packages, "so each earns the offer". Those return caller-owned state in a
       * fresh holder; these return it in a fresh container. Refusing one while allowing the
       * other would make the rule's answer depend on the shape of the wrapper rather than on
       * what the callable does.
       *
       * What keeps that from spreading to callables that do write is unchanged: a write
       * through any of these results is attributed to the caller's own parameter, which is
       * why `writesThroughReturnedContainer` still records `referentMutated=[0]` and is
       * still reported.
       *
       * Three to two when the shared completeness predicate began requiring closed-world
       * callers, and the two are not among the three. Each of the original three is still
       * accurate about its callable; what changed is that none can be *proven* any more. All
       * three are exported, a file's module surface is how a caller outside this program
       * reaches one, and a discharge licensed by what callers substitute cannot rest on an
       * enumeration that may not have seen them all.
       *
       * The two that remain are `localReceiverElements` and its caller, added with that
       * predicate and identical in shape to the first of the three but for being unexported.
       * They are what keeps this number sound in both directions: without them the count
       * would read zero and the feature would look dead rather than scoped, and a probe of
       * this feature reporting no difference would again be indistinguishable from a probe
       * that cannot see it.
       *
       * Whether the price is worth paying is recorded in
       * `doc/planning/prefer-readonly-return-substitution.md`; this number is what it costs.
       *
       * Two to six when the position condition widened to "returning is the only escape".
       * `localBoundElements` binds its call to a `const` before returning and
       * `localWrappedElements` wraps it in an assertion, and each is joined by its caller.
       * Both hand the caller the same value by the same route as `localReceiverElements`, so
       * refusing them was a property of how the return was spelled rather than of what the
       * callable does. */
        expect(messages.filter(function offersReturningReceiver(message,): boolean {
          return message.includes('"rows" can be deeply readonly: `',)
            && (!message.includes('parameter type uses mutable `Array`',));
        },).length,).toBe(6,);
      /* Every offer in the fixture, and there is one. Four defects surfaced here as an
       * offer and each is gone: `row` through a contract-omitted property with no lookup
       * involved, `rows` through a discharged `at` result, and
       * `methodReturnPackagedEffect` and `arrowReturnPackagedEffect` through a callable
       * they package for the callee to call. `effect-summaries.unit.test.ts` carries the
       * written-parameter assertions that keep this list from being vacuous, since a
       * fixture nothing linted would satisfy an empty one too.
       *
       * The surviving offer is a recovery rather than a defect. `second` reaches
       * `mutateOnlyNamedRow` through the `unnamed` property, whose only use in that body
       * is reading `unnamed.label`, so no write reaches `second` at all. It appeared when
       * the call edge stopped repeating an argument's whole origin set on every property
       * slot the callee reads. Reverting `effect-argument-properties.ts` empties this
       * list again, and `narrowingPrecisionCostEffect` goes back to reporting both
       * parameters written in `effect-summaries.unit.test.ts`. */
      /* The packaging pair adds two more, and both are correct rather than tolerated:
       * `packageRowShorthand` and `packageRowExplicit` return the row they are handed and
       * write nothing, so a read-only projection applies to each. They are listed here in
       * full rather than counted, so a future change that turns one of them into a defect
       * has to edit this list and say why. */
      /* The container discharge added the projection offer, and it is the first offer in this
       * fixture that exists because a member was discharged rather than despite it.
       * `containerGrowthEffect` slices its parameter and pushes onto the copy, so nothing
       * reaches the caller's array and `readonly LabelledRow[]` applies. */
        expect(messages.filter(function offersAnyParameter(message,): boolean {
          return message.includes('can be deeply readonly',);
        },)
          .toSorted(),).toEqual([
        /* The container record added this one, and it is the first offer in this fixture
         * that exists because a wrong mutation was withdrawn rather than because a call was
         * discharged. `heldContainerRestructureEffect` pops an array it built around `box`,
         * so it writes nothing the caller owns. */
            'Parameter "box" can be deeply readonly: `label` is writable.',
            'Parameter "held" can be deeply readonly: `label` is writable.',
            'Parameter "held" can be deeply readonly: `label` is writable.',
        /* Three offers from the returned-result discharge stood here, on the callables that
         * hand a container of the caller's rows back, and all three are gone. Removed rather
         * than left with a note, because a list that names offers the rule no longer makes
         * would stop being the exhaustive record it exists to be.
         *
         * They were accurate: `returnsReceiverElements` and `returnsComposedReceiverElements`
         * build a fresh container and write nothing, and `readsReturnedContainerLength` reads
         * a length through one. What changed is not the judgement but what can be proven.
         * All three are exported, so a caller outside this program can reach them, and a
         * discharge licensed by what callers substitute cannot rest on an enumeration that may
         * not have seen every caller.
         *
         * Their absence is the visible price of requiring closed-world callers, and the
         * assertion above keeps it at an explicit zero so the cost stays legible. */
        /* The positive control and its caller, and the only offers here the returned-result
         * discharge still makes. `localReceiverElements` is `returnsReceiverElements` with one
         * difference, that no other file can import it, and `readsLocalContainerLength` reads
         * a length through it. Both write nothing, so read-only describes each accurately.
         *
         * They earn their place by answering the question the six removed guard programs could
         * not. A probe reporting no difference proves nothing until the harness is known to be
         * able to show one, and these are that proof: they are offered, they were not offered
         * while every program here was exported, and they go quiet if the discharge is
         * disabled. Any future probe of this feature belongs beside them. */
            'Parameter "rows" can be deeply readonly: `[number].label` is writable.',
            'Parameter "rows" can be deeply readonly: `[number].label` is writable.',
        /* Four more once returning stopped having to be spelled outright: a call bound to a
         * `const` and one wrapped in an assertion, each with its caller. Listed rather than
         * counted for the same reason as the pair above, so a change that turns any of them
         * into a defect has to edit this list and say why. */
            'Parameter "rows" can be deeply readonly: `[number].label` is writable.',
            'Parameter "rows" can be deeply readonly: `[number].label` is writable.',
            'Parameter "rows" can be deeply readonly: `[number].label` is writable.',
            'Parameter "rows" can be deeply readonly: `[number].label` is writable.',
            'Parameter "rows" can be deeply readonly: `[number].label` is writable; parameter type uses mutable `Array`.',
            'Parameter "second" can be deeply readonly: `label` is writable.',
          ],);
      },
    },),
    it({
      name: 'still offers a written parameter for two measured call-edge shapes',
      fn: async () => {
      /* A ledger, not an approval. Every offer counted here except three names a
       * parameter some callee writes, and each is tracked as its own task with the
       * machinery it needs. Pinning them keeps the count from drifting quietly in either
       * direction: a fix has to lower it deliberately, and a regression raises it.
       *
       * Sound and staying: `first`, which `mutateSecond` only reads; `value`, which
       * `store` only stores; and one `row`, which `Reader.use` only reads.
       *
       * Unsound, by parameter and cause:
       * `row` in `polymorphicEffect`, where static resolution finds a reading base method
       * and the reachable override writes.
       *
       * Two shapes from the same review do not reproduce and have no offer here: a mixed
       * method-and-direct effect, which the packaged-callable scan covers, and a getter
       * body writing a captured parameter, which the caller's own direct-write scan
       * catches because the accessor body sits in the caller's scope. */
        const diagnostics = await lintReadonly('readonly-call-edge-invalid.ts',);
        const messages = diagnostics.map(function diagnosticMessage(diagnostic,): string {
          return preferenceClaim(diagnostic.message,);
        },);
      /**
       Counts offers naming one parameter.
       
       @param parameterName - Authored parameter name.
       
       @returns how many offers name it.
       */
        function offersFor(parameterName: string,): number {
          return messages.filter(function offersParameter(message,): boolean {
            return message.startsWith(`Parameter "${parameterName}" can be deeply readonly`,);
          },).length;
        }
      /* Two, down from four, and both losses come from classifying an assignment that
       * hands a parameter onward. `store` assigns `value` into `slot.value`, so `value`
       * is no longer offered, and `setterPairEffect` calls `store`, so its `row` takes
       * the same opacity through the call edge.
       *
       * One of those two was a documented unsound offer. `setterPairEffect` reaches a
       * setter that writes the assigned value, and it is withheld now for a reason that
       * does not mention setters at all: the value left the callable, and where it went
       * settles the question without resolving what happens there.
       *
       * The other is a precision loss and is tracked rather than accepted quietly.
       * `store` puts one parameter inside another, and both belong to the caller, which
       * already held each of them. Rearranging a graph the caller can already reach
       * grants no capability it lacked, the way returning a piece of a parameter does
       * not, so `value` deserves its offer and the classification cannot yet tell that
       * target apart from one the caller cannot reach. */
        expect(messages.filter(function isOffer(message,): boolean {
          return message.includes('can be deeply readonly',);
        },).length,).toBe(2,);
        expect(offersFor('row',),).toBe(1,);
        expect(offersFor('first',),).toBe(1,);
        expect(offersFor('value',),).toBe(0,);
      /* The seven shapes already fixed, kept as the controls that stop this case from
       * passing on a fixture the rule never reached. An explicit `this` parameter shifting
       * every later formal index. A parameter named only by a shorthand inside an accessor
       * body. A rest formal collecting a later actual, and one spread actual covering two
       * formals, both breaking the relation between argument position and formal index. A
       * default aliasing an earlier formal, from both sides. A write reached from a
       * parameter initializer rather than a body. Reverting any of them raises the counts
       * above, and reverting the default-alias work also restores a `primary` offer. */
        expect(offersFor('handler',),).toBe(0,);
        expect(offersFor('primary',),).toBe(0,);
      },
    },),
    it({
      name: 'withholds an offer from a stored parameter without asking the reader anything',
      fn: async () => {
      /* A store withholds the offer and says nothing, the way a mutation always has. The
       * classification that records it routes through opacity, which is right about the
       * decision and was wrong about the channel: opacity carries a message that names its
       * causes as calls, says the rule could not inspect them, and lists four remedies
       * addressed to an unresolved implementation. A reader who retains a constructor
       * argument can act on none of that, and a sweep measured thirty-two locations in
       * this repository that started saying it while no offer anywhere changed. */
        const diagnostics = await lintReadonly('readonly-structural-store-invalid.ts',);
        const messages = diagnostics.map(function diagnosticMessage(diagnostic,): string {
          return preferenceClaim(diagnostic.message,);
        },);
      /* The store provenance never reaches a reader. It stays recorded on the summary,
       * because every analysis consumer still needs it, and this is the boundary it must
       * not cross. */
        expect(messages.filter(function namesStore(message,): boolean {
          return message.includes('stored into',);
        },).length,).toBe(0,);
      /* Withheld, not merely silent. Seventeen offers, and every one belongs to a shape
       * that keeps nothing beyond the callable: the two nested-store controls, an
       * assignment into a parameter and into an own local, a counter, three readers, a
       * projection, a fresh aggregate, an iteration that retains only a primitive, an
       * iteration that declares its own binding, two callables that hand back what the
       * caller already holds, two that allocate their own, and the two stores of an
       * allocated value through a call. Were retention to stop withholding, ten more would
       * join them.
       *
       * Moved from thirteen by the deferred retention, in two steps and by measurement
       * each time rather than by adjusting the number to fit. Two offers left, whose
       * shapes were falsified: `storeThroughOwnedCall` and `storeIterationThroughCall`.
       * Six arrived with the controls those needed, `firstRow` having already been here:
       * `freshRow`, `storeFreshThroughOwnedCall`, `rowsOf`, `freshRows`,
       * `storeIterationThroughFreshCall`, and the allocating callable each of the last two
       * depends on. Every arrival belongs: a callable that hands back a piece of its own
       * parameter grants the caller no capability it lacked, and one that allocates shares
       * no identity with its argument. */
      /* Nineteen. Three are withheld for a retention the origin walk cannot see on its
       * own: through a call, through an iteration over a call result, and through a local
       * holding one. The rest keep their offers, including three that store a call result
       * and must: two whose callee allocates, and one that stores only a primitive read
       * off a returned value. That last arrival is the control for a gate the retention
       * path was missing, and it belongs among the offers rather than beside them. */
      /* Twenty. Six closure shapes arrived and five of them withhold: three that store a
       * capturing closure past the callable, one that hands its parameter to a callable
       * doing so, and one whose locally invoked closure writes. The twentieth offer is
       * `invokeAssignedLocalClosureWriting`, and it is the odd one: the same write as the
       * shape beside it, reached through an assignment to an already-declared local rather
       * than through an initializer, recording no effect at all.
       *
       * It belongs among the offers only in the sense that the rule currently makes it.
       * Self-limiting rather than false, since the write is on `config` directly and the
       * applied annotation stops type-checking, so no falsification rides on it. Task #65
       * holds the selection question, and this count is what its fix moves. */
      /* Twenty-two once the capture channel landed, and the two arrivals are its controls
       * rather than its subjects. `handFreshCaptureToRetainer` hands a retaining callee a
       * closure that allocates its own row, and `handCaptureToReader` hands a capturing
       * closure to a callee that only invokes it. Both keep their offers, which is what
       * separates attributing what a callable captured from refusing every callable handed to
       * an owned callee. */
      /* Thirty-nine once packaged parameter defaults stopped being attributed unconditionally,
       * and the single arrival is the subject rather than a side effect. Six shapes joined and
       * five of them withhold: the invoked default, the initializer expression that writes on
       * entry, and the three that hand the default outward through a store, a retaining callee
       * and a return. `unreachedDefault` is the arrival, and its whole claim is that a closure
       * nothing invokes and nothing keeps performs no write. Asserted by name below, because a
       * count cannot tell which offer moved. */
      /* Fifty-one once a defaulted producer's invoked result could be stored. Three shapes joined and
       * two of them keep their offers, so the net arrival is two rather than three. Asserted by name
       * in the defaulted-producer group below, for the reason that group records. */
      /* Fifty-five once a concise arrow body earned its returned fact. Five shapes joined and four of
       * them keep an offer: the two helpers, whose own parameters are read or handed back rather than
       * stored, and the two controls. The subject contributes none, which is the whole point of it. */
      /* Fifty-seven once a capture handed to a callback parameter was charged. Four shapes joined and
       * the two controls keep their offers, so the net arrival is two and both subjects contribute
       * none. */
      /* Sixty once a default naming an ordinary function resolved to it. Four shapes joined, the two
       * named helpers keep offers on their own row parameters, the control keeps its configuration,
       * and the subject contributes none. */
      /* Sixty-one once the reach walk followed a conditional callee. Two shapes joined, the control
       * keeps its offer and the subject contributes none. */
      /* Sixty-two once a call result handed to a callback parameter was charged. Two shapes joined, the
       * control keeps its offer and the subject contributes none. */
      /* Sixty-three once the accessor walk recognised the other ways source spells a property read.
       * Four shapes joined, three subjects contributing none and one control keeping its offer. */
      /* Sixty-four once a declared void result stopped answering for a slot. Four shapes joined and
       * exactly one keeps an offer, the control whose callee is a readable declaration. The subject
       * contributes none, the forwarder's own producer is a callable and so was never offered, and the
       * reporter's own parameter is a string. */
      /* Sixty-five once a candidate list stopped standing for a closed set. Four shapes joined and
       * exactly one keeps an offer, the control whose candidate and whose declared result are both
       * leaves. The subject contributes none, and the two forwarders' own producers are callables. */
      /* Sixty-eight once the outward handoffs asked about captures. Six shapes joined, three subjects
       * contributing none and three controls keeping their offers, one pair per handoff syntax. */
      /* Sixty-nine once the value walk followed a destructuring source and a property read. Three
       * shapes joined and exactly one keeps an offer, the control whose held closure allocates. */
      /* Seventy once a tagged template counted as an invocation. Two shapes joined, the control keeping
       * its offer and the subject contributing none. */
      /* Seventy-one once a construction counted as an invocation too. Two shapes joined, the control
       * keeping its offer and the subject contributing none. */
      /* Seventy-two once a decorator counted as an invocation and stopped being gated on the member it
       * decorates. Two shapes joined, the control keeping its offer and the subject contributing none. */
      /* Seventy-five once an awaited completion was judged by what it resolves to. Four shapes joined and
       * three keep offers: the precision subject, whose closure resolves to a label, and both async readers,
       * whose own parameters are read rather than handed out. The control resolving to the caller's row
       * contributes none, which is what shows the promise was looked through and the question was not. */
      /* Seventy-six once the reach walk followed a callee binding filled by assignment. Two shapes joined,
       * the control keeping its offer and the subject contributing none. */
        expect(messages.filter(function isOffer(message,): boolean {
          return message.includes('can be deeply readonly',);
        },).length,).toBe(76,);
      /* The assignment pair. Two earlier placements of the same question read identically on both halves,
       * one by breaking an unrelated invariant and one by changing nothing, so this pair is what tells the
       * working placement from either. */
        expect(messages.filter(function namesAssigned(message,): boolean {
          return message.includes('"assignedGotten"',)
            && message.includes('can be deeply readonly',);
        },),).toEqual([],);
        expect(messages.filter(function namesNeitherAssigned(message,): boolean {
          return message.includes('"neitherAssignedGotten"',);
        },),).toEqual([
          'Parameter "neitherAssignedGotten" can be deeply readonly: `row` is writable; `rows` is writable.',
        ],);
      /* The promise pair, and this one is the reverse of every other pair here: the subject must be OFFERED
       * because the fix recovers precision, and the control must be withheld. A fix that looked through the
       * question rather than the wrapper would offer to both. */
        expect(messages.filter(function namesAsyncLabel(message,): boolean {
          return message.includes('"asyncLabelGotten"',);
        },),).toEqual([
          'Parameter "asyncLabelGotten" can be deeply readonly: `row` is writable; `rows` is writable.',
        ],);
        expect(messages.filter(function namesAsyncRow(message,): boolean {
          return message.includes('"asyncRowGotten"',)
            && message.includes('can be deeply readonly',);
        },),).toEqual([],);
      /* The decorator pair. Two changes were needed together and each was a no-op alone, so this pair is
       * the only thing that distinguishes having both from having either. */
        expect(messages.filter(function namesDecorated(message,): boolean {
          return message.includes('"decoratedGotten"',)
            && message.includes('can be deeply readonly',);
        },),).toEqual([],);
        expect(messages.filter(function namesNeitherDecorated(message,): boolean {
          return message.includes('"neitherDecoratedGotten"',);
        },),).toEqual([
          'Parameter "neitherDecoratedGotten" can be deeply readonly: `row` is writable; `rows` is writable.',
        ],);
      /* The construction pair, asserted as agreement for the same reason the tag pair is. */
        expect(messages.filter(function namesBodyStored(message,): boolean {
          return message.includes('"bodyStoredGotten"',)
            && message.includes('can be deeply readonly',);
        },),).toEqual([],);
        expect(messages.filter(function namesNeitherBodyStored(message,): boolean {
          return message.includes('"neitherBodyStoredGotten"',);
        },),).toEqual([
          'Parameter "neitherBodyStoredGotten" can be deeply readonly: `row` is writable; `rows` is writable.',
        ],);
      /* The tag pair, asserted as agreement. Seeing a tag as an invocation must charge the one storing
       * caller state and spare the one storing a fresh row, and only the pair separates that from a
       * walk that charges every tag or none. */
        expect(messages.filter(function namesTagStored(message,): boolean {
          return message.includes('"tagStoredGotten"',)
            && message.includes('can be deeply readonly',);
        },),).toEqual([],);
        expect(messages.filter(function namesNeitherTagStored(message,): boolean {
          return message.includes('"neitherTagStoredGotten"',);
        },),).toEqual([
          'Parameter "neitherTagStoredGotten" can be deeply readonly: `row` is writable; `rows` is writable.',
        ],);
      /* Both held-callable subjects charged and the shared control offered. Asserted as agreement,
       * because following a receiver too eagerly would charge all three and following nothing charges
       * none, and only the pairing separates those. */
        expect([
          'patternHeldGotten',
          'readHeldGotten',
        ].flatMap(function heldSubjectOffers(name,): readonly string[] {
          return messages.filter(function namesHeldSubject(message,): boolean {
            return message.includes(`"${name}"`,)
              && message.includes('can be deeply readonly',);
          },);
        },),).toEqual([],);
        expect(messages.filter(function namesNeitherHeld(message,): boolean {
          return message.includes('"neitherHeldGotten"',);
        },),).toEqual([
          'Parameter "neitherHeldGotten" can be deeply readonly: `row` is writable; `rows` is writable.',
        ],);
      /* One pair per handoff syntax, asserted as agreement across all six at once. The three subjects
       * must draw no offer and the three controls must each draw exactly one, and only the pairing tells
       * a working channel from one that charges every handed value. */
        expect([
          'constructedGotten',
          'yieldedGotten',
          'thrownGotten',
        ].flatMap(function subjectOffers(name,): readonly string[] {
          return messages.filter(function namesSubject(message,): boolean {
            return message.includes(`"${name}"`,)
              && message.includes('can be deeply readonly',);
          },);
        },),).toEqual([],);
        expect([
          'neitherConstructedGotten',
          'neitherYieldedGotten',
          'neitherThrownGotten',
        ].map(function controlOffers(name,): number {
          return messages.filter(function namesControl(message,): boolean {
            return message.includes(`"${name}"`,)
              && message.includes('can be deeply readonly',);
          },).length;
        },),).toEqual([1, 1, 1,],);
      /* The candidate-list subject is charged and its control is not. Asserted as agreement for the same
       * reason the void pair is: joining the two answers unconditionally would charge both if the
       * declared result decided alone, and trusting the candidate list would offer to both. */
        expect(messages.filter(function namesDefaultedGotten(message,): boolean {
          return message.includes('"defaultedGotten"',);
        },).filter(function isOffer(message,): boolean {
          return message.includes('can be deeply readonly',);
        },),).toEqual([],);
        expect(messages.filter(function namesLabelledGotten(message,): boolean {
          return message.includes('"labelledGotten"',);
        },),).toEqual([
          'Parameter "labelledGotten" can be deeply readonly: `row` is writable; `rows` is writable.',
        ],);
      /* The void-slot subject is charged and its control is not, which is the whole scope of that fix.
       * Asserted as agreement between the two rather than as a count, because a fallback that
       * distrusted every void result would withhold from both and a fallback that trusted every slot
       * would offer to both, and only the pair tells those apart. */
        expect(messages.filter(function namesVoidGotten(message,): boolean {
          return message.includes('"voidGotten"',);
        },).filter(function isOffer(message,): boolean {
          return message.includes('can be deeply readonly',);
        },),).toEqual([],);
        expect(messages.filter(function namesReportedGotten(message,): boolean {
          return message.includes('"reportedGotten"',);
        },),).toEqual([
          'Parameter "reportedGotten" can be deeply readonly: `row` is writable; `rows` is writable.',
        ],);
      /* Withheld and silent, asserted on every diagnostic rather than on offers alone. A
       * caller that says nothing and a caller that reports argument opacity naming the
       * retaining callee both lose their offer and both read `[0]` from the summary, so
       * neither the offer count nor the opaque set can tell the intended outcome from the
       * failure this channel can produce: captures arriving as call-caused. Unique parameter
       * names are what make the count per callable rather than per file. */
        expect(messages.filter(function namesRetained(message,): boolean {
          return message.includes('"retained"',);
        },).length,).toBe(0,);
        expect(messages.filter(function namesNamedRetained(message,): boolean {
          return message.includes('"namedRetained"',);
        },).length,).toBe(0,);
      /* And the controls, which must speak exactly once each and say only that an offer
       * stands. A control that fell silent would look identical to the shapes above. */
        expect(messages.filter(function namesUntouched(message,): boolean {
          return message.includes('"untouched"',);
        },),).toEqual([
          'Parameter "untouched" can be deeply readonly: `row` is writable; `rows` is writable.',
        ],);
        expect(messages.filter(function namesInspected(message,): boolean {
          return message.includes('"inspected"',);
        },),).toEqual([
          'Parameter "inspected" can be deeply readonly: `row` is writable; `rows` is writable.',
        ],);
      /* The forwarded capture speaks, and that is right rather than a leak. Its callee could
       * not account for the callable and named the boundary it handed it to, so the caller
       * inherits a cause a reader can act on. The stored captures above stay silent because
       * their cause is a store, which a reader cannot act on. One channel, two messages,
       * decided by the callee's own provenance rather than by a second rule here. */
        expect(messages.filter(function namesForwarded(message,): boolean {
          return message.includes('"forwarded"',);
        },).length,).toBe(1,);
      /* The store-side alias, withheld and silent for the same reason the handed one is: its
       * cause is a store, which a reader cannot act on. Its control keeps the offer that
       * carries the count above from twenty-two to twenty-three. */
        expect(messages.filter(function namesAliased(message,): boolean {
          return message.includes('"aliased"',);
        },).length,).toBe(0,);
        expect(messages.filter(function namesUnnamed(message,): boolean {
          return message.includes('"unnamed"',);
        },),).toEqual([
          'Parameter "unnamed" can be deeply readonly: `row` is writable; `rows` is writable.',
        ],);
      /* The returned closure, withheld and silent. Its cause is the same retention the stores
       * above carry, because there is no call to name and no boundary a reader could inspect,
       * so the offer is simply not made. */
        expect(messages.filter(function namesProduced(message,): boolean {
          return message.includes('"produced"',);
        },).length,).toBe(0,);
      /* Its two controls, both speaking exactly once and both saying an offer stands. The
       * second is the accepted policy working rather than a control on this change: a direct
       * return of caller state is tracked, callers substitute through it, and it keeps its
       * offer. Without it this would read as a rule against returning caller state, which is
       * what the decision explicitly permits. */
        expect(messages.filter(function namesUnreturned(message,): boolean {
          return message.includes('"unreturned"',);
        },),).toEqual([
          'Parameter "unreturned" can be deeply readonly: `row` is writable; `rows` is writable.',
        ],);
        expect(messages.filter(function namesDirect(message,): boolean {
          return message.includes('"direct"',);
        },),).toEqual([
          'Parameter "direct" can be deeply readonly: `row` is writable; `rows` is writable.',
        ],);
      /* The transitive shapes on all three paths, withheld and silent, because each cause is
       * an escape rather than a call a reader could inspect. */
        expect(messages.filter(function namesRelayed(message,): boolean {
          return message.includes('"relayedThrough"',)
            || message.includes('"relayedArgument"',)
            || message.includes('"relayedReturn"',)
            || message.includes('"recursed"',);
        },).length,).toBe(0,);
      /* And the control carrying the count from twenty-five to twenty-six. */
        expect(messages.filter(function namesRelayedFresh(message,): boolean {
          return message.includes('"relayedFresh"',);
        },),).toEqual([
          'Parameter "relayedFresh" can be deeply readonly: `row` is writable; `rows` is writable.',
        ],);
      /* The conditional, the coalescence and the container, all withheld and all silent. */
        expect(messages.filter(function namesPossibleValues(message,): boolean {
          return message.includes('"chosen"',)
            || message.includes('"coalesced"',)
            || message.includes('"preferred"',)
            || message.includes('"contained"',)
            || message.includes('"leftBiased"',)
            || message.includes('"handedToNew"',)
            || message.includes('"yieldedOut"',)
            || message.includes('"awaitedThrough"',)
            || message.includes('"pushedResult"',)
            || message.includes('"nestedResult"',)
            || message.includes('"keptRow"',)
            || message.includes('"patternBound"',)
            || message.includes('"logicalBound"',)
            || message.includes('"defaultBound"',)
            || message.includes('"defaulted"',)
            || message.includes('"conditionalTarget"',)
            || message.includes('"neverReached"',)
            || message.includes('"neverReturned"',)
            || message.includes('"actuallyReached"',)
            || message.includes('"interpolated"',)
            || message.includes('"iteratedOut"',)
            || message.includes('"viaLocalFunction"',)
            || message.includes('"viaArrowProperty"',)
            || message.includes('"thrownOut"',)
            || message.includes('"defaultReached"',)
            || message.includes('"throughThis"',);
        },).length,).toBe(0,);
      /* The accessor group, and the offer that carries the count from forty-eight to forty-nine. The
       * reach walk follows calls, and a property read is not one, so a closure naming a local whose
       * getter reaches caller state answered empty. `gottenThrough` is the subject and `freshGotten`
       * is the control: collecting every callable a literal declares must not report a literal built
       * from nothing the caller handed in. */
        expect(messages.filter(function namesFreshGetter(message,): boolean {
          return message.includes('"freshGotten"',);
        },),).toEqual([
          'Parameter "freshGotten" can be deeply readonly: `row` is writable; `rows` is writable.',
        ],);
        expect(messages.filter(function namesGottenThrough(message,): boolean {
          return message.includes('"gottenThrough"',)
            && message.includes('can be deeply readonly',);
        },).length,).toBe(0,);
      /* The defaulted-producer group, whose three shapes carry the count from forty-nine to
       * fifty-one: two of them keep an offer and the subject loses one, so the net arrival is two.
       * A store of an invoked result withholds when the callable was handed in, and a defaulted
       * callable is selected when it is stored or handed onward, and neither covered a default that
       * is invoked and whose result is then stored.
       *
       * `producedDefault` is the subject. `untouchedByProducer` controls that selecting a default
       * must not charge a configuration the default never names, and `countedByProducer` controls
       * that the charge belongs to the store rather than to the invocation: the capture does reach
       * the result there, and a primitive read off it lets nothing out. Asserted by name because all
       * three record the same producer effects, so a count cannot tell which one moved. */
        expect(messages.filter(function namesProducedDefault(message,): boolean {
          return message.includes('"producedDefault"',)
            && message.includes('can be deeply readonly',);
        },).length,).toBe(0,);
        expect(messages.filter(function namesUntouchedByProducer(message,): boolean {
          return message.includes('"untouchedByProducer"',);
        },),).toEqual([
          'Parameter "untouchedByProducer" can be deeply readonly: `row` is writable; `rows` is writable.',
        ],);
        expect(messages.filter(function namesCountedByProducer(message,): boolean {
          return message.includes('"countedByProducer"',);
        },),).toEqual([
          'Parameter "countedByProducer" can be deeply readonly: `row` is writable; `rows` is writable.',
        ],);
      /* The concise-body group. General rather than default-specific: the direct scan recorded a
       * returned effect under `isReturnStatement` alone, and a concise arrow body is the callable's
       * own body expression with no return statement anywhere, so such a callable recorded an empty
       * returned set and every caller storing its result was offered. Measured at top level with no
       * default in sight, which is what established the body form as the cause.
       *
       * `concisePassedStored` is the subject. `conciseFreshStored` controls that a concise body
       * handing back something freshly allocated claims no origin, and `conciseCountedStored`
       * controls that the charge belongs to the store rather than to the call. */
        expect(messages.filter(function namesConcisePassedStored(message,): boolean {
          return message.includes('"concisePassedStored"',)
            && message.includes('can be deeply readonly',);
        },).length,).toBe(0,);
        expect(messages.filter(function namesConciseFreshStored(message,): boolean {
          return message.includes('"conciseFreshStored"',);
        },),).toEqual([
          'Parameter "conciseFreshStored" can be deeply readonly: `row` is writable; `rows` is writable.',
        ],);
        expect(messages.filter(function namesConciseCountedStored(message,): boolean {
          return message.includes('"conciseCountedStored"',);
        },),).toEqual([
          'Parameter "conciseCountedStored" can be deeply readonly: `row` is writable; `rows` is writable.',
        ],);
      /* The callback-parameter capture group. A relation names which caller-owned value reached which
       * callback argument position, and the caller can reconstruct that because the caller chose the
       * value. A closure written inside the callee is not the caller's choice, so the relation held
       * nothing while the same closure handed to an unresolvable member recorded opacity: two paths,
       * one relation, disagreeing.
       *
       * `handedToCallback` and `siblingHandedToCallback` are the subjects, the second reaching its
       * capture only through a sibling. `freshHandedToCallback` controls that a closure over nothing
       * the caller owns keeps its offer.
       *
       * `forwardedToCallback` is the control that decides whether the gate belongs on this branch at
       * all: the deferral #75 settled rests on a parameter-derived non-callable keeping its relation
       * and gaining no opacity. If it ever loses its offer, every callback-forwarding shape in the
       * workspace has silently become a withholding one, so it is asserted exactly rather than by
       * count. */
        expect(messages.filter(function namesHandedToCallback(message,): boolean {
          return (message.includes('"handedToCallback"',)
            || message.includes('"siblingHandedToCallback"',))
            && message.includes('can be deeply readonly',);
        },).length,).toBe(0,);
        expect(messages.filter(function namesFreshHandedToCallback(message,): boolean {
          return message.includes('"freshHandedToCallback"',);
        },),).toEqual([
          'Parameter "freshHandedToCallback" can be deeply readonly: `row` is writable; `rows` is writable.',
        ],);
        expect(messages.filter(function namesForwardedToCallback(message,): boolean {
          return message.includes('"forwardedToCallback"',);
        },),).toEqual([
          'Parameter "forwardedToCallback" can be deeply readonly: `row` is writable; `rows` is writable.',
        ],);
      /* The named-default group. The value walk hands back the identifier a default names, and an
       * identifier is not a callable declaration, so the syntax filter that answered for an inline
       * default answered nothing for a named one and built no call edge. The same callee reached
       * directly or through a local alias charged correctly, which located the defect in resolution
       * rather than in substitution. Every candidate value is now resolved rather than tested.
       *
       * `namedDefaultStored` is the subject and `namedFreshStored` controls that resolving a name must
       * not charge a configuration the named callee never hands back. */
        expect(messages.filter(function namesNamedDefaultStored(message,): boolean {
          return message.includes('"namedDefaultStored"',)
            && message.includes('can be deeply readonly',);
        },).length,).toBe(0,);
        expect(messages.filter(function namesNamedFreshStored(message,): boolean {
          return message.includes('"namedFreshStored"',);
        },),).toEqual([
          'Parameter "namedFreshStored" can be deeply readonly: `row` is writable; `rows` is writable.',
        ],);
      /* The source-order pair, which adds no offer and is asserted for agreement rather than for a
       * count. Both defaults resolve to the same two callables and differ only in which branch is
       * written first, so both must answer the same way. One call site carried two edges keyed alike
       * and the consumer built its lookup with `new Map(entries)`, which keeps the last pair, so the
       * half whose returning branch was discarded offered its caller's configuration while its twin
       * withheld. An answer that flips with source order is the diagnosis, and it only became
       * reachable once the shared resolver reached a default naming an ordinary function. */
        expect(messages.filter(function namesOrderPassFirst(message,): boolean {
          return message.includes('"orderPassFirst"',)
            && message.includes('can be deeply readonly',);
        },).length,).toBe(
          messages.filter(function namesOrderAllocFirst(message,): boolean {
            return message.includes('"orderAllocFirst"',)
              && message.includes('can be deeply readonly',);
          },).length,
        );
        expect(messages.filter(function namesEitherOrder(message,): boolean {
          return (message.includes('"orderPassFirst"',)
            || message.includes('"orderAllocFirst"',))
            && message.includes('can be deeply readonly',);
        },).length,).toBe(0,);
      /* The conditional-callee group, and the third site the shared resolver had to reach. The reach
       * walk resolved a callee with the narrow resolver alone, which answers for one declaration and
       * nothing for a conditional, so a handed closure invoking `(pick ? reveal : fresh)()` reached
       * nothing while the same closure invoking one named callee charged correctly. The handed closure
       * names neither the configuration nor the body reading it, which leaves the reach walk as the
       * only channel that can answer.
       *
       * `conditionalReached` is the subject and `neitherReached` controls that following every branch
       * must not report one built from nothing the caller handed in. */
        expect(messages.filter(function namesConditionalReached(message,): boolean {
          return message.includes('"conditionalReached"',)
            && message.includes('can be deeply readonly',);
        },).length,).toBe(0,);
        expect(messages.filter(function namesNeitherReached(message,): boolean {
          return message.includes('"neitherReached"',);
        },),).toEqual([
          'Parameter "neitherReached" can be deeply readonly: `row` is writable; `rows` is writable.',
        ],);
      /* The callback call-result group, and the second thing the callback branch returned before doing.
       * A relation cannot see through an inner call result, because a callee's summary does not exist
       * while its callers are walked, so the retention every argument carries was never recorded here
       * and this was indistinguishable from a control handing over a freshly allocated row, while the
       * same result handed to an unresolvable member recorded opacity.
       *
       * `resultHandedToCallback` is the subject and `freshResultHanded` controls that a retention per
       * argument must leave an offer standing when nothing the caller owns comes back out. The deferral
       * control `forwardedToCallback` asserted above covers this fix too: it forwards a
       * parameter-derived row rather than a call result and must keep its relation. */
        expect(messages.filter(function namesResultHandedToCallback(message,): boolean {
          return message.includes('"resultHandedToCallback"',)
            && message.includes('can be deeply readonly',);
        },).length,).toBe(0,);
        expect(messages.filter(function namesFreshResultHanded(message,): boolean {
          return message.includes('"freshResultHanded"',);
        },),).toEqual([
          'Parameter "freshResultHanded" can be deeply readonly: `row` is writable; `rows` is writable.',
        ],);
      /* The accessor-forms group. Only plain property access was recognised, and three other spellings
       * run a getter just as surely: element access, a destructuring pattern, and a getter declared by a
       * class declaration reached through a construction. The last needed two hops, since a class
       * declaration was excluded beside a class expression and the receiver resolves to `new Holder()`
       * rather than to the class.
       *
       * `neitherClassGotten` controls the construction hop: following a construction to its class must
       * not report a class whose getter hands back nothing the caller owns. */
        expect(messages.filter(function namesAccessorSubjects(message,): boolean {
          return (message.includes('"elementGotten"',)
            || message.includes('"patternGotten"',)
            || message.includes('"classGotten"',))
            && message.includes('can be deeply readonly',);
        },).length,).toBe(0,);
        expect(messages.filter(function namesNeitherClassGotten(message,): boolean {
          return message.includes('"neitherClassGotten"',);
        },),).toEqual([
          'Parameter "neitherClassGotten" can be deeply readonly: `row` is writable; `rows` is writable.',
        ],);
      /* The laundered-completion group, and the three offers that carry the count from forty-five to
       * forty-eight. The gate asks whether a packaged closure's completion can carry mutable state,
       * and a declared type can lie about that in two ways. `erasedThrough` holds a local annotated
       * `() => void` whose callable hands back a row, and `assertedThrough` asserts a row to a
       * string. Both really produce caller state. A completion is now followed to its callable
       * instead of read off an annotation, and an assertion is stripped before judging.
       *
       * `boundThrough` is the third subject and a different miss: the inspection took arguments
       * alone, so a capturing closure reaching an uninspectable implementation as the **receiver**
       * was recorded by nothing.
       *
       * Two of the three arrivals are the precision controls that decide how far following goes.
       * `countedThrough` hands back a count through an owned callee, whose body is followed and says
       * so. `stringifiedThrough` hands back a string through a library callee, whose declared return
       * type stands, because an external declaration's return type is what this rule trusts
       * everywhere else and distrusting it here would withhold on every primitive handed back
       * through a library call. `countedRows` is the third and is incidental: the counting callee's
       * own parameter, offered because a `readonly Row[]` still has mutable elements. */
        expect(messages.filter(function namesLaunderedControls(message,): boolean {
          return message.includes('"countedThrough"',)
            || message.includes('"stringifiedThrough"',)
            || message.includes('"countedRows"',);
        },).length,).toBe(3,);
        expect(messages.filter(function namesLaunderedSubjects(message,): boolean {
          return (message.includes('"erasedThrough"',)
            || message.includes('"assertedThrough"',)
            || message.includes('"boundThrough"',))
            && message.includes('can be deeply readonly',);
        },).length,).toBe(0,);
      /* The default-callback group, and the three offers that carry the count from forty-two to
       * forty-five. A callback relation defers to the caller, because the caller supplies the
       * callback and knows what it does, which is what task #75 settled. A default is supplied by
       * the callee, so there is nobody to defer to and deferring lost the write. `defaultTarget` and
       * `patternTarget` are the subjects, and the first is falsified.
       *
       * Three arrivals rather than two, and the third is worth naming because it is not a control at
       * all: `readRow` is the reading default's own arrow parameter, offered on its own merits
       * because that arrow only reads what it receives. It is here because the rule reports every
       * callable, nested ones included.
       *
       * `suppliedTarget` is the control that keeps #75 settled: the caller supplies the callback,
       * there is somebody to defer to, and the offer stands. `defaultRead` is the precision control:
       * a default that only reads what it receives grants the caller nothing. */
        expect(messages.filter(function namesDefaultCallbackOffers(message,): boolean {
          return message.includes('"suppliedTarget"',)
            || message.includes('"defaultRead"',)
            || message.includes('"readRow"',);
        },).length,).toBe(3,);
        expect(messages.filter(function namesDefaultCallbackSubjects(message,): boolean {
          return (message.includes('"defaultTarget"',)
            || message.includes('"patternTarget"',))
            && message.includes('can be deeply readonly',);
        },).length,).toBe(0,);
      /* The unresolved-boundary group, and the two offers that carry the count from forty to
       * forty-two. Captures lived on owned call edges only, so a capturing closure handed to a call
       * with no owned edge was recorded by nothing, and a possibly-overridden method is treated as
       * unresolved on purpose, which made every instance method that keeps a callback an instance of
       * it. `registeredCapture` is falsified.
       *
       * The two arrivals are the precision the gate exists to keep, and they matter more than the
       * subject. Scoping captures to owned edges was protecting exactly them, and asking what the
       * closure hands back protects them without the scoping: a closure completing only in a string,
       * and one completing in nothing at all, expose nothing whatever an uninspectable callee does
       * with them, because writes the closure performs are charged separately.
       *
       * `timedRow` is the accepted loss, pinned so it stays visible rather than forgotten.
       * `setTimeout` discards what it invokes, so nothing escapes there, and no local property of
       * the call expression can establish that. */
        expect(messages.filter(function namesUnresolvedCaptureOffers(message,): boolean {
          return message.includes('"mappedPrimitive"',)
            || message.includes('"countedVoid"',);
        },).length,).toBe(2,);
      /* Both withheld, and both speaking, which is right for this channel rather than a leak. Their
       * cause is a call, so there is a boundary the reader can inspect, and that is exactly the
       * distinction the store channel is silent for: a reader can do nothing about a store and
       * something about an uninspectable call. */
        expect(messages.filter(function offersRegisteredCapture(message,): boolean {
          return (message.includes('"registeredCapture"',)
            || message.includes('"timedRow"',))
            && message.includes('can be deeply readonly',);
        },).length,).toBe(0,);
        expect(messages.filter(function namesRegisteredCapture(message,): boolean {
          return (message.includes('"registeredCapture"',)
            || message.includes('"timedRow"',))
            && message.includes('is exposed to these unresolved calls',);
        },).length,).toBe(2,);
      /* The invoked-result group. `invokedThrough` is the arrival and it keeps its offer, which is
       * the accepted return policy working: a callee that invokes a handed closure and returns the
       * result hands back caller state, and returning caller state is permitted on the condition
       * that callers substitute through a recorded returned origin. The record is what was
       * missing, so `storedInvoked` beside it is the one that proves the fix: it stores that
       * result, a store is not a permitted return, and it withholds only because the capture now
       * reaches the returned set. */
        expect(messages.filter(function namesInvokedThrough(message,): boolean {
          return message.includes('"invokedThrough"',);
        },),).toEqual([
          'Parameter "invokedThrough" can be deeply readonly: `row` is writable; `rows` is writable.',
        ],);
        expect(messages.filter(function namesStoredInvoked(message,): boolean {
          return message.includes('"storedInvoked"',);
        },).length,).toBe(0,);
      /* The third thing a callee can do with a handed callable: write through what invoking it
       * produced. Withheld, and asserted by name because it moves no count. It arrived beside a
       * callee whose formal is a function type, which is offered nothing, and its own control is
       * `inspected` above, whose callee invokes and keeps only a primitive and which therefore
       * keeps its offer. */
        expect(messages.filter(function namesWrittenThrough(message,): boolean {
          return message.includes('"writtenThrough"',);
        },).length,).toBe(0,);
      /* The packaged-default group, asserted by name because the count alone cannot say which
       * offer moved. One arrival and five that withhold, and the five matter more: each is a way
       * the default leaves the callable, and the activation gate stops attributing the write
       * inside it, so every one of them has to be caught by the channel that carries what a
       * callable can reach rather than by the write itself.
       *
       * `handedDefault` is the one that was not. Nothing named the callable an argument holds
       * when it arrives as a parameter default, so the capture channel saw an unresolvable
       * identifier and said nothing, which offered a configuration whose row the retained
       * closure hands out.
       *
       * `unreachedDefault` is self-limiting rather than a compilable suggestion, in the same
       * sense `invokeAssignedLocalClosureWriting` is: its default closure writes through the
       * parameter, so applying the offer stops the file type-checking and no falsification can
       * ride on it. The measurement it carries is about attribution rather than about a
       * suggestion a reader could take, and the three escape shapes beside it are the reading
       * form precisely so that a falsification can. */
        expect(messages.filter(function namesUnreachedDefault(message,): boolean {
          return message.includes('"unreachedDefault"',);
        },),).toEqual([
          'Parameter "unreachedDefault" can be deeply readonly: `row` is writable; `rows` is writable.',
        ],);
        expect(messages.filter(function namesPackagedDefaults(message,): boolean {
          return message.includes('"reachedDefault"',)
            || message.includes('"entryWritten"',)
            || message.includes('"storedDefault"',)
            || message.includes('"handedDefault"',)
            || message.includes('"returnedDefault"',);
        },).length,).toBe(0,);
      /* The method-receiver channel's control. A holder built from nothing the caller owns keeps
       * the offer, which is what separates asking the receiver as well as the callee from
       * reporting every method call on a local holder. Stated as what it controls for rather than
       * as a transition, because which count it moved is no longer reconstructible and an
       * arithmetic claim nothing backs is worse than none. */
        expect(messages.filter(function namesFreshHolder(message,): boolean {
          return message.includes('"freshHolder"',);
        },),).toEqual([
          'Parameter "freshHolder" can be deeply readonly: `row` is writable; `rows` is writable.',
        ],);
      /* The two pass-three controls, carrying the count from thirty-six to thirty-eight. A thrown
       * message retains nothing writable, and a destructuring default that allocates names nothing
       * the caller owns. */
        expect(messages.filter(function namesPassThreeControls(message,): boolean {
          return message.includes('"thrownLabel"',)
            || message.includes('"defaultFresh"',);
        },).length,).toBe(2,);
      /* The nested-callee control, carrying the count from thirty-five to thirty-six. */
        expect(messages.filter(function namesFreshLocal(message,): boolean {
          return message.includes('"viaFreshLocal"',);
        },),).toEqual([
          'Parameter "viaFreshLocal" can be deeply readonly: `row` is writable; `rows` is writable.',
        ],);
      /* The two pass-two controls, carrying the count from thirty-three to thirty-five. A tag
       * handed a label retains nothing writable, and a returned iterator whose closure allocates
       * its own row captures nothing. */
        expect(messages.filter(function namesInterpolatedLabel(message,): boolean {
          return message.includes('"interpolatedLabel"',)
            || message.includes('"iteratedFresh"',);
        },).length,).toBe(2,);
      /* The cluster's leaf control, and the three offers that carry the count from thirty to
       * thirty-three. A count handed to a collection retains nothing a caller can be written
       * through, and the two returns of caller state are the permitted return whose callers now
       * substitute through a tracked origin. */
        expect(messages.filter(function namesCountedArgument(message,): boolean {
          return message.includes('"countedArgument"',);
        },),).toEqual([
          'Parameter "countedArgument" can be deeply readonly: `row` is writable; `rows` is writable.',
        ],);
        expect(messages.filter(function namesHandedBack(message,): boolean {
          return message.includes('"handedBack"',)
            || message.includes('"projectedOut"',);
        },).length,).toBe(2,);
      /* And their control, carrying the count from twenty-six to twenty-seven. */
        expect(messages.filter(function namesNeither(message,): boolean {
          return message.includes('"neither"',);
        },),).toEqual([
          'Parameter "neither" can be deeply readonly: `row` is writable; `rows` is writable.',
        ],);
        expect(messages.some(function forwardedNamesBoundary(message,): boolean {
          return message.includes('"forwarded"',)
            && message.includes('queueMicrotask',);
        },),).toBe(true,);
      /* What still speaks, and in the words that fit it. Every one is a member call on the
       * parameter, so each keeps the method-specific message rather than the generic one.
       * `storeMemberIntoModuleBinding` is the mixed shape that decides this: it both calls
       * `config.rows.at` and stores the result, and before the split its store joined the
       * boundary list, which is an `every` over that list, and cost it this message.
       *
       * Fifteen since the accessor-forms group arrived, which added four more `.register` receivers on
       * top of the conditional-callee group's two. Two name `.rows.at` and the rest name `.register`
       * or `.keep`, each the receiver of
       * a method the capture channel answers for, so the list is asserted per boundary rather than by
       * one shared substring. Every registry parameter reports this way and none of them is a
       * subject: what the shapes are about is the closure handed to the method, and the receiver
       * speaking is the ordinary consequence of calling a method this rule cannot inspect.
       *
       * Eighteen since the void-slot group arrived, which added three: the two forwarders calling
       * `.keep` on their own registry, and the subject that hands its registry to one of them and
       * receives the propagated report.
       *
       * Twenty-one since the candidate-list group arrived, which added three the same way: its two
       * forwarders call `.keep` on their own registry, and its subject and control each hand a registry
       * to one of them, with one of those four sharing a registry parameter rather than adding a
       * report.
       *
       * Twenty-four since the held-callable group arrived, one per shape: each of its three registry
       * parameters is the receiver of a `.register` call.
       *
       * Twenty-six since the awaited-completion group arrived, one for each of its two registry
       * parameters.
       *
       * Twenty-eight since the assignment group arrived, one for each of its two registry parameters. */
        const opacityMessages = messages.filter(function isOpacity(message,): boolean {
        /* Both phrasings, so the count keeps measuring opacity reports rather than which
         * message text a finding was routed to. */
          return message.includes('is the receiver of these unresolved method calls',)
            || message.includes('is exposed through these unresolved collection calls',);
        },);
        expect(opacityMessages.length,).toBe(28,);
        expect(opacityMessages.every(function namesMemberCall(message,): boolean {
          return message.includes('.rows.at',)
            || message.includes('.register',)
            || message.includes('.keep',);
        },),).toBe(true,);
      /* The subject that introduces the boundary list, held to the same rule as the list.
       * `reportMixedBindingCauses` stores one destructured binding and passes the other to
       * an unresolved call, and each binding owns its own slot, so a subject built from
       * every opaque slot said both were used by `JSON.stringify`. Filtering the list
       * without filtering the subject left that standing. */
        const mixedMessages = messages.filter(function namesCalledBinding(message,): boolean {
          return message.includes('"called"',);
        },);
        expect(mixedMessages.length,).toBe(1,);
        expect(mixedMessages.every(function omitsStoredBinding(message,): boolean {
          return !message.includes('"stored"',);
        },),).toBe(true,);
      /* Both halves of the projected-capability pair, which is where the withhold has to stop. A store
       * decides one verdict, the offer, and no other. The first shape of this silenced
       * every verdict for a stored parameter, including this one, which is about the
       * declared type and has nothing to do with where a value went. No sweep of this
       * repository could catch it: nothing here pairs retention with a capability-bearing declared
       * type, so the count of these reports held constant across three captures while one
       * of them was being suppressed. Two, not one, is the assertion. */
        expect(messages.filter(function isProjectedCapability(message,): boolean {
          return message.includes('uses a readonly projection that retains unresolved callable capability',);
        },).length,).toBe(2,);
      },
    },),
    it({
      name: 'credits a reassigned alias with every parameter it can hold',
      fn: async () => {
        const diagnostics = await lintReadonly('readonly-binding-origin-invalid.ts',);
        const messages = diagnostics.map(function diagnosticMessage(diagnostic,): string {
          return preferenceClaim(diagnostic.message,);
        },);
      /* The whole expected set, pinned rather than probed by absence. Every other
       * claim in this case is that some parameter is *not* offered, which a fixture
       * nothing linted would satisfy too, so `readAliasEffect` is the control that
       * proves the file reached the rule. Measured with `registerBindingOrigin`
       * reverted to overwriting origins and the package rebuilt: this fixture emits
       * three messages instead of one, adding offers for `second` and `shadowed`.
       * The `second` offer is the unsound one, whose annotation fails to compile. */
        expect(messages,).toEqual([
          'Parameter "values" can be deeply readonly: `[number].label` is writable; parameter type uses mutable `Array`.',
        ],);
      /* Neither candidate may be offered, because either can be what the alias holds
       * when the mutation runs. Overwriting credits the mutation to whichever branch
       * registered last and offers the other parameter, whose annotation then fails
       * to compile: `Property 'push' does not exist on type 'readonly Labelled[]'`. */
        expect(messages.filter(function offersEitherCandidate(message,): boolean {
          return message.startsWith('Parameter "first" can be deeply readonly',)
            || message.startsWith('Parameter "second" can be deeply readonly',);
        },),).toEqual([],);
      /* Accumulating origins must not slide into crediting every parameter. The
       * control aliases one parameter and one fresh array, so the fresh array
       * contributes no origin and `flag` is never implicated by either function. */
        expect(messages.filter(function offersUnrelatedParameter(message,): boolean {
          return message.includes('"only"',)
            || message.includes('"flag"',);
        },),).toEqual([],);
      /* The accepted cost of being flow-insensitive, pinned so it stays a decision
       * rather than drift. `shadowed` is displaced before the mutation runs and can
       * never be what the alias holds, yet it is credited and loses an offer it
       * deserves. Withholding an offer is the safe direction; the overwrite this
       * replaced made one that did not compile. */
        expect(messages.filter(function offersDisplacedParameter(message,): boolean {
          return message.includes('"shadowed"',);
        },),).toEqual([],);
      },
    },),
    it({
      name: 'routes each evidence category to exactly one public rule',
      fn: async () => {
        const diagnostics = await lintReadonly('readonly-split-invalid.ts',);
        expect(diagnostics.length,).toBe(4,);
      /**
       Diagnostics indexed by exact public rule ID.
       */
        const diagnosticsByCode = new Map(
          diagnostics.map(function indexDiagnostic(diagnostic,): [string, OxlintDiagnostic] {
            return [diagnostic.code, diagnostic,];
          },),
        );
        expect(
          diagnosticsByCode
            .get('prefer-readonly-parameter-type(prefer-readonly-parameter-types)',)
            ?.message,
        ).toContain('Parameter "state" can be deeply readonly',);
        expect(
          diagnosticsByCode
            .get('prefer-readonly-parameter-type(no-readonly-parameter-mutations)',)
            ?.message,
        ).toContain('analysis proved a reachable mutation',);
        expect(
          diagnosticsByCode
            .get('prefer-readonly-parameter-type(no-opaque-parameter-effects)',)
            ?.message,
        ).toContain('readonly projection that retains unresolved callable capability',);
        expect(
          diagnosticsByCode
            .get('prefer-readonly-parameter-type(no-invalid-parameter-effect-contracts)',)
            ?.message,
        ).toContain('stale @mutates contract',);
        expect(diagnostics.some(function projectedCapabilityIsNotMutation(diagnostic,): boolean {
          return (diagnostic.code
            === 'prefer-readonly-parameter-type(no-readonly-parameter-mutations)')
            && diagnostic.message.includes('runner',);
        },),).toBe(false,);
      },
    },),
    it({
      name: 'renders one-line searchable subjects across every public rule',
      fn: async () => {
        const diagnostics = await lintReadonly('readonly-subject-split-invalid.ts',);
        expect(diagnostics.length,).toBe(4,);
      /**
       Messages indexed by public rule owning each subject.
       */
        const messagesByCode = new Map(diagnostics.map(function indexMessage(
          diagnostic,
        ): readonly [string, string] {
          return [diagnostic.code, diagnostic.message,];
        },),);
        expect(messagesByCode.get(
          'prefer-readonly-parameter-type(prefer-readonly-parameter-types)',
        ),).toContain(
          'Destructured parameter with binding "state" can be deeply readonly',
        );
        expect(messagesByCode.get(
          'prefer-readonly-parameter-type(no-readonly-parameter-mutations)',
        ),).toContain(
          'Destructured parameter with binding "state" is declared readonly',
        );
        expect(messagesByCode.get(
          'prefer-readonly-parameter-type(no-opaque-parameter-effects)',
        ),).toContain(
          'Destructured parameter with binding "runner" uses a readonly projection',
        );
        expect(messagesByCode.get(
          'prefer-readonly-parameter-type(no-invalid-parameter-effect-contracts)',
        ),).toContain(
          'Destructured parameter with binding "label" has stale @mutates contract',
        );
        expect(diagnostics.every(function messageIsOneLine(diagnostic,): boolean {
          return (!diagnostic.message.includes('\n',))
            && (!diagnostic.message.includes('\r',));
        },),).toBe(true,);
      },
    },),
    it({
      name: 'renders every parameter binding shape without raw source text',
      fn: async () => {
        const diagnostics = await lintReadonly('readonly-subject-patterns-invalid.ts',);
        expect(diagnostics.length,).toBe(9,);
      /**
       Stable subjects rendered by preference findings.
       */
        const subjects = diagnostics.map(function diagnosticSubject(diagnostic,): string {
        /**
         Rule-specific suffix following shared parameter subject.
         */
          const suffix = diagnostic.message.includes(' can be deeply readonly:',)
            ? ' can be deeply readonly:'
            : ' has stale @mutates contract.';
          return diagnostic.message.split(suffix,)[0] ?? '';
        },);
        expect(subjects,).toContain('Parameter "value"');
        expect(subjects,).toContain(
          'Destructured parameter with bindings "raw" and "rest"',
        );
        expect(subjects.filter(function aliasedLocal(subject,): boolean {
          return subject === 'Destructured parameter with binding "local"';
        },).length,).toBe(2,);
        expect(subjects,).toContain('Destructured parameter with binding "raw"');
        expect(subjects,).toContain('Destructured parameter with binding "inner"');
        expect(subjects,).toContain('Destructured parameter with binding "rest"');
        expect(subjects,).toContain(
          'Destructured parameter with bindings "first" and "rest"',
        );
        expect(subjects,).toContain('Parameter 1 at this location');
        expect(diagnostics.every(function searchableOneLineSubject(diagnostic,): boolean {
          return (!diagnostic.message.includes('\n',))
            && (!diagnostic.message.includes('\r',))
            && (!diagnostic.message.includes('source:',));
        },),).toBe(true,);
      },
    },),
    it({
      name: 'gives complete action paths for every inferred-origin state',
      fn: async () => {
        const diagnostics = await lintReadonly('readonly-inferred-origin-invalid.ts',);
        expect(diagnostics.length,).toBe(11,);
      /**
       Preference messages emitted by origin decision controls.
       */
        const messages = diagnostics.map(function diagnosticMessage(diagnostic,): string {
          return diagnostic.message;
        },);
        expect(messages.some(function verifiedAuthored(message,): boolean {
          return message.includes('Verified edit: Prefix the authored array type with `readonly`.');
        },),).toBe(true,);
        expect(messages.some(function deepAuthored(message,): boolean {
          return message.includes(
            'Verified edit: Wrap the complete authored parameter type with `import(\'type-fest\').ReadonlyDeep<...>`.',
          );
        },),).toBe(true,);
        expect(messages.some(function externalDeclaration(message,): boolean {
          return message.startsWith('Parameter "error" can be deeply readonly:')
            && message.includes('At least one reported writable path is declared outside this workspace.')
            && message.includes('`Readonly<T>` is shallow');
        },),).toBe(true,);
      /**
       Multi-origin guidance preserving uncertainty without location dump.
       */
        const multipleOrigins = messages.filter(function multiple(message,): boolean {
          return message.includes('has multiple workspace-owned origins',);
        },);
        expect(multipleOrigins.length,).toBe(3,);
        expect(multipleOrigins.every(function omitsOriginDump(message,): boolean {
          return (!message.includes('toLeft',))
            && (!message.includes('toRight',))
            && message.includes(
              'Establish one common deeply readonly element type at their merge boundary',
            );
        },),).toBe(true,);
        expect(messages.some(function namedTypeOrigin(message,): boolean {
          return message.includes(
            'originates in type "NamedMutableRow" at package/test-fixture/oxlint-no-restricted-syntax/src/readonly-inferred-origin-invalid.ts:11',
          )
            && message.includes('No exact type syntax was proved for that producer');
        },),).toBe(true,);
      /**
       Every consumer of judged rows converges on sole mapping callback.
       */
        const callableOrigins = messages.filter(function callableOrigin(message,): boolean {
          return message.includes(
            'originates in callable "toJudged" at package/test-fixture/oxlint-no-restricted-syntax/src/readonly-inferred-origin-invalid.ts:22',
          );
        },);
        expect(callableOrigins.length,).toBe(2,);
        expect(callableOrigins.every(function cautiousProducer(message,): boolean {
          return message.includes('Likely edit: give that callable an explicit deeply readonly return type.')
            && message.includes('No exact type syntax was proved for that producer');
        },),).toBe(true,);
        expect(messages.some(function anonymousArrowOrigin(message,): boolean {
          return message.includes('originates in an anonymous callable at ')
            && message.includes('readonly-inferred-origin-invalid.ts:');
        },),).toBe(true,);
        expect(messages.some(function sameOwnerUnion(message,): boolean {
          return message.includes('originates in callable "toEither" at ')
            && (!message.includes('has multiple workspace-owned origins',));
        },),).toBe(true,);
        expect(messages.every(function oneLine(message,): boolean {
          return (!message.includes('\n',)) && (!message.includes('\r',));
        },),).toBe(true,);
      },
    },),
    it({
      name: 'reports every distinct writable path without a presentation budget',
      fn: async () => {
        const diagnostics = await lintReadonly('readonly-writable-paths-invalid.ts',);
        expect(diagnostics.length,).toBe(1,);
      /**
       Sole complete preference diagnostic for multi-branch type.
       */
        const message = diagnostics[0]?.message ?? '';
        [
          '`alpha.value` is writable',
          '`beta.value` is writable',
          '`byName[string].status` is writable',
          '`children[number].type` is writable',
          '`delta.value` is writable',
          '`epsilon.value` is writable',
          '`eta.value` is writable',
          '`gamma.value` is writable',
          '`zeta.value` is writable',
        ].forEach(function containsCompletePath(path,): void {
          expect(message,).toContain(path,);
        },);
        expect(message.includes('omitted',),).toBe(false,);
        expect(message.includes('\n',),).toBe(false,);
        expect(message.includes('\r',),).toBe(false,);
      },
    },),
    it({
      name: 'distinguishes local expressions from genuine callable return producers',
      fn: async () => {
        const diagnostics = await lintReadonly('readonly-local-origin-invalid.ts',);
      /**
       Preference messages emitted by local producer matrix.
       */
        const messages = diagnostics.map(function diagnosticMessage(diagnostic,): string {
          return diagnostic.message;
        },);
        [
          'localArrayRow',
          'localPromiseRow',
          'localSeedRow',
        ].forEach(function localExpressionParameter(parameterName,): void {
        /**
         Finding for exact local-expression control.
         */
          const message = messages.find(function matchingParameter(candidate,): boolean {
            return candidate.startsWith(`Parameter "${parameterName}" can be deeply readonly:`,);
          },);
          expect(message,).toBeDefined();
          expect(message,).toContain('originates in an inferred local expression at ');
          expect(message?.includes('give that callable an explicit deeply readonly return type',),).toBe(false,);
          expect(message?.includes('localOriginControls',),).toBe(false,);
        },);
        expect(messages.some(function genericCapabilityWithheld(message,): boolean {
          return message.includes('localGenericRow',);
        },),).toBe(false,);
        expect(messages.some(function distinctUnionOrigins(message,): boolean {
          return message.startsWith('Parameter "localUnionRow" can be deeply readonly:')
            && message.includes('has multiple workspace-owned origins');
        },),).toBe(true,);
        expect(messages.some(function genuineReturnProducer(message,): boolean {
          return message.startsWith('Parameter "returnedRow" can be deeply readonly:')
            && message.includes('originates in callable "makeProducedRow" at ')
            && message.includes('give that callable an explicit deeply readonly return type');
        },),).toBe(true,);
        expect(messages.some(function boundReturnProducer(message,): boolean {
          return message.startsWith('Parameter "boundReturnedRow" can be deeply readonly:')
            && message.includes('originates in callable "makeBoundProducedRow" at ')
            && message.includes('give that callable an explicit deeply readonly return type');
        },),).toBe(true,);
      },
    },),
    it({
      name: 'resolves a unique producer across a source-file boundary',
      fn: async () => {
        const diagnostics = await lintReadonly('readonly-cross-file-origin-invalid.ts',);
        expect(diagnostics.length,).toBe(1,);
        expect(diagnostics[0]?.message,).toContain(
          'originates in callable "toCrossFileRow" at package/test-fixture/oxlint-no-restricted-syntax/src/readonly-origin-producer.ts:16',
        );
        expect(diagnostics[0]?.message,).toContain(
          'Likely edit: give that callable an explicit deeply readonly return type.',
        );
      },
    },),
    it({
      name: 'reports readonly preference, stale contracts, and unresolved effects',
      fn: async () => {
        const diagnostics = await lintReadonly('readonly-invalid.ts',);
        expect(diagnostics.length,).toBe(11,);
        const messages = diagnostics.map(function diagnosticMessage(diagnostic,): string {
          return preferenceClaim(diagnostic.message,);
        },);
        expect(messages.some(function shouldReadonly(message,): boolean {
          return message.includes('can be deeply readonly',);
        },),).toBe(true,);
      /* Inherited documented uncertainty no longer demands per-level
       * contracts; the boundary contract is the audit. */
        expect(messages.some(function inheritedUncertainty(message,): boolean {
          return message.includes('but lacks its own @mutates contract',);
        },),).toBe(false,);
      /**
       Resolved nonmutating parameter owns sole stale-contract diagnostic.
       */
        const contractDiagnostics = diagnostics.filter(function contractRule(diagnostic,): boolean {
          return diagnostic.code
            === 'prefer-readonly-parameter-type(no-invalid-parameter-effect-contracts)';
        },);
        expect(contractDiagnostics.length,).toBe(1,);
        expect(contractDiagnostics[0]?.message,).toContain(
          'Parameter "controller" has stale @mutates contract',
        );
        expect(messages.some(function opacityPreemptsReadonlyShape(message,): boolean {
          return message.includes('uses a readonly projection that retains unresolved callable capability',);
        },),).toBe(false,);
      /* Five, up from four, because `destructuredOpaqueEffect` joined them. Its report used to
       * name both bindings of its one destructured parameter and now names `state` alone,
       * which is the assertion immediately below. */
        expect(messages.filter(function contractedOpacity(message,): boolean {
          return message.startsWith(
            'The function input named "state" is exposed to these unresolved calls: JSON.stringify [',
          );
        },).length,).toBe(5,);
      /* Every serialization finding belongs to the unresolved-effect rule regardless of
       * whether its declaration is already deeply readonly. The preference rule emits only
       * the separate proved replacement in this fixture. */
        expect(diagnostics.filter(function stateSerialization(diagnostic,): boolean {
          return diagnostic.message.startsWith(
            'The function input named "state" is exposed to these unresolved calls: JSON.stringify [',
          );
        },).every(function ownedByOpaqueRule(diagnostic,): boolean {
          return diagnostic.code
            === 'prefer-readonly-parameter-type(no-opaque-parameter-effects)';
        },),).toBe(true,);
      /** Plain-language uncertainty diagnostic for unsafe JSON serialization. */
        const opaqueMessage = messages.find(function unsafeJson(message,): boolean {
          return message.startsWith(
            'The function input named "state" is exposed to these unresolved calls: JSON.stringify [',
          )
            && message.includes('An @mutates block alone documents known effects',);
        },);
        if (opaqueMessage === undefined)
          throw new Error('Expected JSON.stringify uncertainty diagnostic.',);
      /* Boundaries carry origin locations and strict proof-preserving remediation. */
        expect(opaqueMessage.includes(
          'src/readonly-invalid.ts:',
        ),).toBe(true,);
        expect(opaqueMessage.includes(
          '\n\nThis rule cannot inspect enough of those calls to know what they might change. They could change the input itself, change an object stored inside it, call a function stored inside it, or arrange for one of those changes to happen later.'
            + '\n\nResolve the call by one of these proof-preserving changes:'
            + '\n1. Include the exact repository-owned implementation in the nearest tsconfig.json so the rule can inspect it.'
            + '\n2. Pass only primitive values or a separately verified isolated snapshot that shares no caller-owned identity or capability.'
            + '\n3. Remove or replace the call so no caller-owned input reaches unresolved code.'
            + '\n4. After source and source-map inference are exhausted, mark exact runtime-owned host input as ForeignHostCapability and document its possible effects with @mutates.'
            + '\n\nAn @mutates block alone documents known effects but cannot make an unresolved implementation safe.',
        ),).toBe(true,);
      /* `destructuredOpaqueEffect({ state, label })` calls `JSON.stringify({ state, label })`,
       * and `label` is a `string`. A primitive cannot carry the state an unresolved call might
       * change, so naming it in the report told the reader to look at something that could not
       * be the problem. It was named because both bindings shared one parameter index, which
       * `ST9` makes the ordinary shape rather than an unusual one: the report could only name
       * every binding of the parameter or none.
       *
       * Now the opacity is recorded against `state`'s own slot and the report says so.
       * `effect-affected-bindings.ts` is what turns that slot fact into a name, and removing it
       * puts `label` back. */
        expect(messages.some(function namesPrimitiveSibling(message,): boolean {
          return message.startsWith(
            'The function inputs named "state" and "label" are exposed to these unresolved calls: JSON.stringify [',
          );
        },),).toBe(false,);
      /** Method-specific diagnostic explaining state changes without assignment. */
        const methodMessage = messages.find(function unknownMethod(message,): boolean {
          return message.startsWith(
            'The function input named "service" is the receiver of these unresolved method calls: service.write [',
          );
        },);
        if (methodMessage === undefined)
          throw new Error('Expected unknown method diagnostic.',);
        expect(methodMessage.includes(
          'A method can change state stored inside its receiver or in the system it controls.',
        ),).toBe(true,);
      /** Opaque call remains rejected despite unrelated authored contract. */
        const incompleteContractMessage = messages.find(function unrelatedLinkContract(message,): boolean {
          return message.startsWith(
            'The function input named "state" is exposed to these unresolved calls: opaqueExternalMutation [',
          );
        },);
        if (incompleteContractMessage === undefined)
          throw new Error('Expected incomplete-contract uncertainty diagnostic.',);
        expect(incompleteContractMessage.includes(
          'An @mutates block alone documents known effects but cannot make an unresolved implementation safe.',
        ),).toBe(true,);
      /** Global String remains an ordinary unresolved bodyless host call. */
        const stringMessage = messages.find(function stringCoercion(message,): boolean {
          return message.startsWith(
            'The function input named "error" is exposed to these unresolved calls: String [',
          );
        },);
        if (stringMessage === undefined)
          throw new Error('Expected global String object coercion diagnostic.',);
        expect(stringMessage.includes(
          'An @mutates block alone documents known effects but cannot make an unresolved implementation safe.',
        ),).toBe(true,);
        expect(messages.some(function incompleteStringContract(message,): boolean {
          return message.startsWith(
            'The function input named "incomplete" is exposed to these unresolved calls: String [',
          );
        },),).toBe(true,);
      },
    },),
    it({
      name: 'does not identify same-named external callable as global String',
      fn: async () => {
        const diagnostics = await lintReadonly('readonly-string-lookalike-invalid.ts',);
        expect(diagnostics.length,).toBe(1,);
        expect(diagnostics[0]?.message.startsWith(
          'The function input named "value" is exposed to these unresolved calls: String [',
        ),).toBe(true,);
        expect(diagnostics[0]?.message.includes(
          'passed to global String',
        ),).toBe(false,);
      },
    },),
    it({
      name: 'reports overload contracts whose union differs from implementation effects',
      fn: async () => {
        const diagnostics = await lintReadonly('readonly-overload-invalid.ts',);
        expect(diagnostics.length,).toBe(1,);
        expect(diagnostics[0]?.message,).toBe(
          'Mutation contracts disagree across callable signatures.',
        );
      /* The one report is the over-claiming overload, whose `@mutates controller`
       * names a transition the implementation never performs. `invokedStepOverload`
       * in the same fixture invokes a step lifted out of its parameter and is not
       * reported, because an invoked capability is not a referent mutation and no
       * authored contract can express one. Its `invokedStepPlain` control has the
       * identical body without overloads and was already silent, which is what
       * places the difference in the comparison rather than in the effect.
       *
       * Comparing the union again would report all three, and the length assertion
       * above is what catches that. */
      },
    },),
    it({
      name: 'keeps a receiver-held tuple out of the exposure discharge',
      fn: async () => {
        const diagnostics = await lintReadonly(
          'readonly-tuple-exposure-invalid.ts',
        );
      /** Fixture source read back for exact span attribution. */
        const fixtureSource = readFileSync(resolveFixtureTarget({
          fixtureSourceRoot: FIXTURES,
          fixturePath: 'readonly-tuple-exposure-invalid.ts',
        },), 'utf8',);
      /* The control is offered and the writers are not. `readStoredPair` iterates the
       * same receiver through the same member and writes nothing, so wrapping its
       * parameter is sound, and `readonly-tuple-exposure-invalid.ts` declares that offer
       * allowed. The two writers must stay silent: `pair[0] = 'rewritten'` on a tuple the
       * array holds mutates `rows`, so offering either one would recommend a wrapping
       * their own bodies defeat. A tuple is caller-owned state whatever its positions
       * are, because the tuple itself is writable, and only a member that builds the
       * tuple fresh could discharge it. That is the container relation
       * `FRESH_CONTAINER_MEMBER_NAMES` records as unbuilt.
       *
       * History: under TypeScript 7.0.2 this fixture was fully silent, because
       * `Type.isTupleType()` answered false for these tuple references and the
       * classifier fell through to `deep-readonly`, making every offer impossible for
       * the wrong reason. TypeScript 7.1 answers true (the tuple rework behind
       * `TupleTypeReference.getTarget().readonly`), so the classifier sees the mutable
       * element tuple and the control's sound offer appears. The recursion experiment
       * the fixture warns about stays unsound, measured rather than argued: with the
       * tuple's own positions recursed into, `rewriteMutableStoredPair` is offered while
       * its body runs `pair[0] = 'rewritten'`. */
        expect(diagnostics.length,).toBe(1,);
      /* The one offer lands on the control's parameter, by exact source offset. An offer
       * on either writer would carry an identical message, so only the span attributes
       * it, and the attribution pin in `effect-summaries.unit.test.ts`
       * (`rewriteMutableStoredPair` and `rewriteStoredPair` at `referentMutated=[0]`) is
       * what must withhold the writers' offers at the gate. The two move together. */
        expect(
          diagnostics[0]?.labels?.[0]?.span.offset,
        ).toBe(
          fixtureSource.indexOf('rows', fixtureSource.indexOf('function readStoredPair',),),
        );
        expect(diagnostics.some(function offersReadonly(diagnostic,): boolean {
          return diagnostic.message
            .includes('can be deeply readonly',);
        },),).toBe(true,);
      },
    },),
    it({
      name: 'keeps foreign ownership anchored to a marker through recursion',
      fn: async () => {
        const diagnostics = await lintReadonly(
          'readonly-recursive-ownership-invalid.ts',
        );
      /* Two, and the count distinguishes all three states this can be in. Before
       * grounding it was one: `markerlessRecursion` was suppressed by a foreign
       * candidate no marker fed, because the greatest fixed point seeds every
       * parameter of a callable holding an inbound and a self-edge then sustains it.
       * If grounding over-removed, it would be four, because `markerFedRecursion` and
       * `markedRecursionEntry` would lose a suppression a marker does feed.
       *
       * That middle case is the one worth being careful about. Removing the marker
       * from `markedRecursionEntry` and re-linting yields four offers, which is what
       * establishes the two suppressions here are marker-driven rather than an
       * artifact of the helper being unexported. */
        expect(diagnostics.length,).toBe(2,);
      /* Neither offer may name the marked parameter. `markerlessPlain` and
       * `markerlessRecursion` both take `value`, so the messages are identical and
       * only this separates an over-removal that happened to keep the count. */
        expect(diagnostics.some(function offersMarkedParameter(diagnostic,): boolean {
          return diagnostic.message
            .includes('"marked"',);
        },),).toBe(false,);
      },
    },),
    it({
      name: 'suggests readonly arrays only through explicit suggestion fixes',
      fn: async () => {
      /** Source with deeply projectable primitive array parameter. */
        const source = `/**\n * Reads values.\n *\n * @param values - Values read without mutation.\n */\nexport function countValues(values: string[],): number {\n  return values.length;\n}\n`;
        const ordinarilyFixed = await fixReadonlyGeneratedSource({
          source,
          fixSuggestions: false,
        },);
        expect(ordinarilyFixed.includes('values: string[]',),).toBe(true,);
        const suggestionFixed = await fixReadonlyGeneratedSource({
          source,
          fixSuggestions: true,
        },);
        expect(suggestionFixed.includes('values: readonly string[]',),).toBe(true,);
      },
    },),
    it({
      name: 'suggests standard ReadonlyArray owner for readonly reachable types',
      fn: async () => {
      /** Source with standard mutable Array type reference. */
        const source = '/** Reads array. @param values - Values. */\nexport function read(values: Array<string>,): number { return values.length; }\n';
        const ordinarilyFixed = await fixReadonlyGeneratedSource({
          source,
          fixSuggestions: false,
        },);
        expect(ordinarilyFixed.includes('Array<string>',),).toBe(true,);
        const suggestionFixed = await fixReadonlyGeneratedSource({
          source,
          fixSuggestions: true,
        },);
        expect(suggestionFixed.includes('ReadonlyArray<string>',),).toBe(true,);
      },
    },),
    it({
      name: 'suggests type-fest ReadonlyDeep for structural data without depending on an import',
      fn: async () => {
      /** Source with mutable nested data and aliased type-fest import. */
        const source = `import type { ReadonlyDeep as DeepReadonly, } from 'type-fest';

/**
 * Reads nested state.
 *
 * @param state - Nested state read without mutation.
 */
export function readNested(state: { nested: { value: string; }; },): string {
  return state.nested.value;
}
`;
        const ordinarilyFixed = await fixReadonlyGeneratedSource({
          source,
          fixSuggestions: false,
        },);
        expect(ordinarilyFixed.includes('state: { nested:',),).toBe(true,);
        const suggestionFixed = await fixReadonlyGeneratedSource({
          source,
          fixSuggestions: true,
        },);
      /* This asserted `DeepReadonly<{ nested:` and the alias preservation it pinned is
       * gone on purpose. The suggestion used to fire only for a file already importing
       * `ReadonlyDeep`, and it emitted that local name, so it depended on a statement it
       * could not keep alive: until the suggestion is applied the import is unused, and
       * the unused-import fix removes it in the same pass. Measured end to end, a file
       * that type-checked clean before `oxlint --fix --fix-suggestions` failed afterwards
       * with `TS2552: Cannot find name 'ReadonlyDeep'`. An inline import type needs no
       * statement, so an alias has nothing left to name. */
        expect(suggestionFixed.includes(
          "state: import('type-fest').ReadonlyDeep<{ nested:",
        ),).toBe(true,);
      },
    },),
    it({
      name: 'suggests the structural projection for a source that imports nothing',
      fn: async () => {
      /* The half the import gate blocked. Measured across the workspace before this
       * changed: thirty-two offers in ten files, of which one file imported
       * `ReadonlyDeep`, so nine files carried an offer the rule could not help with for a
       * reason that was mechanical rather than about correctness. Every guard that is
       * about correctness stays, which the array case beside this one covers. */
        const source = `/**
 * Reads nested state.
 *
 * @param state - Nested state read without mutation.
 */
export function readNested(state: { nested: { value: string; }; },): string {
  return state.nested.value;
}
`;
        const suggestionFixed = await fixReadonlyGeneratedSource({
          source,
          fixSuggestions: true,
        },);
        expect(suggestionFixed.includes(
          "state: import('type-fest').ReadonlyDeep<{ nested:",
        ),).toBe(true,);
      /* Nothing was added above the declaration, which is the whole point of the inline
       * form: no import statement means no statement to race with. */
        expect(suggestionFixed.includes('from \'type-fest\'',),).toBe(false,);
      },
    },),
    it({
      name: 'keeps stale contracts under ordinary fix and removes them through suggestions',
      fn: async () => {
      /** Source with one semantically stale mutation block. */
        const source = `/**\n * Reads signal.\n *\n * @param controller - Capability read only.\n *\n * @mutates controller - Claims absent transition.\n */\nexport function readSignal(controller: AbortController,): AbortSignal {\n  return controller.signal;\n}\n`;
        const ordinarilyFixed = await fixReadonlyGeneratedSource({
          source,
          fixSuggestions: false,
        },);
        expect(ordinarilyFixed.includes('@mutates controller',),).toBe(true,);
        const suggestionFixed = await fixReadonlyGeneratedSource({
          source,
          fixSuggestions: true,
        },);
        expect(suggestionFixed.includes('@mutates controller',),).toBe(false,);
      },
    },),
  ],
},);
