/**
 Reads the deletion commands in one shell source text.
 
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
import {
  type DeletionProgram,
  deletionProgramOf,
  deletionTargetsOf,
  type DeletionTarget,
  FIND_DELETE_ACTION,
  findDeletesThePathsItFound,
  NO_DELETION_PROGRAM,
} from './deletion-programs.ts';
import {
  type DeletionFindingDraft,
  deletionFindingDrafts,
} from './deletion-target-findings.ts';
import { lastPathSegment, } from './executable-path-words.ts';
import type { LiteralRmPathFinding, } from './literal-rm-path-finding.ts';
import {
  heredocLinesFrom,
  nestedReadOf,
  nestedReads,
  type SourceRead,
  type SourceReadResult,
} from './deletion-source-read.ts';
import { rawCommandWordRuns, } from './raw-command-word-runs.ts';
import { shellScriptPayloads, } from './shell-script-payloads.ts';
import { shellWordCarriesExpansion, } from './shell-word-carries-expansion.ts';
import type { ShellCommandInfo, } from './types.ts';

//region Constants

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

//region Predicates

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
 Reads the words one deletion command would delete, with their spellings.
 
 @param argSources - the program's own words, without its name
 
 @param optionsWithValue - options whose value is the next word
 
 @returns targets in source order
 
 @example
 ```ts
 deletionTargetsFromSources({ argSources: [{ value: '-rf', sourceText: '-rf' }, { value: '/x', sourceText: '/x' }], optionsWithValue: new Set() });
 ```
 */
function deletionTargetsFromSources({
  argSources,
  optionsWithValue,
}: {
  readonly argSources: readonly DeletionTarget[];
  readonly optionsWithValue: ReadonlySet<string>;
},): readonly DeletionTarget[] {
  return deletionTargetsOf({
    words: argSources,
    optionsWithValue,
  },);
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
   Program word of the innermost command, which decides what deletes here.
   */
  const [programWord = '',] = wrapped.words;
  /**
   Program specification when that word names a program that deletes.
   */
  const program = deletionProgramOf(programWord,);
  /**
   The program's own words, which are the words after the program word.
   
   Unwrapping only drops words in front of the program,
   so the program's words are the last words this record spells.
   */
  const programSources = info.argSources
    .slice(info.argSources
      .length
      - Math.max(
        wrapped.words
          .length
          - 1,
        0,
      ),)
    .map(function sourceSpelling(arg,): DeletionTarget {
      return {
        value: arg.value,
        sourceText: arg.sourceText,
      };
    },);
  /**
   Drafts the deletion checks named for this record.
   */
  const drafts = deletionDraftsOfCommand({
    program,
    programWord,
    programSources,
    words: wrapped.words,
    read,
    wrapsPathsFromOutside: wrapped.wrapperNames
      .includes(XARGS_WRAPPER_NAME,),
    carriesExpansion: shellWordCarriesExpansion,
  },);

  return {
    findings: drafts
      .map(function findingForDraft(draft,): LiteralRmPathFinding {
        return {
          command,
          word: draft.word,
          check: draft.check,
          ...(draft.detail === undefined ? {} : { detail: draft.detail, }),
          ...(read.file === undefined ? {} : { file: read.file, }),
          ...(line === undefined ? {} : { line, }),
        };
      },),
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
        homeFolder: read.homeFolder,
      },];
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
 Reads the deletion drafts one command line produces.
 
 A program that deletes has its own targets judged,
 and `find` is judged by its `-delete` action instead,
 because the paths that action deletes are never written in the command.
 
 @param program - program specification when the command word deletes, else the sentinel
 
 @param programWord - word naming that program
 
 @param programSources - the program's own words, with their source spellings
 
 @param words - innermost program words, program first
 
 @param read - source text the command was read from
 
 @param wrapsPathsFromOutside - whether a wrapper hands this program the paths it read
 
 @param carriesExpansion - decides whether one written spelling is computed
 
 @returns drafts the deletion checks named, empty when this command deletes nothing
 
 @example
 ```ts
 deletionDraftsOfCommand({ program, programWord: 'rm', programSources: [], words: ['rm'], read, wrapsPathsFromOutside: false, carriesExpansion: shellWordCarriesExpansion });
 ```
 */
function deletionDraftsOfCommand({
  program,
  programWord,
  programSources,
  words,
  read,
  wrapsPathsFromOutside,
  carriesExpansion,
}: {
  readonly program: DeletionProgram | typeof NO_DELETION_PROGRAM;
  readonly programWord: string;
  readonly programSources: readonly DeletionTarget[];
  readonly words: readonly string[];
  readonly read: SourceRead;
  readonly wrapsPathsFromOutside: boolean;
  readonly carriesExpansion: (sourceText: string,) => boolean;
},): readonly DeletionFindingDraft[] {
  if (program !== NO_DELETION_PROGRAM) {
    return deletionFindingDrafts({
      programWord,
      targets: deletionTargetsFromSources({
        argSources: programSources,
        optionsWithValue: program.optionsWithValue,
      },),
      unwrittenPaths: read.unwrittenPaths || wrapsPathsFromOutside,
      homeFolder: read.homeFolder,
      carriesExpansion,
    },);
  }
  if (!findDeletesThePathsItFound(words,))
    return [];
  return [{
    word: FIND_DELETE_ACTION,
    check: 'find-deletes-the-paths-it-found',
  },];
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
    for (const draft of deletionDraftsOfCommand({
      program: deletionProgramOf(wrapped.words
        .at(0,)
        ?? '',),
      programWord: wrapped.words
        .at(0,)
        ?? '',
      programSources: wrapped.words
        .slice(1,)
        .map(function rawTarget(word,): DeletionTarget {
          return {
            value: word,
            sourceText: word,
          };
        },),
      words: wrapped.words,
      read,
      wrapsPathsFromOutside: wrapped.wrapperNames
        .includes(XARGS_WRAPPER_NAME,),
      carriesExpansion: rawWordCarriesExpansion,
    },)) {
      collected.findings
        .push({
          command: read.origin
            ?? command,
          word: draft.word,
          check: draft.check,
          ...(draft.detail === undefined ? {} : { detail: draft.detail, }),
          ...(read.file === undefined ? {} : { file: read.file, }),
        },);
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
  deletionDraftsOfCommand,
  deletionTargetsFromSources,
  rawWordCarriesExpansion,
  readOneSource,
};
