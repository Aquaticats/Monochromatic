/**
 Top-level Figma document to Penpot document conversion.
 
 @module figma-to-penpot-document
 */

import {
  FIGMA_DOCUMENT_ABSENT,
  type FigmaFile,
} from '@monochromatic-dev/figma-kiwi/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import { figmaColorToHex, } from './color.ts';
import {
  PAGE_BACKGROUND_DECK,
  PAGE_BACKGROUND_FIG,
  ZERO_UUID,
} from './constants.ts';
import { parseParentIndex, } from './geometry.ts';
import {
  makeFile,
  makeManifest,
} from './manifest.ts';
import {
  type ConvertContext,
  convertNode,
} from './node.ts';
import {
  asString,
  type FigmaRecord,
  isRecord,
  numberOr,
  recordArray,
  SKIP,
  stringOr,
} from './read.ts';
import { makeRootFrame, } from './shape.ts';
import type {
  ConvertOptions,
  PenpotDocument,
  PenpotPage,
  Uuid,
} from './types.ts';
import {
  guidToUuid,
  nextUuid,
} from './uuid.ts';

/**
 Page-source node: a Figma canvas or slide that becomes one Penpot page.
 */
type PageSource = {
  key: string;
  nc: Record<string, unknown>;
};

/**
 Build a composite GUID lookup key from session and local id components.
 
 @param sessionId - Figma session id
 
 @param localId - Figma local id
 
 @returns `"sessionId:localId"` key
 
 @example
 ```ts
 guidKey({ sessionId: 0, localId: 1, }); // "0:1"
 ```
 */
function guidKey(
  {
    sessionId,
    localId,
  }: Readonly<{
    sessionId: number;
    localId: number;
  }>,
): string {
  return `${sessionId}:${localId}`;
}

/**
 Read a node's GUID components, or `SKIP` when the node carries no GUID.
 
 @param nc - Figma NodeChange {@link FigmaRecord}
 
 @returns session/local id pair, or {@link SKIP}
 
 @example
 ```ts
 const guid = nodeGuid(nc);
 if (guid !== SKIP) { ... }
 ```
 */
function nodeGuid(
  nc: FigmaRecord,
): {
  sessionId: number;
  localId: number
} | typeof SKIP {
  /**
   Per-node GUID struct guarded before reading id components.
   */
  const {guid} = nc;
  if (!isRecord(guid,))
    return SKIP;
  return {
    sessionId: numberOr({
      value: guid.sessionID,
      fallback: 0,
    },),
    localId: numberOr({
      value: guid.localID,
      fallback: 0,
    },),
  };
}

/**
 Index every node: assign stable UUIDs and build GUID and parent-to-children maps.
 
 @param nodeChanges - raw Figma NodeChange {@link FigmaRecord} entries
 
 @returns {@link ConvertContext} with populated lookup tables
 
 @example
 ```ts
 const ctx = indexNodes(nodeChanges);
 ```
 */
function indexNodes(nodeChanges: readonly FigmaRecord[],): ConvertContext {
  /**
   Conversion context whose maps are populated by the single pass below.
   */
  const ctx: ConvertContext = {
    nodeByGuid: new Map<string, Record<string, unknown>>(),
    childrenByParent: new Map<string, string[]>(),
    guidToUuidMap: new Map<string, Uuid>(),
    shapes: new Map(),
  };
  for (const nc of nodeChanges) {
    /**
     GUID components for this node; nodes without a GUID are not indexed.
     */
    const guid = nodeGuid(nc,);
    if (guid === SKIP)
      continue;
    /**
     Composite key shared by all three lookup maps.
     */
    const key = guidKey(guid,);
    ctx.guidToUuidMap
      .set(
      key,
      guidToUuid(guid,),
      );
    ctx.nodeByGuid
      .set(
      key,
      nc,
      );
    /**
     Parent reference; `SKIP` means the node is unparented.
     */
    const parent = parseParentIndex(nc.parentIndex,);
    if (parent !== SKIP) {
      /**
       Parent's composite key in `childrenByParent`.
       */
      const parentKey = guidKey(parent.parentGuid,);
      /**
       Existing child list for the parent, or a fresh array.
       */
      const children = ctx.childrenByParent
        .get(parentKey,)
        ?? [];
      children.push(key,);
      ctx.childrenByParent
        .set(
        parentKey,
        children,
        );
    }
  }
  return ctx;
}

/**
 Test whether a `.fig`/`.jam` canvas is Figma's hidden internal-only canvas.
 
 @param nc - Figma canvas NodeChange {@link FigmaRecord}
 
 @returns whether the canvas should be excluded from pages
 
 @example
 ```ts
 if (isInternalCanvas(nc)) continue;
 ```
 */
function isInternalCanvas(nc: FigmaRecord,): boolean {
  /**
   Whether the node carries either internal-only marker Figma emits.
   */
  const hasEditInfo = (nc.editInfo !== null) && (nc.editInfo !== undefined);
  /**
   Best-effort internal-only flag combining both markers.
   */
  const internalOnly = (nc.internalOnly === true) || hasEditInfo;
  return internalOnly
    && asString(nc.name,)
      .toLowerCase()
      .includes('internal',);
}

/**
 Select the Figma nodes that each become a Penpot page.
 
 Deck files map slides to pages; `.fig`/`.jam` map canvases (minus the hidden
 internal canvas).
 
 @param nodeChanges - raw Figma NodeChange {@link FigmaRecord} entries
 
 @param isDeck - whether the source is a `.deck` file
 
 @returns page-source nodes in document order
 
 @example
 ```ts
 const sources = selectPageSources({ nodeChanges, isDeck, });
 ```
 */
function selectPageSources(
  {
    nodeChanges,
    isDeck,
  }: Readonly<{
    nodeChanges: readonly FigmaRecord[];
    isDeck: boolean;
  }>,
): PageSource[] {
  /**
   Accumulated page sources in document order.
   */
  const sources: PageSource[] = [];
  for (const nc of nodeChanges) {
    /**
     Figma node type used to filter for slides or canvases.
     */
    const nodeType = asString(nc.type,);
    if (isDeck ? (nodeType !== 'NodeType.SLIDE') : (nodeType !== 'NodeType.CANVAS'))
      continue;
    if ((!isDeck) && isInternalCanvas(nc,))
      continue;
    /**
     GUID components; nodes without a GUID cannot anchor a page.
     */
    const guid = nodeGuid(nc,);
    if (guid === SKIP)
      continue;
    sources.push({
      key: guidKey(guid,),
      nc,
    },);
  }
  return sources;
}

/**
 Build one Penpot page: register its root frame and convert its children.
 
 @param source - page-source node (canvas or slide)
 
 @param pageIndex - 0-based page order index
 
 @param isDeck - whether the source is a `.deck` file
 
 @param ctx - conversion context whose `shapes` map is written
 
 @returns the {@link PenpotPage} metadata
 
 @example
 ```ts
 const page = buildPage({ source, pageIndex, isDeck, ctx, });
 ```
 */
function buildPage(
  {
    source,
    pageIndex,
    isDeck,
    ctx,
  }: {
    source: PageSource;
    pageIndex: number;
    isDeck: boolean;
    ctx: ConvertContext;
  },
): PenpotPage {
  /**
   Fresh UUID for this page.
   */
  const pageId = nextUuid();
  /**
   Optional canvas/slide background struct.
   */
  const bgColor = source.nc
    .backgroundColor;
  /**
   Page background hex, falling back per file type when none is set.
   */
  const background = isRecord(bgColor,)
    ? figmaColorToHex(bgColor,)
    : (isDeck ? PAGE_BACKGROUND_DECK : PAGE_BACKGROUND_FIG);
  /**
   Implicit root frame Penpot requires on every page.
   */
  const rootShape = makeRootFrame(pageId,);
  ctx.shapes
    .set(
    ZERO_UUID,
    rootShape,
    );
  /**
   Penpot UUIDs of converted top-level children for this page.
   */
  const childUuids: Uuid[] = [];
  for (const childKey of ctx.childrenByParent
    .get(source.key,)
    ?? []) {
    /**
     Converted child UUID; {@link SKIP} children are dropped.
     */
    const childUuid = convertNode({
      nodeKey: childKey,
      parentUuid: ZERO_UUID,
      frameUuid: ZERO_UUID,
      pageId,
      ctx,
    },);
    if (childUuid !== SKIP)
      childUuids.push(childUuid,);
  }
  rootShape.shapes = childUuids;
  return {
    id: pageId,
    name: stringOr({
      value: source.nc
        .name,
      fallback: `Page ${pageIndex + 1}`,
    },),
    background,
    index: pageIndex,
  };
}

/**
 Convert a parsed Figma file to a Penpot document.
 
 @param figmaFile - fully decoded Figma file from {@link parseFigmaFile}
 
 @param options - conversion options
 
 @returns Penpot document ready for ZIP serialization
 
 @example
 ```ts
 const doc = convertFigmaToPenpot({ figmaFile, });
 ```
 */
export function convertFigmaToPenpot(
  {
    figmaFile,
    options = {},
  }: {
    readonly figmaFile: ForeignBorrowed<FigmaFile>;
    readonly options?: ConvertOptions;
  },
): PenpotDocument {
  /**
   File UUID assigned up front so manifest, file, and paths share it.
   */
  const fileId = nextUuid();
  /**
   Display name: caller override beats Figma's `meta.fileName`.
   */
  const fileName = options.fileName
    ?? stringOr({
      value: figmaFile.meta
        .fileName,
      fallback: 'Untitled',
    },);
  /**
   ISO timestamp stamped on created/modified so the file looks freshly minted.
   */
  const now = new Date().toISOString();
  /**
   Raw NodeChange entries lifted from the Figma document.
   */
  const nodeChanges = figmaFile.document === FIGMA_DOCUMENT_ABSENT
    ? []
    : recordArray(figmaFile.document
      .nodeChanges,);
  /**
   Conversion context with GUID, parent, and shape maps populated.
   */
  const ctx = indexNodes(nodeChanges,);
  /**
   Whether slides (deck) or canvases (fig/jam) source the pages.
   */
  const isDeck = figmaFile.fileType === 'deck';
  /**
   Page metadata accumulator keyed by page UUID.
   */
  const pages = new Map<Uuid, PenpotPage>();
  for (
    const [pageIndex, source,] of selectPageSources({
      nodeChanges,
      isDeck,
    },)
      .entries()
  ) {
    /**
     Built page; registers its shapes through the shared context.
     */
    const page = buildPage({
      source,
      pageIndex,
      isDeck,
      ctx,
    },);
    pages.set(
      page.id,
      page,
    );
  }

  if (pages.size === 0) {
    /**
     Fallback page Penpot needs even when the Figma file produced none.
     */
    const pageId = nextUuid();
    pages.set(
      pageId,
      {
        id: pageId,
        name: 'Page 1',
        background: PAGE_BACKGROUND_DECK,
        index: 0,
      },
    );
    ctx.shapes
      .set(
      ZERO_UUID,
      makeRootFrame(pageId,),
      );
  }

  return {
    manifest: makeManifest({
      fileId,
      fileName,
      generatedBy: options.generatedBy
        ?? 'figma-to-penpot/0.1.0',
    },),
    file: makeFile({
      fileId,
      fileName,
      now,
    },),
    pages,
    shapes: ctx.shapes,
    media: new Map(),
    storageObjects: new Map(),
    components: new Map(),
    colors: new Map(),
    typographies: new Map(),
    thumbnails: [],
  };
}
