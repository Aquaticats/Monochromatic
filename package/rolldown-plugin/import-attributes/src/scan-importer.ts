/**
 Scans importer source files to discover import attribute types
 for dynamic imports that rolldown's Rust scanner processes
 before `transform` runs.
 
 Uses AST parsing via rolldown's `parseSync` instead of regex
 for robust, formatting-independent matching.
 
 @module
 */

import { readFile, } from 'node:fs/promises';

import {
  type ESTree,
  parseSync,
  Visitor,
} from 'rolldown/utils';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import {
  tagged,
  type Logger,
} from '@monochromatic-dev/logger-core/ts';

import {
  extractTypeFromOptions,
  getStringLiteralValue,
  NO_ATTR_TYPE,
  staticDeclAttrType,
} from './ast-extract.ts';

//region Logger

/**
 Logger root for importer source scanning.
 
 @example
 ```ts
 const rl = tagged({ tag: scanImporterForAttribute.name, l, });
 ```
 */
const l = tagged({ tag: 'import-attributes', },);

//endregion Logger

//region Importer source

/**
 Sentinel returned by {@link readImporterSource} when the importer cannot be read.
 A `unique symbol` so the caller never needs a nullish union to model absence.
 */
const UNREADABLE_IMPORTER: unique symbol = Symbol('import-attributes/importer-source-could-not-be-read',);

/**
 Reads a file fresh on every call, reporting a read failure through the logger.
 
 Reading per call rather than caching across builds is deliberate: a cache in the
 plugin instance outlives every rebuild, so an importer edited in watch mode
 would keep answering with the clause it had when the first build read it.
 
 @param importerPath - absolute path of the importer to read
 
 @param parentLogger - logger of the calling scan, which this adds a tag to
 
 @returns the file's text, or {@link UNREADABLE_IMPORTER} when it could not be read
 
 @example
 ```ts
 const source = await readImporterSource({ importerPath: '/project/src/main.ts', parentLogger: l });
 if (source === UNREADABLE_IMPORTER) {
   // nothing to scan
 }
 ```
 */
async function readImporterSource({
  importerPath,
  parentLogger,
}: {
  readonly importerPath: string;
  readonly parentLogger: Logger;
},): Promise<string | typeof UNREADABLE_IMPORTER> {
  try {
    return await readFile(
      importerPath,
      'utf8',
    );
  }
  catch (error: unknown) {
    /**
     Logger tagged for this reader, so the record names who tried to read the file.
     */
    const rl = tagged({
      tag: readImporterSource.name,
      l: parentLogger,
    },);
    // Best-effort scan: an unreadable or virtual importer (e.g. a plugin-generated
    // module whose id is not a real filesystem path) has no source to scan, so
    // report no attribute rather than failing the build; the caught error rides in
    // the message because the logger takes one string and the cause is what a
    // reader needs to diagnose it.
    rl.warn(
      `could not read importer ${importerPath} while scanning for import attributes; treating as no attribute type: ${String(error,)}`,
    );
    return UNREADABLE_IMPORTER;
  }
}

//endregion Importer source

//region Static declaration match

/**
 Reads the attribute type of one static declaration whose source is the scanned specifier.
 
 @param source - because only the declaration naming this module carries its clause
 
 @param attributes - because the clause is where the type is written
 
 @param specifier - because the scan looks for one module's attribute
 
 @returns the declaration's attribute type, or {@link NO_ATTR_TYPE} when it names another module
 */
function specifierAttrType({
  source,
  attributes,
  specifier,
}: Readonly<{
  source: ESTree.StringLiteral;
  attributes: readonly ESTree.ImportAttribute[];
  specifier: string;
}>,): string | typeof NO_ATTR_TYPE {
  if (source.value
    !== specifier)
    return NO_ATTR_TYPE;
  return staticDeclAttrType(attributes,);
}

//endregion Static declaration match

//region Importer scan

/**
 Scans an importer file's source code to find the import attribute type
 associated with a given specifier.
 
 This is used for dynamic imports where rolldown's Rust scanner discovers
 dependencies before the `transform` hook can rewrite them. The matched
 declaration's attribute type is read with {@link extractTypeFromAttributes}
 (static forms) or {@link extractTypeFromOptions} (dynamic `import()`).
 
 @param specifier - import specifier to look for (e.g. `./sample.sql`)
 
 @param importerPath - absolute path to the importing file
 
 @returns attribute type string if found and supported, {@link NO_ATTR_TYPE} otherwise
 
 @example
 ```ts
 const attrType = await scanImporterForAttribute({
   specifier: './sample.sql',
   importerPath: '/project/src/main.ts',
 },);
 // 'text' if main.ts contains: import x from './sample.sql' with { type: 'text' }
 ```
 */
export async function scanImporterForAttribute({
  specifier,
  importerPath,
}: {
  readonly specifier: string;
  readonly importerPath: string;
},): Promise<string | typeof NO_ATTR_TYPE> {
  /**
   Logger tagged for this scanner; the tag chain names both the plugin and the
   function instead of a hand-written prefix inside the message.
   */
  const rl = tagged({
    tag: scanImporterForAttribute.name,
    l,
  },);

  /**
   Importer source text, read fresh on every call.
   */
  const source = await readImporterSource({
    importerPath,
    parentLogger: rl,
  },);
  if (source === UNREADABLE_IMPORTER)
    return NO_ATTR_TYPE;

  if (!source.includes(specifier,))
    return NO_ATTR_TYPE;

  /**
   Parsed AST root produced by rolldown's parser.
   */
  const result = parseSync(
    importerPath,
    source,
  );
  /**
   Mutable accumulator written by the visitor when a matching specifier is encountered.
   */
  let found: string | typeof NO_ATTR_TYPE = NO_ATTR_TYPE;

  //region AST visitor

  /**
   AST visitor that records the attribute type on the first matching specifier.
   */
  const visitor = new Visitor({
    ImportDeclaration(node: ForeignBorrowed<ESTree.ImportDeclaration>,): void {
      if (found !== NO_ATTR_TYPE)
        return;
      found = specifierAttrType({
        source: node.source,
        attributes: node.attributes,
        specifier,
      },);
    },

    ExportNamedDeclaration(node: ForeignBorrowed<ESTree.ExportNamedDeclaration>,): void {
      if (found !== NO_ATTR_TYPE)
        return;
      /**
       Declaration without a source names no module to scan.
       */
      if (node.source
        === null)
        return;
      found = specifierAttrType({
        source: node.source,
        attributes: node.attributes,
        specifier,
      },);
    },

    ExportAllDeclaration(node: ForeignBorrowed<ESTree.ExportAllDeclaration>,): void {
      if (found !== NO_ATTR_TYPE)
        return;
      found = specifierAttrType({
        source: node.source,
        attributes: node.attributes,
        specifier,
      },);
    },

    ImportExpression(node: ForeignBorrowed<ESTree.ImportExpression>,): void {
      if (found !== NO_ATTR_TYPE)
        return;
      if (node.options
        === null)
        return;
      /**
       Literal specifier text; computed sources are skipped.
       */
      const sourceValue = getStringLiteralValue(node.source,);
      if (sourceValue !== specifier)
        return;
      found = extractTypeFromOptions(node.options,);
    },
  },);

  //endregion AST visitor

  visitor.visit(result.program,);
  return found;
}

//endregion Importer scan
