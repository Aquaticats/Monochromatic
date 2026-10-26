import type { Processor, } from 'unified';

//region Without indented code
// A PARSE IN WHICH A LINE INDENTED FOUR SPACES IS NOT CODE, as the page holds it. The strict grammar disables indented
// code (`micromark-extension-mdx-md@2.0.0`, `index.js`: `codeIndented` is among the disabled constructs), so a line
// that CommonMark reads as a lazy line of the paragraph above it, or as a block of indented code, starts whatever block
// its text starts on the page: a footnote definition at any indent, say. A parse with the construct turned off finds
// the blocks where the page does.

/**
 The construct of the plain reading that reads indented code, which the strict grammar disables.
 */
const INDENTED_CODE_CONSTRUCT = 'codeIndented';

/**
 Makes the plugin that turns off, in the parser it is used with, the construct that reads indented code.

 @returns Plugin for the parser

 @example
 ```ts
 const parser = unified().use(remarkParse,).use(withoutIndentedCode(),);
 ```
 */
export function withoutIndentedCode(): (this: Processor) => undefined {
  return function addExtension(this: Processor,): undefined {
    /**
     The parser's data, in which the extensions of the micromark parse are listed.
     */
    const data = this.data();
    data.micromarkExtensions = [
      ...(data.micromarkExtensions ?? []),
      { disable: { null: [INDENTED_CODE_CONSTRUCT,], }, },
    ];
  };
}

//endregion Without indented code
