/**
 Document-level arbitraries: whole valid TOML texts for the fuzz properties.
 
 Validity is structural, not a filter. Every block in a document owns a unique
 first-segment name drawn from one global unique pool, so no two blocks can
 collide (a top-level key and a `[table]` of the same name, a redefined table,
 and so on). Within a block, keys are likewise unique. Blocks are key-values
 (optionally dotted, optionally with a trailing comment), standard tables, and
 arrays-of-tables, interleaved with header comments and blank lines.
 
 Order is load-bearing: TOML binds every bare key-value to the most recently
 opened table header, so a bare top-level key emitted after a `[table]` would
 silently land inside that table and could collide there. All key-value blocks
 are therefore emitted before any sectioned (`[table]` / `[[aot]]`) block.
 
 @module
 */

import {
  type Arbitrary,
  constantFrom,
  oneof,
  uniqueArray,
} from 'fast-check';

import { drawEach, } from './arb-combinators.ts';
import {
  bareName,
  type KeySegment,
  keySegmentArbitrary,
} from './arb-keys.ts';
import { valueTextArbitrary, } from './arb-values.ts';

/**
 Resolved key name of a segment, used as a uniqueness selector.
 
 @param segment - Key segment whose decoded name to return.
 
 @returns Segment's decoded key name.
 */
function segmentName(segment: KeySegment,): string {
  return segment.name;
}

/**
 Build a `key = value` line arbitrary for a body entry.
 
 @param key - Key segment used as the left side of the entry.
 
 @returns Arbitrary single line with no trailing newline.
 */
function tableEntryLine(key: KeySegment,): Arbitrary<string> {
  return valueTextArbitrary.map(function line(valueText,) {
    return `${key.text} = ${valueText}`;
  },);
}

/**
 Maximum number of blocks in one generated document.
 */
const MAX_BLOCKS = 6;

/**
 Maximum number of entries inside one table or array-of-tables instance.
 */
const MAX_BODY_ENTRIES = 4;

/**
 Maximum number of instances generated for one array-of-tables block.
 */
const MAX_AOT_INSTANCES = 3;

/**
 Optional decoration placed before a block: nothing, a blank line, a comment,
 or a comment after a blank line. Comments and blanks are valid between any
 two statements, so this never threatens document validity.
 */
const blockPrefixArbitrary: Arbitrary<string> = constantFrom(
  '',
  '\n',
  '# section\n',
  '\n# section\n',
);

/**
 Optional trailing same-line comment for a key-value block.
 */
const trailingCommentArbitrary: Arbitrary<string> = constantFrom(
  '',
  ' # trailing',
  '\t# tabbed',
);

/**
 One table body: unique single-segment keys mapped to generated value texts.
 
 @returns Arbitrary of joined `key = value` lines (no trailing newline).
 */
const tableBodyArbitrary: Arbitrary<string> = uniqueArray(
  keySegmentArbitrary,
  {
    maxLength: MAX_BODY_ENTRIES,
    selector: segmentName,
  },
)
  .chain(function fill(keys: readonly KeySegment[],) {
    return drawEach({
      items: keys,
      make: tableEntryLine,
    },)
      .map(function join(lines,) { return lines.join('\n',); },);
  },);

/**
 Build a key-value block under `owner`, optionally dotted, optionally commented.
 
 @returns Arbitrary block text ending in a newline.
 */
function keyValueBlockArbitrary({ owner, }: { readonly owner: string; },): Arbitrary<string> {
  return oneof(
    keySegmentArbitrary,
    keySegmentArbitrary,
  )
    .chain(function withTail(tail,) {
      return oneof(
        valueTextArbitrary.map(function bare(valueText,) { return {
          key: owner,
          valueText,
        }; },),
        valueTextArbitrary.map(function dotted(valueText,) {
          return {
            key: `${owner}.${tail.text}`,
            valueText,
          };
        },),
      );
    },)
    .chain(function withComment(entry: {
      readonly key: string;
      readonly valueText: string;
    },) {
      return trailingCommentArbitrary.map(function render(comment,) {
        return `${entry.key} = ${entry.valueText}${comment}\n`;
      },);
    },);
}

/**
 Build a standard-table block headed `[owner]`.
 
 @returns Arbitrary block text ending in a newline.
 */
function standardTableBlockArbitrary({ owner, }: { readonly owner: string; },): Arbitrary<string> {
  return tableBodyArbitrary.map(function render(body,) {
    return body.length === 0 ? `[${owner}]\n` : `[${owner}]\n${body}\n`;
  },);
}

/**
 Build an array-of-tables block of one to three `[[owner]]` instances.
 
 @returns Arbitrary block text ending in a newline.
 */
function arrayOfTablesBlockArbitrary({ owner, }: { readonly owner: string; },): Arbitrary<string> {
  /**
   Render one instance for this array-of-tables owner.

   @returns Arbitrary section text.

   @example
   ```ts
   instance();
   ```
   */
  function instance(): Arbitrary<string> {
    return tableBodyArbitrary.map(function render(body,) {
      return body.length === 0 ? `[[${owner}]]\n` : `[[${owner}]]\n${body}\n`;
    },);
  }
  return uniqueArray(
    keySegmentArbitrary,
    {
      minLength: 1,
      maxLength: MAX_AOT_INSTANCES,
      selector: segmentName,
    },
  )
    .chain(function instances(markers: readonly KeySegment[],) {
      return drawEach({
        items: markers,
        make: instance,
      },)
        .map(function join(blocks,) { return blocks.join('',); },);
    },);
}

/**
 A generated block tagged by category so the document can order key-values
 ahead of every sectioned block.
 */
type Block = {
  /**
   `'keyvalue'` for bare top-level entries; `'section'` for table headers.
   */
  readonly category: 'keyvalue' | 'section';
  /**
   Rendered block text including its optional prefix decoration.
   */
  readonly text: string;
};

/**
 Build one decorated, category-tagged block for `owner`, choosing its kind
 uniformly.

 @param owner - Key used to name generated blocks.

 @returns Arbitrary block including its category and optional prefix decoration.
 */
function blockArbitrary(owner: string,): Arbitrary<Block> {
  return oneof(
    keyValueBlockArbitrary({ owner, },)
      .map(function tag(text,) {
        return {
          category: 'keyvalue',
          text,
        } as Block;
      },),
    standardTableBlockArbitrary({ owner, },)
      .map(function tag(text,) {
        return {
          category: 'section',
          text,
        } as Block;
      },),
    arrayOfTablesBlockArbitrary({ owner, },)
      .map(function tag(text,) {
        return {
          category: 'section',
          text,
        } as Block;
      },),
  )
    .chain(function decorate(block,) {
      return blockPrefixArbitrary.map(function render(prefix,) {
        return {
          category: block.category,
          text: `${prefix}${block.text}`,
        };
      },);
    },);
}

/**
 Deterministic document examples spanning the major structural shapes.
 */
export const DOCUMENT_EXAMPLES: readonly string[] = [
  '',
  'a = 1\n',
  '# header\nname = "x"\n\n[tbl]\nk = true\n',
  '[[items]]\nid = 1\n[[items]]\nid = 2\n',
  'a.b.c = 1\nflag = false # note\n',
  'arr = [ 1, 2, [ 3, 4 ] ]\ninline = { x = 1, y = "two" }\n',
];

/**
 Whole-document arbitrary producing valid TOML text. Key-value blocks are
 emitted before any sectioned block so no bare entry is captured by a table.
 */
export const documentArbitrary: Arbitrary<string> = uniqueArray(
  bareName,
  {
    maxLength: MAX_BLOCKS,
  },
)
  .chain(function build(owners: readonly string[],) {
    return drawEach({
      items: owners,
      make: blockArbitrary,
    },)
      .map(function join(blocks,) {
    /**
     Bare key-value blocks, kept in generation order ahead of the sections.
     */
        const keyValues = blocks.filter(function isKeyValue(block,) {
          return block.category === 'keyvalue';
        },);
    /**
     Table and array-of-tables blocks, each opening its own header.
     */
        const sections = blocks.filter(function isSection(block,) {
          return block.category === 'section';
        },);
        return [
          ...keyValues,
          ...sections,
        ].map(function text(block,) { return block.text; },)
          .join('',);
      },);
  },);
