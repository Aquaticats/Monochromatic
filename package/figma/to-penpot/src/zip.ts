/**
 Penpot document to `.penpot` ZIP (binfile-v3) serialization.
 
 @module figma-to-penpot-zip
 */

import { ZipWriter, } from '@monochromatic-dev/zip-writer-core/ts';

import { ZERO_UUID, } from './constants.ts';
import type { PenpotDocument, } from './types.ts';

/**
 JSON indentation width for every emitted entry.
 */
const JSON_INDENT = 2;

/**
 Static MIME-type to file-extension lookup for objects Penpot stores.
 */
const MTYPE_EXTENSIONS: Record<string, string> = {
  'image/png': '.png',
  'image/jpeg': '.jpg',
  'image/svg+xml': '.svg',
  'image/webp': '.webp',
  'image/gif': '.gif',
  'font/woff': '.woff',
  'font/woff2': '.woff2',
  'font/ttf': '.ttf',
  'font/otf': '.otf',
};

/**
 Map a MIME type to its storage-object file extension.
 
 @param mtype - MIME type string
 
 @returns matching extension, or `.bin` when unknown
 
 @example
 ```ts
 mtypeToExtension('image/png'); // ".png"
 ```
 */
export function mtypeToExtension(mtype: string,): string {
  return MTYPE_EXTENSIONS[mtype]
    ?? '.bin';
}

/**
 Serialize a JSON value into the archive at a given path.
 
 @param zip - archive writer to append to
 
 @param path - entry path inside the archive
 
 @param value - JSON-serializable value
 
 @mutates value through JSON.stringify serialization hooks
 
 @example
 ```ts
 addJson({ zip, path: 'manifest.json', value: doc.manifest, });
 ```
 */
function addJson(
  {
    zip,
    path,
    value,
  }: {
    readonly zip: ZipWriter;
    readonly path: string;
    readonly value: unknown
  },
): void {
  /**
   Serialized primitive separated from caller-owned value provenance before archive storage.
   */
  const content: string = JSON.stringify(
    value,
    null,
    JSON_INDENT,
  );
  zip.add(
    path,
    content,
  );
}

/**
 Write every shape for one page (root frame first) into the archive.
 
 @param zip - {@link ZipWriter} archive writer
 
 @param doc - converted {@link PenpotDocument}
 
 @param pageId - page whose shapes are written
 
 @param pageDir - archive directory prefix for the page
 
 @mutates doc through JSON.stringify hooks
 
 @example
 ```ts
 addPageShapes({ zip, doc, pageId: page.id, pageDir, });
 ```
 */
function addPageShapes(
  {
    zip,
    doc,
    pageId,
    pageDir,
  }: {
    readonly zip: ZipWriter;
    readonly doc: PenpotDocument;
    readonly pageId: string;
    readonly pageDir: string;
  },
): void {
  /**
   Root-frame shape; written first when it belongs to this page.
   */
  const rootShape = doc.shapes
    .get(ZERO_UUID,);
  if ((rootShape !== undefined) && (rootShape.pageId === pageId)) {
    addJson({
      zip,
      path: `${pageDir}/${ZERO_UUID}.json`,
      value: rootShape,
    },);
  }
  for (const [shapeId, shape,] of doc.shapes) {
    if ((shape.pageId === pageId) && (shapeId !== ZERO_UUID)) {
      addJson({
        zip,
        path: `${pageDir}/${shapeId}.json`,
        value: shape,
      },);
    }
  }
}

/**
 Serialize a Penpot document to a `.penpot` ZIP buffer.
 
 Produces a binfile-v3 archive importable into Penpot.
 
 @param doc - converted {@link PenpotDocument}
 
 @returns ZIP archive bytes
 
 @mutates doc through JSON.stringify hooks and module-zip-writer 0.0.1 index.ts sha256 8f8368a6425fa203195cc48ec66396d1a47684f3fdd1ef4103583febfa2e1dff mutates receiver entries, owns encoded string bytes, and retains supplied Uint8Array content
 
 @example
 ```ts
 const bytes = await serializePenpotZip(doc);
 ```
 */
export function serializePenpotZip(doc: PenpotDocument,): Uint8Array {
  /**
   Writer accumulating every JSON and binary entry; `build()` emits the archive.
   */
  const zip = new ZipWriter();
  /**
   File UUID spliced into every file-scoped entry path.
   */
  const fileId = doc.file
    .id;

  addJson({
    zip,
    path: 'manifest.json',
    value: doc.manifest,
  },);
  addJson({
    zip,
    path: `files/${fileId}.json`,
    value: doc.file,
  },);

  for (const [, page,] of doc.pages) {
    /**
     Directory prefix for this page's JSON and its shape entries.
     */
    const pageDir = `files/${fileId}/pages/${page.id}`;
    addJson({
      zip,
      path: `${pageDir}.json`,
      value: page,
    },);
    addPageShapes({
      zip,
      doc,
      pageId: page.id,
      pageDir,
    },);
  }

  for (const [mediaId, mediaObj,] of doc.media) {
    addJson({
      zip,
      path: `files/${fileId}/media/${mediaId}.json`,
      value: mediaObj,
    },);
  }

  for (const [objectId, {
    meta,
    data,
  },] of doc.storageObjects) {
    addJson({
      zip,
      path: `objects/${objectId}.json`,
      value: meta,
    },);
    zip.add(
      `objects/${objectId}${mtypeToExtension(meta.contentType,)}`,
      data,
    );
  }

  for (const [compId, compData,] of doc.components) {
    addJson({
      zip,
      path: `files/${fileId}/components/${compId}.json`,
      value: compData,
    },);
  }

  for (const [colorId, colorData,] of doc.colors) {
    addJson({
      zip,
      path: `files/${fileId}/colors/${colorId}.json`,
      value: colorData,
    },);
  }

  for (const [typoId, typoData,] of doc.typographies) {
    addJson({
      zip,
      path: `files/${fileId}/typographies/${typoId}.json`,
      value: typoData,
    },);
  }

  if (doc.tokens !== undefined) {
    addJson({
      zip,
      path: `files/${fileId}/tokens.json`,
      value: doc.tokens,
    },);
  }

  for (const thumb of doc.thumbnails) {
    addJson({
      zip,
      path: thumb.path,
      value: thumb.data,
    },);
  }

  return zip.build();
}
