/**
 * Block-boundary markdown chunker shared by the server (no-JS path,
 * import endpoint, seed script) and the client worker.
 *
 * Splits a markdown source string into a sequence of `(md, html, char_count)`
 * chunks, cutting only at block-level boundaries (a blank line outside a
 * fenced code block) so each chunk is independently renderable. A chunk is
 * emitted when its accumulated rendered HTML reaches `CHUNK_TARGET_BYTES`,
 * with a hard cap at `CHUNK_HARD_CAP_BYTES`.
 *
 * Cross-runtime: uses only `micromark` and platform string APIs. No Node-
 * specific imports so it can be bundled into the browser worker.
 */

import { BYTES_PER_KIB, } from '@monochromatic-dev/const-core/ts';
import { micromark, } from 'micromark';

/**
 * Soft chunk target in kibibytes.
 */
const CHUNK_TARGET_KIB = 32;

/**
 * Hard chunk cap in kibibytes.
 */
const CHUNK_HARD_CAP_KIB = 256;

/**
 * Soft target for the rendered HTML size of one chunk. The chunker
 * accumulates blocks until reaching this threshold, then emits.
 *
 * 32 KiB chosen as a compromise: small enough that a single chunk fetch
 * paints the first viewport on a 3G link in under 100 ms, large enough
 * that million-message corpora do not explode chunk count.
 */
export const CHUNK_TARGET_BYTES: number = CHUNK_TARGET_KIB * BYTES_PER_KIB;

/**
 * Hard upper bound for one chunk. A single block that produces more
 * rendered HTML than this still ships as one chunk (we never split inside
 * a block), but the soft target stops accepting additional blocks once
 * the running size reaches the hard cap.
 */
export const CHUNK_HARD_CAP_BYTES: number = CHUNK_HARD_CAP_KIB * BYTES_PER_KIB;

/**
 * One emitted chunk: paired source markdown and pre-rendered HTML, plus
 * the source character count for `messages.char_count` aggregation.
 */
export type RenderedChunk = {
  /**
   * Source markdown for this chunk. Stored in `chunks.md`.
   */
  readonly md: string;
  /**
   * Pre-rendered safe HTML. Stored in `chunks.html`. Safe to inject as-is.
   */
  readonly html: string;
  /**
   * Count of characters in `md`. Aggregated into `messages.char_count`.
   */
  readonly charCount: number;
};

/**
 * Detects whether a line opens or closes a fenced code block.
 * CommonMark allows three or more backticks/tildes optionally indented up
 * to three spaces. We do the cheap regex check; full CommonMark fence
 * matching (info string sanity, matching fence char) is left to the
 * downstream parser.
 *
 * @param line - one source line, no trailing newline
 *
 * @returns `true` if the line is a code fence delimiter, `false` otherwise
 */
function isCodeFence(line: string,): boolean {
  // oxlint-disable-next-line no-restricted-syntax/no-regex -- anchored prefix on one line of bounded length; matches CommonMark fence opener (0-3 spaces + ```/~~~) without backtracking
  return /^ {0,3}(```|~~~)/u.test(line,);
}

/**
 * Splits a markdown source string into block-boundary segments.
 *
 * A segment ends when we hit a blank line outside any fenced code
 * block, or end-of-input. Code-fence content is kept whole even if it
 * contains blank lines, so syntax integrity is preserved across chunk
 * boundaries.
 *
 * @param md - full markdown source
 *
 * @returns generator of block strings in source order
 *
 * @example
 * ```ts
 * for (const block of segmentBlocks('# hi\n\nbody')) {
 *   // '# hi', then 'body'
 * }
 * ```
 */
export function* segmentBlocks(md: string,): Generator<string, void, void> {
  /* oxlint-disable no-restricted-syntax/no-function-root-let -- streaming block segmenter: `buffer` accumulates lines until the blank-line boundary cuts a block; `inFence` toggles on each code-fence line so blanks inside fences do not split */
  /**
   * Accumulator for the current block; yielded by `flush` when a blank-line boundary is reached.
   */
  let buffer = '';
  /**
   * Tracks whether the walker is currently inside a fenced code block.
   */
  let inFence = false;
  /* oxlint-enable no-restricted-syntax/no-function-root-let */

  /**
   * Closes the running buffer, yielding it if non-empty, and resets.
   * Local helper closure; avoids repeating the conditional yield.
   *
   * @returns generator that yields at most one buffered block
   *
   * @example
   * ```ts
   * yield* flush();
   * ```
   */
  function* flush(): Generator<string, void, void> {
    if (buffer.length
      > 0) {
      yield buffer;
      buffer = '';
    }
  }

  // Process line by line. We split on '\n' rather than walking codepoints
  // because CommonMark blanks are line-anchored and split() is the
  // simplest correct primitive for a demo. For multi-GB inputs the worker
  // calls this on chunks of the buffer, never the full thing.
  /**
   * Source split on `\n`; CommonMark blanks are line-anchored so this is the simplest correct primitive.
   */
  const lines = md.split('\n',);
  for (const line of lines) {
    /**
     * Blank-line detection drives the block-boundary cut outside fences.
     */
    const isBlank = line.trim()
      .length
      === 0;

    if (isCodeFence(line,)) {
      inFence = !inFence;
      buffer += `${line}\n`;
      continue;
    }

    if (isBlank && (!inFence)) {
      yield* flush();
      continue;
    }

    buffer += `${line}\n`;
  }

  yield* flush();
}

/**
 * Splits a markdown source into rendered chunks targeting `CHUNK_TARGET_BYTES`
 * of HTML each, cutting only at block boundaries.
 *
 * Each chunk's HTML is produced by `micromark` and is safe to inject
 * (`micromark` html-escapes user input by default; we do not pass
 * `allowDangerousHtml`).
 *
 * @param md - full markdown source
 *
 * @returns generator of `RenderedChunk` objects in source order
 *
 * @example
 * ```ts
 * const chunks = [...renderChunks('# hi\n\nbody')];
 * // chunks[0].html === '<h1>hi</h1>', chunks[0].md === '# hi\n'
 * ```
 */
export function* renderChunks(md: string,): Generator<RenderedChunk, void, void> {
  /* oxlint-disable no-restricted-syntax/no-function-root-let -- streaming chunker: `pendingMd` and `pendingHtml` accumulate across `segmentBlocks` iterations until the soft- or hard-cap threshold triggers an `emit` flush */
  /**
   * Accumulator of source markdown across blocks; flushed at chunk boundaries.
   */
  let pendingMd = '';
  /**
   * Accumulator of rendered HTML; the soft-target threshold compares against its length.
   */
  let pendingHtml = '';
  /* oxlint-enable no-restricted-syntax/no-function-root-let */

  /**
   * Emits the accumulated chunk and resets the buffers.
   * Local helper closure to keep the conditional yield in one place.
   *
   * @returns generator that yields at most one accumulated chunk
   *
   * @example
   * ```ts
   * yield* emit();
   * ```
   */
  function* emit(): Generator<RenderedChunk, void, void> {
    if (pendingMd.length
      === 0)
      return;
    yield {
      md: pendingMd,
      html: pendingHtml,
      charCount: pendingMd.length,
    };
    pendingMd = '';
    pendingHtml = '';
  }

  for (const block of segmentBlocks(md,)) {
    /**
     * Rendered HTML for one block; compared against the hard cap before merging into pending.
     */
    const blockHtml = micromark(block,);

    // A single block over the hard cap still ships as its own chunk;
    // we never split inside a block. Flush any pending chunk first so
    // the oversized block lands alone.
    if (blockHtml.length
      > CHUNK_HARD_CAP_BYTES) {
      yield* emit();
      yield {
        md: block,
        html: blockHtml,
        charCount: block.length,
      };
      continue;
    }

    // If adding this block would push the running HTML past the hard cap,
    // flush the pending chunk before adding.
    if ((pendingHtml.length
      + blockHtml
      .length) > CHUNK_HARD_CAP_BYTES)
      yield* emit();

    pendingMd += block;
    pendingHtml += blockHtml;

    // Soft target: emit when reaching the target threshold. Subsequent
    // blocks start a new chunk.
    if (pendingHtml.length
      >= CHUNK_TARGET_BYTES)
      yield* emit();
  }

  yield* emit();
}

/**
 * Extracts a plain-text excerpt from the start of a markdown source.
 * Used to derive `messages.preview` at finalize time. Strips simple
 * markdown structural characters (#, *, -, \>, backticks, link
 * brackets) so the preview reads as prose. Limited to `maxLength`
 * characters; falls back to the placeholder if the excerpt is empty
 * after stripping (e.g. when the first chunk is a pure code block).
 *
 * @param md - source markdown (typically the first chunk's md)
 *
 * @param maxLength - maximum excerpt length in characters
 *
 * @returns plain-text excerpt or the placeholder
 *
 * @example
 * ```ts
 * extractPreview({ md: '# Hello\n\nWorld', maxLength: 50 }); // 'Hello World'
 * extractPreview({ md: '```\ncode\n```', maxLength: 50 });   // '(no text preview)'
 * ```
 */
export function extractPreview({
  md,
  maxLength,
}: {
  readonly md: string;
  readonly maxLength: number;
},): string {
  /* oxlint-disable no-restricted-syntax/no-regex -- preview extractor strips markdown structural markup from a chunk-sized excerpt (capped well below the chunk hard cap). Each pattern uses lazy quantifiers or negated character classes, so matching is linear in input length with no nested unbounded quantifiers. */
  /**
   * Successively stripped form; built up by chained replaces before length check.
   */
  const stripped = md
    .replaceAll(
      /```[\s\S]*?```/gu,
      ' ',
    )
    .replaceAll(
      /~~~[\s\S]*?~~~/gu,
      ' ',
    )
    .replaceAll(
      /`[^`]*`/gu,
      ' ',
    )
    .replaceAll(
      /!\[[^\]]*\]\([^)]*\)/gu,
      ' ',
    )
    .replaceAll(
      /\[([^\]]*)\]\([^)]*\)/gu,
      '$1',
    )
    .replaceAll(
      /^[ \t]*[#>*\-+][ \t]+/gmu,
      '',
    )
    .replaceAll(
      /[*_~]+/gu,
      '',
    )
    .replaceAll(
      /\s+/gu,
      ' ',
    )
    .trim();
  /* oxlint-enable no-restricted-syntax/no-regex */
  if (stripped.length
    === 0)
    return '(no text preview)';
  return stripped.length
    <= maxLength
    ? stripped
    : stripped.slice(
      0,
      maxLength,
    );
}
