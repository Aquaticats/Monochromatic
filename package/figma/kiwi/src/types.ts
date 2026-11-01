/**
 Shared Kiwi and Figma parser domain types.
 
 @example
 ```ts
 KIWI_PRIMITIVES.includes('string');
 // true
 ```
 */

/**
 Kiwi primitive type names, indexed by their inverted type code.
 
 @example
 ```ts
 KIWI_PRIMITIVES[5];
 // 'string'
 ```
 */
export const KIWI_PRIMITIVES = [
  'bool',
  'byte',
  'int',
  'uint',
  'float',
  'string',
  'int64',
  'uint64',
] as const;

/**
 Sentinel returned when a decoded Kiwi value has no representable payload.
 
 @example
 ```ts
 typeof KIWI_VALUE_ABSENT;
 // 'symbol'
 ```
 */
export const KIWI_VALUE_ABSENT: unique symbol = Symbol('kiwi decoded value is absent');

/**
 Sentinel returned when a Figma export carries no decoded document payload.
 
 @example
 ```ts
 typeof FIGMA_DOCUMENT_ABSENT;
 // 'symbol'
 ```
 */
export const FIGMA_DOCUMENT_ABSENT: unique symbol = Symbol('figma export document payload is absent');

/**
 Kiwi primitive type name union.
 
 @example
 ```ts
 const name: KiwiPrimitiveName = 'uint';
 ```
 */
export type KiwiPrimitiveName = (typeof KIWI_PRIMITIVES)[number];

/**
 Kind of type definition in a Kiwi schema.
 
 @example
 ```ts
 const kind: KiwiDefinitionKind = 'MESSAGE';
 ```
 */
export type KiwiDefinitionKind = 'ENUM' | 'STRUCT' | 'MESSAGE';

/**
 Enum field in a Kiwi schema.
 
 @example
 ```ts
 const field: KiwiEnumField = { name: 'VALUE', isArray: false, value: 1 };
 ```
 */
export type KiwiEnumField = {
  readonly isArray: boolean;
  readonly name: string;
  readonly value: number;
};

/**
 Field within a struct or message definition.
 
 @example
 ```ts
 const field: KiwiStructField = { name: 'id', type: -4, isArray: false, value: 1 };
 ```
 */
export type KiwiStructField = {
  readonly isArray: boolean;
  readonly name: string;
  readonly type: number;
  readonly value: number;
};

/**
 Enum definition.
 
 @example
 ```ts
 const item: KiwiEnum = { kind: 'ENUM', name: 'Kind', fields: [] };
 ```
 */
export type KiwiEnum = {
  readonly fields: readonly KiwiEnumField[];
  readonly kind: 'ENUM';
  readonly name: string;
};

/**
 Struct or message definition.
 
 @example
 ```ts
 const item: KiwiStruct = { kind: 'STRUCT', name: 'Point', fields: [] };
 ```
 */
export type KiwiStruct = {
  readonly fields: readonly KiwiStructField[];
  readonly kind: 'MESSAGE' | 'STRUCT';
  readonly name: string;
};

/**
 Type definition in a Kiwi schema.
 
 @example
 ```ts
 const item: KiwiDefinition = { kind: 'ENUM', name: 'Kind', fields: [] };
 ```
 */
export type KiwiDefinition = KiwiEnum | KiwiStruct;

/**
 Fully parsed Kiwi schema.
 
 @example
 ```ts
 const schema: KiwiSchema = { definitions: [], enumByName: new Map(), structByName: new Map() };
 ```
 */
export type KiwiSchema = {
  readonly definitions: readonly KiwiDefinition[];
  readonly enumByName: ReadonlyMap<string, KiwiEnum>;
  readonly structByName: ReadonlyMap<string, KiwiStruct>;
};

/**
 Decoded Kiwi value types.
 
 @example
 ```ts
 const value: KiwiDecodedValue = 'Enum.VALUE';
 ```
 */
export type KiwiDecodedValue =
  | boolean
    | number
    | string
    | Uint8Array
    | readonly KiwiDecodedValue[]
    | Record<string, unknown>
    | typeof KIWI_VALUE_ABSENT;

/**
 Figma file type determined by canvas magic bytes.
 
 @example
 ```ts
 const fileType: FigmaFileType = 'fig';
 ```
 */
export type FigmaFileType = 'deck' | 'fig' | 'jam';

/**
 Metadata extracted from meta.json inside the ZIP archive.
 
 @example
 ```ts
 const meta: FigmaMeta = fallbackFigmaMeta();
 ```
 */
export type FigmaMeta = {
  readonly backgroundColor: {
    readonly a: number;
    readonly b: number;
    readonly g: number;
    readonly r: number;
  };
  readonly developerRelatedLinks: readonly unknown[];
  readonly exportedAt: string;
  readonly fileName: string;
  readonly renderCoordinates: {
    readonly height: number;
    readonly width: number;
    readonly x: number;
    readonly y: number;
  };
  readonly thumbnailSize: {
    readonly height: number;
    readonly width: number;
  };
};

/**
 Fully decoded Figma file with all components: {@link FigmaFileType}, {@link FigmaMeta}, and
 {@link KiwiSchema}, alongside decoded document, image, and thumbnail bytes.
 
 @example
 ```ts
 const absent = file.document === FIGMA_DOCUMENT_ABSENT;
 ```
 */
export type FigmaFile = {
  /**
   Little-endian uint32 canvas.fig format version, such as 101 or 106.
   
   Always set by `parseFigmaFile`; optional so values built by hand stay valid.
   */
  readonly canvasVersion?: number;
  readonly document: Record<string, unknown> | typeof FIGMA_DOCUMENT_ABSENT;
  readonly fileType: FigmaFileType;
  readonly images: ReadonlyMap<string, Uint8Array>;
  readonly meta: FigmaMeta;
  readonly schema: KiwiSchema;
  readonly thumbnail: Uint8Array;
};
