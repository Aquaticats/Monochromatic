#!/usr/bin/env node

/**
 Append utility for adding lines to files.
 
 This utility appends text lines to a specified file. It validates that the file
 exists and has write permissions before appending.
 
 Usage:
   task-append "my new line" --to myfile.md
   task-append "my new line1" --to myfile.md
   task-append "my new line1\nMy new line2" --to myfile.md
   task-append "my new line1" "my new line2" --to myfile.md
 
 @example
 ```bash
 # Append a single line
 task-append "my new line" --to myfile.md
 
 # Append multiple lines as separate arguments
 task-append "my new line1" "my new line2" --to myfile.md
 
 # Append multiline text
 task-append "my new line1\\nMy new line2" --to myfile.md
 ```
 */

// TODO: deprecate Optique
import { object, } from '@optique/core/constructs';
// TODO: deprecate Optique
import { multiple, } from '@optique/core/modifiers';
// TODO: deprecate Optique
import {
  argument,
  option,
} from '@optique/core/primitives';
// TODO: deprecate Optique
import { string, } from '@optique/core/valueparser';
// TODO: deprecate Optique
import { runSync, } from '@optique/run';
import { constants, } from 'node:fs';
import {
  access,
  appendFile,
} from 'node:fs/promises';



/**
 Error messages for append operations
 */
const ERROR_MESSAGES = {
  fileNotFound: function fileNotFound(path: string,): string {
    return `File not found: ${path}`;
  },
  noWritePermission: function noWritePermission(path: string,): string {
    return `No write permission for file: ${path}`;
  },
  noTextProvided: 'No text provided to append',
} as const;

/**
 Node filesystem error code for absent paths.
 */
const FILE_NOT_FOUND_ERROR_CODE = 'ENOENT';

/**
 Validates that a file exists and has write permissions.
 
 @param filePath - Absolute or relative path to validate
 
 @throws When file does not exist or lacks write permissions
 
 @example
 ```ts
 await validateFile('./output.md');
 ```
 */
async function validateFile(filePath: string,): Promise<void> {
  try {
    await access(
      filePath,
      constants.W_OK,
    );
  }
  catch (error) {
    if (!(Error.isError(error,)))
      throw new Error(
        ERROR_MESSAGES.noWritePermission(filePath,),
        { cause: error, },
      );

    if (!('code' in error))
      throw new Error(
        ERROR_MESSAGES.noWritePermission(filePath,),
        { cause: error, },
      );

    /**
     Node filesystem error code attached to the failed permission check.
     */
    const { code, } = error as { readonly code: unknown; };
    if (code === FILE_NOT_FOUND_ERROR_CODE)
      throw new Error(
        ERROR_MESSAGES.fileNotFound(filePath,),
        { cause: error, },
      );

    throw new Error(
      ERROR_MESSAGES.noWritePermission(filePath,),
      { cause: error, },
    );
  }
}

/**
 Options for {@link appendLinesToFile}.
 
 @example
 ```ts
 const options: AppendLinesToFileOptions = {
   filePath: './output.md',
   lines: ['line 1', 'line 2'],
 };
 ```
 */
type AppendLinesToFileOptions = {
  /**
   Absolute or relative path to append to
   */
  readonly filePath: string;
  /**
   Lines of text to append
   */
  readonly lines: readonly string[];
};

/**
 Appends lines to a file, joining them with newlines and adding a trailing newline.
 
 @param filePath - Absolute or relative path to append to
 
 @param lines - Lines of text to append
 
 @example
 ```ts
 await appendLinesToFile({ filePath: './output.md', lines: ['line 1', 'line 2'] });
 ```
 */
async function appendLinesToFile({
  filePath,
  lines,
}: AppendLinesToFileOptions,): Promise<void> {
  /**
   Joined payload with a trailing newline so subsequent appends start on a fresh line.
   */
  const content = `${lines.join('\n',)}\n`;
  await appendFile(
    filePath,
    content,
  );
}

//region Parser definition: required --to option and variadic positional text lines

/**
 TODO: deprecate Optique
 Optique parser for the task-append CLI
 */
const parser = object({
  to: option(
    '-t',
    '--to',
    string(),
  ),
  lines: multiple(
    argument(string(),),
  ),
},);

//endregion Parser definition

/**
 TODO: deprecate Optique
 Parsed CLI arguments from process.argv
 */
const args = runSync(
  parser,
  {
    programName: 'task-append',
    help: 'option',
  },
);

if (args.lines
  .length
  === 0)
  throw new Error(ERROR_MESSAGES.noTextProvided,);

// Validate the target file
await validateFile(args.to,);

// Append all lines to the file
await appendLinesToFile({
  filePath: args.to,
  lines: args.lines,
},);
