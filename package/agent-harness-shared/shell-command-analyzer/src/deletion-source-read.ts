/**
 The read record one shell source text travels in.
 
 Every source the reader opens carries what the reader knows about it:
 the file it came from when it is a file's whole text,
 the line mapping it can prove,
 whether every deletion in it counts,
 and the home folder a target may not name.
 
 Nested sources keep the file and the home folder but no line mapping,
 because their offsets count from their own first character.
 
 @module
 */

import type { LiteralRmPathFinding, } from './literal-rm-path-finding.ts';

//region Constants

/**
 Character that ends one line and starts the next.
 */
const LINE_BREAK = '\n';

/**
 Separator between the identity parts of one read, which no text carries.
 */
const READ_IDENTITY_SEPARATOR = '\u0000';

//endregion Constants

//region Types

/**
 One shell source text still to read, and what the reader knows about it.
 */
type SourceRead = {
  /**
   Shell source text to read.
   */
  readonly source: string;
  /**
   File the source text came from, when it is a file's whole text or a piece of one.
   */
  readonly file?: string;
  /**
   Maps an offset in {@link source} to a line number, when that is possible.
   */
  readonly linesFrom?: (offset: number,) => number;
  /**
   Whether every `rm` in this source is a finding whatever its operands spell,
   because the program running this source supplies the paths.
   */
  readonly unwrittenPaths: boolean;
  /**
   Command line a finding reports instead of the line it was read from,
   which names the program that supplied the paths.
   */
  readonly origin?: string;
  /**
   The account's home folder, which a deletion may not name as its target.
   */
  readonly homeFolder: string;
};

/**
 What reading one shell source text produced.
 */
type SourceReadResult = {
  /**
   Findings this source named.
   */
  readonly findings: readonly LiteralRmPathFinding[];
  /**
   Sources this source launches, still to read.
   */
  readonly reads: readonly SourceRead[];
};

//endregion Types

//region Line reads

/**
 Counts the lines that end before one offset.
 
 @param text - source text the offset belongs to
 
 @param offset - offset to locate
 
 @returns 1-based line number of that offset
 
 @example
 ```ts
 lineOfOffset({ text: 'a\nb', offset: 2, }); // 2
 ```
 */
function lineOfOffset({
  text,
  offset,
}: {
  readonly text: string;
  readonly offset: number;
},): number {
  /**
   Lines counted so far, before any line break is added.
   */
  let line = 1;
  for (let index = 0; index < offset; index += 1) {
    if (text[index] === LINE_BREAK)
      line += 1;
  }
  return line;
}

/**
 Copies what a nested read keeps from the read that launched it.
 
 A nested source is a string the shell parses on its own,
 so its offsets count from its own first character and cannot name a line
 in the enclosing file.
 
 @param read - the read that launched this nested source
 
 @param extra - what this nested read adds
 
 @returns read of the nested source
 
 @example
 ```ts
 nestedReadOf({ read: { source: 'x', unwrittenPaths: false }, extra: { source: 'rm -rf $d', unwrittenPaths: false } });
 ```
 */
function nestedReadOf({
  read,
  extra,
}: {
  readonly read: SourceRead;
  readonly extra: {
    readonly source: string;
    readonly unwrittenPaths: boolean;
    readonly origin?: string;
  };
},): SourceRead {
  return {
    source: extra.source,
    ...(read.file === undefined ? {} : { file: read.file, }),
    ...(extra.origin === undefined ? {} : { origin: extra.origin, }),
    unwrittenPaths: extra.unwrittenPaths,
    homeFolder: read.homeFolder,
  };
}

/**
 Maps offsets in a here-document body to lines in the file holding it.
 
 A body the parser copied unchanged sits at its own offset,
 so a line inside it adds to the line the body starts on.
 A body the parser rewrote (`<<-` drops leading tabs) maps to that first line only.
 
 @param read - the read that declared the here-document
 
 @param bodyOffset - offset of the body in the declaring source, when the parser reported one
 
 @param body - here-document body text
 
 @returns line-mapping property to spread, empty when no line is readable
 
 @example
 ```ts
 heredocLinesFrom({ read, bodyOffset: 21, body: 'rm -rf "$d"\n' });
 ```
 */
function heredocLinesFrom({
  read,
  bodyOffset,
  body,
}: {
  readonly read: SourceRead;
  readonly bodyOffset?: number;
  readonly body: string;
},): { readonly linesFrom?: (offset: number,) => number } {
  if ((read.linesFrom === undefined) || (bodyOffset === undefined))
    return {};
  /**
   Line the body starts on, which every body maps at least to.
   */
  const bodyStartLine = read.linesFrom(bodyOffset,);
  if (!read.source
    .startsWith(
    body,
    bodyOffset,
    )) {
    return {
      linesFrom(): number {
        return bodyStartLine;
      },
    };
  }
  return {
    linesFrom(offset: number,): number {
      return (bodyStartLine + lineOfOffset({
        text: body,
        offset,
      },)) - 1;
    },
  };
}

//endregion Line reads

/**
 Collects reads for the sources one command line launches.
 
 @param read - source text the launching record was read from
 
 @param sources - shell source strings the record runs
 
 @param unwrittenPaths - whether the launched program supplies the paths
 
 @param origin - command line a finding reports instead, when the launching line names more
 
 @returns one read per launched source
 
 @example
 ```ts
 nestedReads({ read, sources: ['rm -rf $d'], unwrittenPaths: false });
 ```
 */
function nestedReads({
  read,
  sources,
  unwrittenPaths,
  origin,
}: {
  readonly read: SourceRead;
  readonly sources: readonly string[];
  readonly unwrittenPaths: boolean;
  readonly origin?: string;
},): readonly SourceRead[] {
  return sources
    .map(function nestedRead(source,): SourceRead {
      return nestedReadOf({
        read,
        extra: {
          source,
          unwrittenPaths,
          ...(origin === undefined ? {} : { origin, }),
        },
      },);
    },);
}

/**
 Reads the words of one raw run with their quote characters dropped./**
 Reports whether one source text was already read under the same rule.
 
 Reading a source once is what ends a source that launches its own text.
 
 @param source - shell source text being read
 
 @param unwrittenPaths - whether every deletion in that source counts
 
 @returns identity of that read
 
 @example
 ```ts
 readIdentity({ source: 'rm -rf $d', unwrittenPaths: false });
 ```
 */
function readIdentity({
  source,
  unwrittenPaths,
}: {
  readonly source: string;
  readonly unwrittenPaths: boolean;
},): string {
  return `${String(unwrittenPaths,)}${READ_IDENTITY_SEPARATOR}${source}`;
}

export {
  heredocLinesFrom,
  lineOfOffset,
  nestedReadOf,
  nestedReads,
  readIdentity,
};
export type { SourceRead, };
export type { SourceReadResult, };
