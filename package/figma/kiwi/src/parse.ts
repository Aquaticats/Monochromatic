/**
 Top-level Figma export parser.
 
 @example
 ```ts
 await parseFigmaFile(new Uint8Array());
 ```
 */

import { readFile, } from 'node:fs/promises';

import { parseCanvasFig, } from './canvas.ts';
import { decodeDocument, } from './decode.ts';
import { parseMetaJson, } from './meta.ts';
import { parseKiwiSchema, } from './schema.ts';
import type { FigmaFile, } from './types.ts';
import { extractZipEntries, } from './zip.ts';

/**
 ZIP entry name for main Figma payload.
 */
const CANVAS_ENTRY_NAME = 'canvas.fig';

/**
 ZIP entry name for metadata payload.
 */
const META_ENTRY_NAME = 'meta.json';

/**
 ZIP entry name for thumbnail payload.
 */
const THUMBNAIL_ENTRY_NAME = 'thumbnail.png';

/**
 Prefix for image asset entries.
 */
const IMAGE_ENTRY_PREFIX = 'images/';

/**
 Parses a Figma export file (.fig, .deck, or .jam).
 
 @param filePathOrBuffer - Path to file or raw ZIP content.
 
 @returns Fully parsed {@link FigmaFile} with decoded schema and document.
 
 @example
 ```ts
 await parseFigmaFile(new Uint8Array());
 ```
 */
export async function parseFigmaFile(filePathOrBuffer: string | Uint8Array,): Promise<FigmaFile> {
  /**
   Whole-file ZIP bytes.
   */
  const rawBuffer = await readFigmaBytes({ filePathOrBuffer, },);
  /**
   ZIP entries from export archive.
   */
  const zipEntries = await extractZipEntries(rawBuffer,);
  /**
   canvas.fig bytes.
   */
  const canvasFig = requiredEntry({
    zipEntries,
    entryName: CANVAS_ENTRY_NAME,
  },);
  /**
   meta.json bytes.
   */
  const metaJson = requiredEntry({
    zipEntries,
    entryName: META_ENTRY_NAME,
  },);
  /**
   Parsed canvas sections.
   */
  const {
    fileType,
    schemaBytes,
    documentBytes,
    version,
  } = await parseCanvasFig(canvasFig,);
  /**
   Parsed Kiwi schema.
   */
  const schema = parseKiwiSchema(schemaBytes,);

  return {
    canvasVersion: version,
    document: decodeDocument({
      documentData: documentBytes,
      schema,
    },),
    fileType,
    images: imageEntries({ zipEntries, },),
    meta: parseMetaJson(metaJson,),
    schema,
    thumbnail: zipEntries.get(THUMBNAIL_ENTRY_NAME,) ?? new Uint8Array(),
  };
}

/**
 Reads parser input into bytes.
 
 @param filePathOrBuffer - File path or raw bytes.
 
 @returns Raw ZIP bytes.
 
 @example
 ```ts
 await readFigmaBytes({ filePathOrBuffer: new Uint8Array() });
 // Uint8Array []
 ```
 */
async function readFigmaBytes(
  { filePathOrBuffer, }: { readonly filePathOrBuffer: string | Uint8Array; },
): Promise<Uint8Array> {
  if ((typeof filePathOrBuffer) === 'string')
    return new Uint8Array(await readFile(filePathOrBuffer,),);
  return filePathOrBuffer;
}

/**
 Reads required ZIP entry or throws.
 
 @param zipEntries - ZIP entries.
 
 @param entryName - Required entry name.
 
 @returns Entry bytes.
 
 @example
 ```ts
 requiredEntry({ zipEntries: new Map([['a', new Uint8Array()]]), entryName: 'a' });
 // Uint8Array []
 ```
 */
function requiredEntry(
  {
    zipEntries,
    entryName,
  }: {
    readonly entryName: string;
    readonly zipEntries: ReadonlyMap<string, Uint8Array>;
  },
): Uint8Array {
  /**
   Requested entry bytes.
   */
  const entry = zipEntries.get(entryName,);
  if (entry === undefined)
    throw new Error(`Missing ${entryName} in Figma export file`);
  return entry;
}

/**
 Extracts image entries from ZIP entries.
 
 @param zipEntries - ZIP entries.
 
 @returns Map keyed by image filename without prefix.
 
 @example
 ```ts
 imageEntries({ zipEntries: new Map([['images/a.png', new Uint8Array()]]) }).has('a.png');
 // true
 ```
 */
function imageEntries(
  { zipEntries, }: { readonly zipEntries: ReadonlyMap<string, Uint8Array>; },
): Map<string, Uint8Array> {
  return new Map([...zipEntries]
    .filter(function isImageEntry(entry,): boolean {
      return entry[0]
        .startsWith(IMAGE_ENTRY_PREFIX,);
    },)
    .map(function imageEntry(entry,): readonly [
      string,
      Uint8Array
    ] {
      return [
        entry[0]
          .slice(IMAGE_ENTRY_PREFIX.length,),
        entry[1],
      ];
    },),);
}
