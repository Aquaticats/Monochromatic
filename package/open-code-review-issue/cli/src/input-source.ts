/**
 Positional OCR input loading boundary.
 
 @module
 */

import { readStructuredInputFile, } from './file-input.ts';
import type { CliInputArgument, } from './cli-input.ts';
import { parseStructuredInput, } from './ingest.ts';
import type { NormalizedInput, } from './model.ts';

/**
 Reports positional value that is neither inline JSON nor an existing file.
 */
export class CliInputNotFoundError extends Error {
  /**
   Creates unavailable-input failure without reading standard input.
   
   @param input - Positional value interpreted as named path.
   
   @param cause - Filesystem evidence proving path absence.
   
   @example
   ```ts
   new CliInputNotFoundError({ input: 'review.json', cause: new Error('missing') });
   ```
   */
  public constructor({
    input,
    cause,
  }: {
    readonly input: string;
    readonly cause: unknown;
  },) {
    super(
      `positional input ${JSON.stringify(input,)} is neither inline JSON nor an existing named file`,
      { cause, },
    );
    this.name = 'CliInputNotFoundError';
  }
}

/**
 Node filesystem error shape carrying operation code.
 */
type NodeFileError = Error & {
  readonly code?: unknown;
};

/**
 Detects absent named-file evidence.
 
 @param value - Caught filesystem value.
 
 @returns Whether Node reported `ENOENT`.
 */
function isAbsentFileError(value: unknown,): value is NodeFileError {
  return (Error.isError(value,))
    && ((value as NodeFileError).code === 'ENOENT');
}

/**
 Loads shell-quoted inline JSON or required named file.
 
 @param input - Validated positional input union.
 
 @returns Atomically normalized OCR input.
 
 @throws {@link CliInputNotFoundError} when named path does not exist.
 
 @example
 ```ts
 await loadCliInput({ input: { kind: 'file', path: 'review.json' } });
 ```
 */
export async function loadCliInput({
  input,
}: {
  readonly input: CliInputArgument;
},): Promise<NormalizedInput> {
  if (input.kind === 'inline-json') {
    return parseStructuredInput({ text: input.text, },);
  }
  try {
    return await readStructuredInputFile({ path: input.path, });
  }
  catch (error: unknown) {
    if (isAbsentFileError(error,)) {
      throw new CliInputNotFoundError({
        input: input.path,
        cause: error,
      },);
    }
    throw error;
  }
}
