import remarkGfm from 'remark-gfm';
import remarkMath from 'remark-math';
import remarkParse from 'remark-parse';
import {
  type Data,
  type Processor,
  unified,
} from 'unified';

import {
  firstNestingExcess,
  isStackOverflow,
} from '@monochromatic-dev/translation-repair-markup/ts';
import { maskHtmlComments, } from './mask-html-comments.ts';
import { maskInvisibleLines, } from './mask-invisible-lines.ts';

//region Module line starts
// WHERE THE STRICT GRAMMAR STARTS AN ES MODULE, read off the grammar's own tokenizer and not off a list of shapes. MDX
// reads a flow block at the first column of the document's own level that starts with `import ` or `export ` as
// JavaScript (`micromark-extension-mdxjs-esm@3.0.0`, `lib/syntax.js`, `tokenizeExportImport`: it gives up when the
// tokenizer is only sniffing a block that may interrupt a paragraph, `self.interrupt`, and when the column is past the
// first). What decides whether a line opens a block is the tokenizer's own state: the containers the line continues, a
// setext underline, a thematic break, a footnote definition that stays open for the line after it, a list marker
// alone. A reader of the lines cannot be right about all of them; the tokenizer is, by construction.
//
// THE TOKENIZER RUNS HERE WITH A CONSTRUCT THAT RECORDS AND REFUSES. At the two letters the module construct is
// registered at, it checks the same two conditions, reads the word and one space, notes where the line starts, and then
// refuses, so the line is read as what it is once its first letter is written as a character reference: the start of
// a paragraph. Refusing is what lets every later line be read in the state the page will hold. A line the module
// construct accepts would swallow every line up to the next blank one, and a second module line among them would never
// be seen.
//
// WHAT IS READ AS THE PACKAGE READS A PAGE. The text is masked as `parseDocument` masks a body before the strict parse:
// a line of invisible characters is blanked first (it is not blank to the tokenizer, which keeps a line of a byte order
// mark in the paragraph above it), then the comments, which MDX has no syntax for. The extensions are those of the
// strict parse that decide where a block starts: GFM (footnote definitions, tables), the site's math, and what MDX
// turns off (`micromark-extension-mdx-md@2.0.0`: autolinks, indented code, and HTML both as a block and inline). The
// two that read braces and tags, the expression and the tag, are left out: they start no block in a text whose
// braces and less-than signs are written as text, which is the only text this is asked about, and they refuse a text
// the tokenizer must still read.
//
// THE TEXT STANDS IN THE MIDDLE OF A PAGE. Two line endings are put before it, so a byte order mark that opens it is
// an ordinary character, where the tokenizer would drop a mark that opens its input, and the text starts a block.

/**
 What is put before the text so that it stands in the middle of a page: a blank line after other content.
 */
const PAGE_LEAD = '\n\n';

/**
 What an ES module of imports starts with: the word and the one space MDX requires after it.
 */
const IMPORT_WORD = 'import ';

/**
 What an ES module of exports starts with: the word and the one space MDX requires after it.
 */
const EXPORT_WORD = 'export ';

/**
 The code of `i`, which `import` starts with.
 */
const LETTER_I = 105;

/**
 The code of `e`, which `export` starts with.
 */
const LETTER_E = 101;

/**
 What MDX turns off, which the strict grammar's block structure is read without.
 */
const MDX_DISABLED = [
  'autolink',
  'codeIndented',
  'htmlFlow',
  'htmlText',
];

/**
 A micromark extension, as the parser's data takes it.
 */
type Extension = NonNullable<Data['micromarkExtensions']>[number];

/**
 A construct of the flow, which the extension registers.
 */
type FlowConstruct = Exclude<NonNullable<NonNullable<Extension['flow']>[number]>, readonly unknown[]>;

/**
 The tokenizer of a construct.
 */
type Tokenizer = FlowConstruct['tokenize'];

/**
 What a tokenizer reads and effects the tokens by.
 */
type Context = ThisParameterType<Tokenizer>;

/**
 The effects a tokenizer is given.
 */
type Effects = Parameters<Tokenizer>[0];

/**
 A state of a tokenizer.
 */
type State = Parameters<Tokenizer>[1];

/**
 Makes the construct that notes where a module line starts and refuses it.

 @param starts - list the offsets are added to, in the order the tokenizer reaches them

 @param word - word the line must start with, and the one space after it

 @returns Construct to register at the first letter of the word

 @example
 ```ts
 const construct = noteModuleLine({ starts: [], word: 'import ', },);
 ```
 */
function noteModuleLine(
  {
    starts,
    word,
  }: {
    readonly starts: number[];
    readonly word: string;
  },
): FlowConstruct {
  return {
    tokenize: function tokenizeModuleLine(
      this: Context,
      effects: Effects,
      _ok: State,
      nok: State,
    ): State {
      // The tokenizer only sniffs a block that may interrupt a paragraph, or a paragraph is open: not a block start.
      if (this.interrupt === true)
        return nok;
      /**
       Where the line starts.
       */
      const origin = this.now();
      /**
       How many codes of the word are read.
       */
      const read = { count: 0, };

      /**
       Reads one code of the word, and notes the line at the space that ends it.

       @param code - code read

       @returns Next state, refused at the end of the word and at a code that is not the word's
       */
      function inWord(code: Parameters<State>[0],): ReturnType<State> {
        if ((code === null) || (code !== word.codePointAt(read.count,)))
          return nok(code,);
        effects.consume(code,);
        read.count += 1;
        if (read.count < word.length)
          return inWord;
        starts.push(origin.offset,);
        return nok;
      }

      return function atStart(code,): ReturnType<State> {
        if (origin.column > 1)
          return nok(code,);
        return inWord(code,);
      };
    },
  };
}

/**
 Makes the plugin that adds the extension to the parser.

 @param starts - list the offsets are added to

 @returns Plugin for the parser

 @example
 ```ts
 const parser = unified().use(remarkParse,).use(noting({ starts: [], },),);
 ```
 */
function noting({ starts, }: { readonly starts: number[]; },): (this: Processor) => undefined {
  return function addExtension(this: Processor,): undefined {
    /**
     The parser's data, in which the extensions of the micromark parse are listed.
     */
    const data = this.data();
    data.micromarkExtensions = [
      ...(data.micromarkExtensions ?? []),
      {
        flow: {
          [LETTER_E]: noteModuleLine({
            starts,
            word: EXPORT_WORD,
          },),
          [LETTER_I]: noteModuleLine({
            starts,
            word: IMPORT_WORD,
          },),
        },
        disable: { null: MDX_DISABLED, },
      },
    ];
  };
}

/**
 Where the strict grammar starts an ES module in a text that stands in the middle of a page.

 A TEXT NESTED PAST THE BOUND IS READ AS HOLDING NONE: the strict grammar refuses it before it reads a line
 (`parse-mdx.ts`), so no line of it starts a module, and the engine's own stack exhaustion is that refusal too.

 @param body - text of a run, as the page holds it: its braces and less-than signs already written as text, its
 comments and lines of invisible characters as they stand

 @returns Index in the body of the first letter of each line that opens a fresh block at the first column and starts
 with `import ` or `export `, in text order

 @example
 ```ts
 const at = moduleLineStartsOf({ body: 'x\n\nimport a\n===\nexport b\n', },);
 ```
 */
export function moduleLineStartsOf({ body, }: { readonly body: string; },): readonly number[] {
  if (firstNestingExcess({
    body,
    grammar: 'mdx',
  },)
    .kind
    === 'beyond') {
    return [];
  }
  /**
   Offsets the tokenizer notes, in the order it reaches them.
   */
  const starts: number[] = [];
  /**
   The text as `parseDocument` hands it to the strict parse.
   */
  const { masked: unwelded, } = maskInvisibleLines({ text: `${PAGE_LEAD}${body}`, },);
  /**
   The text with its comments blanked as well.
   */
  const { masked, } = maskHtmlComments({ text: unwelded, },);
  try {
    unified()
      .use(remarkParse,)
      .use(remarkGfm,)
      .use(remarkMath,)
      .use(noting({ starts, },),)
      .parse(masked,);
  }
  catch (error) {
    // Only the engine's stack exhaustion is the refusal the strict grammar gives such a text; any other failure is an
    // unexpected state that must keep propagating.
    if (!isStackOverflow(error,))
      throw error;
    return [];
  }
  return [...new Set(starts,),]
    .map(function inBody(at,): number {
      return at - PAGE_LEAD.length;
    },)
    .toSorted(function ascending(
      left,
      right,
    ): number {
      return left - right;
    },);
}

//endregion Module line starts
