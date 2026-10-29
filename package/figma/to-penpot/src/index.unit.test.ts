import { homedir, } from 'node:os';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  type FigmaFile as FigmaFileType,
  parseFigmaFile,
} from '@monochromatic-dev/figma-kiwi/ts';
import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';
import {
  convertFigmaToPenpot,
  figmaColorToHex,
  type PenpotDocument,
  type PenpotFill,
  type PenpotShape,
  serializePenpotZip,
} from './index.ts';

type FigmaFile = FigmaFileType;

/**
 Directory holding the maintainer's personal Figma export fixtures.
 
 These integration tests read real `.fig`/`.deck`/`.jam` exports that are
 not checked into the repo; they only run on a machine where the files are
 present at this path under the home directory. `homedir()` keeps the path
 independent of the `/home` vs `/var/home` mount.
 */
const FIGMA_EXPORT_DIR = join(
  homedir(),
  'Seafile',
  'Plain',
  'Text',
  'Reference',
  'Figma export',
);

// region Test fixtures

/** Build a minimal FigmaFile for testing without real file I/O. */
function buildTestFigmaFile(fileType: FigmaFile['fileType'] = 'fig',): FigmaFile {
  const nodeChanges: Record<string, unknown>[] = [
    // Document root
    { __type: 'NodeChange', guid: { sessionID: 0, localID: 0, },
      type: 'NodeType.DOCUMENT', name: 'Document', phase: 'NodePhase.CREATED', },
    // Canvas (page)
    {
      __type: 'NodeChange',
      guid: { sessionID: 0, localID: 1, },
      type: 'NodeType.CANVAS',
      name: 'Page 1',
      phase: 'NodePhase.CREATED',
      parentIndex: { guid: { sessionID: 0, localID: 0, }, position: '!', },
      backgroundColor: { r: 1, g: 1, b: 1, a: 1, },
    },
    // Frame on the canvas
    {
      __type: 'NodeChange',
      guid: { sessionID: 0, localID: 2, },
      type: 'NodeType.FRAME',
      name: 'My Frame',
      phase: 'NodePhase.CREATED',
      parentIndex: { guid: { sessionID: 0, localID: 1, }, position: '!', },
      transform: { m00: 1, m01: 0, m02: 100, m10: 0, m11: 1, m12: 200, },
      size: { x: 300, y: 400, },
      opacity: 0.8,
      visible: true,
      fillPaints: [{ __type: 'Paint', type: 'PaintType.SOLID',
        color: { r: 1, g: 0, b: 0, a: 1, }, opacity: 1, },],
      strokePaints: [{ __type: 'Paint', type: 'PaintType.SOLID',
        color: { r: 0, g: 0, b: 0, a: 1, }, },],
      strokeWeight: 2,
      strokeAlign: 'CENTER',
      cornerRadius: 8,
    },
    // Text inside the frame
    {
      __type: 'NodeChange',
      guid: { sessionID: 0, localID: 3, },
      type: 'NodeType.TEXT',
      name: 'Hello',
      phase: 'NodePhase.CREATED',
      parentIndex: { guid: { sessionID: 0, localID: 2, }, position: '!', },
      transform: { m00: 1, m01: 0, m02: 110, m10: 0, m11: 1, m12: 210, },
      size: { x: 200, y: 30, },
      characters: 'Hello World',
      fontSize: 24,
      fontName: 'Inter',
      fontWeight: 400,
      fillPaints: [{ __type: 'Paint', type: 'PaintType.SOLID',
        color: { r: 0, g: 0, b: 0, a: 1, }, },],
    },
  ];

  return {
    fileType,
    meta: {
      backgroundColor: { r: 1, g: 1, b: 1, a: 1, },
      thumbnailSize: { width: 800, height: 600, },
      renderCoordinates: { x: 0, y: 0, width: 800, height: 600, },
      fileName: 'Test File',
      exportedAt: '2025-01-01T00:00:00Z',
      developerRelatedLinks: [],
    },
    thumbnail: new Uint8Array(0,),
    schema: { definitions: [], enumByName: new Map(), structByName: new Map(), },
    document: { __type: 'Message', type: 'MessageType.NODE_CHANGES', nodeChanges, },
    images: new Map(),
  };
}

/** Build a FigmaFile that simulates a deck with slides. */
function buildTestDeckFile(): FigmaFile {
  const nodeChanges: Record<string, unknown>[] = [
    // Document root
    { __type: 'NodeChange', guid: { sessionID: 0, localID: 0, },
      type: 'NodeType.DOCUMENT', name: 'Document', phase: 'NodePhase.CREATED', },
    // Canvas container
    { __type: 'NodeChange', guid: { sessionID: 0, localID: 1, }, type: 'NodeType.CANVAS',
      name: 'Page 1', phase: 'NodePhase.CREATED',
      parentIndex: { guid: { sessionID: 0, localID: 0, }, position: '!', }, },
    // Internal Only Canvas (should be skipped)
    {
      __type: 'NodeChange',
      guid: { sessionID: 0, localID: 2, },
      type: 'NodeType.CANVAS',
      name: 'Internal Only Canvas',
      phase: 'NodePhase.CREATED',
      parentIndex: { guid: { sessionID: 0, localID: 0, }, position: '~', },
      internalOnly: true,
    },
    // Slide grid
    { __type: 'NodeChange', guid: { sessionID: 0, localID: 3, },
      type: 'NodeType.SLIDE_GRID', name: 'Presentation', phase: 'NodePhase.CREATED',
      parentIndex: { guid: { sessionID: 0, localID: 1, }, position: '!', }, },
    // Slide row
    { __type: 'NodeChange', guid: { sessionID: 1, localID: 29, },
      type: 'NodeType.SLIDE_ROW', name: 'Slide row', phase: 'NodePhase.CREATED',
      parentIndex: { guid: { sessionID: 0, localID: 3, }, position: '!', }, },
    // Slide
    { __type: 'NodeChange', guid: { sessionID: 1, localID: 85, }, type: 'NodeType.SLIDE',
      name: 'Slide 1', phase: 'NodePhase.CREATED',
      parentIndex: { guid: { sessionID: 1, localID: 29, }, position: '!', }, },
    // Text inside the slide
    {
      __type: 'NodeChange',
      guid: { sessionID: 1, localID: 86, },
      type: 'NodeType.TEXT',
      name: 'Title',
      phase: 'NodePhase.CREATED',
      parentIndex: { guid: { sessionID: 1, localID: 85, }, position: '!', },
      characters: 'My Title',
      fontSize: 48,
      fontName: 'Inter',
      fontWeight: 700,
      fillPaints: [{ __type: 'Paint', type: 'PaintType.SOLID',
        color: { r: 0, g: 0, b: 0, a: 1, }, },],
    },
  ];

  return {
    fileType: 'deck',
    meta: {
      backgroundColor: { r: 1, g: 1, b: 1, a: 1, },
      thumbnailSize: { width: 1_920, height: 1_080, },
      renderCoordinates: { x: 0, y: 0, width: 1_920, height: 1_080, },
      fileName: 'Test Deck',
      exportedAt: '2025-01-01T00:00:00Z',
      developerRelatedLinks: [],
    },
    thumbnail: new Uint8Array(0,),
    schema: { definitions: [], enumByName: new Map(), structByName: new Map(), },
    document: { __type: 'Message', type: 'MessageType.NODE_CHANGES', nodeChanges, },
    images: new Map(),
  };
}

// endregion

await describe({
  name: '',
  children: [
    // region figmaColorToHex
    describe({
      name: figmaColorToHex.name,
      children: [
        it({
          name: 'converts white',
          fn: async () => {
            expect(figmaColorToHex({ r: 1, g: 1, b: 1, a: 1, },),).toBe('#FFFFFF',);
          },
        },),
        it({
          name: 'converts black',
          fn: async () => {
            expect(figmaColorToHex({ r: 0, g: 0, b: 0, a: 1, },),).toBe('#000000',);
          },
        },),
        it({
          name: 'converts red',
          fn: async () => {
            expect(figmaColorToHex({ r: 1, g: 0, b: 0, a: 1, },),).toBe('#FF0000',);
          },
        },),
        it({
          name: 'converts mid-gray',
          fn: async () => {
            expect(figmaColorToHex({ r: 0.5, g: 0.5, b: 0.5, a: 1, },),).toBe('#808080',);
          },
        },),
      ],
    },),
    // endregion

    // region convertFigmaToPenpot (fig)
    describe({
      name: 'convertFigmaToPenpot (fig)',
      children: [
        it({
          name: 'creates one page per canvas',
          fn: async () => {
            const figmaFile = buildTestFigmaFile();
            const doc = convertFigmaToPenpot({ figmaFile, },);
            expect(doc.pages.size,).toBe(1,);
          },
        },),
        it({
          name: 'sets file name from meta',
          fn: async () => {
            const figmaFile = buildTestFigmaFile();
            const doc = convertFigmaToPenpot({ figmaFile, },);
            expect(doc.file.name,).toBe('Test File',);
          },
        },),
        it({
          name: 'converts frame with fills and strokes',
          fn: async () => {
            const figmaFile = buildTestFigmaFile();
            const doc = convertFigmaToPenpot({ figmaFile, },);
            // Find the frame shape (not root frame)
            const shapes = [...doc.shapes.values(),].filter(s => s.name === 'My Frame');
            expect(shapes,).toHaveLength(1,);
            const frame = nonNullishOrThrow(shapes[0],);
            expect(frame.type,).toBe('frame',);
            expect(frame.fills,).toHaveLength(1,);
            expect(nonNullishOrThrow(frame.fills[0],).fillColor,).toBe('#FF0000',);
            expect(frame.strokes,).toHaveLength(1,);
            expect(nonNullishOrThrow(frame.strokes[0],).strokeWidth,).toBe(2,);
            expect(frame.r1,).toBe(8,);
          },
        },),
        it({
          name: 'converts text node',
          fn: async () => {
            const figmaFile = buildTestFigmaFile();
            const doc = convertFigmaToPenpot({ figmaFile, },);
            const textShapes = [...doc.shapes.values(),].filter(s => s.type === 'text');
            expect(textShapes,).toHaveLength(1,);
            const text = nonNullishOrThrow(textShapes[0],);
            expect(text.name,).toBe('Hello',);
            expect(text.growType,).toBe('auto-width',);
          },
        },),
        it({
          name: 'skips DOCUMENT and NONE types',
          fn: async () => {
            const figmaFile = buildTestFigmaFile();
            const doc = convertFigmaToPenpot({ figmaFile, },);
            // Only 3 shapes should exist: root frame + frame + text
            const nonRootShapes = [...doc.shapes.values(),].filter(s =>
              s.id !== '00000000-0000-0000-0000-000000000000'
            );
            expect(nonRootShapes.length,).toBe(2,);
          },
        },),
      ],
    },),
    // endregion

    // region convertFigmaToPenpot (deck)
    describe({
      name: 'convertFigmaToPenpot (deck)',
      children: [
        it({
          name: 'creates one page per slide',
          fn: async () => {
            const deckFile = buildTestDeckFile();
            const doc = convertFigmaToPenpot({ figmaFile: deckFile, },);
            expect(doc.pages.size,).toBe(1,);
            const page = nonNullishOrThrow([...doc.pages.values(),][0],);
            expect(page.name,).toBe('Slide 1',);
          },
        },),
        it({
          name: 'skips internal-only canvases',
          fn: async () => {
            const deckFile = buildTestDeckFile();
            const doc = convertFigmaToPenpot({ figmaFile: deckFile, },);
            // Only 1 page (the slide), not 2 (slide + internal canvas)
            const pageNames = [...doc.pages.values(),].map(p => p.name);
            expect(pageNames,).not.toContain('Internal Only Canvas',);
          },
        },),
        it({
          name: 'converts slide child text',
          fn: async () => {
            const deckFile = buildTestDeckFile();
            const doc = convertFigmaToPenpot({ figmaFile: deckFile, },);
            const textShapes = [...doc.shapes.values(),].filter(s => s.type === 'text');
            expect(textShapes,).toHaveLength(1,);
            expect(nonNullishOrThrow(textShapes[0],).name,).toBe('Title',);
          },
        },),
      ],
    },),
    // endregion

    // region convertFigmaToPenpot (jam)
    describe({
      name: 'convertFigmaToPenpot (jam)',
      children: [
        it({
          name: 'creates pages from canvases',
          fn: async () => {
            const jamFile = buildTestFigmaFile('jam',);
            const doc = convertFigmaToPenpot({ figmaFile: jamFile, },);
            expect(doc.pages.size,).toBe(1,);
          },
        },),
      ],
    },),
    // endregion

    // region serializePenpotZip
    describe({
      name: serializePenpotZip.name,
      children: [
        it({
          name: 'produces a valid ZIP with manifest',
          fn: async () => {
            const figmaFile = buildTestFigmaFile();
            const doc = convertFigmaToPenpot({ figmaFile, },);
            const zipBuffer = await serializePenpotZip(doc,);
            expect(zipBuffer.length,).toBeGreaterThan(0,);

            // Verify it's a valid ZIP by checking magic bytes
            expect(zipBuffer[0],).toBe(0x50,); // 'P'
            expect(zipBuffer[1],).toBe(0x4B,); // 'K'
          },
        },),
        it({
          name: 'manifest has penpot/export-files type',
          fn: async () => {
            const figmaFile = buildTestFigmaFile();
            const doc = convertFigmaToPenpot({ figmaFile, },);
            expect(doc.manifest.type,).toBe('penpot/export-files',);
            expect(doc.manifest.version,).toBe(1,);
          },
        },),
      ],
    },),
    // endregion

    // region Integration: real file conversion
    describe({
      name: 'integration: real file conversion',
      children: [
        it({
          name: 'converts a real .fig file end-to-end',
          timeout: 30_000,
          fn: async () => {
            const figFile = await parseFigmaFile(
              join(FIGMA_EXPORT_DIR, 'Color palette - base.fig',),
            );
            const figDoc = convertFigmaToPenpot({ figmaFile: figFile, },);
            const figZip = serializePenpotZip(figDoc,);
            expect(figDoc.pages.size,).toBeGreaterThan(0,);
            expect(figDoc.shapes.size,).toBeGreaterThan(0,);
            expect(figZip.length,).toBeGreaterThan(1_000,);
          },
        },),
        it({
          name: 'converts a real .deck file end-to-end',
          timeout: 30_000,
          fn: async () => {
            const deckFile = await parseFigmaFile(
              join(FIGMA_EXPORT_DIR, 'MTM6162-040 participation 2 cover.deck',),
            );
            const deckDoc = convertFigmaToPenpot({ figmaFile: deckFile, },);
            expect(deckDoc.pages.size,).toBeGreaterThanOrEqual(1,);
            for (const page of deckDoc.pages.values())
              expect(page.name,).not.toBe('Internal Only Canvas',);
          },
        },),
        it({
          name: 'converts a real .jam file end-to-end',
          timeout: 30_000,
          fn: async () => {
            const jamFile = await parseFigmaFile(
              join(FIGMA_EXPORT_DIR, 'Todo app - Brainstorming.jam',),
            );
            const jamDoc = convertFigmaToPenpot({ figmaFile: jamFile, },);
            expect(jamDoc.pages.size,).toBeGreaterThan(0,);
            const frames = [...jamDoc.shapes.values(),].filter(s =>
              (s.type === 'frame') && (s.name !== 'Root Frame')
            );
            expect(frames.length,).toBeGreaterThan(0,);
          },
        },),
      ],
    },),
    // endregion
  ],
},);
