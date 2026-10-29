/**
 Parser-owned shell structure for permission review. @module
 */
import { parse, } from 'unbash';
import { tagged, } from '@monochromatic-dev/logger-core/ts';

//region Syntax projection retains operations while keeping source archives local.
/**
 Incomplete syntax cannot silently become an empty reviewed operation.
 */
export class ShellReviewSyntaxError extends Error {
  /**
   Identify incomplete syntax at the model-input boundary.
   */
  override readonly name = 'ShellReviewSyntaxError';
}

/**
 Structural parser fields, excluding source archives and locations.
 */
const structuralFields: ReadonlySet<string> = new Set([
  'type',
  'commands',
  'operators',
  'command',
  'background',
  'name',
  'prefix',
  'suffix',
  'redirects',
  'operator',
  'descriptor',
  'target',
  'delimiter',
  'body',
  'quoted',
  'clause',
  'then',
  'else',
  'wordlist',
  'kind',
  'items',
  'word',
  'pattern',
  'terminator',
  'initialize',
  'test',
  'update',
  'expression',
  'left',
  'right',
  'operand',
  'consequent',
  'alternate',
  'value',
  'parts',
  'parameter',
  'index',
  'operation',
  'offset',
  'length',
  'replacement',
  'append',
  'elements',
  'script',
]);

/**
 Narrow parser records before selecting structural fields.
 
 @param value - parser-owned node or scalar
 
 @returns whether fields can be inspected
 */
function isRecord(value: unknown): value is Record<string, unknown> {
  return (value !== null) && ((typeof value) === 'object')
    && (!Array.isArray(value));
}

/**
 Project materialized parser nodes without duplicated raw scripts.
 
 @param value - parser-owned node or scalar
 
 @returns syntax data without source archives
 
 @throws ShellReviewSyntaxError for incomplete syntax
 */
function projectNode(value: unknown): unknown {
  if (!isRecord(value)) return value;
  /**
   Parser node kind determines which syntax fields require special handling.
   */
  const { type, } = value;
  if (type === 'Unknown')
    throw new ShellReviewSyntaxError('Bash review encountered an unsupported parameter operation');
  if ((type === 'Script') && Array.isArray(value.errors)
    && (value.errors
      .length
      > 0))
    throw new ShellReviewSyntaxError('Bash parser reported incomplete syntax; automatic review requires a complete parsed operation');
  if ((type === 'Word') || (type === 'ArithmeticWord')) {
    if (Array.isArray(value.parts) && (value.parts
      .length
      > 0)) return {
        type,
        parts: value.parts,
      };
    return {
      type,
      value: value.value,
    };
  }
  if (type === 'SimpleExpansion') {
    if (((typeof value.text) !== 'string') || (!value.text
      .startsWith('$')))
      throw new ShellReviewSyntaxError('Bash parameter node has no parser-owned parameter spelling');
    return {
      type,
      parameter: value.text
        .slice(1),
    };
  }
  // Token spelling preserves escaped glob characters without repeating enclosing scripts.
  if (type === 'Literal') return {
    type,
    value: value.value,
    spelling: value.text,
  };
  if (type === 'HereDocBody') return Array.isArray(value.parts) && (value.parts
    .length
    > 0)
    ? {
      type,
      parts: value.parts,
    }
    : {
      type,
      literal: value.text,
    };
  if (type === 'BraceExpansion') return {
    type,
    pattern: value.text,
    parts: value.parts,
  };
  if (type === 'Time') return {
    type,
    posix: value.posix !== undefined,
    command: value.command,
  };
  if (((type === 'CommandExpansion') || (type === 'ProcessSubstitution')
    || (type === 'ArithmeticCommandExpansion')) && (value.script === undefined))
    throw new ShellReviewSyntaxError('Bash nested execution has no complete parsed script');
  if (((type === 'ArithmeticExpansion') || (type === 'ArithmeticCommand')) && (value.expression === undefined))
    throw new ShellReviewSyntaxError('Bash arithmetic has no complete parsed expression');
  return Object.fromEntries(Object.entries(value)
    .filter(function structuralField([field,]) {
      return structuralFields.has(field) && (!((type === 'ArithmeticCommand') && (field === 'body')));
    }));
}

/**
 Parse Bash locally, retaining control flow, quoting, redirects, and expansions for review.
 Literal arguments stay data; parsing does not establish runtime values or command effects.
 
 @param command - original Bash source retained locally for execution
 
 @returns structural syntax without complete source scripts or source positions
 
 @throws ShellReviewSyntaxError when complete structure cannot be projected
 
 @example
 ```ts
 const structure = parseShellReviewStructure('ls project | wc -l');
 ```
 */
export function parseShellReviewStructure(command: string): Readonly<Record<string, unknown>> {
  /**
   Function-scoped logs omit the original command text.
   */
  const l = tagged({ tag: parseShellReviewStructure.name, });
  l.debug('parsing shell structure for permission review');
  try {
    /**
     JSON serialization materializes the parser's lazy word and nested-script fields.
     */
    const syntax: unknown = JSON.parse(JSON.stringify(
      parse(command),
      function projectMaterializedNode(
        _key: string,
        value: unknown
      ) { return projectNode(value); }
    ));
    if (!isRecord(syntax)) throw new ShellReviewSyntaxError('Bash parser did not produce a structural script');
    l.debug('prepared shell structure without source scripts');
    return Object.freeze({
      language: 'bash',
      syntax,
      expansionValues: 'not-evaluated',
      effects: 'not-established',
    });
  } catch (error) {
    if (error instanceof ShellReviewSyntaxError) {
      l.debug(String(error));
      throw error;
    }
    l.error(String(error));
    throw new ShellReviewSyntaxError(
      'Bash structure could not be parsed for automatic review',
      { cause: error, }
    );
  }
}
//endregion
