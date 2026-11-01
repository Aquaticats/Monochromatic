/**
 Convert `unbash` nodes to public analyzer command records.
 
 @module
 */

import type {
  AssignmentPrefix as UnbashAssignmentPrefix,
  Command as UnbashCommand,
  CommandArgument as UnbashCommandArgument,
  Redirection as UnbashRedirect,
  Word as UnbashWord,
} from 'unbash';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import type {
  ShellCommandContext,
  ShellCommandInfo,
  ShellEnvAssignment,
  ShellRedirect,
  ShellRedirectKind,
  ShellWordSource,
} from './types.ts';

import {
  assignmentSource,
  assignmentValue,
} from './assignment.ts';

//region Redirect conversion

/**
 Operators include file redirects, here-documents and here-strings.
 */
type UnbashRedirectOperator = UnbashRedirect['operator'];

/**
 Operators that write output to file-like targets.
 */
const WRITE_FILE_OPERATORS: ReadonlySet<UnbashRedirectOperator> = new Set([
  '>',
  '>>',
  '<>',
  '>|',
  '&>',
  '&>>',
],);

/**
 Operators that read input from file-like targets.
 */
const READ_FILE_OPERATORS: ReadonlySet<UnbashRedirectOperator> = new Set([
  '<',
  '<>',
],);

/**
 Operators whose target can be a file descriptor when numeric.
 */
const DESCRIPTOR_OPERATORS: ReadonlySet<UnbashRedirectOperator> = new Set([
  '<&',
  '>&',
],);

/**
 Operators that carry heredoc or here-string bodies.
 */
const HEREDOC_OPERATORS: ReadonlySet<UnbashRedirectOperator> = new Set([
  '<<',
  '<<-',
  '<<<',
],);

/**
 Whether target text names a numeric file descriptor.
 
 @param target - redirect target text
 
 @returns whether target is composed only of decimal digits
 
 @example
 ```ts
 isNumericDescriptor('2');
 ```
 */
function isNumericDescriptor(target: string,): boolean {
  if (target.length === 0)
    return false;
  for (const c of target) {
    if ((c < '0') || (c > '9'))
      return false;
  }
  return true;
}

/**
 Classify redirect target kind.
 
 @param operator - redirect operator
 
 @param target - redirect target text, when present
 
 @returns redirect kind used by callers
 
 @example
 ```ts
 redirectKind({ operator: '>&', target: '1' });
 ```
 */
function redirectKind(
  {
    operator,
    target,
  }: {
    readonly operator: UnbashRedirectOperator;
    readonly target?: string;
  },
): ShellRedirectKind {
  if ((operator === '<<') || (operator === '<<-'))
    return 'heredoc';
  if (operator === '<<<')
    return 'hereString';
  if (DESCRIPTOR_OPERATORS.has(operator,) && (target !== undefined)
    && isNumericDescriptor(target,))
    return 'fileDescriptor';
  return 'file';
}

/**
 Whether redirect writes to a file-like target.
 
 @param operator - redirect operator
 
 @param kind - redirect target classification
 
 @returns whether redirect writes file content
 
 @example
 ```ts
 redirectWritesFile({ operator: '>', kind: 'file' });
 ```
 */
function redirectWritesFile(
  {
    operator,
    kind,
  }: {
    readonly operator: UnbashRedirectOperator;
    readonly kind: ShellRedirectKind;
  },
): boolean {
  if (kind !== 'file')
    return false;
  return WRITE_FILE_OPERATORS.has(operator,)
    || (operator === '>&');
}

/**
 Whether redirect reads from a file-like target.
 
 @param operator - redirect operator
 
 @param kind - redirect target classification
 
 @returns whether redirect reads file content
 
 @example
 ```ts
 redirectReadsFile({ operator: '<', kind: 'file' });
 ```
 */
function redirectReadsFile(
  {
    operator,
    kind,
  }: {
    readonly operator: UnbashRedirectOperator;
    readonly kind: ShellRedirectKind;
  },
): boolean {
  if (kind !== 'file')
    return false;
  return READ_FILE_OPERATORS.has(operator,)
    || (operator === '<&');
}

/**
 Convert one `unbash` redirect to public redirect record.
 
 @param redirect - parsed redirect node
 
 @returns public redirect record
 
 @example
 ```ts
 redirectToInfo(redirect);
 ```
 */
function redirectToInfo(redirect: ForeignBorrowed<UnbashRedirect>,): ShellRedirect {
  /**
   Parsed target text, when `unbash` produced one.
   */
  const targetWord = redirect.type === 'HereDoc' ? redirect.delimiter : redirect.target;
  /**
   Parsed delimiter or target, never a here-document body interpreted as a path.
   */
  const target = targetWord?.value;
  /**
   Original source spelling paired with parsed target value.
   */
  const targetSourceText = targetWord?.text;
  /**
   Descriptor nodes distinguish numeric descriptors from allocated descriptor variables.
   */
  const {descriptor} = redirect;
  /**
   Numeric descriptor, when explicitly supplied.
   */
  const fileDescriptor = descriptor?.type === 'FileDescriptor' ? descriptor.value : undefined;
  /**
   Allocated descriptor variable, when explicitly supplied.
   */
  const fileDescriptorVariable = descriptor?.type === 'FileDescriptorVariable' ? descriptor.name : undefined;
  /**
   Target classification.
   */
  const kind = redirectKind({
    operator: redirect.operator,
    ...(target === undefined ? {} : { target, }),
  },);

  return {
    operator: redirect.operator,
    ...(target === undefined ? {} : { target, }),
    ...(targetSourceText === undefined ? {} : { targetSourceText, }),
    ...(fileDescriptor === undefined ? {} : { fileDescriptor, }),
    ...(fileDescriptorVariable === undefined ? {} : { fileDescriptorVariable, }),
    kind,
    writesFile: redirectWritesFile({
      operator: redirect.operator,
      kind,
    },),
    readsFile: redirectReadsFile({
      operator: redirect.operator,
      kind,
    },),
  };
}

/**
 Extract file-like redirect targets.
 
 @param redirects - public redirect records
 
 @returns target text for file redirects only
 
 @example
 ```ts
 redirectTargets([redirect]);
 ```
 */
function redirectTargets(redirects: readonly ShellRedirect[],): string[] {
  return redirects.flatMap(function targetForRedirect(redirect,): string[] {
    if (redirect.kind !== 'file')
      return [];
    if (redirect.target === undefined)
      return [];
    return [redirect.target,];
  },);
}

/**
 Extract file-like redirect values with original source spellings.
 
 @param redirects - public redirect records
 
 @returns target values and source text for file redirects only
 
 @example
 ```ts
 redirectTargetSources([redirect]);
 ```
 */
function redirectTargetSources(redirects: readonly ShellRedirect[],): ShellWordSource[] {
  return redirects.flatMap(function sourceForRedirect(redirect,): ShellWordSource[] {
    if (redirect.kind !== 'file')
      return [];
    if ((redirect.target === undefined) || (redirect.targetSourceText === undefined))
      return [];
    return [{
      value: redirect.target,
      sourceText: redirect.targetSourceText,
    },];
  },);
}

//endregion Redirect conversion

//region Command conversion

/**
 Pair word value with source spelling, excluding process substitution placeholders.
 
 @param word - suffix word from `unbash`
 
 @returns singleton source record, or empty array for process substitution syntax
 
 @example
 ```ts
 wordToSource(word);
 ```
 */
function wordToSource(word: ForeignBorrowed<UnbashCommandArgument>,): ShellWordSource[] {
  if (word.type === 'Assignment')
    return [assignmentSource(word,),];
  if ((word.parts ?? [])
    .some(function isProcessSubstitution(
      part: ForeignBorrowed<NonNullable<UnbashWord['parts']>[number]>,
    ): boolean {
      return part.type === 'ProcessSubstitution';
    },)) {
    return [];
  }
  return [{
    value: word.value,
    sourceText: word.text,
  },];
}

/**
 Convert one assignment prefix to zero or one environment assignment.
 
 @param assignment - assignment prefix from `unbash`
 
 @returns analyzer assignment entries
 
 @example
 ```ts
 assignmentToEnvAssignment(assignment);
 ```
 */
function assignmentToEnvAssignment(assignment: ForeignBorrowed<UnbashAssignmentPrefix>,): ShellEnvAssignment[] {
  if (assignment.name === undefined)
    return [];
  return [{
    name: assignment.name,
    value: assignmentValue(assignment,),
  },];
}

/**
 Convert one simple command node to analyzer command info.
 
 @param command - simple command node to convert
 
 @param inheritedRedirects - redirects inherited from wrapping statements
 
 @param paramRefs - parameter references pre-scanned from raw source text
 
 @param context - execution context inherited by command
 
 @returns command info used by guardrail signal checks
 
 @example
 ```ts
 commandToInfo({ command, inheritedRedirects: [], paramRefs: [], context });
 ```
 */
function commandToInfo(
  {
    command,
    inheritedRedirects,
    paramRefs,
    context,
  }: {
    readonly command: ForeignBorrowed<UnbashCommand>;
    readonly inheritedRedirects: readonly ForeignBorrowed<UnbashRedirect>[];
    readonly paramRefs: readonly string[];
    readonly context: ShellCommandContext;
  },
): ShellCommandInfo {
  /**
   Converted redirects from command and surrounding statement.
   */
  const redirects = [
    ...command.redirects,
    ...inheritedRedirects,
  ]
    .map(function mapRedirectToInfo(
      redirect: ForeignBorrowed<UnbashRedirect>,
    ): ShellRedirect {
      return redirectToInfo(redirect,);
    },);
  /**
   Suffix argument values paired with exact shell source spelling.
   */
  const argSources = command.args
    .flatMap(wordToSource,);

  return {
    name: command.name
      ?.value
      ?? '',
    envAssignments: command.prefix
      .filter(function isAssignment(prefix,) {
        return prefix.type === 'Assignment';
      },)
      .flatMap(assignmentToEnvAssignment,),
    args: argSources.map(function sourceValue(source,): string {
      return source.value;
    },),
    argSources,
    redirects,
    redirectTargets: redirectTargets(redirects,),
    redirectTargetSources: redirectTargetSources(redirects,),
    paramRefs: [...paramRefs,],
    context,
  };
}

/**
 Emit synthetic command for redirects attached to compound syntax.
 
 @param redirects - redirects attached to compound syntax
 
 @param paramRefs - parameter references pre-scanned from raw source text
 
 @param context - execution context inherited by redirect command
 
 @returns command info carrying redirect targets only
 
 @example
 ```ts
 redirectOnlyCommand({ redirects, paramRefs: [], context });
 ```
 */
function redirectOnlyCommand(
  {
    redirects,
    paramRefs,
    context,
  }: {
    readonly redirects: readonly ForeignBorrowed<UnbashRedirect>[];
    readonly paramRefs: readonly string[];
    readonly context: ShellCommandContext;
  },
): ShellCommandInfo {
  /**
   Converted redirects from compound syntax.
   */
  const convertedRedirects = redirects.map(function mapRedirectToInfo(redirect,): ShellRedirect {
    return redirectToInfo(redirect,);
  },);
  return {
    name: '',
    envAssignments: [],
    args: [],
    argSources: [],
    redirects: convertedRedirects,
    redirectTargets: redirectTargets(convertedRedirects,),
    redirectTargetSources: redirectTargetSources(convertedRedirects,),
    paramRefs: [...paramRefs,],
    context,
  };
}

//endregion Command conversion

export {
  commandToInfo,
  HEREDOC_OPERATORS,
  redirectOnlyCommand,
};
