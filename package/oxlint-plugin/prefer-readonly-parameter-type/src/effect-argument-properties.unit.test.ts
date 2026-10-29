import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { isFunctionLikeDeclaration, } from 'typescript/ast/is';
import { readFileSync, } from 'node:fs';
import { fileURLToPath, } from 'node:url';

import {
  buildEffectSummaryIndex,
  closeSemanticBridge,
  NO_EFFECT_SUMMARY,
  openSemanticFile,
} from '../dist/final/node/index.mjs';

/** Fixture pairing one narrowing shape per caller with the callee it hands rows to. */
const NARROWING_PATH = fileURLToPath(new URL(
  '../../../test-fixture/oxlint-no-restricted-syntax/src/readonly-slot-narrowing-invalid.ts',
  import.meta.url,
),);

/** Current narrowing-fixture text. */
const NARROWING_SOURCE = readFileSync(
  NARROWING_PATH,
  'utf8',
);

await describe({
  name: 'caller-side property matching',
  concurrency: 1,
  children: [
    it({
      name: 'narrows to the property that fills a slot, and withholds it wherever that would lose a write',
      fn: async () => {
        /* Both directions in one place, because either alone reads as the other's bug.
         * A narrowing that never fires looks like precision that was never recovered, and
         * one that fires everywhere looks the same as a correct one until something
         * measures a shape it should have refused. */
        const session = openSemanticFile({
          fileName: NARROWING_PATH,
          sourceText: NARROWING_SOURCE,
          hasBOM: false,
        },);
        const index = buildEffectSummaryIndex({
          project: session.project,
          activeSourceFile: session.sourceFile,
        },);
        /**
         Reads the written parameter positions of one fixture caller.
         
         Reads `referentMutatedParameterIndexes` rather than the union with the invoked
         set, because the readonly offer is gated on that set alone.
         
         @param functionName - Exported fixture caller to inspect.
         
         @returns written parameter positions in ascending order.
         */
        function writtenIndexes(functionName: string,): readonly number[] {
          /**
           Name node of the declaration under test.
           */
          const nameNode = session.nodeAtOffset(
            NARROWING_SOURCE.indexOf(`function ${functionName}`,)
              + 'function '.length,
          );
          /**
           Declaration the name belongs to.
           */
          const declaration = nameNode.parent;
          if (!isFunctionLikeDeclaration(declaration,))
            throw new Error(`Expected a declaration for ${functionName}.`,);
          /**
           Summary of that declaration.
           */
          const summary = index.get(declaration,);
          if (summary === NO_EFFECT_SUMMARY)
            throw new Error(`Expected an effect summary for ${functionName}.`,);
          /* Explicit numeric compare, since the default sort is lexicographic. */
          return [...summary.referentMutatedParameterIndexes,]
            .toSorted(function byIndex(
              left: number,
              right: number,
            ): number {
              return left - right;
            },);
        }
        /**
         Reads the parameter positions one fixture caller cannot account for.
         
         @param functionName - Exported fixture caller to inspect.
         
         @returns opaque parameter positions in ascending order.
         */
        function opaqueIndexes(functionName: string,): readonly number[] {
          /**
           Name node of the declaration under test.
           */
          const nameNode = session.nodeAtOffset(
            NARROWING_SOURCE.indexOf(`function ${functionName}`,)
              + 'function '.length,
          );
          /**
           Declaration the name belongs to.
           */
          const declaration = nameNode.parent;
          if (!isFunctionLikeDeclaration(declaration,))
            throw new Error(`Expected a declaration for ${functionName}.`,);
          /**
           Summary of that declaration.
           */
          const summary = index.get(declaration,);
          if (summary === NO_EFFECT_SUMMARY)
            throw new Error(`Expected an effect summary for ${functionName}.`,);
          return [...summary.opaqueParameterIndexes,]
            .toSorted(function byIndex(
              left: number,
              right: number,
            ): number {
              return left - right;
            },);
        }
        /** Callback and its argument packaged into one destructured parameter. */
        const packagedCallback = opaqueIndexes('packagedCallbackInvocation',);
        /** The same shape, which proves no write rather than proving one. */
        const packagedCallbackWritten = writtenIndexes('packagedCallbackInvocation',);
        /** Each row handed to the property the callee names. */
        const plainKey = writtenIndexes('plainKeyNarrowing',);
        /** Both rows handed over through shorthand properties. */
        const shorthandKey = writtenIndexes('shorthandKeyNarrowing',);
        /** Keys quoted where the callee reads them unquoted. */
        const quotedKey = writtenIndexes('quotedKeyNarrowing',);
        /** Callee binding the written property under another name. */
        const renamedKey = writtenIndexes('renamedKeyNarrowing',);
        /** Numeric key spelled `1e0` against a callee reading `1`. */
        const numericKey = writtenIndexes('numericKeyNarrowing',);
        /** Row packaged one literal below the property the callee names. */
        const nestedValue = writtenIndexes('nestedValueNarrowing',);
        /** Row returned by a method the callee calls. */
        const methodResult = writtenIndexes('methodResultNarrowing',);
        /** Spread placed before the exact key that shadows it. */
        const spreadBefore = writtenIndexes('spreadBeforeKeyNarrowing',);
        /** Spread placed after the key it can overwrite. */
        const spreadAfter = writtenIndexes('spreadAfterKeyBroadcast',);
        /** Property filled under a key resolved at runtime. */
        const computedKey = writtenIndexes('computedKeyBroadcast',);
        /** Property served from an explicit prototype rather than an own key. */
        const prototypeKey = writtenIndexes('prototypeKeyBroadcast',);
        /** Prototype accessor reaching a sibling key through its receiver. */
        const inheritedAccessor = writtenIndexes('inheritedAccessorBroadcast',);
        /** Two literals reaching one rest formal. */
        const restIndexSpread = writtenIndexes('restIndexSpreadBroadcast',);
        /** Both rows packaged into a local before the call. */
        const localLiteral = writtenIndexes('localLiteralProvenance',);
        /** Row held by a class expression's static member. */
        const classMember = writtenIndexes('classMemberPackaging',);
        /** Receiver of an assignment whose declaration is a setter. */
        const throughSetter = writtenIndexes('assignThroughSetter',);
        /** Holder whose setter retains the row assigned into it, and that row. */
        const setterRetention = writtenIndexes('retainThroughSetter',);
        /** Row a callable writes through its explicit `this` formal. */
        const throughThis = writtenIndexes('writeThroughThis',);
        /** Row written by a method called on it as the receiver. */
        const thisReceiver = writtenIndexes('explicitThisReceiver',);
        /** Property defined by a getter with an origin-free setter after it. */
        const accessorKey = writtenIndexes('accessorKeyBroadcast',);
        /** Getter reaching its row through `this` rather than through its own value. */
        const thisAccessor = writtenIndexes('thisAccessorBroadcast',);
        /** Literal handed to a rest formal that destructures it by index. */
        const restIndex = writtenIndexes('restIndexBroadcast',);
        closeSemanticBridge();
        /* Narrowing, measured. Every one of these read `[0, 1]` while the edge repeated an
         * argument's whole origin set on every property slot, and the second parameter in
         * each is one the callee only reads. Quoting, renaming and numeric spelling are
         * separate assertions because each is a distinct way for the two sides of the key
         * comparison to disagree, and disagreement drops the origin rather than widening
         * it. */
        expect(plainKey,).toEqual([0,],);
        expect(shorthandKey,).toEqual([0,],);
        expect(quotedKey,).toEqual([0,],);
        expect(renamedKey,).toEqual([0,],);
        expect(numericKey,).toEqual([0,],);
        expect(nestedValue,).toEqual([0,],);
        expect(methodResult,).toEqual([0,],);
        /* A later exact key shadows an earlier spread at runtime, so the spread's row is
         * not reachable through that property and the walk stops at the exact match. */
        expect(spreadBefore,).toEqual([0,],);
        /* Withheld, and each for its own reason. A spread after the key can overwrite it,
         * so both rows stay named. */
        expect(spreadAfter,).toEqual([
          0,
          1,
        ],);
        /* A computed key could name the property the callee reads, so it contributes and
         * the walk continues past it. */
        expect(computedKey,).toEqual([
          0,
          1,
        ],);
        /* A literal that sets a prototype is not decomposed at all, so both rows stay named even
         * though only `second` is reachable through the written property. Measured three ways to
         * show why the refusal earns its imprecision:
         *
         * 1. `__proto__` read as an ordinary key: `[]` here and `[]` for the inherited-accessor
         *    shape. Both unsound.
         * 2. `__proto__` read as a wildcard carrying the origins inside the prototype: `[1]` here,
         *    which is exactly right, and still `[]` for the inherited-accessor shape, because the
         *    origin that one reaches sits outside the prototype entirely. Shipped in
         *    `0f787856f` and unsound.
         * 3. Refusing to decompose: both shapes name both rows.
         *
         * Anything that restores precision here has to answer measurement 2. */
        expect(prototypeKey,).toEqual([
          0,
          1,
        ],);
        /* The shape that ruled out the wildcard. An inherited getter runs with the outer literal
         * as its receiver, so `{ __proto__: { get named() { return this.hidden } }, hidden: first }`
         * reaches `first` through a sibling key that no walk looking for `named` considers, and
         * the getter body names no caller binding at all. Reads `[]` without the refusal, which
         * offers `readonly` for a row the callee writes. */
        expect(inheritedAccessor,).toEqual([
          0,
          1,
        ],);
        /* An accessor pair whose setter comes last carries no origin, so stopping at the
         * first exact match from the end would drop the getter's row. */
        expect(accessorKey,).toEqual([
          0,
          1,
        ],);
        /* A getter reaching its row through `this` puts the origin under a different key
         * entirely, which no walk of the accessor body finds. */
        expect(thisAccessor,).toEqual([
          0,
          1,
        ],);
        /* A local holding the literal carries what the literal packages, so the callee's write
         * reaches the row that property holds. This read `[]` while the provenance walk had no
         * case for an aggregate, which offered both rows read-only while one of them is written.
         * Both stay named because the actual is an identifier the edge cannot decompose; naming
         * `first` alone would need the local's own property structure, which is a separate
         * question from carrying its origins at all. */
        expect(localLiteral,).toEqual([
          0,
          1,
        ],);
        /* A class expression is neither a callable the walk routes to a body scan nor a literal
         * it descends, so a row held by a static member was reachable by the callee and invisible
         * to the walk. It read no written parameter while the callee writes through `holder.row`,
         * which offered a row something mutates. */
        expect(classMember,).toEqual([0,],);
        /* Assignment through a declared setter, pinned because it was suspected of losing the
         * write and does not. The store names its receiver whether or not the setter body is
         * inspected, which is the sound answer: running the setter changes state the receiver
         * controls.
         *
         * Retention names the receiver alone, and that is also sound. `holder.latest = row`
         * stores `row` inside `holder` without mutating `row`, and annotating `row` deeply
         * readonly would make the store a type error, so the offer this leaves available cannot
         * be taken without TypeScript objecting. A setter body writing a different parameter
         * would be a real gap and cannot be built in one callable: the holder would have to have
         * captured that parameter already, which the analysis sees as the capture. */
        expect(throughSetter,).toEqual([0,],);
        expect(setterRetention,).toEqual([0,],);
        /* Both halves of a write made through an explicit `this`. The callable side reported
         * nothing because a `this` expression is not an identifier, so the provenance walk had
         * no root to resolve; the caller side reported nothing because the receiver is the value
         * before the dot and the formal-to-actual mapping has no position for it. Each offered a
         * row that is written, and the caller half was untestable until the callable half landed,
         * since an empty effect set propagates as nothing whatever the edge does with it. */
        expect(throughThis,).toEqual([0,],);
        expect(thisReceiver,).toEqual([0,],);
        /* A callee invoking a callback it destructured, with the row that callback writes
         * packaged beside it. The edge cannot name which body runs, because the actual at that
         * argument position is an object literal rather than a callable, so the invocation is
         * unresolved and the row it reaches takes opacity. It read no written parameter and no
         * opacity at all before, which offered a row the packaged callback mutates.
         *
         * Opacity rather than a write on purpose: nothing here proved what the callback does,
         * only that this rule cannot say. Naming the declaration a caller packaged in a property
         * would prove it, and is tracked separately. */
        expect(packagedCallback,).toEqual([0,],);
        expect(packagedCallbackWritten,).toEqual([],);
        /* A rest formal's key `0` names an array index rather than a property of the literal, so
         * resolving it against the literal finds nothing. One parameter here, and it has to stay
         * named: this catches the empty result, which is the dangerous one. */
        expect(restIndex,).toEqual([0,],);
        /* Two literals reaching the same rest formal, which the single-literal shape cannot test:
         * every actual that can fill a rest formal contributes to its property slots, not only
         * the one whose position matches. The callee writes through rest index zero, so naming
         * the second row too is the stated conservatism, and naming only the second would be the
         * inversion this rules out. */
        expect(restIndexSpread,).toEqual([
          0,
          1,
        ],);
      },
    },),
  ],
},);
