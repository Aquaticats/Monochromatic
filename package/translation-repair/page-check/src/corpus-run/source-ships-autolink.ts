import {
  autolinkWritten,
  cutByTable,
} from './source-ships-links.ts';
import type {
  SourceSpans,
  TextRange,
} from './source-ships-ranges.ts';

//region Source ships autolink
// AN ANGLE AUTOLINK, which MDX reads as a tag, becomes a link, except inside the text of a link, which holds none: its
// address is written as text there, without the brackets, and where a table cuts it at a pipe, which leaves no autolink in
// the source, so it is text as it stands.

/**
 What a reading of an autolink comes to before it finds anything.
 */
const NOTHING: SourceSpans = {
  text: [],
  literal: [],
  edits: [],
  insideLinks: [],
  streamStarts: [],
};

/**
 Reads an angle autolink.

 @param source - text

 @param url - target the parse gives the autolink

 @param range - where the autolink stands, brackets included

 @param tables - where the strict grammar reads a table in the text

 @param insideLink - whether the autolink stands inside the text of a link

 @returns Autolink as the stretch read as text where a table cuts it, otherwise the edit that writes it as a link, or as
 its address inside the text of a link

 @example
 ```ts
 const { text, edits, } = autolinkReading({ source, url: node.url, range, tables, insideLink: false, },);
 ```
 */
export function autolinkReading(
  {
    source,
    url,
    range,
    tables,
    insideLink,
  }: {
    readonly source: string;
    readonly url: string;
    readonly range: TextRange;
    readonly tables: readonly TextRange[];
    readonly insideLink: boolean;
  },
): SourceSpans {
  if (cutByTable({
    source,
    range,
    tables,
  },)) {
    return {
      ...NOTHING,
      text: [range,],
    };
  }
  return {
    ...NOTHING,
    edits: [
      {
        ...range,
        replacement: autolinkWritten({
          source,
          url,
          range,
          insideLink,
        },),
      },
    ],
  };
}

//endregion Source ships autolink
