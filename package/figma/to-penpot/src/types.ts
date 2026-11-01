/**
 Penpot binfile-v3 data model produced by the Figma converter.
 
 @module figma-to-penpot-types
 */

/**
 UUID v4 string.
 */
export type Uuid = string;

/**
 Penpot shape type discriminant.
 */
export type PenpotShapeType =
  | 'frame'
    | 'group'
    | 'bool'
    | 'rect'
    | 'circle'
    | 'path'
    | 'text'
    | 'image'
    | 'svg-raw';

/**
 Penpot fill object.
 */
export type PenpotFill = {
  fillColor?: string;
  fillOpacity?: number;
  fillColorGradient?: Record<string, unknown>;
  fillImage?: {
    name: string;
    width: number;
    height: number;
    mtype: string;
    id: Uuid;
  };
};

/**
 Penpot stroke object.
 */
export type PenpotStroke = {
  strokeStyle: 'solid' | 'dotted' | 'dashed' | 'mixed';
  strokeAlignment: 'center' | 'inner' | 'outer';
  strokeWidth: number;
  strokeColor?: string;
  strokeOpacity?: number;
  strokeColorGradient?: Record<string, unknown>;
  strokeCapStart?: string;
  strokeCapEnd?: string;
};

/**
 Penpot 2D transform matrix.
 */
export type PenpotTransform = {
  a: number;
  b: number;
  c: number;
  d: number;
  e: number;
  f: number;
};

/**
 Penpot rect (selection rectangle).
 */
export type PenpotSelRect = {
  x: number;
  y: number;
  width: number;
  height: number;
  x1: number;
  y1: number;
  x2: number;
  y2: number;
};

/**
 Penpot point.
 */
export type PenpotPoint = {
  x: number;
  y: number;
};

/**
 Four corner points of a shape's bounding rect, in Penpot order.
 */
export type PenpotPoints = [
  PenpotPoint,
  PenpotPoint,
  PenpotPoint,
  PenpotPoint,
];

/* oxlint-disable no-restricted-syntax/no-nullish-union -- Penpot binfile-v3 stores explicit JSON null for x/y/width/height on shapes without computed geometry, and always for flipX/flipY; the importer treats a null value differently from an absent key, so these cannot collapse to optional `?:` properties */
/**
 A Penpot shape object (core data model), composed of a {@link PenpotShapeType} discriminant,
 {@link PenpotSelRect}, {@link PenpotPoints}, {@link PenpotTransform}, and lists of
 {@link PenpotFill} and {@link PenpotStroke} entries.
 */
export type PenpotShape = {
  id: Uuid;
  name: string;
  type: PenpotShapeType;
  x: number | null;
  y: number | null;
  width: number | null;
  height: number | null;
  rotation: number;
  selrect: PenpotSelRect;
  points: PenpotPoints;
  transform: PenpotTransform;
  transformInverse: PenpotTransform;
  parentId: Uuid;
  frameId: Uuid;
  flipX: boolean | null;
  flipY: boolean | null;
  proportion: number;
  proportionLock: boolean;
  opacity?: number;
  fills: PenpotFill[];
  strokes: PenpotStroke[];
  pageId: Uuid;
  // Container types (frame, group, bool)
  shapes?: Uuid[];
  // Path/bool content
  content?: string | Record<string, unknown>;
  // Text content
  growType?: string;
  // Bool type
  boolType?: string;
  // Frame-specific
  hideFillOnExport?: boolean;
  showContent?: boolean;
  hideInViewer?: boolean;
  // Border radius (rect/frame)
  r1?: number;
  r2?: number;
  r3?: number;
  r4?: number;
  // Layout (frame)
  layout?: string;
  layoutGap?: Record<string, number>;
  layoutPadding?: Record<string, number>;
  layoutFlexDir?: string;
  layoutAlignItems?: string;
  layoutJustifyContent?: string;
  layoutAlignContent?: string;
  layoutWrapType?: string;
  layoutItemHSizing?: string;
  layoutItemVSizing?: string;
  layoutGapType?: string;
  layoutPaddingType?: string;
  // Constraints
  constraintsH?: string;
  constraintsV?: string;
  // Component references
  componentId?: Uuid;
  componentFile?: Uuid;
  componentRoot?: boolean;
  shapeRef?: Uuid;
  mainInstance?: boolean;
  // Image metadata
  metadata?: {
    name: string;
    width: number;
    height: number;
    mtype: string;
    id: Uuid;
  };
  // Touched flags
  touched?: string[];
  // Blocked/hidden
  blocked?: boolean;
  hidden?: boolean;
  locked?: boolean;
  // Blend mode
  blendMode?: string;
  // Shadow
  shadow?: Record<string, unknown>[];
  // Blur
  blur?: Record<string, unknown>;
};
/* oxlint-enable no-restricted-syntax/no-nullish-union */

/**
 A Penpot page.
 */
export type PenpotPage = {
  id: Uuid;
  name: string;
  background: string;
  index: number;
};

/**
 A Penpot media object.
 */
export type PenpotMedia = {
  id: Uuid;
  name: string;
  width: number;
  height: number;
  mtype: string;
  mediaId: Uuid;
  thumbnailId?: Uuid;
  isLocal: boolean;
  createdAt: string;
};

/**
 Penpot manifest entry for a file.
 */
export type PenpotManifestFile = {
  id: Uuid;
  name: string;
  features: string[];
};

/**
 The full Penpot export manifest.
 */
export type PenpotManifest = {
  type: 'penpot/export-files';
  version: number;
  generatedBy: string;
  referer: string;
  files: PenpotManifestFile[];
  relations: [
    Uuid,
    Uuid,
  ][];
};

/**
 A Penpot storage object metadata entry.
 */
export type PenpotStorageObject = {
  id: Uuid;
  size: number;
  contentType: string;
  bucket: string;
  hash?: string;
};

/**
 A Penpot file-level metadata object.
 */
export type PenpotFile = {
  id: Uuid;
  name: string;
  revn: number;
  vern: number;
  createdAt: string;
  modifiedAt: string;
  isShared: boolean;
  hasMediaTrimmed: boolean;
  version: number;
  features: string[];
  options?: Record<string, unknown>;
  migrations?: string[];
};

/**
 A converted Penpot document ready for ZIP serialization.
 */
export type PenpotDocument = {
  manifest: PenpotManifest;
  file: PenpotFile;
  pages: Map<Uuid, PenpotPage>;
  shapes: Map<Uuid, PenpotShape>;
  media: Map<Uuid, PenpotMedia>;
  storageObjects: Map<Uuid, {
    meta: PenpotStorageObject;
    data: Uint8Array;
  }>;
  components: Map<Uuid, Record<string, unknown>>;
  colors: Map<Uuid, Record<string, unknown>>;
  typographies: Map<Uuid, Record<string, unknown>>;
  tokens?: Record<string, unknown>;
  thumbnails: {
    path: string;
    data: Record<string, unknown>;
  }[];
};

/**
 Options for the converter.
 */
export type ConvertOptions = {
  /**
   File name override (defaults to Figma meta.fileName).
   */
  fileName?: string;
  /**
   Generator string for manifest.
   */
  generatedBy?: string;
};
