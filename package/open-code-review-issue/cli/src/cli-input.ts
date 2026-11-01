/**
 Positional CLI input grammar.
 
 @module
 */

import {
  CliInvocationError,
  MissingCliInputError,
} from './cli-invocation-error.ts';

/**
 Required positional input interpreted without consulting standard input.
 */
export type CliInputArgument =
  | {
    readonly kind: 'file';
    readonly path: string;
  }
    | {
      readonly kind: 'inline-json';
      readonly text: string;
    };

/**
 Validates exactly one positional input argument.
 
 @param positionals - Raw positional arguments.
 
 @param interactive - Whether inline structured JSON is accepted.
 
 @returns Named-file or shell-quoted inline JSON input.
 
 @throws {@link MissingCliInputError} when positional input is omitted.
 
 @throws {@link CliInvocationError} for multiple or unsupported positionals.
 
 @example
 ```ts
 parsePositionalInput({ positionals: ['review.json'], interactive: true });
 ```
 */
export function parsePositionalInput({
  positionals,
  interactive,
}: {
  readonly positionals: readonly string[];
  readonly interactive: boolean;
},): CliInputArgument {
  if (positionals.length === 0) {
    throw new MissingCliInputError();
  }
  if (positionals.length !== 1) {
    throw new CliInvocationError('every mode requires exactly one positional input',);
  }
  /**
   Sole required positional value.
   */
  const value = positionals.at(0,);
  if (value === undefined) {
    throw new MissingCliInputError();
  }
  if (value === '-') {
    throw new CliInvocationError('`-` is not an input source; pass a named file path',);
  }
  /**
   Leading-whitespace-insensitive syntax discriminator preserving original JSON text.
   */
  const candidate = value.trimStart();
  /**
   Whether positional syntax unambiguously opens a JSON object or array.
   */
  const inlineJson = candidate.startsWith('{',) || candidate.startsWith('[',);
  if (inlineJson && (!interactive)) {
    throw new CliInvocationError('inline JSON positional input requires `--interactive`',);
  }
  return inlineJson
    ? {
      kind: 'inline-json',
      text: value,
    }
    : {
      kind: 'file',
      path: value,
    };
}
