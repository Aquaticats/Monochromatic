//region Footnote definition lines inside a container
// A CODE SPAN THAT STARTS BEFORE A FOOTNOTE DEFINITION LINE OF A BLOCK QUOTE OR A LIST ITEM, in every position the strict
// grammar starts the definition at: behind the mark of the quote, on a lazy line of the quote indented up to three
// spaces, and at the content column of the list item or short of it. The plain reading keeps the span across the line;
// the strict grammar ends the paragraph there and starts the definition, so what stands in the span after the colon is
// text of the definition and a brace or a tag in it is syntax.
//
// TEST SUPPORT, NOT PACKAGE SOURCE.

/**
 A shape: what it is, for a case name, and the text.
 */
export type ContainerFootnoteShape = readonly [
  string,
  string,
];

/**
 The indents of the footnote definition line, as spaces: the strict grammar starts a definition after none to three.
 */
const INDENTS: readonly string[] = [
  '',
  ' ',
  '  ',
  '   ',
];

/**
 Texts that put a code span across a footnote definition line inside a container, one for each position and indent.

 @param body - what stands after the colon of the definition and before the span closes

 @returns Name of the shape and the text, a block quote with the line marked, a block quote with a lazy line and a list
 item for each indent from none to three spaces

 @example
 ```ts
 const texts = containerFootnoteTexts({ body: '<b>', },);
 ```
 */
export function containerFootnoteTexts({ body, }: { readonly body: string; },): readonly ContainerFootnoteShape[] {
  return INDENTS.flatMap(function atIndent(spaces,): readonly ContainerFootnoteShape[] {
    /**
     The indent for a case name.
     */
    const named = `an indent of ${String(spaces.length,)}, holding ${body}`;
    return [
      [
        `a code span across a footnote line behind the mark of a block quote, ${named}`,
        `> a \`x\n>${spaces}[^1]: ${body}\` {d}\n`,
      ],
      [
        `a code span across a lazy footnote line of a block quote, ${named}`,
        `> a \`x\n${spaces}[^1]: ${body}\` {d}\n`,
      ],
      [
        `a code span across a footnote line of a list item, ${named}`,
        `- a \`x\n${spaces}[^1]: ${body}\` {d}\n`,
      ],
    ];
  },);
}

//endregion Footnote definition lines inside a container
