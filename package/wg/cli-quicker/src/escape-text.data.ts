/**
 An inclusive range of code points.
 */
export type CodePointRange = {
  /**
   First code point of the range.
   */
  readonly first: number;

  /**
   Last code point of the range.
   */
  readonly last: number;
};

/**
 Code points that can change how a terminal or a reader shows text,
 and that therefore never reach a log line as they are:
 the C0 controls,
 delete and the C1 controls,
 the Arabic letter mark,
 zero-width and direction marks,
 line and paragraph separators,
 embedding and override controls,
 word joiners and invisible operators,
 isolates,
 the byte order mark,
 and the invisible tag characters.
 */
export const UNSAFE_CODE_POINT_RANGES: readonly CodePointRange[] = [
  {
    first: 0x00,
    last: 0x1F,
  },
  {
    first: 0x7F,
    last: 0x9F,
  },
  {
    first: 0x6_1C,
    last: 0x6_1C,
  },
  {
    first: 0x20_0B,
    last: 0x20_0F,
  },
  {
    first: 0x20_28,
    last: 0x20_29,
  },
  {
    first: 0x20_2A,
    last: 0x20_2E,
  },
  {
    first: 0x20_60,
    last: 0x20_64,
  },
  {
    first: 0x20_66,
    last: 0x20_69,
  },
  {
    first: 0xFE_FF,
    last: 0xFE_FF,
  },
  {
    first: 0xE_00_00,
    last: 0xE_00_7F,
  },
];

/**
 Hexadecimal digits an escaped code point is padded to.
 */
export const ESCAPE_DIGITS = 4;

/**
 Radix of the digits of an escaped code point.
 */
export const ESCAPE_RADIX = 16;

/**
 Entities for the characters that are syntax in notification markup.
 */
export const MARKUP_ENTITIES: ReadonlyMap<string, string> = new Map([
  [
    '&',
    '&amp;',
  ],
  [
    '<',
    '&lt;',
  ],
  [
    '>',
    '&gt;',
  ],
  [
    '"',
    '&quot;',
  ],
  [
    '\'',
    '&apos;',
  ],
],);
