import type { Processor, } from 'unified';

//region Without raw HTML
// A PARSE IN WHICH A LINE OF RAW HTML IS TEXT, as the page holds it. The page writes every `<` of an untranslated
// original as text, and the strict grammar has no raw HTML (`micromark-extension-mdx-md@2.0.0`, `index.js`: the HTML
// constructs, as a block and inline, are disabled), so a line that opens an HTML block in the plain reading is a line of
// the paragraph above it on the page. A parse with the two constructs turned off finds the blocks, and the footnote
// definitions among them, where the page does.

/**
 The constructs of the plain reading that read raw HTML, which the strict grammar disables.
 */
const RAW_HTML_CONSTRUCTS: readonly string[] = [
  'htmlFlow',
  'htmlText',
];

/**
 Makes the plugin that turns off, in the parser it is used with, the constructs that read raw HTML.

 @returns Plugin for the parser

 @example
 ```ts
 const parser = unified().use(remarkParse,).use(withoutRawHtml(),);
 ```
 */
export function withoutRawHtml(): (this: Processor) => undefined {
  return function addExtension(this: Processor,): undefined {
    /**
     The parser's data, in which the extensions of the micromark parse are listed.
     */
    const data = this.data();
    data.micromarkExtensions = [
      ...(data.micromarkExtensions ?? []),
      { disable: { null: [...RAW_HTML_CONSTRUCTS,], }, },
    ];
  };
}

//endregion Without raw HTML
