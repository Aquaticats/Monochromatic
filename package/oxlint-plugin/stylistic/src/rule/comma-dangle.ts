import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import type {
  Context,
  CreateOnceRule,
  Node,
  VisitorWithHooks,
} from '@oxlint/plugins';

import {
  checkTrailingComma,
  lastEnumMember,
  lastFieldNode,
  lastImportExpressionItem,
  lastNamedImportSpecifier,
} from '../utility/comma-dangle.ts';

//region Rule
// The exported rule: metadata, one delegation per container kind, and the
// visitor that maps every comma-delimited list onto its check.

/**
 Requires trailing commas at every supported comma-delimited list tail.
 
 This mirrors `@stylistic/comma-dangle` in its plain `"always"` behavior
 without exposing configuration. Empty lists are ignored, and final rest
 elements are ignored because JavaScript grammar rejects a comma after them.
 
 @example
 ```ts
 // Bad
 const value = [one];
 function read(value: string): void {}
 
 // Good
 const value = [one,];
 function read(value: string,): void {}
 ```
 */
export const commaDangle: CreateOnceRule = {
  meta: {
    type: 'layout',
    fixable: 'code',
    schema: [],
    docs: {
      description: 'Require trailing commas in supported comma-delimited lists.',
      recommended: true,
    },
    messages: {
      missingComma: 'Missing trailing comma.',
    },
  },
  /**
   Handles effectful plugin callback.
   
   @param context - Foreign callback value carrying diagnostic capability.
   
   @mutates context - Emits Oxlint diagnostics through foreign rule context.
   
   @example
   ```ts
   createOnce(context);
   ```
   */
  createOnce(context: ForeignBorrowed<Context>,): VisitorWithHooks {
    //region Element, property, and argument lists
    // `[...]`, `{...}`, and call arguments read their last item as a field of
    // the container, with the container's own penultimate token as the anchor.

    /**
     Checks array element lists and array-pattern element lists.
     
     Delegates to {@link checkTrailingComma}, locating the last element via
     {@link lastFieldNode}.
     
     @param node - array-like container node
     */
    function checkElements(node: ForeignBorrowed<Node>,): void {
      checkTrailingComma({
        context,
        container: node,
        lastItem: lastFieldNode({
          node,
          fieldName: 'elements',
        },),
        useContainerPenultimateToken: true,
      },);
    }

    /**
     Checks object property lists and object-pattern property lists.
     
     Delegates to {@link checkTrailingComma}, locating the last property via
     {@link lastFieldNode}.
     
     @param node - object-like container node
     */
    function checkProperties(node: ForeignBorrowed<Node>,): void {
      checkTrailingComma({
        context,
        container: node,
        lastItem: lastFieldNode({
          node,
          fieldName: 'properties',
        },),
        useContainerPenultimateToken: true,
      },);
    }

    /**
     Checks call or constructor arguments.
     
     Delegates to {@link checkTrailingComma}, locating the last argument via
     {@link lastFieldNode}.
     
     @param node - call-like node
     */
    function checkArguments(node: ForeignBorrowed<Node>,): void {
      checkTrailingComma({
        context,
        container: node,
        lastItem: lastFieldNode({
          node,
          fieldName: 'arguments',
        },),
        useContainerPenultimateToken: true,
      },);
    }
    //endregion Element, property, and argument lists

    //region Parameter lists
    // Every function-like node shares one check, which reads its `params` field.

    /**
     Checks function-like parameter lists.
     
     Delegates to {@link checkTrailingComma}, locating the last parameter via
     {@link lastFieldNode}.
     
     @param node - function-like node
     */
    function checkParams(node: ForeignBorrowed<Node>,): void {
      checkTrailingComma({
        context,
        container: node,
        lastItem: lastFieldNode({
          node,
          fieldName: 'params',
        },),
      },);
    }
    //endregion Parameter lists

    //region Import, export, and dynamic-import lists
    // A declaration can carry two comma-delimited lists: its specifiers and its
    // import attributes, so each check reports both.

    /**
     Checks import specifiers and import attributes.
     
     Delegates to {@link checkTrailingComma}, locating the last specifier via
     {@link lastNamedImportSpecifier} and the last attribute via {@link lastFieldNode}.
     
     @param node - import declaration node
     */
    function checkImportDeclaration(node: ForeignBorrowed<Node>,): void {
      checkTrailingComma({
        context,
        container: node,
        lastItem: lastNamedImportSpecifier(node,),
      },);
      checkTrailingComma({
        context,
        container: node,
        lastItem: lastFieldNode({
          node,
          fieldName: 'attributes',
        },),
      },);
    }

    /**
     Checks export specifiers and import attributes.
     
     Delegates to {@link checkTrailingComma}, locating the last specifier and
     the last attribute via {@link lastFieldNode}.
     
     @param node - export named declaration node
     */
    function checkExportNamedDeclaration(node: ForeignBorrowed<Node>,): void {
      checkTrailingComma({
        context,
        container: node,
        lastItem: lastFieldNode({
          node,
          fieldName: 'specifiers',
        },),
      },);
      checkTrailingComma({
        context,
        container: node,
        lastItem: lastFieldNode({
          node,
          fieldName: 'attributes',
        },),
      },);
    }

    /**
     Checks export-all import attributes.
     
     Delegates to {@link checkTrailingComma}, locating the last attribute via
     {@link lastFieldNode}.
     
     @param node - export all declaration node
     */
    function checkExportAllDeclaration(node: ForeignBorrowed<Node>,): void {
      checkTrailingComma({
        context,
        container: node,
        lastItem: lastFieldNode({
          node,
          fieldName: 'attributes',
        },),
      },);
    }

    /**
     Checks dynamic import arguments.
     
     Delegates to {@link checkTrailingComma}, locating the last argument via
     {@link lastImportExpressionItem}.
     
     @param node - dynamic import expression node
     */
    function checkImportExpression(node: ForeignBorrowed<Node>,): void {
      checkTrailingComma({
        context,
        container: node,
        lastItem: lastImportExpressionItem(node,),
        useContainerPenultimateToken: true,
      },);
    }
    //endregion Import, export, and dynamic-import lists

    //region Enum and type lists
    // TypeScript-only containers: enum members, type parameters, and tuple
    // element types.

    /**
     Checks enum body members.
     
     Delegates to {@link checkTrailingComma}, locating the last member via
     {@link lastEnumMember}.
     
     @param node - enum declaration node
     */
    function checkEnumDeclaration(node: ForeignBorrowed<Node>,): void {
      checkTrailingComma({
        context,
        container: node,
        lastItem: lastEnumMember(node,),
      },);
    }

    /**
     Checks type parameter declarations.
     
     Delegates to {@link checkTrailingComma}, locating the last parameter via
     {@link lastFieldNode}.
     
     @param node - type parameter declaration node
     */
    function checkTypeParameters(node: ForeignBorrowed<Node>,): void {
      checkTrailingComma({
        context,
        container: node,
        lastItem: lastFieldNode({
          node,
          fieldName: 'params',
        },),
      },);
    }

    /**
     Checks tuple element types.
     
     Delegates to {@link checkTrailingComma}, locating the last element type via
     {@link lastFieldNode}.
     
     @param node - tuple type node
     */
    function checkTupleElements(node: ForeignBorrowed<Node>,): void {
      checkTrailingComma({
        context,
        container: node,
        lastItem: lastFieldNode({
          node,
          fieldName: 'elementTypes',
        },),
      },);
    }
    //endregion Enum and type lists

    //region Visitor wiring
    // One entry per container kind, several of them sharing a single check.

    return {
      ArrayExpression: checkElements,
      ObjectExpression: checkProperties,
      ArrayPattern: checkElements,
      ObjectPattern: checkProperties,
      ImportDeclaration: checkImportDeclaration,
      ExportNamedDeclaration: checkExportNamedDeclaration,
      ExportAllDeclaration: checkExportAllDeclaration,
      FunctionDeclaration: checkParams,
      FunctionExpression: checkParams,
      ArrowFunctionExpression: checkParams,
      CallExpression: checkArguments,
      NewExpression: checkArguments,
      ImportExpression: checkImportExpression,
      TSEnumDeclaration: checkEnumDeclaration,
      TSTypeParameterDeclaration: checkTypeParameters,
      TSTupleType: checkTupleElements,
      TSDeclareFunction: checkParams,
      TSEmptyBodyFunctionExpression: checkParams,
      TSFunctionType: checkParams,
      TSMethodSignature: checkParams,
      TSCallSignatureDeclaration: checkParams,
      TSConstructSignatureDeclaration: checkParams,
      TSConstructorType: checkParams,
    };
    //endregion Visitor wiring
  },
};
//endregion Rule
