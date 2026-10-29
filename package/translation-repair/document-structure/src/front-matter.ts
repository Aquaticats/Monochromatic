import {
  errorOrThrow,
  nonNullishOrThrow,
} from '@monochromatic-dev/or-throw-core/ts';
import {
  parse as parseYaml,
  YAMLError,
} from 'yaml';

import { NAMED_POSITION_UNSTATED, } from './refusal-text.ts';

//region Front matter model
// Upstream strips metadata before MDX compilation and repairs must never rewrite this
// zone, so the split keeps an exact raw slice alongside parsed data.

/**
 Verbatim YAML front matter block split from a corpus MDX document.

 @example
 ```ts
 const block: FrontMatterBlock = { raw: '---\nname: x\n---\n', data: { name: 'x', }, startOffset: 0, };
 ```
 */
export type FrontMatterBlock = {
  /**
   Exact source slice spanning both fence lines, preserved code unit for code
   unit after decoding, which is what keeps every downstream offset valid.
   A byte order mark opening the page is not part of it.
   */
  readonly raw: string;

  /**
   Parsed YAML value; typed `unknown` because corpus metadata shapes vary per entry.
   */
  readonly data: unknown;

  /**
   Offset of the opening fence in the text as written: zero, or one where the
   page opens with a byte order mark, which is no content and stays where the
   original had it.
   */
  readonly startOffset: number;
};

/**
 Result of splitting optional front matter from remaining MDX body.

 @example
 ```ts
 const split: SplitMdxDocument = { frontMatter: undefined, body: '# t', bodyOffset: 0, };
 ```
 */
export type SplitMdxDocument = {
  /**
   Present only when source opens with a well-terminated `---` fence pair,
   after a byte order mark if it opens with one.
   */
  readonly frontMatter?: FrontMatterBlock;

  /**
   MDX source following front matter; equals whole input when front matter is absent,
   a byte order mark opening it included.
   */
  readonly body: string;

  /**
   Character offset of body start inside original text;
   anchors mdast positions back onto full-document offsets.
   */
  readonly bodyOffset: number;
};

//endregion Front matter model

//region Front matter splitting

/**
 The byte order mark a page may open with, which is no content of the page.
 */
const BYTE_ORDER_MARK = '\uFEFF';

/**
 How many characters of a page's start are a byte order mark, which is not
 content: front matter opens after it, and whatever the package writes for
 the page keeps it where the original had it.

 @param text - whole document text as written

 @returns One when the text opens with the mark, otherwise zero

 @example
 ```ts
 const width = leadingMarkWidth({ text: '\uFEFF---\nname: Mittens\n---\n', },);
 ```
 */
export function leadingMarkWidth({ text, }: { readonly text: string; },): number {
  return text.startsWith(BYTE_ORDER_MARK,)
    ? BYTE_ORDER_MARK.length
    : 0;
}

/**
 Describes where a YAML refusal stopped, quoting nothing it read.

 READ THROUGH THE PARSER'S OWN CLASS, whose position and code are typed.
 `yaml` raises a syntax refusal as a `YAMLError` carrying both, its position
 always set from the line counter `parse` creates, and an anchor fault (an
 alias naming no anchor, an anchor holding itself) as a plain
 `ReferenceError` or `TypeError` carrying neither. Shape checks on an
 unknown value stood here, and their stand-ins for a value not an `Error`
 and for a malformed position answered states the parser never produces,
 where a changed parser should fail loudly instead (T8, twenty-third batch).

 @param cause - parser's refusal

 @returns Phrase naming position and fault code

 @throws {@link Error} when a syntax refusal carries no position, which the
 parser's own `parse` never leaves unset

 @example
 ```ts
 `refused to parse ${yamlRefusalSite({ cause, },)}`;
 ```
 */
export function yamlRefusalSite({ cause, }: { readonly cause: Readonly<Error>; },): string {
  if (!(cause instanceof YAMLError))
    return `at ${NAMED_POSITION_UNSTATED} (${cause.name})`;

  /**
   Position the refusal begins at.
   */
  const [start,] = nonNullishOrThrow(cause.linePos,);
  return `at line ${String(start.line,)} column ${String(start.col,)} (${cause.code})`;
}

/**
 Signals YAML inside a front matter fence pair that refuses to parse;
 corpus metadata parses upstream, so failure here indicates corruption.

 @example
 ```ts
 throw new FrontMatterParseError({ cause: error, },);
 ```
 */
export class FrontMatterParseError extends Error {
  /**
   Declares this message safe to forward: it states where the parser stopped
   and what it called the fault, and repeats no front matter.
   */
  readonly messageNamesOnly: true = true;

  /**
   Builds failure stating where YAML stopped, never what it read.

   DOES NOT CARRY THE PARSER ERROR AS `cause`, deliberately. A
   `YAMLParseError` message embeds a source code frame, measured on five
   failure shapes and present in all five, and Node's uncaught-exception
   reporter renders a cause chain. Front matter names a person, so carrying
   the original would publish it through any printer that never asked to.

   Nothing is lost for diagnosis: the position and the parser's own code are
   what a reader acts on, and the file is on disk to open at that line.

   @param cause - underlying YAML parser error, read for position and code

   @example
   ```ts
   new FrontMatterParseError({ cause: error, },);
   ```
   */
  public constructor({ cause, }: { readonly cause: Readonly<Error>; },) {
    super(
      `Front matter fence pair found but YAML inside refused to parse ${
        yamlRefusalSite({ cause, },)
      }; corpus metadata parses upstream, so this signals corruption.`,
    );
    this.name = 'FrontMatterParseError';
  }
}

/**
 The YAML refusal a catch around {@link splitFrontMatter} holds, for a catch
 that acts on the refusal alone.

 SHARED RATHER THAN REPEATED, as `requireMdxRefusal` (`parse-mdx.ts`) is
 for the Markdown grammar: the splitter raises every refusal as a
 {@link FrontMatterParseError}, so a rethrow written in each catch is a
 statement no case reaches. The narrowing stands here once, where a case
 reaches the rethrow.

 @param error - what the catch caught

 @returns Refusal

 @throws The caught value unchanged when it is anything but the YAML
 refusal, an unexpected state that must keep propagating

 @example
 ```ts
 const refusal = requireFrontMatterRefusal({ error, },);
 ```
 */
export function requireFrontMatterRefusal({ error, }: { readonly error: unknown; },): FrontMatterParseError {
  if (error instanceof FrontMatterParseError)
    return error;
  throw error;
}

/**
 Fence line delimiting YAML front matter on both sides.
 */
const FRONT_MATTER_FENCE = '---';

/**
 Opening sequence a document must start with, after any byte order mark, for
 front matter to exist.
 */
const FENCE_OPEN = `${FRONT_MATTER_FENCE}\n`;

/**
 Closing sequence terminating front matter when body content follows.
 */
const FENCE_CLOSE_INNER = `\n${FRONT_MATTER_FENCE}\n`;

/**
 Closing sequence terminating front matter at end of input.
 */
const FENCE_CLOSE_EOF = `\n${FRONT_MATTER_FENCE}`;

/**
 The three fence strings for one document's line ending.
 */
type FenceSet = {
  /**
   Sequence the document must open with.
   */
  readonly open: string;

  /**
   Sequence closing front matter when a body follows.
   */
  readonly closeInner: string;

  /**
   Sequence closing front matter at end of input.
   */
  readonly closeEof: string;
};

/**
 Fences matching the line ending this document actually uses.

 The fences were fixed to `\n`, so a document written with CRLF failed
 `startsWith` and reported NO front matter at all. One corpus original does
 exactly that, and the consequence is not a missing field: the whole YAML
 block is then parsed as body, where `---` becomes a thematic break and
 `name: Ara` becomes a setext heading. The critics receive the metadata as
 content to compare, and the identity context built from front matter is
 empty for the one entry whose names most needed declaring.

 Matching rather than rewriting, because every offset downstream indexes the
 original text and normalising line endings would move all of them.

 @param text - whole document text

 @param start - where the opening fence stands, after any leading mark

 @returns Fence strings to match with

 @example
 ```ts
 const fences = fencesFor({ text, start: 0, },);
 ```
 */
function fencesFor(
  {
    text,
    start,
  }: {
    readonly text: string;
    readonly start: number;
  },
): FenceSet {
  /**
   Opening fence spelled with a carriage return.
   */
  const carriageOpen = `${FRONT_MATTER_FENCE}\r\n`;
  if (!text.startsWith(
    carriageOpen,
    start,
  )) {
    return {
      open: FENCE_OPEN,
      closeInner: FENCE_CLOSE_INNER,
      closeEof: FENCE_CLOSE_EOF,
    };
  }

  return {
    open: carriageOpen,
    closeInner: `\r\n${FRONT_MATTER_FENCE}\r\n`,
    closeEof: `\r\n${FRONT_MATTER_FENCE}`,
  };
}

/**
 Parses YAML between fences, converting parser failures into domain errors.

 @param yamlSource - text between fence lines

 @returns Parsed YAML value; corpus shapes vary, so no schema is imposed here

 @throws {@link FrontMatterParseError} when YAML refuses to parse

 @example
 ```ts
 parseFrontMatterYaml({ yamlSource: 'name: mittens', },);
 ```
 */
function parseFrontMatterYaml(
  { yamlSource, }: { readonly yamlSource: string; },
): unknown {
  try {
    return parseYaml(yamlSource,);
  }
  catch (error) {
    // A REFUSAL IS AN ERROR: the parser throws a YAMLError, or a
    // ReferenceError or TypeError for an anchor fault, and nothing else
    // (ledger T8, twenty-third batch). A caught value that is not an Error
    // is an unexpected state, which `errorOrThrow` fails on loudly, naming
    // what arrived, rather than wrapping it as a refusal.
    throw new FrontMatterParseError({ cause: errorOrThrow(error,), },);
  }
}

/**
 Builds split result once closing fence position is known.

 @param text - full document source

 @param start - where the opening fence stands, after any leading mark

 @param closeStart - index of newline beginning closing fence sequence

 @param rawEnd - end index (exclusive) of raw front matter slice

 @param openLength - length of the opening fence line, newline included, which
 is where the YAML starts

 @returns Split with parsed front matter and offset-adjusted body

 @throws {@link FrontMatterParseError} when YAML between fences refuses to parse

 @example
 ```ts
 buildSplit({ text: '---\nname: n\n---\n', start: 0, closeStart: 11, rawEnd: 16, openLength: 4, },);
 ```
 */
function buildSplit(
  {
    text,
    start,
    closeStart,
    rawEnd,
    openLength,
  }: {
    readonly text: string;
    readonly start: number;
    readonly closeStart: number;
    readonly rawEnd: number;
    readonly openLength: number;
  },
): SplitMdxDocument {
  /**
   Parsed metadata value from YAML between fence lines.
   */
  const data: unknown = parseFrontMatterYaml({
    yamlSource: text.slice(
      start + openLength,
      closeStart,
    ),
  },);

  return {
    frontMatter: {
      raw: text.slice(
        start,
        rawEnd,
      ),
      data,
      startOffset: start,
    },
    body: text.slice(rawEnd,),
    bodyOffset: rawEnd,
  };
}

/**
 Splits optional YAML front matter from MDX body without regex,
 mirroring how upstream strips metadata before MDX compilation.

 Unterminated fences are treated as body text:
 remark parses stray `---` lines as thematic breaks,
 so returning whole input preserves that interpretation instead of guessing.

 A byte order mark opening the text is not content:
 the opening fence is read after it, the mark stays outside the raw slice,
 and every offset the split names indexes the text as written.
 A mark anywhere else is content and is left alone.

 @param text - full document source possibly opening with YAML front matter

 @returns Split parts plus body offset for absolute position anchoring

 @throws {@link FrontMatterParseError} when fences exist but YAML refuses to parse

 @example
 ```ts
 const { frontMatter, body, bodyOffset, } = splitFrontMatter({
   text: '---\nname: n\n---\n\nBody',
 },);
 ```
 */
export function splitFrontMatter({ text, }: { readonly text: string; },): SplitMdxDocument {
  /**
   Where the opening fence stands: after a byte order mark opening the text.
   */
  const start = leadingMarkWidth({ text, },);

  /**
   Fences spelled with this document's own line ending.
   */
  const fences = fencesFor({
    text,
    start,
  },);
  if (!text.startsWith(
    fences.open,
    start,
  ))
    return {
      body: text,
      bodyOffset: 0,
    };

  /**
   Index of newline beginning closing fence sequence; -1 when only EOF close can apply.
   Search starts at opening fence's newline so empty front matter still terminates.
   */
  const {
    open,
    closeInner,
    closeEof,
  } = fences;

  /**
   Offset of the opening fence's own line break, counted from the fence.
   */
  const breakInFence = open.length - closeInner.indexOf(
    FRONT_MATTER_FENCE,
    0,
  );

  /**
   Where the search for a closing fence begins: at the opening fence's own
   line break, so empty front matter still terminates.
   */
  const searchFrom = start + breakInFence;

  /**
   Index of the line break beginning the closing fence; -1 when only an
   end-of-input close can apply.
   */
  const closeStart = text.indexOf(
    closeInner,
    searchFrom,
  );

  if (closeStart !== (-1)) {
    return buildSplit({
      text,
      start,
      closeStart,
      rawEnd: closeStart + closeInner.length,
      openLength: open.length,
    },);
  }

  /**
   Candidate index of newline beginning EOF-terminated closing fence.
   */
  const eofCloseStart = text.length - closeEof.length;

  // The closing fence's line break may be the opening fence's own, which is
  // what empty front matter looks like, but it may not start before it.
  //
  // That boundary is where the opening fence's TEXT ends, never `open.length`
  // minus one: subtracting one assumes a single-character terminator, so under
  // CRLF the bound landed one past the line break and `---\r\n---` was refused
  // outright. Every other CRLF shape parsed, which is what kept this hidden.
  if (text.endsWith(closeEof,) && (eofCloseStart >= (start + FRONT_MATTER_FENCE.length))) {
    return buildSplit({
      text,
      start,
      closeStart: eofCloseStart,
      rawEnd: text.length,
      openLength: open.length,
    },);
  }

  return {
    body: text,
    bodyOffset: 0,
  };
}

//endregion Front matter splitting
