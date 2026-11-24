/**
 Reads the `rm` commands in one shell source text.
 
 This module owns everything one source text can answer:
 the command records the parser reports,
 the raw word runs it rejects,
 the here-document bodies it declares,
 and the sources those records launch.
 What a launched source is silent about,
 such as the file a script sits in,
 is carried in the read itself.
 
 Nested sources are handed back as reads rather than followed here,
 so a caller reads each one under the line mapping it can prove.
 
 @module
 */

import { analyzeShellCommand, } from './analyzer.ts';
import { unwrapWrappers, } from './command-wrapper-words.ts';
import { lastPathSegment, } from './executable-path-words.ts';
import type { LiteralRmPathFinding, } from './literal-rm-path-finding.ts';
import { rawCommandWordRuns, } from './raw-command-word-runs.ts';
import { shellScriptPayloads, } from './shell-script-payloads.ts';
import { shellWordCarriesExpansion, } from './shell-word-carries-expansion.ts';
import type { ShellCommandInfo, } from './types.ts';

//region Constants

/**
 Program name whose operands this reader inspects.
 */
const RM_PROGRAM_NAME = 'rm';

/**
 Wrapper that hands its program the paths it read, instead of the program naming them.
 */
const XARGS_WRAPPER_NAME = 'xargs';

/**
 Character a quote-free raw word keeps where an expansion stood.
 */
const DOLLAR_SIGN = '$';

/**
 Character that runs the text it encloses as a command.
 */
const BACKTICK = '`';

/**
 Character that ends one line and starts the next.
 */
const LINE_BREAK = '\n';

/**
 Separator between the identity parts of one read, which no text carries.
 */
const READ_IDENTITY_SEPARATOR = '\u0000';

/**
 Characters that spell the same word whether or not a raw scan honoured quoting.
 */
const QUOTE_CHARACTERS: readonly string[] = [
  '"',
  '\'',
  '\\',
];

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

//region Predicates

/**
 Reports whether one word names the `rm` program.
 
 The word is read by its last path segment, so `/usr/bin/rm` and `./rm` are
 the same program.
 A different program that takes `rm` as a subcommand (`git rm`, `cargo rm`)
 is not this program, because only the command word is read here.
 
 @param word - command word of a parsed record or a raw run
 
 @returns whether the word runs `rm`
 
 @example
 ```ts
 isRmProgramWord('/usr/bin/rm'); // true
 isRmProgramWord('git'); // false
 ```
 */
function isRmProgramWord(word: string,): boolean {
  return lastPathSegment(word,) === RM_PROGRAM_NAME;
}

/**
 Reports whether a quote-free raw word carries an expansion.
 
 Raw words keep no quotes,
 so a single-quoted `$d` reaches this reader as an expansion it cannot tell
 from an unquoted one.
 
 @param word - word from a raw word run
 
 @returns whether the word spells a dollar sign or a backtick
 
 @example
 ```ts
 rawWordCarriesExpansion('$d'); // true
 ```
 */
function rawWordCarriesExpansion(word: string,): boolean {
  return word.includes(DOLLAR_SIGN,)
    || word.includes(BACKTICK,);
}

/**
 Lists the words of one `rm` command that make its paths unreadable.
 
 @param unwrittenPaths - whether the paths come from outside this command
 
 @param programWord - word naming the `rm` program
 
 @param operandSourceTexts - operand words as written, or quote-free raw words
 
 @param carriesExpansion - decides whether one operand word spells an expansion
 
 @returns offending words, the program word alone when the paths come from outside
 
 @example
 ```ts
 rmOperandFindings({
   unwrittenPaths: false,
   programWord: 'rm',
   operandSourceTexts: ['-rf', '"$d"'],
   carriesExpansion: shellWordCarriesExpansion,
 }); // ['"$d"']
 ```
 */
function rmOperandFindings({
  unwrittenPaths,
  programWord,
  operandSourceTexts,
  carriesExpansion,
}: {
  readonly unwrittenPaths: boolean;
  readonly programWord: string;
  readonly operandSourceTexts: readonly string[];
  readonly carriesExpansion: (sourceText: string,) => boolean;
},): readonly string[] {
  if (unwrittenPaths)
    return [programWord,];
  return operandSourceTexts
    .filter(carriesExpansion,);
}

/**
 Reports whether one source text was already read under the same rule.
 
 Reading a source once is what ends a source that launches its own text.
 
 @param source - shell source text being read
 
 @param unwrittenPaths - whether every `rm` in that source counts
 
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

//endregion Predicates

//region Command reads

/**
 Spells one command record as the command line it was written as.
 
 @param info - parsed command record
 
 @returns command word followed by every operand's source spelling
 
 @example
 ```ts
 commandLineText(info); // 'rm -rf "$d"'
 ```
 */
function commandLineText(info: ShellCommandInfo,): string {
  return [
    info.name,
    ...info.argSources
      .map(function sourceSpelling(arg,): string {
        return arg.sourceText;
      },),
  ]
    .join(' ',);
}

/**
 Reads one command record and the sources it launches.
 
 @param info - parsed command record
 
 @param read - source text the record was read from
 
 @returns findings the record named and the sources it launches
 
 @example
 ```ts
 commandRmRead({ info, read: { source: 'rm -rf "$d"', unwrittenPaths: false, } });
 ```
 */
function commandRmRead({
  info,
  read,
}: {
  readonly info: ShellCommandInfo;
  readonly read: SourceRead;
},): SourceReadResult {
  /**
   Innermost program words and every source the stripped wrappers revealed.
   */
  const wrapped = unwrapWrappers({
    words: [
      info.name,
      ...info.args,
    ],
  },);
  /**
   Command line this record writes, used to name what was found.
   */
  const command = commandLineText(info,);
  /**
   Line of this command in its file, when the reader can locate it.
   */
  const line = ((read.linesFrom === undefined) || (info.sourceOffset === undefined))
    ? undefined
    : read.linesFrom(info.sourceOffset,);
  /**
   Operand words as written, which are the words after the program word.
   
   Unwrapping only drops words in front of the program,
   so the program's operands are the last words this record spells.
   */
  const operandSourceTexts = info.argSources
    .slice(info.argSources
      .length
      - Math.max(
        wrapped.words
          .length
          - 1,
        0,
      ),)
    .map(function sourceSpelling(arg,): string {
      return arg.sourceText;
    },);

  return {
    findings: isRmProgramWord(wrapped.words
      .at(0,)
      ?? '',)
      ? rmOperandFindings({
        unwrittenPaths: read.unwrittenPaths
          || wrapped.wrapperNames
            .includes(XARGS_WRAPPER_NAME,),
        programWord: wrapped.words
          .at(0,)
          ?? RM_PROGRAM_NAME,
        operandSourceTexts,
        carriesExpansion: shellWordCarriesExpansion,
      },)
        .map(function findingForWord(word,): LiteralRmPathFinding {
          return {
            command,
            word,
            ...(read.file === undefined ? {} : { file: read.file, }),
            ...(line === undefined ? {} : { line, }),
          };
        },)
      : [],
    reads: [
      ...heredocReads({
        info,
        read,
      }),
      ...nestedReads({
        read,
        sources: wrapped.payloads,
        unwrittenPaths: false,
      },),
      ...nestedReads({
        read,
        sources: wrapped.argumentCommandLines,
        unwrittenPaths: true,
        origin: command,
      },),
      ...nestedReads({
        read,
        sources: shellScriptPayloads({ words: wrapped.words, },),
        unwrittenPaths: false,
      },),
    ],
  };
}

/**
 Collects the here-document bodies one command record declares.
 
 The shell may read a body as a script whatever program the redirect feeds,
 so each body is read as shell source of its own.
 
 @param info - parsed command record
 
 @param read - source text the record was read from
 
 @returns one read per declared body, none when the record declares no body
 
 @example
 ```ts
 heredocReads({ info, read });
 ```
 */
function heredocReads({
  info,
  read,
}: {
  readonly info: ShellCommandInfo;
  readonly read: SourceRead;
},): readonly SourceRead[] {
  return info.redirects
    .flatMap(function heredocRead(redirect,): SourceRead[] {
      if (redirect.heredocBody === undefined)
        return [];
      return [{
        source: redirect.heredocBody,
        ...(read.file === undefined ? {} : { file: read.file, }),
        ...heredocLinesFrom({
          read,
          ...(redirect.heredocBodyOffset === undefined
            ? {}
            : { bodyOffset: redirect.heredocBodyOffset, }),
          body: redirect.heredocBody,
        },),
        unwrittenPaths: false,
      },];
    },);
}

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
 Reads the words of one raw run with their quote characters dropped.
 
 A source that ends inside a quote is read twice, once honouring the quote and
 once treating it as text, so the same run reaches this reader in two spellings.
 Comparing the words without their quote characters keeps one finding for it.
 
 @param words - words of one raw run
 
 @returns the run's words with quote characters removed
 
 @example
 ```ts
 rawRunIdentity(['rm', '-rf', '"$d"']); // `rm\u0000-rf\u0000$d`
 ```
 */
function rawRunIdentity(words: readonly string[],): string {
  return words
    .map(function unquotedWord(word,): string {
      return QUOTE_CHARACTERS
        .reduce(
        function withoutCharacter(
          current,
          character,
        ): string {
          return current.replaceAll(
            character,
            '',
          );
        },
        word,
        );
    },)
    .join(READ_IDENTITY_SEPARATOR,);
}

/**
 Reads the raw word runs of a source the parser rejected.
 
 The visible text is the only evidence an unparsable source carries,
 so it is read rather than answered with "no".
 
 @param read - source text the parser rejected
 
 @returns findings the raw runs named and the sources they launch
 
 @example
 ```ts
 rawSourceRmRead({ read: { source: 'rm -rf "$d" "unterminated', unwrittenPaths: false, } });
 ```
 */
function rawSourceRmRead({ read, }: { readonly read: SourceRead; },): SourceReadResult {
  /**
   Findings and launched sources collected from every run.
   */
  const collected: {
    findings: LiteralRmPathFinding[];
    reads: SourceRead[]
  } = {
    findings: [],
    reads: [],
  };
  /**
   Runs already read, keyed without their quote characters.
   */
  const readRuns = new Set<string>();
  for (const words of rawCommandWordRuns({ source: read.source, },)) {
    if (readRuns.has(rawRunIdentity(words,),))
      continue;
    readRuns.add(rawRunIdentity(words,),);
    /**
     Innermost program words of this run, with its wrapper layers stripped.
     */
    const wrapped = unwrapWrappers({ words, },);
    /**
     Command line this run writes, used to name what was found.
     */
    const command = words.join(' ',);
    if (isRmProgramWord(wrapped.words
      .at(0,)
      ?? '',)) {
      for (const word of rmOperandFindings({
        unwrittenPaths: read.unwrittenPaths
          || wrapped.wrapperNames
            .includes(XARGS_WRAPPER_NAME,),
        programWord: wrapped.words
          .at(0,)
          ?? RM_PROGRAM_NAME,
        operandSourceTexts: wrapped.words
          .slice(1,),
        carriesExpansion: rawWordCarriesExpansion,
      },)) {
        collected.findings
          .push({
            command: read.origin
              ?? command,
            word,
            ...(read.file === undefined ? {} : { file: read.file, }),
          },);
      }
    }
    collected.reads
      .push(
      ...nestedReads({
        read,
        sources: [
          ...wrapped.payloads,
          ...shellScriptPayloads({ words: wrapped.words, },),
        ],
        unwrittenPaths: false,
      },),
      ...nestedReads({
        read,
        sources: wrapped.argumentCommandLines,
        unwrittenPaths: true,
        origin: read.origin
          ?? command,
      },),
      );
  }
  return collected;
}

/**
 Reads one source text and reports what it names.
 
 @param read - source text to read and what the reader knows about it
 
 @returns this source's findings and the sources it launches
 
 @example
 ```ts
 readOneSource({ read: { source: 'rm -rf "$d"', unwrittenPaths: false } });
 ```
 */
function readOneSource({ read, }: { readonly read: SourceRead; },): SourceReadResult {
  /**
   Parsed shell command analysis.
   */
  const analysis = analyzeShellCommand(read.source,);
  if (!analysis.parsed)
    return rawSourceRmRead({ read, },);

  /**
   Findings this source's records named.
   */
  const findings: LiteralRmPathFinding[] = [];
  /**
   Sources this source's records launch.
   */
  const reads: SourceRead[] = [];
  for (const info of analysis.commands) {
    /**
     What this record named and launched.
     */
    const record = commandRmRead({
      info,
      read,
    },);
    findings.push(...record.findings,);
    reads.push(...record.reads,);
  }
  return {
    findings,
    reads,
  };
}

//endregion Command reads

export {
  isRmProgramWord,
  lineOfOffset,
  rawWordCarriesExpansion,
  readIdentity,
  readOneSource,
  rmOperandFindings,
};
export type { SourceRead, };
