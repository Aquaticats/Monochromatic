/**
 Extractor tests for the AST helpers in `ast-extract.ts`.
 
 Every visitor in the package delegates the attribute-type decision to these, so
 each supported attribute shape and each rejected shape is checked here through
 the parser's own nodes.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type { ESTree, } from 'rolldown/utils';

import {
  dynamicImportOptionsOf,
  importAttributesOf,
  stringLiteralSourceOf,
} from './test-support.ts';
import {
  extractTypeFromAttributes,
  extractTypeFromOptions,
  getStringLiteralValue,
  NO_ATTR_TYPE,
  NON_STRING_NODE,
  staticDeclAttrType,
} from '../dist/final/node/index.mjs';

/** Attribute list of a declaration that carries no clause, the shape a bare import hands the extractor. */
const NO_ATTRIBUTES: readonly ESTree.ImportAttribute[] = [];

await describe({
  name: '',
  children: [
    describe({
      name: getStringLiteralValue.name,
      children: [
        it({
          name: 'returns the value of a string literal',
          fn: async () => {
            expect(getStringLiteralValue(stringLiteralSourceOf("import x from './schema.sql';",),),)
              .toBe('./schema.sql',);
          },
        },),
        it({
          name: 'reports a non-literal expression as no string',
          fn: async () => {
            expect(getStringLiteralValue(dynamicImportOptionsOf("import('./a.sql', { with: { type: 'text' } });",),),)
              .toBe(NON_STRING_NODE,);
          },
        },),
      ],
    },),
    describe({
      name: extractTypeFromAttributes.name,
      children: [
        it({
          name: 'reads a supported type from a static clause',
          fn: async () => {
            expect(extractTypeFromAttributes(
              importAttributesOf("import x from './a.sql' with { type: 'text' };",),
            ),).toBe('text',);
          },
        },),
        it({
          name: 'reads a type whose key is a string literal',
          fn: async () => {
            expect(extractTypeFromAttributes(
              importAttributesOf("import x from './a.sql' with { 'type': 'text' };",),
            ),).toBe('text',);
          },
        },),
        it({
          name: 'rejects a type no handler registers',
          fn: async () => {
            expect(extractTypeFromAttributes(
              importAttributesOf("import x from './a.sql' with { type: 'unknown' };",),
            ),).toBe(NO_ATTR_TYPE,);
          },
        },),
        it({
          name: 'rejects a declaration whose clause carries another key only',
          fn: async () => {
            expect(extractTypeFromAttributes(
              importAttributesOf("import x from './a.sql' with { kind: 'text' };",),
            ),).toBe(NO_ATTR_TYPE,);
          },
        },),
      ],
    },),
    describe({
      name: staticDeclAttrType.name,
      children: [
        it({
          name: 'reads the type of a declaration carrying an attribute clause',
          fn: async () => {
            expect(staticDeclAttrType(
              importAttributesOf("export { default as x, } from './a.sql' with { type: 'text' };",),
            ),).toBe('text',);
          },
        },),
        it({
          name: 'reports no type for a declaration carrying no attributes',
          fn: async () => {
            expect(staticDeclAttrType(NO_ATTRIBUTES,),).toBe(NO_ATTR_TYPE,);
          },
        },),
        it({
          name: 'reports no type for an unsupported type on a declaration with attributes',
          fn: async () => {
            expect(staticDeclAttrType(
              importAttributesOf("import x from './a.sql' with { type: 'unknown' };",),
            ),).toBe(NO_ATTR_TYPE,);
          },
        },),
      ],
    },),
    describe({
      name: extractTypeFromOptions.name,
      children: [
        it({
          name: 'reads the nested with.type value of a dynamic import',
          fn: async () => {
            expect(extractTypeFromOptions(
              dynamicImportOptionsOf("import('./a.sql', { with: { type: 'text' } });",),
            ),).toBe('text',);
          },
        },),
        it({
          name: 'rejects options that are not an object literal',
          fn: async () => {
            expect(extractTypeFromOptions(
              dynamicImportOptionsOf("import('./a.sql', options);",),
            ),).toBe(NO_ATTR_TYPE,);
          },
        },),
        it({
          name: 'rejects a with entry whose value is not an object literal',
          fn: async () => {
            expect(extractTypeFromOptions(
              dynamicImportOptionsOf("import('./a.sql', { with: 'text' });",),
            ),).toBe(NO_ATTR_TYPE,);
          },
        },),
        it({
          name: 'rejects a nested type whose value is not a string literal',
          fn: async () => {
            expect(extractTypeFromOptions(
              dynamicImportOptionsOf("import('./a.sql', { with: { type: 1 } });",),
            ),).toBe(NO_ATTR_TYPE,);
          },
        },),
        it({
          name: 'rejects an unsupported nested type value',
          fn: async () => {
            expect(extractTypeFromOptions(
              dynamicImportOptionsOf("import('./a.sql', { with: { type: 'unknown' } });",),
            ),).toBe(NO_ATTR_TYPE,);
          },
        },),
        it({
          name: 'rejects the deprecated assert key',
          fn: async () => {
            expect(extractTypeFromOptions(
              dynamicImportOptionsOf("import('./a.sql', { assert: { type: 'text' } });",),
            ),).toBe(NO_ATTR_TYPE,);
          },
        },),
        it({
          name: 'rejects an options object whose with entry has another key only',
          fn: async () => {
            expect(extractTypeFromOptions(
              dynamicImportOptionsOf("import('./a.sql', { with: { kind: 'text' } });",),
            ),).toBe(NO_ATTR_TYPE,);
          },
        },),
      ],
    },),
  ],
},);
