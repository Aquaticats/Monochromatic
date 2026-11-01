/**
 Deep readonly classification for standard collection and tuple shapes.
 
 @module
 */

import type {
  Checker,
  Type,
  TypeReference,
} from 'typescript/sync';

import { combineClassifications, } from './readonly-classification-combine.ts';
import {
  mutableReadonlyClassification,
  prefixReadonlyClassification,
  type ReadonlyClassification,
} from './readonly-classification-model.ts';

/**
 Sentinel when type is not a collection shape handled by this module.
 */
export const READONLY_COLLECTION_CLASSIFICATION_UNAVAILABLE: unique symbol = Symbol(
  'readonly collection classification unavailable',
);

/**
 Standard readonly generic collections classified through reachable type arguments.
 */
const READONLY_GENERIC_COLLECTION_OWNERS: ReadonlySet<string> = new Set([
  'ReadonlyMap',
  'ReadonlySet',
]);

/**
 Classifier callback for recursively reached collection values.
 */
type NestedReadonlyClassifier = (type: Type) => ReadonlyClassification;

/**
 Classifies mutable array container and nested indexed values.
 
 @param checker - Semantic checker exposing index types.
 
 @param type - Standard mutable array type.
 
 @param classify - Recursive classifier for element values.
 
 @returns combined container and element classification.
 */
function classifyMutableArray({
  checker,
  type,
  classify,
}: {
  readonly checker: Checker;
  readonly type: Type;
  readonly classify: NestedReadonlyClassifier;
}): ReadonlyClassification {
  return combineClassifications([
    mutableReadonlyClassification({
      kind: 'array',
      segments: [],
      declarationOwners: [],
    },),
    ...checker
      .getIndexInfosOfType(type,)
      .map(function classifyArrayValue(indexInfo,): ReadonlyClassification {
        return prefixReadonlyClassification({
          classification: classify(indexInfo.valueType,),
          segment: {
            kind: 'index',
            keyType: checker.typeToString(indexInfo.keyType,),
          },
        },);
      },),
  ],);
}

/**
 Classifies readonly array indexed values.
 
 @param checker - Semantic checker exposing index types.
 
 @param type - Standard readonly array type.
 
 @param classify - Recursive classifier for element values.
 
 @returns combined nested element classification.
 */
function classifyReadonlyArray({
  checker,
  type,
  classify,
}: {
  readonly checker: Checker;
  readonly type: Type;
  readonly classify: NestedReadonlyClassifier;
}): ReadonlyClassification {
  return combineClassifications(checker
    .getIndexInfosOfType(type,)
    .map(function classifyArrayValue(indexInfo,): ReadonlyClassification {
      return prefixReadonlyClassification({
        classification: classify(indexInfo.valueType,),
        segment: {
          kind: 'index',
          keyType: checker.typeToString(indexInfo.keyType,),
        },
      },);
    },),);
}

/**
 Classifies readonly generic collection key and value arguments.
 
 @param checker - Semantic checker exposing type arguments.
 
 @param type - Readonly map or set reference.
 
 @param owner - Standard collection owner name.
 
 @param classify - Recursive classifier for reachable values.
 
 @returns combined key and value classification.
 */
function classifyReadonlyGeneric({
  checker,
  type,
  owner,
  classify,
}: {
  readonly checker: Checker;
  readonly type: TypeReference;
  readonly owner: string;
  readonly classify: NestedReadonlyClassifier;
}): ReadonlyClassification {
  return combineClassifications(checker
    .getTypeArguments(type,)
    .map(function classifyCollectionArgument(
      argument,
      index,
    ): ReadonlyClassification {
      return prefixReadonlyClassification({
        classification: classify(argument,),
        segment: {
          kind: 'index',
          keyType: owner === 'ReadonlyMap'
            ? (index === 0 ? 'key' : 'value')
            : 'value',
        },
      },);
    },),);
}

/**
 Classifies tuple mutability and every indexed element.
 
 @param checker - Semantic checker exposing tuple arguments.
 
 @param type - Semantic tuple type.
 
 @param classify - Recursive classifier for tuple elements.
 
 @returns combined tuple and element classification.
 */
function classifyTuple({
  checker,
  type,
  classify,
}: {
  readonly checker: Checker;
  readonly type: Type;
  readonly classify: NestedReadonlyClassifier;
}): ReadonlyClassification {
  if (!type.isTupleType())
    return { kind: 'deep-readonly', };
  /**
   Element causes addressed by tuple position.
   */
  const tupleElementResults = checker
    .getTypeArguments(type,)
    .map(function classifyTupleElement(
      element,
      index,
    ): ReadonlyClassification {
      return prefixReadonlyClassification({
        classification: classify(element,),
        segment: {
          kind: 'index',
          keyType: String(index,),
        },
      },);
    },);
  return combineClassifications([
    ...type
      .getTarget()
      .readonly
      ? []
      : [mutableReadonlyClassification({
        kind: 'tuple',
        segments: [],
        declarationOwners: [],
      },),],
    ...tupleElementResults,
  ],);
}

/**
 Classifies collection shape or returns unavailable sentinel for ordinary objects.
 
 @param checker - Semantic checker owning collection type.
 
 @param type - Current type candidate.
 
 @param owner - Declared type owner name.
 
 @param classify - Recursive classifier for reachable values.
 
 @returns complete collection classification or unavailable sentinel.
 
 @example
 ```ts
 classifyReadonlyCollection({ checker, type, owner, classify });
 ```
 */
export function classifyReadonlyCollection({
  checker,
  type,
  owner,
  classify,
}: {
  readonly checker: Checker;
  readonly type: Type;
  readonly owner: string;
  readonly classify: NestedReadonlyClassifier;
}): ReadonlyClassification | typeof READONLY_COLLECTION_CLASSIFICATION_UNAVAILABLE {
  if (checker.isArrayType(type,) && (owner === 'Array')) {
    return classifyMutableArray({
      checker,
      type,
      classify,
    },);
  }
  if (READONLY_GENERIC_COLLECTION_OWNERS.has(owner,) && type.isTypeReference()) {
    return classifyReadonlyGeneric({
      checker,
      type,
      owner,
      classify,
    },);
  }
  if (owner === 'ReadonlyArray') {
    return classifyReadonlyArray({
      checker,
      type,
      classify,
    },);
  }
  if (type.isTupleType()) {
    return classifyTuple({
      checker,
      type,
      classify,
    },);
  }
  return READONLY_COLLECTION_CLASSIFICATION_UNAVAILABLE;
}
