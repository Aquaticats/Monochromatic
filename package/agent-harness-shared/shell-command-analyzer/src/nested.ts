/**
 Nested shell script extraction for command and process substitutions.
 
 @module
 */

import type {
  ArithmeticCommandExpansion as UnbashArithmeticCommandExpansion,
  CommandExpansionPart as UnbashCommandExpansionPart,
  ProcessSubstitutionPart as UnbashProcessSubstitutionPart,
  ParsedScript as UnbashScript,
} from 'unbash';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import { tagged, } from '@monochromatic-dev/logger-core/ts';
import type {
  ShellCommandContext,
  ShellParseError,
} from './types.ts';
import {
  EMPTY_VISIT_RESULT,
  NO_SCRIPT,
  type ParsedUnbashScript,
  type VisitResult,
} from './internal-types.ts';
import { statementWorkItems, } from './work-items.ts';

//region Logging

/**
 Logger root for shell-command analyzer.
 */
const parentLogger = tagged({ tag: 'shell-command-analyzer', },);

/**
 Tagged logger for nested parsing.
 */
const moduleLogger = tagged({
  tag: 'nested',
  l: parentLogger,
},);

//endregion Logging

/**
 Convert `unbash` parse diagnostics to public diagnostics.
 
 @param script - parsed script to inspect
 
 @returns parser diagnostics in source order
 
 @example
 ```ts
 parseErrorsFromScript(script);
 ```
 */
function parseErrorsFromScript(script: ForeignBorrowed<UnbashScript>,): ShellParseError[] {
  return [
    ...(script.errors ?? []),
  ]
    .map(function parseError(error,): ShellParseError {
      return {
        message: error.message,
        pos: error.pos,
      };
    },);
}

/**
 Resolve nested script from expansion node.
 
 @param expansion - expansion containing nested shell source
 
 @returns parsed script, parse errors, or sentinel when no source exists
 
 @example
 ```ts
 scriptFromExpansion(expansion);
 ```
 */
function scriptFromExpansion(
  expansion:
    | ForeignBorrowed<UnbashArithmeticCommandExpansion>
      | ForeignBorrowed<UnbashCommandExpansionPart>
      | ForeignBorrowed<UnbashProcessSubstitutionPart>,
): {
  readonly script: ParsedUnbashScript;
  readonly parseErrors: readonly ShellParseError[]
} | typeof NO_SCRIPT {
  /**
   The parser owns nested-script parsing and its source positions in unbash 5.
   */
  const innerLogger = tagged({
    tag: scriptFromExpansion.name,
    l: moduleLogger,
  },);
  if (expansion.script === undefined) {
    innerLogger.debug('unbash supplied no parsed nested script; retaining the substitution flag',);
    return NO_SCRIPT;
  }
  innerLogger.debug('Using the original unbash nested script and diagnostics',);
  return {
    script: expansion.script,
    parseErrors: parseErrorsFromScript(expansion.script,),
  };
}

/**
 Build script work from command, process, or arithmetic command expansion.
 
 @param expansion - expansion node containing nested shell source
 
 @param context - execution context inherited by nested commands
 
 @returns statement work, feature flags, and nested parse diagnostics
 
 @example
 ```ts
 visitExpansion({ expansion, context });
 ```
 */
function visitExpansion(
  {
    expansion,
    context,
  }: {
    readonly expansion:
      | ForeignBorrowed<UnbashArithmeticCommandExpansion>
        | ForeignBorrowed<UnbashCommandExpansionPart>
        | ForeignBorrowed<UnbashProcessSubstitutionPart>;
    readonly context: ShellCommandContext;
  },
): VisitResult {
  /**
   Parsed nested script, or sentinel when no source is available.
   */
  const nested = scriptFromExpansion(expansion,);
  if (nested === NO_SCRIPT) {
    return {
      ...EMPTY_VISIT_RESULT,
      flags: {
        ...EMPTY_VISIT_RESULT.flags,
        hasCommandSubstitution: expansion.type !== 'ProcessSubstitution',
        hasProcessSubstitution: expansion.type === 'ProcessSubstitution',
      },
    };
  }
  return {
    ...EMPTY_VISIT_RESULT,
    workItems: statementWorkItems({
      statements: nested.script
        .commands,
      context,
    },),
    flags: {
      ...EMPTY_VISIT_RESULT.flags,
      hasCommandSubstitution: expansion.type !== 'ProcessSubstitution',
      hasProcessSubstitution: expansion.type === 'ProcessSubstitution',
    },
    parseErrors: nested.parseErrors,
  };
}

export {
  parseErrorsFromScript,
  visitExpansion,
};
