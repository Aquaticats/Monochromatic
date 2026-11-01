/**
 Deep readonly and capability classification over TypeScript 7 semantic types.
 
 @module
 */

import {
  SignatureKind,
  SymbolFlags,
  TypeFlags,
  type Checker,
  type Project,
  type Symbol as TypeScriptSymbol,
  type Type,
} from 'typescript/sync';

import { SemanticBridgeError, } from './semantic-bridge-error.ts';
import { isForeignBorrowedType, } from './foreign-borrowed-identity.ts';
import {
  readonlyOwnerName,
  typeClaimsReadonlyProjection,
} from './readonly-owner.ts';
import { declarationIsReadonly, } from './readonly-declaration.ts';
import {
  CALLABLE_CAPABILITY,
  typeHasCallableCapability,
} from './readonly-callable-capability.ts';
import {
  combineClassifications,
  DEEP_READONLY,
} from './readonly-classification-combine.ts';
import {
  classifyReadonlyCollection,
  READONLY_COLLECTION_CLASSIFICATION_UNAVAILABLE,
} from './readonly-collection-classification.ts';
import {
  mutableReadonlyClassification,
  prefixReadonlyClassification,
  type ReadonlyClassification,
} from './readonly-classification-model.ts';
import {
  writableIndexOwners,
  writablePropertyOwners,
} from './readonly-writable-declaration.ts';

/**
 Bit position of hidden TypeScript 7 mapped-property readonly state.
 */
const CHECK_FLAGS_READONLY_BIT = 3;

/**
 Hidden TypeScript 7 mapped-property readonly bit audited from upstream Go source.
 */
const CHECK_FLAGS_READONLY = 1 << CHECK_FLAGS_READONLY_BIT;

/**
 Sentinel while recursive type is being classified.
 */
const CLASSIFICATION_ACTIVE: unique symbol = Symbol('ReadonlyClassification traversal active for type',);

export type { ReadonlyClassification, } from './readonly-classification-model.ts';

/**
 Determines whether property symbol is declared or mapped readonly.
 
 @param project - TypeScript project resolving declaration handles.
 
 @param property - Property symbol to inspect.
 
 @returns whether every declaration is readonly or mapped readonly bit is set.
 
 @throws {@link SemanticBridgeError} when unstable check flag is unavailable.
 
 @example
 ```ts
 const readonly = propertyIsReadonly({ project, property });
 ```
 */
export function propertyIsReadonly({
  project,
  property,
}: {
  readonly project: Project;
  readonly property: TypeScriptSymbol;
},): boolean {
  if ((typeof property.checkFlags) !== 'number') {
    throw new SemanticBridgeError({
      reason: 'readonly-capability-unavailable',
      message: 'TypeScript Symbol.checkFlags is unavailable for readonly classification.',
    },);
  }
  if ((property.checkFlags & CHECK_FLAGS_READONLY) !== 0)
    return true;
  if (property
    .declarations
    .length
    === 0)
    return false;
  return property
    .declarations
    .every(function declarationReadonly(handle,): boolean {
      /**
       Declaration node resolved in owning project.
       */
      const declaration = handle.resolve(project,);
      return (declaration !== undefined)
        && declarationIsReadonly(declaration,);
    },);
}

/**
 Settled classifications by semantic type ID, per project.
 
 The memo inside `classifyReadonlyType` is created per call, which is correct for the cycle
 marker it also holds and far narrower than the repetition it faces: a `Row` named by two
 hundred callables was walked two hundred times, and each walk redid every property, element
 and signature beneath it. Measured at 4.2ms per callable in
 `doc/planning/oxlint-warm-sweep-attribution.md`.
 
 Safe to share because the classification depends on nothing but the type. Every use of
 `checker` and `project` inside the walk is derived from the type being classified: its own
 declaration handle, its base constraint, whether it is an array, its type arguments.
 
 Keyed on the project because a type ID means nothing outside the checker that issued it, which
 is the same boundary the result is valid within, so the key cannot collide across instances.
 `effect-final-index-cache.ts` keys its own store the same way.
 
 Withholding an assumed result is what makes the store an identity rather than an accident of
 order. `classify` answers `HONEST_READONLY` for a type already being walked above it, and that
 answer is only resolved by the walk that made the assumption. Every other member of the cycle
 finishes standing on it, so publishing those would let whichever parameter a worker classified
 first decide what a later one means.
 */
const settledClassificationsByProject = new WeakMap<
  Project,
  Map<number, ReadonlyClassification>
  >();

/**
 One finished classification and whether an unresolved type above it decided the answer.
 */
type TraversalOutcome = {
  readonly result: ReadonlyClassification;
  readonly assumed: boolean;
};

/**
 Classifies deep readonly soundy for one resolved type graph.
 
 Recursive calls are bounded by unique TypeScript type IDs and break cycles
 through active-state memoization.
 
 @param checker - TypeScript checker owning type graph.
 
 @param project - TypeScript project resolving symbols and declarations.
 
 @param type - Parameter type to classify.
 
 @returns deep readonly and capability classification.
 
 @example
 ```ts
 const classification = classifyReadonlyType({ checker, project, type });
 ```
 */
export function classifyReadonlyType({
  checker,
  project,
  type,
}: {
  readonly checker: Checker;
  readonly project: Project;
  readonly type: Type;
},): ReadonlyClassification {
  /**
   Memoized outcome or active traversal marker by semantic type ID.
   */
  const memo = new Map<number, TraversalOutcome | typeof CLASSIFICATION_ACTIVE>();
  /**
   How many answers this walk took from a type it had not finished.
   
   Compared before and after a type's own walk, so an unchanged count proves nothing beneath it
   stood on an assumption. Counting rather than flagging keeps nested walks independent: an
   inner cycle taints its own members without tainting a sibling that never met one.
   */
  const assumptions = { count: 0, };
  /**
   Classifications this project has already settled, shared across every call.
   */
  const settled = settledClassificationsByProject.get(project,)
    ?? new Map<number, ReadonlyClassification>();
  settledClassificationsByProject.set(
    project,
    settled,
  );

  /**
   Recursively classifies one type with cycle-aware memoization.
   
   @param current - Current semantic type node.
   
   @returns classification for current graph root.
   */
  function classify(current: Type,): ReadonlyClassification {
    /**
     Result this project settled for the type on an earlier call, when it has.
     */
    const shared = settled.get(current.id,);
    if (shared !== undefined)
      return shared;
    /**
     Prior outcome or active cycle marker for current type ID.
     */
    const cached = memo.get(current.id,);
    if (cached === CLASSIFICATION_ACTIVE) {
      assumptions.count += 1;
      return DEEP_READONLY;
    }
    if (cached !== undefined) {
      /* Reading an assumed outcome carries the assumption to whoever reads it. Without this a
       * sibling reached after the cycle closed would look unconditional, since it meets the
       * finished entry rather than the marker that produced it. */
      if (cached.assumed)
        assumptions.count += 1;
      return cached.result;
    }
    /**
     Assumption count before this type's own walk, for comparison once it finishes.
     */
    const assumptionsBefore = assumptions.count;
    memo.set(
      current.id,
      CLASSIFICATION_ACTIVE,
    );

    /**
     Stores completed classification for current type.
     
     @param result - Completed current-type result.
     
     @returns same result after memoization.
     */
    function finish(result: ReadonlyClassification,): ReadonlyClassification {
      /**
       Whether anything beneath this type answered from a type still being walked.
       */
      const assumed = assumptions.count !== assumptionsBefore;
      memo.set(
        current.id,
        {
          result,
          assumed,
        },
      );
      if (!assumed) {
        settled.set(
          current.id,
          result,
        );
      }
      return result;
    }

    if (isForeignBorrowedType({
      project,
      type: current,
    },)) {
      return finish({
        kind: 'opaque-capability',
        reason: 'foreign API dictates mutable borrowed handle contract',
      },);
    }

    if ((current.flags & (TypeFlags.Any | TypeFlags.Unknown)) !== 0) {
      return finish({
        kind: 'opaque-capability',
        reason: 'unknown runtime value may carry mutable caller-owned capability',
      },);
    }

    if (current.isUnionType() || current.isIntersectionType()) {
      /**
       Constituent classifications for combined type.
       */
      const constituentResults = current.getTypes()
        .map(classify,);
      return finish(combineClassifications(constituentResults,),);
    }
    if (current.isTypeParameter()) {
      /**
       Base constraint providing reachable shape when available.
       */
      const constraint = checker.getBaseConstraintOfType(current,);
      return finish(constraint === undefined
        ? {
          kind: 'opaque-capability',
          reason: 'unconstrained type parameter has unknown reachable state',
        }
        : classify(constraint,),);
    }
    if ((current.flags & TypeFlags.NonPrimitive) !== 0) {
      return finish({
        kind: 'opaque-capability',
        reason: 'broad object type may carry caller-defined properties, accessors, or proxy capability',
      },);
    }
    if (typeHasCallableCapability({
      checker,
      type: current,
    },))
      return finish(CALLABLE_CAPABILITY,);
    if (!current.isObjectType())
      return finish(DEEP_READONLY,);
    /**
     Declared owner identity used by standard projection policy.
     */
    const currentOwner = readonlyOwnerName(current,);
    /**
     Collection-specific mutability and reachable element classification.
     */
    const collectionClassification = classifyReadonlyCollection({
      checker,
      type: current,
      owner: currentOwner,
      classify,
    },);
    if (collectionClassification !== READONLY_COLLECTION_CLASSIFICATION_UNAVAILABLE)
      return finish(collectionClassification,);

    /**
     Whether authored type claims readonly projection semantics.
     */
    const projectionClaimed = typeClaimsReadonlyProjection(current,);
    /**
     Classification of every reachable named property.
     */
    const propertyResults = checker.getPropertiesOfType(current,)
      .map(function classifyProperty(property,): ReadonlyClassification {
        /**
         Whether symbol is declared as method rather than data property.
         */
        const isMethod = (property.flags & SymbolFlags.Method) !== 0;
        /**
         Value declaration candidate before type-only fallback.
         */
        const declarationHandleResult = { value: property.valueDeclaration, };
        declarationHandleResult.value ??= property
          .declarations
          .at(0,);
        /**
         Value declaration or first type-only declaration fallback.
         */
        const { value: declarationHandle, } = declarationHandleResult;
        if (declarationHandle === undefined) {
          return {
            kind: 'opaque-capability',
            reason: `property ${property.name} has no resolvable declaration`,
          };
        }
        /**
         Declaration node used for property type and provenance.
         */
        const declaration = declarationHandle.resolve(project,);
        if (declaration === undefined) {
          return {
            kind: 'opaque-capability',
            reason: `property ${property.name} declaration cannot be resolved`,
          };
        }
        /**
         Property type at exact declaring location.
         */
        const propertyType = checker.getTypeOfSymbolAtLocation(
          property,
          declaration,
        );
        /**
         Call signatures retained by ordinary and mapped method properties.
         */
        const callSignatures = checker.getSignaturesOfType(
          propertyType,
          SignatureKind.Call,
        );
        /**
         Whether property is behavior rather than assignable data slot.
         */
        const callable = isMethod || (callSignatures.length > 0);
        if (callable) {
          return projectionClaimed
            ? {
              kind: 'projected-readonly-capability',
              reason: `${currentOwner}.${property.name} retains unknown callable capability`,
            }
            : {
              kind: 'opaque-capability',
              reason: `${currentOwner}.${property.name} has unresolved callable effect`,
            };
        }
        /**
         Access segment retained whether property itself or nested state is writable.
         */
        const segment = {
          kind: 'property' as const,
          name: property.name,
        };
        if (!propertyIsReadonly({
          project,
          property,
        },)) {
          return mutableReadonlyClassification({
            kind: 'property',
            segments: [segment,],
            declarationOwners: writablePropertyOwners({
              property,
              project,
            },),
          },);
        }
        return prefixReadonlyClassification({
          classification: classify(propertyType,),
          segment,
        },);
      },);
    /**
     Classification of every reachable index signature.
     */
    const indexResults = checker.getIndexInfosOfType(current,)
      .map(function classifyIndex(indexInfo,): ReadonlyClassification {
        /**
         Access segment for index key type rendered by semantic checker.
         */
        const segment = {
          kind: 'index' as const,
          keyType: checker.typeToString(indexInfo.keyType,),
        };
        if (!indexInfo.isReadonly) {
          return mutableReadonlyClassification({
            kind: 'index',
            segments: [segment,],
            declarationOwners: writableIndexOwners({
              indexInfo,
              project,
            },),
          },);
        }
        return prefixReadonlyClassification({
          classification: classify(indexInfo.valueType,),
          segment,
        },);
      },);
    return finish(combineClassifications([
      ...propertyResults,
      ...indexResults,
    ],),);
  }

  /**
   Classification for the type this call was asked about.
   */
  const requested = classify(type,);
  /* Published whether or not it stood on an assumption, unlike everything beneath it. A walk
   * starting here is what an unshared classifier computes for this type, since the memo it starts
   * from is empty, so recording it under its own identity repeats that answer rather than
   * inventing one. The assumption a cycle head makes is about itself, and its own walk resolves
   * it; the members below it are the ones left holding an answer nothing resolved. */
  settled.set(
    type.id,
    requested,
  );
  return requested;
}
