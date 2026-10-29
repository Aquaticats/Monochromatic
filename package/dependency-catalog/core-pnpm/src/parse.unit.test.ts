/**
 Unit tests for the built pnpm workspace catalog parser.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  flattenCatalogEntries,
  parseCatalogFromYaml,
} from '../dist/final/node/index.mjs';

await describe({
  name: parseCatalogFromYaml.name,
  children: [
    //region YAML shapes

    it({
      name: 'parses single-quoted default entries and comments',
      fn: async () => {
        /**
         Workspace text using the repository's single-quoted catalog style.
         */
        const content = [
          'catalog:',
          '  # pinned exact, see note',
          "  'oxlint': '>=1.71.0'",
          "  '@types/node': '>=24.0.0'",
        ].join('\n',);
        /**
         Parsed default map used for the independent expected-value assertions.
         */
        const catalog = parseCatalogFromYaml(content,).defaultCatalog;
        expect(catalog,).toEqual({
          oxlint: '>=1.71.0',
          '@types/node': '>=24.0.0',
        },);
      },
    },),

    it({
      name: 'parses double-quoted keys and values',
      fn: async () => {
        /**
         Workspace text using double-quoted YAML scalars.
         */
        const content = [
          'catalog:',
          '  "foo": ">=1.2.3"',
        ].join('\n',);
        expect(parseCatalogFromYaml(content,).defaultCatalog,).toEqual({
          foo: '>=1.2.3',
        },);
      },
    },),

    it({
      name: 'parses default and named catalogs without merging them',
      fn: async () => {
        /**
         Workspace text with the same package key in separate catalog blocks.
         */
        const content = [
          'catalog:',
          '  react: ^19.0.0',
          'catalogs:',
          '  legacy:',
          '    react: ^18.0.0',
        ].join('\n',);
        /**
         Parsed document containing separate default and named maps.
         */
        const document = parseCatalogFromYaml(content,);
        expect(document.defaultCatalog.react,).toBe('^19.0.0',);
        expect(document.namedCatalogs.legacy?.react,).toBe('^18.0.0',);
        expect(flattenCatalogEntries({ document, },),).toEqual([{
          catalogKey: 'react',
          catalogValue: '^19.0.0',
        },],);
        expect(flattenCatalogEntries({
          document,
          includeNamedCatalogs: true,
        },),).toEqual([
          {
            catalogKey: 'react',
            catalogValue: '^19.0.0',
          },
          {
            catalogKey: 'react',
            catalogName: 'legacy',
            catalogValue: '^18.0.0',
          },
        ],);
      },
    },),

    it({
      name: 'preserves npm alias values without decoding them',
      fn: async () => {
        /**
         Alias value whose target and selector must remain one raw scalar.
         */
        const content = [
          'catalog:',
          "  zod: 'npm:@jsr/zod__zod@>=4.1.8'",
        ].join('\n',);
        expect(parseCatalogFromYaml(content,).defaultCatalog.zod,)
          .toBe('npm:@jsr/zod__zod@>=4.1.8',);
      },
    },),

    it({
      name: 'returns empty safe maps when no catalog exists',
      fn: async () => {
        /**
         Workspace text containing only a package glob.
         */
        const content = [
          'packages:',
          "  - 'package/*'",
        ].join('\n',);
        /**
         Empty parsed document whose maps are still safe containers.
         */
        const document = parseCatalogFromYaml(content,);
        expect(document.defaultCatalog,).toEqual({});
        expect(document.namedCatalogs,).toEqual({});
        expect(Object.getPrototypeOf(document.defaultCatalog,),).toBeNull();
        expect(Object.getPrototypeOf(document.namedCatalogs,),).toBeNull();
      },
    },),

    //endregion YAML shapes

    //region Safety

    it({
      name: 'skips invalid keys while preserving valid siblings',
      fn: async () => {
        /**
         Workspace text containing a crafted key and a valid package name.
         */
        const content = [
          'catalog:',
          "  '__proto__': '>=9.9.9'",
          "  foo: '>=1.2.3'",
        ].join('\n',);
        /**
         Parsed map after invalid-key filtering.
         */
        const catalog = parseCatalogFromYaml(content,).defaultCatalog;
        expect(catalog.foo,).toBe('>=1.2.3',);
        expect(Object.hasOwn(catalog, '__proto__',),).toBe(false,);
        expect(Object.getPrototypeOf(catalog,),).toBeNull();
        expect(({} as Record<string, unknown>).polluted,).toBeUndefined();
      },
    },),

    it({
      name: 'keeps name-shaped reserved words inert in every catalog map',
      fn: async () => {
        /**
         Workspace text using a valid npm-shaped key that is also a prototype name.
         */
        const content = [
          'catalog:',
          "  constructor: '>=1.0.0'",
          'catalogs:',
          '  legacy:',
          "    prototype: '>=1.0.0'",
        ].join('\n',);
        /**
         Parsed document with null-prototype maps at both levels.
         */
        const document = parseCatalogFromYaml(content,);
        expect(Object.getPrototypeOf(document.defaultCatalog,),).toBeNull();
        expect(Object.getPrototypeOf(document.namedCatalogs,),).toBeNull();
        expect(Object.getPrototypeOf(document.namedCatalogs.legacy,),).toBeNull();
        expect(document.defaultCatalog.constructor,).toBe('>=1.0.0',);
        expect(document.namedCatalogs.legacy?.prototype,).toBe('>=1.0.0',);
      },
    },),

    it({
      name: 'keeps reserved named-catalog keys in safe maps',
      fn: async () => {
        /**
         Named catalog whose name resembles an inherited object property.
         */
        const content = [
          'catalogs:',
          '  __proto__:',
          "    foo: '>=1.2.3'",
        ].join('\n',);
        /**
         Parsed named catalog map with an own property for the reserved name.
         */
        const document = parseCatalogFromYaml(content,);
        expect(Object.hasOwn(document.namedCatalogs, '__proto__',),).toBe(true,);
        expect(Object.getOwnPropertyDescriptor(
          document.namedCatalogs,
          '__proto__',
        ),).toMatchObject({
          value: { foo: '>=1.2.3', },
        },);
        expect(Object.getPrototypeOf(document.namedCatalogs,),).toBeNull();
      },
    },),

    it({
      name: 'treats malformed catalog blocks as empty safe maps',
      fn: async () => {
        /**
         Workspace text using sequences where catalog mappings are required.
         */
        const content = [
          'catalog: []',
          'catalogs: []',
        ].join('\n',);
        /**
         Parsed document after malformed optional blocks are ignored.
         */
        const document = parseCatalogFromYaml(content,);
        expect(document.defaultCatalog,).toEqual({});
        expect(document.namedCatalogs,).toEqual({});
      },
    },),

    it({
      name: 'skips non-string values without dropping string entries',
      fn: async () => {
        /**
         Workspace text mixing a scalar number with a valid string range.
         */
        const content = [
          'catalog:',
          '  numeric: 1',
          "  valid: '>=1.2.3'",
        ].join('\n',);
        /**
         Parsed map containing only the usable string value.
         */
        const catalog = parseCatalogFromYaml(content,).defaultCatalog;
        expect(catalog,).toEqual({ valid: '>=1.2.3', });
      },
    },),

    //endregion Safety
  ],
},);
