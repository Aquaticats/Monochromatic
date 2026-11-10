/**
 Unit tests for the parse diagnostics the public analyzer reports.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { analyzeShellCommand, } from '@monochromatic-dev/agent-harness-shared-shell-command-analyzer';

//region Parser-throw fixtures

/**
 Text the rejected source's parser read raises, so the diagnostic can be traced back to it.
 */
const PARSER_FAILURE_TEXT = 'source reader rejected the command text';

/**
 Reads the first parse diagnostic message from an analyzed source.
 
 @param source - value passed to the analyzer as shell source
 
 @returns first diagnostic message, or empty text when the source produced none
 
 @example
 ```ts
 firstParseErrorMessage(sourceTheParserRejects());
 ```
 */
function firstParseErrorMessage(source: string,): string {
  /**
   First diagnostic the analyzer reported, defaulted so a missing diagnostic reads as empty text.
   */
  const [diagnostic = {
    message: '',
    pos: 0,
  },] = analyzeShellCommand(source,).parseErrors;
  return diagnostic.message;
}

/**
 Builds a source the pre-scan reads as text and `unbash.parse` rejects.
 
 `unbash.parse` throws only for a non-string source, and the analyzer reads the raw text before
 parsing, so the stub answers character reads for the pre-scan and raises on the character-code
 reads only the parser performs.
 
 @returns parser input that throws while the parser reads it
 
 @example
 ```ts
 analyzeShellCommand(sourceTheParserRejects());
 ```
 */
function sourceTheParserRejects(): string {
  return {
    length: 2,
    charAt: function readCharacter(): string {
      return 'b';
    },
    charCodeAt: function readCharacterCode(): number {
      throw new RangeError(PARSER_FAILURE_TEXT,);
    },
  } as unknown as string;
}

//endregion Parser-throw fixtures

await describe({
  name: 'shell command analyzer parse diagnostics',
  children: [
    describe({
      name: analyzeShellCommand.name,
      children: [
        it({
          name: 'keeps the thrown parser value text in the parse diagnostic',
          fn: async function testThrownValueText() {
            /**
             Analysis of source the parser rejects while reading it.
             */
            const analysis = analyzeShellCommand(sourceTheParserRejects(),);

            expect(analysis.parsed,).toBe(false,);
            expect(
              firstParseErrorMessage(sourceTheParserRejects(),),
            )
              .toContain(PARSER_FAILURE_TEXT,);
            expect(analysis.parseErrors[0]?.pos,).toBe(0,);
          },
        },),
        it({
          name: 'distinguishes different thrown parser values',
          fn: async function testThrownValuesDiffer() {
            /**
             Diagnostic for a source whose character codes cannot be read at all.
             */
            const nonStringSourceMessage = firstParseErrorMessage(42 as unknown as string,);
            /**
             Diagnostic for a source that raises its own text while the parser reads it.
             */
            const rejectedSourceMessage = firstParseErrorMessage(sourceTheParserRejects(),);

            expect(nonStringSourceMessage,).not.toBe(rejectedSourceMessage,);
            expect(nonStringSourceMessage,).toContain('charCodeAt',);
            expect(rejectedSourceMessage,).toContain(PARSER_FAILURE_TEXT,);
          },
        },),
      ],
    },),
  ],
},);
