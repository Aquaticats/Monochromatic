/**
 Handler tests for {@link textHandler} and the {@link HANDLERS} registry.
 
 The handler is the last thing between a file's bytes and the module a build
 consumes, so its output has to be a syntactically valid module for any content,
 including quotes, newlines and separators JavaScript treats specially.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { parseSync, } from 'rolldown/utils';

import {
  HANDLERS,
  textHandler,
} from '../dist/final/node/index.mjs';

/**
 Parses one module so a case can prove the handler's output is syntactically
 valid rather than only round-trippable.
 
 @param code - module source to parse
 
 @throws Error when the source does not parse
 */
function parseModule(code: string,): void {
  parseSync('handlers-case.ts', code,);
}

/** Prefix every handler output starts with, the module's default export. */
const DEFAULT_EXPORT_PREFIX = 'export default ';

/**
 Reads the exported string back out of the module source the handler produced,
 as JSON rather than as JavaScript, so the case checks the content's meaning.
 
 @param moduleSource - module source `textHandler` produced
 
 @returns content the module's default export carries
 
 @example
 ```ts
 exportedContent('export default "SELECT 1";'); // 'SELECT 1'
 ```
 */
function exportedContent(moduleSource: string,): string {
  if (!moduleSource.startsWith(DEFAULT_EXPORT_PREFIX,))
    throw new Error(`not a default export: ${moduleSource}`,);
  if (!moduleSource.endsWith(';',))
    throw new Error(`not a statement: ${moduleSource}`,);
  return JSON.parse(moduleSource.slice(DEFAULT_EXPORT_PREFIX.length, -1,),) as string;
}

/**
 Content shapes a raw file can carry that a naive string-templating handler
 would corrupt.
 */
const CONTENT_CASES: readonly (readonly [string, string])[] = [
  ['double quotes', 'SELECT "quoted" FROM t;',],
  ['single quotes', "SELECT 'quoted' FROM t;",],
  ['a newline', 'SELECT 1;\nSELECT 2;',],
  ['a carriage return', 'SELECT 1;\r\nSELECT 2;',],
  ['a line separator', 'SELECT 1;\u2028SELECT 2;',],
  ['a paragraph separator', 'SELECT 1;\u2029SELECT 2;',],
  ['a backslash', String.raw`SELECT '\d+';`,],
  ['a script-closing sequence', 'SELECT 1; <!-- </script> -->',],
];

await describe({
  name: '',
  children: [
    describe({
      name: textHandler.name,
      children: [
        it({
          name: 'produces a module whose default export is the content',
          fn: async () => {
            expect(exportedContent(textHandler('SELECT 1',),),).toBe('SELECT 1',);
          },
        },),
        ...CONTENT_CASES.map(function mapContentCase([name, content,],) {
          return it({
            name: `round-trips content holding ${name}`,
            fn: async () => {
              expect(exportedContent(textHandler(content,),),).toBe(content,);
            },
          },);
        },),
        it({
          name: 'produces a module the parser accepts for every content case',
          fn: async () => {
            expect(function parses() {
              parseModule(textHandler(CONTENT_CASES.map(function contentOf([, content,],) {
                return content;
              },)
                .join('\n',),),);
            },).not.toThrow();
          },
        },),
      ],
    },),
    describe({
      name: 'HANDLERS',
      children: [
        it({
          name: 'registers the text handler under the text type',
          fn: async () => {
            expect(HANDLERS['text'],).toBe(textHandler,);
          },
        },),
        it({
          name: 'registers no handler for an unregistered type',
          fn: async () => {
            expect(HANDLERS['unknown'],).toBeUndefined();
          },
        },),
      ],
    },),
  ],
},);
