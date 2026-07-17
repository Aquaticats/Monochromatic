/**
 * Exact package-owned intrinsic effects.
 *
 * @module
 */

import type { IntrinsicEffectEntry, } from './intrinsic-effect-catalog.ts';
import { LEZER_PACKAGE_EFFECTS, } from './lezer-package-effect-catalog.ts';
import { NANO_SPAWN_PACKAGE_EFFECTS, } from './nano-spawn-package-effect-catalog.ts';
import { OPTIQUE_PACKAGE_EFFECTS, } from './optique-package-effect-catalog.ts';
import { receiverEffect, } from './package-receiver-effect.ts';
import { PI_PACKAGE_EFFECTS, } from './pi-package-effect-catalog.ts';
import { POSTCSS_PACKAGE_EFFECTS, } from './postcss-package-effect-catalog.ts';
import { TURSO_PACKAGE_EFFECTS, } from './turso-package-effect-catalog.ts';

/**
 * Package effects audited by exact current-lock major.
 */
export const PACKAGE_EFFECTS: readonly IntrinsicEffectEntry[] = [
  ...LEZER_PACKAGE_EFFECTS,
  ...NANO_SPAWN_PACKAGE_EFFECTS,
  ...OPTIQUE_PACKAGE_EFFECTS,
  ...PI_PACKAGE_EFFECTS,
  ...POSTCSS_PACKAGE_EFFECTS,
  ...TURSO_PACKAGE_EFFECTS,
  receiverEffect({
    provenance: {
      kind: 'package',
      packageName: '@oxlint/plugins',
      major: 1,
    },
    ownerType: 'Context',
    member: 'report',
    evidence: '@oxlint/plugins 1.73 Context report diagnostic emission',
  },),
  ...[
    'add',
    'ignores',
    'test',
  ].map(function ignoreMatcherMutation(member,): IntrinsicEffectEntry {
    return receiverEffect({
      provenance: {
        kind: 'package',
        packageName: 'ignore',
        major: 7,
      },
      ownerType: 'Ignore',
      member,
      evidence: 'ignore 7.0.6 shipped implementation updates matcher rules or result caches',
    },);
  },),
  ...[
    'getAllComments',
    'getCommentsBefore',
    'getCommentsInside',
    'getDeclaredVariables',
    'getFirstToken',
    'getIndexFromLoc',
    'getLastToken',
    'getLocFromIndex',
    'getScope',
    'getText',
    'getTokenAfter',
    'getTokenBefore',
    'getTokensBetween',
    'insertTextAfter',
    'insertTextAfterRange',
    'insertTextBefore',
    'insertTextBeforeRange',
    'replaceText',
    'replaceTextRange',
  ].map(function oxlintObservation(member,): IntrinsicEffectEntry {
    return {
      provenance: {
        kind: 'package',
        packageName: '@oxlint/plugins',
        major: 1,
      },
      ownerType: '__type',
      member,
      targets: [],
      evidence: '@oxlint/plugins 1.73 index.d.ts source and fixer descriptor operations',
    };
  },),
  {
    provenance: {
      kind: 'package',
      packageName: 'toml-eslint-parser',
      major: 1,
    },
    ownerType: 'globalThis',
    member: 'getStaticTOMLValue',
    targets: [{
      kind: 'argument',
      index: 0,
    },],
    evidence: 'toml-eslint-parser 1.0.3 source reads parser AST fields and can invoke caller-owned hooks',
  },
  ...[
    'isArrayLiteralExpression',
    'isArrayTypeNode',
    'isBinaryExpression',
    'isBindingElement',
    'isCallExpression',
    'isCallSignatureDeclaration',
    'isClassLikeDeclaration',
    'isConstructSignatureDeclaration',
    'isConstructorTypeNode',
    'isDeleteExpression',
    'isElementAccessExpression',
    'isExportDeclaration',
    'isFunctionLikeDeclaration',
    'isFunctionTypeNode',
    'isForOfStatement',
    'isGetAccessorDeclaration',
    'isIdentifier',
    'isImportClause',
    'isImportDeclaration',
    'isImportSpecifier',
    'isMethodDeclaration',
    'isMethodSignatureDeclaration',
    'isNamedExports',
    'isNamedImports',
    'isNamespaceImport',
    'isObjectLiteralExpression',
    'isPropertyAssignment',
    'isPropertyDeclaration',
    'isPostfixUnaryExpression',
    'isPrefixUnaryExpression',
    'isPropertyAccessExpression',
    'isReturnStatement',
    'isSetAccessorDeclaration',
    'isShorthandPropertyAssignment',
    'isSpreadAssignment',
    'isSpreadElement',
    'isSourceFile',
    'isStringLiteral',
    'isTypeReferenceNode',
    'isVariableDeclaration',
    'isVariableDeclarationList',
  ].map(function typescriptAstObservation(member,): IntrinsicEffectEntry {
    return {
      provenance: {
        kind: 'package',
        packageName: 'typescript',
        major: 7,
      },
      ownerType: 'globalThis',
      member,
      targets: [],
      evidence: 'typescript 7.0.2 dist/ast predicate declarations',
    };
  },),
  ...[
    'getAliasedSymbol',
    'getBaseConstraintOfType',
    'getIndexInfosOfType',
    'getPropertiesOfType',
    'getResolvedSignature',
    'getResolvedSymbol',
    'getSignaturesOfType',
    'getShorthandAssignmentValueSymbol',
    'getSymbolAtLocation',
    'getTypeArguments',
    'getTypeAtLocation',
    'getTypeFromTypeNode',
    'getTypeOfSymbolAtLocation',
    'isArrayType',
    'isTupleType',
  ].map(function typescriptCheckerObservation(member,): IntrinsicEffectEntry {
    return {
      provenance: {
        kind: 'package',
        packageName: 'typescript',
        major: 7,
      },
      ownerType: 'Checker',
      member,
      targets: [],
      evidence: 'typescript 7.0.2 dist/api/sync/api.d.ts Checker query declaration',
    };
  },),
  ...[
    'getAliasSymbol',
    'getAliasTypeArguments',
    'getSymbol',
    'isIntersectionType',
    'isObjectType',
    'isTupleType',
    'isTypeParameter',
    'isTypeReference',
    'isUnionType',
  ].map(function typescriptTypeObservation(member,): IntrinsicEffectEntry {
    return {
      provenance: {
        kind: 'package',
        packageName: 'typescript',
        major: 7,
      },
      ownerType: 'Type',
      member,
      targets: [],
      evidence: 'typescript 7.0.2 dist/api/sync/types.d.ts Type query declaration',
    };
  },),
  {
    provenance: {
      kind: 'package',
      packageName: 'typescript',
      major: 7,
    },
    ownerType: 'Type',
    member: 'getTarget',
    targets: [],
    evidence: 'typescript 7.0.2 dist/api/sync/api.js TypeObject.getTarget reads target through ObjectRegistry.fetchType; only library-internal identity cache changes',
  },
  {
    provenance: {
      kind: 'package',
      packageName: 'typescript',
      major: 7,
    },
    ownerType: 'UnionOrIntersectionType',
    member: 'getTypes',
    targets: [],
    evidence: 'typescript 7.0.2 dist/api/sync/types.d.ts union constituent query',
  },
  ...[
    'getSourceFile',
    'getStart',
    'getText',
  ].map(function typescriptNodeObservation(member,): IntrinsicEffectEntry {
    return {
      provenance: {
        kind: 'package',
        packageName: 'typescript',
        major: 7,
      },
      ownerType: 'Node',
      member,
      targets: [],
      evidence: 'typescript 7.0.2 dist/ast/index.d.ts Node source query',
    };
  },),
  {
    provenance: {
      kind: 'package',
      packageName: 'typescript',
      major: 7,
    },
    ownerType: 'NodeHandle',
    member: 'resolve',
    targets: [],
    evidence: 'typescript 7.0.2 dist/api/sync/api.d.ts NodeHandle resolution',
  },
  ...[
    'getSourceFile',
    'getSourceFileNames',
    'isSourceFileDefaultLibrary',
    'isSourceFileFromExternalLibrary',
  ].map(function typescriptProgramObservation(member,): IntrinsicEffectEntry {
    return {
      provenance: {
        kind: 'package',
        packageName: 'typescript',
        major: 7,
      },
      ownerType: 'Program',
      member,
      targets: [],
      evidence: 'typescript 7.0.2 dist/api/sync/api.d.ts Program source query',
    };
  },),
  ...[
    'clearSourceFileCache',
    'close',
    'updateSnapshot',
  ].map(function typescriptApiEffect(member,): IntrinsicEffectEntry {
    return receiverEffect({
      provenance: {
        kind: 'package',
        packageName: 'typescript',
        major: 7,
      },
      ownerType: 'API',
      member,
      evidence: 'typescript 7.0.2 dist/api/sync/api.d.ts API declaration',
    },);
  },),
  receiverEffect({
    provenance: {
      kind: 'package',
      packageName: 'typescript',
      major: 7,
    },
    ownerType: 'Snapshot',
    member: 'dispose',
    evidence: 'typescript 7.0.2 dist/api/sync/api.d.ts Snapshot declaration',
  },),
];
