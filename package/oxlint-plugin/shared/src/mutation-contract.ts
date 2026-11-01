/**
 Shared scanner for project-specific `@mutates` TSDoc contracts.
 
 @module
 */

import {
  isWhitespaceChar,
  isWordChar,
} from './text-character.ts';

/**
 Block tags that terminate an open `@mutates` block.
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
]);

/**
 Triple-backtick delimiter controlling fenced example state.
 */
const FENCE_DELIMITER = '```';

/**
 Sentinel while scanner has no open mutation segment.
 */
const OPEN_MUTATION_UNAVAILABLE: unique symbol = Symbol(
  'mutation scanner lacks open contract segment',
);

/**
 Parsed mutation contract shared by documentation and semantic rules.
 
 @example
 ```ts
 const block: ParsedMutationContractBlock = {
   parameterName: 'state',
   description: 'Updates shared state.',
   hasDescription: true,
   lineOffset: 4,
   blockStartOffset: 20,
   blockEndOffset: 67,
 };
 ```
 */
export type ParsedMutationContractBlock = {
  readonly parameterName: string;
  readonly description: string;
  readonly hasDescription: boolean;
  readonly lineOffset: number;
  readonly blockStartOffset: number;
  readonly blockEndOffset: number;
};

/**
 One normalized comment line with source offsets.
 */
type ContractLine = {
  readonly text: string;
  readonly lineOffset: number;
  readonly rawStartOffset: number;
  readonly rawEndOffset: number;
  readonly inFence: boolean;
};

/**
 Open mutation segment accumulating continuation text.
 */
type OpenMutationSegment = {
  readonly parts: string[];
  readonly lineOffset: number;
  readonly blockStartOffset: number;
};

/**
 Readonly projection accepted by segment finalization.
 */
type ReadonlyOpenMutationSegment = {
  readonly parts: readonly string[];
  readonly lineOffset: number;
  readonly blockStartOffset: number;
};

/**
 Returns exclusive end of first non-whitespace token.
 
 @param text - Text containing token.
 
 @returns token end offset.
 */
function firstTokenEnd(text: string,): number {
  for (let index = 0; index < text.length; index++) {
    if (isWhitespaceChar(text.charAt(index,),))
      return index;
  }
  return text.length;
}

/**
 Returns leading `@word` tag.
 
 @param text - Normalized comment line.
 
 @returns tag text or empty string when absent.
 */
function leadingBlockTag(text: string,): string {
  if (!text.startsWith('@',))
    return '';
  for (let index = 1; index < text.length; index++) {
    if (!isWordChar(text.charAt(index,),))
      return index === 1 ? '' : text.slice(
        0,
        index,
      );
  }
  return text.length === 1 ? '' : text;
}

/**
 Normalizes comment body lines while retaining source offsets and fence state.
 
 @param commentValue - Oxlint-style block comment body without delimiters.
 
 @returns normalized lines in source order.
 */
function normalizedContractLines(commentValue: string,): readonly ContractLine[] {
  /**
   Fold state carrying source cursor, fence state, and rows.
   */
  const state: {
    cursor: number;
    insideFence: boolean;
    readonly lines: ContractLine[];
  } = {
    cursor: 0,
    insideFence: false,
    lines: [],
  };
  commentValue.split('\n',)
    .forEach(function normalize(
      rawLine,
      lineOffset,
    ): void {
    /**
     Raw line start before newline separator.
     */
      const rawStartOffset = state.cursor;
    /**
     Raw line end before newline separator.
     */
      const rawEndOffset = rawStartOffset + rawLine.length;
    /**
     Content after indentation and optional comment star.
     */
      const leadingTrimmed = rawLine.trimStart();
    /**
     Content after leading star marker.
     */
      const withoutStar = leadingTrimmed.startsWith('*',)
        ? leadingTrimmed.slice(1,)
        : leadingTrimmed;
    /**
     Normalized content used for tag and fence scanning.
     */
      const text = withoutStar.trimStart();
    /**
     Whether current line opens or closes fenced code.
     */
      const fenceLine = text.startsWith(FENCE_DELIMITER,);
      state.lines
        .push({
          text,
          lineOffset,
          rawStartOffset,
          rawEndOffset,
          inFence: state.insideFence || fenceLine,
        },);
      if (fenceLine)
        state.insideFence = !state.insideFence;
      state.cursor = rawEndOffset + 1;
    },);
  return state.lines;
}

/**
 Converts open segment to parsed mutation contract.
 
 @param segment - Open segment to finalize.
 
 @param blockEndOffset - Exclusive source offset ending block.
 
 @returns parsed block.
 */
function finalizeSegment({
  segment,
  blockEndOffset,
}: {
  readonly segment: ReadonlyOpenMutationSegment;
  readonly blockEndOffset: number;
},): ParsedMutationContractBlock {
  /**
   Joined content after `@mutates` tag.
   */
  const text = segment.parts
    .join('\n',)
    .trimStart();
  /**
   Exclusive parameter-name token end.
   */
  const nameEnd = firstTokenEnd(text,);
  /**
   Raw first token, with separator-only token treated as absent.
   */
  const rawName = text.slice(
    0,
    nameEnd,
  );
  /**
   Text following parameter name before separator normalization.
   */
  const afterName = text.slice(nameEnd,)
    .trimStart();
  /**
   Description with one optional hyphen separator removed.
   */
  const description = (afterName.startsWith('-',)
    ? afterName.slice(1,)
    : afterName).trim();
  return {
    parameterName: rawName === '-' ? '' : rawName,
    description,
    hasDescription: description.length > 0,
    lineOffset: segment.lineOffset,
    blockStartOffset: segment.blockStartOffset,
    blockEndOffset,
  };
}

/**
 Parses every project `@mutates` block from one TSDoc comment body.
 
 Fenced examples are ignored. Recognized TSDoc block tags terminate an open
 mutation block, while ordinary continuation lines contribute description.
 
 @param commentValue - Block comment body without opening and closing delimiters.
 
 @returns parsed mutation contracts in source order.
 
 @example
 ```ts
 const blocks = parseMutationContractBlocks({
   commentValue: '* @mutates state - Updates shared state.',
 });
 ```
 */
export function parseMutationContractBlocks({
  commentValue,
}: {
  readonly commentValue: string;
},): readonly ParsedMutationContractBlock[] {
  /**
   Parsed result blocks in source order.
   */
  const blocks: ParsedMutationContractBlock[] = [];
  /**
   Mutable holder for current segment without nullish state.
   */
  const current: {
    value: OpenMutationSegment | typeof OPEN_MUTATION_UNAVAILABLE;
  } = { value: OPEN_MUTATION_UNAVAILABLE, };
  normalizedContractLines(commentValue,)
    .forEach(function scan(line,): void {
    /**
     Leading block tag outside fenced examples.
     */
      const tag = line.inFence ? '' : leadingBlockTag(line.text,);
      if ((tag !== '') && BLOCK_TAG_NAMES.has(tag,)) {
        if (current.value !== OPEN_MUTATION_UNAVAILABLE) {
          blocks.push(finalizeSegment({
            segment: current.value,
            blockEndOffset: line.rawStartOffset,
          },),);
          current.value = OPEN_MUTATION_UNAVAILABLE;
        }
        if (tag === '@mutates') {
          current.value = {
            parts: [line.text
              .slice(tag.length,),],
            lineOffset: line.lineOffset,
            blockStartOffset: line.rawStartOffset,
          };
        }
        return;
      }
      if (current.value !== OPEN_MUTATION_UNAVAILABLE)
        current.value
          .parts
          .push(line.text,);
    },);
  if (current.value !== OPEN_MUTATION_UNAVAILABLE) {
    blocks.push(finalizeSegment({
      segment: current.value,
      blockEndOffset: commentValue.length,
    },),);
  }
  return blocks;
}
