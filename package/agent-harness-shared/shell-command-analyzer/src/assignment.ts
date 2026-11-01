/**
 Assignment values and argument spellings from unbash declaration nodes.

 @module
 */

import type {
  Assignment,
  Word,
} from 'unbash';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import type { ShellWordSource, } from './types.ts';

/**
 Render assignment values without expanding shell expressions.

 @param assignment - original parsed assignment

 @returns scalar value or joined array elements, preserving the analyzer's value convention

 @example
 ```ts
 assignmentValue(assignment);
 ```
 */
export function assignmentValue(assignment: ForeignBorrowed<Assignment>,): string {
  /**
   Value discriminant distinguishes scalar words from array containers.
   */
  const { value, } = assignment;
  if (value.type === 'Word')
    return value.value;
  return value.elements
    .map(function wordValue(word: ForeignBorrowed<Word>,): string {
      return word.value;
    },)
    .join(' ',);
}

/**
 Keep declaration arguments distinct from bare assignment values and exact source spelling.

 @param assignment - original suffix assignment received by a declaration builtin

 @returns quote-removed argument paired with unchanged source text

 @example
 ```ts
 assignmentSource(assignment);
 ```
 */
export function assignmentSource(assignment: ForeignBorrowed<Assignment>,): ShellWordSource {
  /**
   Parsed index remains an expression, not an evaluated array position.
   */
  const {index} = assignment;
  /**
   Destination spelling retains optional indexing.
   */
  const destination = `${assignment.name}${index === undefined ? '' : `[${index.value}]`}`;
  /**
   Append assignments must not be reported as replacement assignments.
   */
  const operator = assignment.append === true ? '+=' : '=';
  /**
   Parsed values keep substitutions unevaluated.
   */
  const value = assignmentValue(assignment,);
  /**
   Array delimiters distinguish an array argument from a scalar argument.
   */
  const argumentValue = assignment.value
    .type
    === 'ArrayValue' ? `(${value})` : value;
  return {
    value: `${destination}${operator}${argumentValue}`,
    sourceText: assignment.text,
  };
}
