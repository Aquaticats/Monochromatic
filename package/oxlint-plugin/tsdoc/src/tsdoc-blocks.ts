/**
 In-house TSDoc block scanner.
 
 Splits a comment into its `@param` blocks and `@returns` block and derives
 the tag-presence flags the rules need, replacing the `@microsoft/tsdoc`
 `DocComment` tree with a flat single-pass scan over the comment lines.
 
 @module
 */

import { parseMutationContractBlocks, } from '@monochromatic-dev/oxlint-plugin-shared/ts';
import type { Comment, } from '@oxlint/plugins';
import type { ReadonlyDeep, } from 'type-fest';

import {
  isWhitespaceChar,
  leadingTag,
  type NormalizedLine,
  normalizeLines,
  stripInlineCodeAndEscapes,
  tokenEnd,
} from './comment-text.ts';
import type {
  ParsedDocComment,
  ParsedMutatesBlock,
  ParsedParamBlock,
  ParsedReturnsBlock,
} from './tsdoc-doc-model.ts';

/**
 TSDoc block tags whose appearance at line start opens a new content block.
 
 Includes every standard BlockTag plus the plugin's custom `@yields`, so a
 following block tag correctly ends a preceding `@param`/`@returns` block.
 */
const BLOCK_TAG_NAMES: ReadonlySet<string> = new Set([
  '@decorator',
  '@defaultValue',
  '@deprecated',
  '@example',
  '@param',
  '@privateRemarks',
  '@remarks',
  '@returns',
  '@see',
  '@throws',
  '@typeParam',
  '@jsx',
  '@jsxRuntime',
  '@jsxFrag',
  '@jsxImportSource',
  '@mutates',
  '@yields',
],);

/**
 One block being accumulated as the scanner walks comment lines.
 */
type OpenSegment = {
  /**
   Block tag including the `@`, e.g. `'@param'`.
   */
  readonly tag: string;
  /**
   Zero-based opening-line offset from comment start.
   */
  readonly lineOffset: number;
  /**
   Content fragments after the tag, one per contributing line.
   */
  readonly parts: string[];
};

/**
 A finalized block: its tag plus the joined content text.
 */
type Segment = {
  /**
   Block tag including the `@`.
   */
  readonly tag: string;
  /**
   Zero-based opening-line offset from comment start.
   */
  readonly lineOffset: number;
  /**
   Joined content after the tag, across all the block's lines.
   */
  readonly text: string;
};

/**
 Checks whether `s` contains any non-whitespace character.
 
 @param s - text to inspect
 
 @returns true when at least one character is not whitespace
 */
function hasNonWhitespace(s: string,): boolean {
  for (const ch of s) {
    if (!isWhitespaceChar(ch,))
      return true;
  }
  return false;
}

/**
 Removes a single leading hyphen separator (with surrounding whitespace) so
 the remaining text is the block description.
 
 @param s - block content after the parameter name or returns tag
 
 @returns description text with one leading `-` separator removed
 */
function stripLeadingHyphen(s: string,): string {
  /**
   Content with leading whitespace removed so a leading `-` is detectable.
   */
  const trimmed = s.trimStart();
  return trimmed.startsWith('-',) ? trimmed.slice(1,) : trimmed;
}

/**
 Parses a `@param` block's content into its name and description presence.
 
 The parameter name is the first whitespace-delimited token; a lone `-` is
 treated as a missing name. The remainder, after one hyphen separator, is the
 description.
 
 @param text - block content after the `@param` tag
 
 @returns parsed param block
 */
function parseParamSegment(text: string,): ParsedParamBlock {
  /**
   Content with leading whitespace removed so the name token starts at 0.
   */
  const trimmed = text.trimStart();
  /**
   Exclusive end of the first token; bounds the parameter name.
   */
  const nameEnd = tokenEnd({
    text: trimmed,
    start: 0,
  },);
  /**
   First token of the content; a bare `-` means no name was given.
   */
  const rawName = trimmed.slice(
    0,
    nameEnd,
  );
  /**
   Description text after the name and one hyphen separator.
   */
  const description = stripLeadingHyphen(trimmed.slice(nameEnd,),);
  return {
    parameterName: rawName === '-' ? '' : rawName,
    hasDescription: hasNonWhitespace(description,),
  };
}

/**
 Parses a `@returns` block's content into its description presence.
 
 @param text - block content after the `@returns` tag
 
 @returns parsed returns block
 */
function parseReturnsSegment(text: string,): ParsedReturnsBlock {
  return {
    hasDescription: hasNonWhitespace(stripLeadingHyphen(text,),),
  };
}

/**
 Groups normalized lines into block segments keyed by their opening tag.
 
 A non-fence line starting with a block tag opens a new segment; subsequent
 lines (including fenced example bodies and blank lines) extend the current
 segment (the last pushed one) until the next block tag or end of comment.
 Lines before the first block tag (the summary) have no open segment and are
 dropped.
 
 @param normalizedLines - fence-aware normalized comment lines
 
 @returns finalized block segments in source order
 */
function buildSegments(normalizedLines: readonly NormalizedLine[],): readonly Segment[] {
  /**
   Block segments in source order; the last entry is the currently open one.
   */
  const segments: OpenSegment[] = [];

  normalizedLines.forEach(function collectSegment(
    line,
    lineOffset,
  ): void {
    if (!line.inFence) {
      /**
       Leading tag of the line, or {@link NO_TAG} when the line is untagged.
       */
      const tag = leadingTag(line.text,);
      if (((typeof tag) === 'string') && BLOCK_TAG_NAMES.has(tag,)) {
        segments.push({
          tag,
          lineOffset,
          parts: [line.text
            .slice(tag.length,),],
        },);
        return;
      }
    }
    /**
     Currently open segment, or undefined before the first block tag.
     */
    const last = segments.at(-1,);
    if (last !== undefined)
      last.parts
        .push(line.text,);
  },);

  return segments
    .map(function finalize(segment: ReadonlyDeep<OpenSegment>,): Segment {
      return {
        tag: segment.tag,
        lineOffset: segment.lineOffset,
        text: segment.parts
          .join('\n',),
      };
    },);
}

/**
 Inline `@inheritDoc` opener, detected anywhere on a line.
 */
const INHERIT_DOC_INLINE = '{@inheritDoc';

/**
 Collects tag-presence flags across all non-fence lines.
 
 `@example` and `@internal` count only as a line's leading tag (block and
 modifier tags occupy the line start), so the words appearing in prose (e.g.
 "missing @example") do not register. `@inheritDoc` counts as a leading tag
 or as the inline `{@inheritDoc ...}` form.
 
 @param normalizedLines - fence-aware normalized comment lines
 
 @returns presence of `@example`, `@inheritDoc`, and `@internal`
 */
function collectPresence(normalizedLines: readonly NormalizedLine[],): {
  example: boolean;
  inheritDoc: boolean;
  internal: boolean;
} {
  /**
   Running presence flags toggled as each non-fence line is inspected.
   */
  const found = {
    example: false,
    inheritDoc: false,
    internal: false,
  };
  for (const line of normalizedLines) {
    if (line.inFence)
      continue;
    /**
     Leading tag of the line, or undefined when the line is untagged.
     */
    const tag = leadingTag(line.text,);
    /**
     Whether the line carries the inline `{@inheritDoc ...}` form.
     */
    const inlineInheritDoc = stripInlineCodeAndEscapes(line.text,)
      .includes(INHERIT_DOC_INLINE,);
    if (tag === '@example')
      found.example = true;
    if (tag === '@internal')
      found.internal = true;
    if ((tag === '@inheritDoc') || inlineInheritDoc)
      found.inheritDoc = true;
  }
  return found;
}

/**
 Scans a TSDoc comment into the minimal parsed model the rules consume,
 combining block segments from {@link buildSegments} with tag-presence
 flags from {@link collectPresence}.
 
 @param comment - block comment AST node to scan
 
 @returns parsed doc model with param/returns facts and tag-presence flags
 
 @example
 ```ts
 const docComment = splitDocComment({ comment });
 if (docComment.returnsBlock === undefined) {
   // function lacks a @returns tag
 }
 ```
 */
export function splitDocComment({
  comment,
}: {
  /**
   Block comment whose body is scanned.
   */
  readonly comment: ReadonlyDeep<Comment>;
},): ParsedDocComment {
  /**
   Fence-aware normalized lines shared by segmentation and presence scans.
   */
  const normalizedLines = normalizeLines({ comment, },);
  /**
   Block segments keyed by opening tag.
   */
  const segments = buildSegments(normalizedLines,);
  /**
   Tag-presence flags for `@example`/`@inheritDoc`/`@internal`.
   */
  const presence = collectPresence(normalizedLines,);

  /**
   Parsed `@param` blocks in source order.
   */
  const blocks = segments
    .filter(function isParam(segment,): boolean {
      return segment.tag === '@param';
    },)
    .map(function toParamBlock(segment,): ParsedParamBlock {
      return parseParamSegment(segment.text,);
    },);

  /**
   Parsed `@mutates` blocks in source order.
   */
  const mutatesBlocks = parseMutationContractBlocks({
    commentValue: comment.value,
  },)
    .map(function toTsdocMutatesBlock(block,): ParsedMutatesBlock {
      return {
        parameterName: block.parameterName,
        hasDescription: block.hasDescription,
        lineOffset: block.lineOffset,
      };
    },);

  /**
   First `@returns` segment, or undefined when none is present.
   */
  const returnsSegment = segments.find(function isReturns(segment,): boolean {
    return segment.tag === '@returns';
  },);
  /**
   Optional returns block, spread in only when a `@returns` tag exists so the
   optional property is omitted (not set to `undefined`) when absent.
   */
  const returns = returnsSegment === undefined
    ? {}
    : { returnsBlock: parseReturnsSegment(returnsSegment.text,), };

  return {
    params: { blocks, },
    mutates: { blocks: mutatesBlocks, },
    ...returns,
    hasExampleTag: presence.example,
    hasInheritDocTag: presence.inheritDoc,
    hasInternalModifier: presence.internal,
  };
}
