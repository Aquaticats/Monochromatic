/**
 Foreign ownership coverage through bindings and packaged expressions.
 
 @module
 */

import type {
  BindingName,
  Node,
} from 'typescript/ast';
import {
  isArrayLiteralExpression,
  isBindingElement,
  isElementAccessExpression,
  isIdentifier,
  isObjectLiteralExpression,
  isPropertyAccessExpression,
  isPropertyAssignment,
  isShorthandPropertyAssignment,
  isSpreadAssignment,
  isSpreadElement,
} from 'typescript/ast/is';
import type {
  Project,
  Type,
} from 'typescript/sync';

import { isForeignBorrowedType, } from './foreign-borrowed-identity.ts';
import { classifyReadonlyType, } from './readonly-classifier.ts';
//region Ownership coverage
// The three coverages a type's caller-observable mutable state can have.

/**
 Ownership coverage of caller-observable mutable state in one expression.
 */
type ForeignCoverage = 'foreign' | 'owned' | 'primitive';

//endregion Ownership coverage

//region Classifying one type
// The semantic classifier, and the combination that keeps one foreign field from
// hiding owned state beside it.

/**
 Classifies semantic type by foreign, owned, or primitive state coverage.
 
 @param project - TypeScript project resolving marker and primitive identity.
 
 @param type - Semantic type to classify.
 
 @returns ownership coverage.
 */
function typeForeignCoverage({
  project,
  type,
}: {
  readonly project: Project;
  readonly type: Type;
},): ForeignCoverage {
  if (isForeignBorrowedType({
    project,
    type,
  },))
    return 'foreign';
  /**
   Readonly contract classification for unmarked value.
   */
  const classification = classifyReadonlyType({
    checker: project.checker,
    project,
    type,
  },);
  return classification.kind === 'deep-readonly'
    ? 'primitive'
    : 'owned';
}

/**
 Combines nested value coverage without letting one foreign field hide owned state.
 
 @param coverages - Nested property or element coverage.
 
 @returns combined ownership coverage.
 */
function combinedForeignCoverage(
  coverages: readonly ForeignCoverage[],
): ForeignCoverage {
  if (coverages.includes('owned',))
    return 'owned';
  return coverages.includes('foreign',) ? 'foreign' : 'primitive';
}

//endregion Classifying one type

//region Classifying bindings
// Parameter bindings and their destructuring, classified by mutable ownership.

/**
 Classifies parameter binding and nested destructuring by mutable ownership.
 
 @param project - TypeScript project resolving binding types.
 
 @param name - Parameter binding identifier or destructuring pattern.
 
 @returns combined ownership coverage of bound values.
 */
function bindingForeignCoverage({
  project,
  name,
}: {
  readonly project: Project;
  readonly name: BindingName;
},): ForeignCoverage {
  if (isIdentifier(name,)) {
    /**
     Semantic type of current bound identifier.
     */
    const type = project.checker
      .getTypeAtLocation(name,);
    return type === undefined
      ? 'owned'
      : typeForeignCoverage({
        project,
        type,
      },);
  }
  return combinedForeignCoverage(name.elements
    .filter(function definedBindingElement(element,): element is typeof element & { readonly name: BindingName; } {
      return isBindingElement(element,) && (element.name !== undefined);
    },)
    .map(function nestedBindingCoverage(element,): ForeignCoverage {
      return bindingForeignCoverage({
        project,
        name: element.name,
      },);
    },),);
}

//endregion Classifying bindings

//region Classifying expressions
// Authored expressions and packaged values in the same terms.

/**
 Tests whether all mutable state in parameter binding is explicitly foreign-owned.
 
 @param project - TypeScript project resolving binding types.
 
 @param name - Parameter binding identifier or destructuring pattern.
 
 @returns whether every mutable bound value has exact marker provenance.
 
 @example
 ```ts
 bindingContainsForeignBorrowed({ project, name: parameter.name });
 ```
 */
export function bindingContainsForeignBorrowed({
  project,
  name,
}: {
  readonly project: Project;
  readonly name: BindingName;
},): boolean {
  return bindingForeignCoverage({
    project,
    name,
  },) === 'foreign';
}

/**
 Classifies authored expression and packaged values by mutable ownership.
 
 @param project - TypeScript project resolving exact marker identity.
 
 @param node - Call argument or receiver expression to inspect.
 
 @returns ownership coverage of expression.
 */
function expressionForeignCoverage({
  project,
  node,
}: {
  readonly project: Project;
  readonly node: Node;
},): ForeignCoverage {
  /**
   Semantic type directly attached to current expression.
   */
  const directType = project.checker
    .getTypeAtLocation(node,);
  if ((directType !== undefined) && isForeignBorrowedType({
    project,
    type: directType,
  },))
    return 'foreign';
  if (isPropertyAccessExpression(node,) || isElementAccessExpression(node,)) {
    /**
     Ownership coverage retained from property or element receiver.
     */
    const rootCoverage = expressionForeignCoverage({
      project,
      node: node.expression,
    },);
    if (rootCoverage === 'foreign')
      return 'foreign';
  }
  if (isObjectLiteralExpression(node,)) {
    return combinedForeignCoverage(node.properties
      .map(function propertyCoverage(property,): ForeignCoverage {
        if (isPropertyAssignment(property,)) {
          return expressionForeignCoverage({
            project,
            node: property.initializer,
          },);
        }
        if (isShorthandPropertyAssignment(property,)) {
          /**
           Value symbol hidden by shorthand property symbol.
           */
          const valueSymbol = project.checker
            .getShorthandAssignmentValueSymbol(property,);
          /**
           Semantic type of shorthand value.
           */
          const valueType = valueSymbol === undefined
            ? undefined
            : project.checker
              .getTypeOfSymbolAtLocation(
              valueSymbol,
              property.name,
              );
          return valueType === undefined
            ? 'owned'
            : typeForeignCoverage({
              project,
              type: valueType,
            },);
        }
        return isSpreadAssignment(property,)
          ? expressionForeignCoverage({
            project,
            node: property.expression,
          },)
          : 'primitive';
      },),);
  }
  if (isArrayLiteralExpression(node,)) {
    return combinedForeignCoverage(node.elements
      .map(function elementCoverage(element,): ForeignCoverage {
        return expressionForeignCoverage({
          project,
          node: isSpreadElement(element,) ? element.expression : element,
        },);
      },),);
  }
  return directType === undefined
    ? 'owned'
    : typeForeignCoverage({
      project,
      type: directType,
    },);
}

//endregion Classifying expressions

//region Call arguments
// Whether every mutable state an argument carries is foreign-owned.

/**
 Tests whether all mutable state in call argument has foreign provenance.
 
 @param project - TypeScript project resolving exact marker identity.
 
 @param node - Call argument or receiver expression to inspect.
 
 @returns whether every mutable value is foreign-owned.
 
 @example
 ```ts
 expressionContainsForeignBorrowed({ project, node: argument });
 ```
 */
export function expressionContainsForeignBorrowed({
  project,
  node,
}: {
  readonly project: Project;
  readonly node: Node;
},): boolean {
  return expressionForeignCoverage({
    project,
    node,
  },) === 'foreign';
}
//endregion Call arguments
