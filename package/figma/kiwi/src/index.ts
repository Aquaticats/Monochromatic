/**
 Figma Kiwi binary format parser public API.
 
 @example
 ```ts
 import { parseFigmaFile } from '@monochromatic-dev/figma-kiwi/ts';
 ```
 */

export {
  type BinaryReader,
  createBinaryReader,
} from './binary-reader.ts';
export {
  CANVAS_FIG_MAGIC,
  CANVAS_HEADER_SIZE,
  type CanvasFigSections,
  type CanvasHeader,
  decompressZstd,
  parseCanvasFig,
  parseCanvasHeader,
} from './canvas.ts';
export {
  KiwiDecodeError,
  decodeDocument,
  decodeMessage,
  decodeStruct,
  decodeValue,
} from './decode.ts';
export { parseMetaJson, } from './meta.ts';
export { parseFigmaFile, } from './parse.ts';
export {
  parseKiwiSchema,
  resolveTypeName,
} from './schema.ts';
export {
  FIGMA_DOCUMENT_ABSENT,
  KIWI_PRIMITIVES,
  KIWI_VALUE_ABSENT,
  type FigmaFile,
  type FigmaFileType,
  type FigmaMeta,
  type KiwiDecodedValue,
  type KiwiDefinition,
  type KiwiDefinitionKind,
  type KiwiEnum,
  type KiwiEnumField,
  type KiwiPrimitiveName,
  type KiwiSchema,
  type KiwiStruct,
  type KiwiStructField,
} from './types.ts';
export { extractZipEntries, } from './zip.ts';
