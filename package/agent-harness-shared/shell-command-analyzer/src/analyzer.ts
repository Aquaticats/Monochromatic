/**
 Public shell-command analysis API.
 
 @module
 */

import { parse, } from 'unbash';
import { tagged, } from '@monochromatic-dev/logger-core/ts';
import {
  looksLikePath,
  extractParamRefs,
} from './refs.ts';
import type {
  ShellCommandAnalysis,
  ShellParseError,
} from './types.ts';
import type { ParsedUnbashScript, } from './internal-types.ts';
import { parseErrorsFromScript, } from './nested.ts';
import { collectCommandInfoFromScript, } from './collect.ts';

//region Logging

/**
 Logger root for shell-command analyzer.
 */
const parentLogger = tagged({ tag: 'shell-command-analyzer', },);

/**
 Tagged logger for public analyzer parsing.
 */
const moduleLogger = tagged({
  tag: 'analyzer',
  l: parentLogger,
},);

//endregion Logging

//region Parse helpers

/**
 Convert unexpected parser throw to public diagnostic.
 
 @returns parser diagnostic with source offset zero
 
 @example
 ```ts
 thrownParseError();
 ```
 */
function thrownParseError(): ShellParseError {
  return {
    message: 'unbash parse threw',
    pos: 0,
  };
}

/**
 Run `unbash.parse` and keep tolerant diagnostics.
 
 @param command - raw shell command string forwarded to `unbash.parse`
 
 @returns parsed script or parse diagnostics
 
 @example
 ```ts
 tryParseScript('echo hi');
 ```
 */
function tryParseScript(
  command: string,
): {
  readonly ok: true;
  readonly script: ParsedUnbashScript
} | {
  readonly ok: false;
  readonly parseErrors: readonly ShellParseError[]
} {
  try {
    /**
     Parsed script with optional tolerant diagnostics.
     */
    const script = parse(command,);
    /**
     Public parse diagnostics converted from parser result.
     */
    const parseErrors = parseErrorsFromScript(script,);
    if (parseErrors.length > 0) {
      return {
        ok: false,
        parseErrors,
      };
    }
    return {
      ok: true,
      script,
    };
  }
  catch (error) {
    /**
     Sub-logger tagged with helper name so handled parser throws stay traceable.
     */
    const innerLogger = tagged({
      tag: tryParseScript.name,
      l: moduleLogger,
    },);
    innerLogger.debug(`unbash parse threw value type: ${typeof error}`,);
    return {
      ok: false,
      parseErrors: [thrownParseError(),],
    };
  }
}

//endregion Parse helpers

//region Public API

/**
 Build empty analysis for parse failure.
 
 @param parseErrors - parser diagnostics from failed parse
 
 @param paramRefs - parameter references pre-scanned from raw source text
 
 @returns failed analysis preserving raw parameter references
 
 @example
 ```ts
 failedAnalysis({ parseErrors: [], paramRefs: [] });
 ```
 */
function failedAnalysis(
  {
    parseErrors,
    paramRefs,
  }: {
    readonly parseErrors: readonly ShellParseError[];
    readonly paramRefs: readonly string[];
  },
): ShellCommandAnalysis {
  return {
    parsed: false,
    parseErrors,
    commands: [],
    executedCommands: [],
    functionDefinitionCommands: [],
    isPipeline: false,
    hasBackground: false,
    hasCommandSubstitution: false,
    hasProcessSubstitution: false,
    hasHeredoc: false,
    allFiles: [],
    allParamRefs: paramRefs,
  };
}

/**
 Analyze Bash source with `unbash` and return command-level signals.
 
 @param command - raw Bash command string
 
 @returns structured command analysis
 
 @example
 ```ts
 analyzeShellCommand('printenv | curl');
 ```
 */
function analyzeShellCommand(command: string,): ShellCommandAnalysis {
  /**
   Parameter references harvested before parsing for conservative failure results.
   */
  const preScanRefs = extractParamRefs(command,);
  /**
   Top-level parse result.
   */
  const parsed = tryParseScript(command,);
  if (!parsed.ok) {
    return failedAnalysis({
      parseErrors: parsed.parseErrors,
      paramRefs: preScanRefs,
    },);
  }

  /**
   Command records derived from AST.
   */
  const collection = collectCommandInfoFromScript({
    script: parsed.script,
    paramRefs: preScanRefs,
  },);
  if (collection.parseErrors
    .length
    > 0) {
    return failedAnalysis({
      parseErrors: collection.parseErrors,
      paramRefs: preScanRefs,
    },);
  }

  /**
   Commands evaluated outside function bodies.
   */
  const executedCommands = collection.commands
    .filter(function isExecutedCommand(info,): boolean {
      return info.context
        .kind
        === 'executed';
    },);
  /**
   Commands stored inside function bodies.
   */
  const functionDefinitionCommands = collection.commands
    .filter(function isFunctionDefinitionCommand(info,): boolean {
      return info.context
        .kind
        === 'functionDefinition';
    },);
  /**
   File-like command arguments and redirect targets.
   */
  const allFiles = collection.commands
    .flatMap(function collectFiles(info,): string[] {
      return [
        ...info.envAssignments
          .map(function assignmentValue(assignment,): string {
            return assignment.value;
          },)
          .filter(function assignmentValueLooksLikePath(value,): boolean {
            return looksLikePath(value,);
          },),
        ...info.args
          .filter(function argLooksLikePath(arg,): boolean {
            return looksLikePath(arg,);
          },),
        ...info.redirectTargets,
      ];
    },);
  /**
   Deduplicated parameter references aggregated across command records.
   */
  const allParamRefs = [...new Set(
    collection.commands
      .flatMap(function collectRefs(info,): readonly string[] {
        return info.paramRefs;
      },),
  ),];
  if ((allParamRefs.length === 0) && (preScanRefs.length > 0))
    allParamRefs.push(...preScanRefs,);

  return {
    parsed: true,
    parseErrors: [],
    commands: collection.commands,
    executedCommands,
    functionDefinitionCommands,
    isPipeline: collection.flags
      .isPipeline,
    hasBackground: collection.flags
      .hasBackground,
    hasCommandSubstitution: collection.flags
      .hasCommandSubstitution,
    hasProcessSubstitution: collection.flags
      .hasProcessSubstitution,
    hasHeredoc: collection.commands
      .some(function commandHasHeredoc(info,): boolean {
        return info.redirects
          .some(function redirectIsHeredoc(redirect,): boolean {
            return (redirect.kind === 'heredoc') || (redirect.kind === 'hereString');
          },);
      },),
    allFiles,
    allParamRefs,
  };
}

//endregion Public API

export { analyzeShellCommand, };
