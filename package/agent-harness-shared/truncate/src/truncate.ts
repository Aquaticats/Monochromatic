/**
 Neutral tool-output truncation and size formatting.

 Ported from Pi's truncation helpers so every host adapter renders identical model-visible output;
 the parity suite in `truncate-parity.unit.test.ts` proves the behaviors match.

 @module
 */

//region Constants

/**
 Bytes in one kibibyte, the unit Pi's size text divides by.
 */
const BYTES_PER_KIBIBYTE: number = 1_024;

/**
 Bytes in one mebibyte, where size text switches suffix again.
 */
const BYTES_PER_MEBIBYTE: number = BYTES_PER_KIBIBYTE * BYTES_PER_KIBIBYTE;

/**
 Kibibytes in the default byte ceiling, matching Pi's tool-output limits.
 */
const DEFAULT_MAX_KIBIBYTES: number = 50;

/**
 Decimal places size text keeps above whole bytes.
 */
const SIZE_DECIMAL_PLACES: number = 1;

/**
 Line index of the first kept line, which carries no separator before it.
 */
const FIRST_LINE_INDEX: number = 0;

/**
 Separator bytes counted between two kept lines.
 */
const LINE_SEPARATOR_BYTES: number = 1;

/**
 Default line ceiling for model-visible output.
 */
export const DEFAULT_MAX_LINES: number = 2_000;

/**
 Default byte ceiling for model-visible output.
 */
export const DEFAULT_MAX_BYTES: number = DEFAULT_MAX_KIBIBYTES * BYTES_PER_KIBIBYTE;

//endregion Constants

//region Types

/**
 Limits a truncation call may override.
 */
export type TruncationOptions = {
  /**
   Line ceiling applied before the byte ceiling.
   */
  readonly maxLines?: number;
  /**
   Byte ceiling applied before the line ceiling.
   */
  readonly maxBytes?: number;
};

/**
 Record describing what a truncation call kept and dropped.
 */
export type TruncationResult = {
  /**
   Model-visible content after truncation.
   */
  readonly content: string;
  /**
   Whether any content was dropped.
   */
  readonly truncated: boolean;
  /**
   Ceiling that caused the drop, or null when nothing was dropped.
   */
  // oxlint-disable-next-line no-restricted-syntax/no-nullish-union -- mirrors Pi's exported TruncationResult, where `truncatedBy` is `'lines' | 'bytes' | null`; the parity suite compares records field by field, so a sentinel would break equality with the external API being mirrored.
  readonly truncatedBy: 'lines' | 'bytes' | null;
  /**
   Line count of the original content.
   */
  readonly totalLines: number;
  /**
   Byte count of the original content.
   */
  readonly totalBytes: number;
  /**
   Line count kept for the model.
   */
  readonly outputLines: number;
  /**
   Byte count kept for the model.
   */
  readonly outputBytes: number;
  /**
   Whether the kept tail ends mid-line.
   */
  readonly lastLinePartial: boolean;
  /**
   Whether the first line alone exceeded the byte ceiling.
   */
  readonly firstLineExceedsLimit: boolean;
  /**
   Line ceiling in force for this call.
   */
  readonly maxLines: number;
  /**
   Byte ceiling in force for this call.
   */
  readonly maxBytes: number;
};

/**
 Kept-line walk outcome shared by the truncation record builders.
 */
type KeptHeadLines = {
  /**
   Whole lines that fit under both ceilings.
   */
  readonly lines: readonly string[];
  /**
   Byte count of kept lines, counting separators between them.
   */
  readonly bytes: number;
  /**
   Ceiling that stopped the walk.
   */
  readonly truncatedBy: 'lines' | 'bytes';
};

//endregion Types

//region Helpers

/**
 Split content into countable lines, dropping the empty tail a trailing newline implies.

 @param content - raw text being truncated

 @returns lines as counted against the line ceiling

 @example
 ```ts
 splitLinesForCounting('a\nb\n'); // ['a', 'b']
 ```
 */
function splitLinesForCounting(content: string,): readonly string[] {
  if (content.length === 0)
    return [];

  /**
   Lines produced by splitting on newline.
   */
  const lines = content.split('\n',);
  if (content.endsWith('\n',))
    lines.pop();
  return lines;
}

/**
 Format byte counts the way tool descriptions quote truncation limits.

 @param bytes - byte count to render

 @returns compact size text with one decimal above whole bytes

 @example
 ```ts
 formatSize(2_048); // '2.0KB'
 ```
 */
export function formatSize(bytes: number,): string {
  if (bytes < BYTES_PER_KIBIBYTE)
    return `${String(bytes,)}B`;
  if (bytes < BYTES_PER_MEBIBYTE)
    return `${(bytes / BYTES_PER_KIBIBYTE)
      .toFixed(SIZE_DECIMAL_PLACES,)}KB`;
  return `${(bytes / BYTES_PER_MEBIBYTE)
    .toFixed(SIZE_DECIMAL_PLACES,)}MB`;
}

/**
 Truncate content from the head so the model sees the beginning of a response.

 Keeps whole lines only; when the first line alone exceeds the byte ceiling nothing is kept,
 because a partial first line would read as complete content to the model.

 @param content - full response text the model should see the beginning of

 @param options - ceilings to override for tests

 @returns truncation record with visible content and the ceilings used

 @example
 ```ts
 truncateHead({ content: 'a\nb\nc', options: { maxLines: 2 } }).content; // 'a\nb'
 ```
 */
export function truncateHead(
  {
    content,
    options = {},
  }: {
    /**
     Full response text.
     */
    readonly content: string;
    /**
     Ceilings to override for tests.
     */
    readonly options?: TruncationOptions;
  },
): TruncationResult {
  /**
   Line ceiling for this call.
   */
  const maxLines = options.maxLines ?? DEFAULT_MAX_LINES;
  /**
   Byte ceiling for this call.
   */
  const maxBytes = options.maxBytes ?? DEFAULT_MAX_BYTES;
  /**
   Byte count of the original content.
   */
  const totalBytes = Buffer.byteLength(
    content,
    'utf8',
  );
  /**
   Countable lines of the original content.
   */
  const lines = splitLinesForCounting(content,);
  /**
   Line count of the original content.
   */
  const totalLines = lines.length;

  if ((totalLines <= maxLines) && (totalBytes <= maxBytes))
    return {
      content,
      truncated: false,
      truncatedBy: null,
      totalLines,
      totalBytes,
      outputLines: totalLines,
      outputBytes: totalBytes,
      lastLinePartial: false,
      firstLineExceedsLimit: false,
      maxLines,
      maxBytes,
    };

  /**
   Byte count of the first line alone.
   */
  const firstLineBytes = Buffer.byteLength(
    lines[FIRST_LINE_INDEX] ?? '',
    'utf8',
  );
  if (firstLineBytes > maxBytes)
    return {
      content: '',
      truncated: true,
      truncatedBy: 'bytes',
      totalLines,
      totalBytes,
      outputLines: 0,
      outputBytes: 0,
      lastLinePartial: false,
      firstLineExceedsLimit: true,
      maxLines,
      maxBytes,
    };

  /**
   Lines kept for the model plus the ceiling that stopped the walk.
   */
  const kept = (function collectKeptHeadLines(): KeptHeadLines {
    /**
     Whole lines kept for the model.
     */
    const keptLines: string[] = [];
    /**
     Byte count of kept lines, counting separators between them.
     */
    let keptBytes = 0;
    /**
     Ceiling that stopped the walk.
     */
    let stoppedBy: 'lines' | 'bytes' = 'lines';

    for (
      let index = FIRST_LINE_INDEX;
      (index < lines.length) && (index < maxLines);
      index += 1
    ) {
      /**
       Line candidate at the current walk position.
       */
      const line = lines[index] ?? '';
      /**
       Bytes this line adds, counting the separator it shares with earlier kept lines.
       */
      const lineBytes = Buffer.byteLength(
        line,
        'utf8',
      ) + (
        index > FIRST_LINE_INDEX ? LINE_SEPARATOR_BYTES : FIRST_LINE_INDEX
      );
      if ((keptBytes + lineBytes) > maxBytes) {
        stoppedBy = 'bytes';
        break;
      }
      keptLines.push(line,);
      keptBytes += lineBytes;
    }

    if ((keptLines.length >= maxLines) && (keptBytes <= maxBytes))
      stoppedBy = 'lines';

    return {
      lines: keptLines,
      bytes: keptBytes,
      truncatedBy: stoppedBy,
    };
  })();

  /**
   Visible content assembled from kept lines.
   */
  const outputContent = kept
    .lines
    .join('\n',);
  return {
    content: outputContent,
    truncated: true,
    truncatedBy: kept.truncatedBy,
    totalLines,
    totalBytes,
    outputLines: kept
      .lines
      .length,
    outputBytes: Buffer.byteLength(
      outputContent,
      'utf8',
    ),
    lastLinePartial: false,
    firstLineExceedsLimit: false,
    maxLines,
    maxBytes,
  };
}

//endregion Helpers
