/**
 Unit tests for the guardrail helper over shell syntax the analyzer cannot read.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  analyzeShellCommand,
  hasNotUnderstoodShellSyntax,
  NOT_UNDERSTOOD_SHELL_SYNTAX_BAN_REASON,
} from '@monochromatic-dev/agent-harness-shared-shell-command-analyzer';

/**
 Nesting past the parser's `MAX_SYNTAX_NESTING` of 256,
 where it keeps a sub-field's raw text without a child list.
 */
const NESTING_PAST_THE_PARSER_BUDGET = 300;

await describe({
  name: '',
  children: [
    describe({
      name: hasNotUnderstoodShellSyntax.name,
      children: [
        it({
          name: 'never reads a brace word past the parser nesting budget as holding no command',
          fn: async function testBraceWordPastTheBudget() {
            /**
             Brace word holding a substitution under more default-value nesting than the parser reads.
             */
            const source = `printf %s ${'${a:-'.repeat(NESTING_PAST_THE_PARSER_BUDGET,)}{b,$(printf nested)}${'}'.repeat(NESTING_PAST_THE_PARSER_BUDGET,)}`;
            /**
             Analyzer's answer for the deep word.
             */
            const analysis = analyzeShellCommand(source,);

            expect(analysis.hasCommandSubstitution || (analysis.notUnderstoodNodeKinds.length > 0),).toBe(true,);
          },
        },),
        it({
          name: 'reports a parameter operation the parser cannot classify',
          fn: async function testUnknownOperation() {
            expect(hasNotUnderstoodShellSyntax(`printf %s \${value;text}`,),).toBe(true,);
          },
        },),
        it({
          name: 'reports an unread operation inside a nested command',
          fn: async function testUnknownOperationInSubstitution() {
            expect(hasNotUnderstoodShellSyntax(`printf %s "$(printf %s \${value;text})"`,),).toBe(true,);
          },
        },),
        ...[
          `printf %s \${value:-text}`,
          `printf %s \${value#pattern}`,
          `printf %s \${value/pattern/replacement}`,
          `printf %s \${value:1:2}`,
          `printf %s \${value^^pattern}`,
          `printf %s \${value@Q}`,
          'printf %s {a,$(printf nested)}',
          'printf %s @(a|$(printf nested))',
          'printf %s $(( 1$(printf nested) ))',
          'printf %s plain',
          'printf %s {a,b}',
          'printf %s {1..3}',
          'printf %s file.{ts,md}',
          'printf %s @(a|b)',
        ].map(function readSyntax(source,) {
          return it({
            name: `reports nothing unread in ${source}`,
            fn: async function testReadSyntax() {
              expect(hasNotUnderstoodShellSyntax(source,),).toBe(false,);
            },
          },);
        },),
        it({
          name: 'leaves incomplete syntax to the callers that refuse unparsable commands',
          fn: async function testIncompleteSyntax() {
            expect(hasNotUnderstoodShellSyntax('printf %s "unterminated',),).toBe(false,);
          },
        },),
      ],
    },),
    describe({
      name: 'NOT_UNDERSTOOD_SHELL_SYNTAX_BAN_REASON',
      children: [
        it({
          name: 'names the field a guard reads and the operations the analyzer reads',
          fn: async function testBanReason() {
            expect(NOT_UNDERSTOOD_SHELL_SYNTAX_BAN_REASON.includes('notUnderstoodNodeKinds',),)
              .toBe(true,);
            expect(NOT_UNDERSTOOD_SHELL_SYNTAX_BAN_REASON.includes('`Unknown`',),).toBe(true,);
            expect(NOT_UNDERSTOOD_SHELL_SYNTAX_BAN_REASON.includes('refuses',),).toBe(true,);
          },
        },),
      ],
    },),
  ],
},);
