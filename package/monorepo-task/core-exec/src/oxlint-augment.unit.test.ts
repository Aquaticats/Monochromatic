import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  augmentOxlintOutput,
  extractRuleName,
  formatGuidanceLine,
  isHelpLine,
  NO_RULE,
  RULE_GUIDANCE,
  stripAnsi,
} from '../dist/final/node/testing.mjs';

/** Iteration count for long-run equivalence cases; large enough to exercise the linear scan, fast to compare. */
const LONG_RUN = 100_000;
/** Sequence count for O(n^2)-sensitive accumulator cases; modest so the pre-refactor array-spread stays fast. */
const MANY_SEQUENCES = 5_000;

/** Returns configured guidance text for a rule in assertions. */
function getRuleGuidance(ruleName: string,): string {
  /**
   Guidance entry configured for the requested rule.
   */
  const ruleGuidance = RULE_GUIDANCE[ruleName];
  if (ruleGuidance === undefined)
    return '';
  return ruleGuidance.guidance;
}

//region augmentOxlintOutput helpers

/** Convenience helper to build a diagnostic block in oxlint's miette format. */
function buildDiagnostic({ rule, plugin, message, file, helpText, }: {
  rule: string;
  plugin: string;
  message: string;
  file: string;
  helpText?: string;
},): string[] {
  const lines = [
    `  x ${plugin}(${rule}): ${message}`,
    `    ,-[${file}:93:30]`,
    ' 92 |   const form = document.querySelector();',
    " 93 |   form?.addEventListener('submit', async function handleSubmit(event) {",
    '    |                                    ^^^^',
    '    `----',
  ];
  if (helpText !== undefined)
    lines.push(`  help: ${helpText}`,);
  return lines;
}

//endregion augmentOxlintOutput helpers

//region stripAnsi

await describe({
  name: '',
  children: [
    describe({
      name: stripAnsi.name,
      children: [
        it({
          name: 'removes single color code',
          fn: async () => {
            expect(stripAnsi('\u001B[31merror\u001B[0m',),).toBe('error',);
          },
        },),
        it({
          name: 'removes multiple color codes',
          fn: async () => {
            expect(stripAnsi('\u001B[1;31mx\u001B[0m \u001B[33mwarning\u001B[0m',),)
              .toBe('x warning',);
          },
        },),
        it({
          name: 'returns plain text unchanged',
          fn: async () => {
            expect(stripAnsi('no colors here',),).toBe('no colors here',);
          },
        },),
        it({
          name: 'handles empty string',
          fn: async () => {
            expect(stripAnsi('',),).toBe('',);
          },
        },),
      ],
    },),

    //endregion stripAnsi

    //region stripAnsi edge cases

    describe({
      name: 'stripAnsi edge cases',
      children: [
        it({
          name: 'returns all-whitespace input unchanged',
          fn: async () => {
            expect(stripAnsi('   ',),).toBe('   ',);
          },
        },),
        it({
          name: 'preserves a lone ESC not followed by [',
          fn: async () => {
            expect(stripAnsi('Z',),).toBe('Z',);
          },
        },),
        it({
          name: 'preserves an ESC[ sequence with no digits',
          fn: async () => {
            expect(stripAnsi('[m',),).toBe('[m',);
          },
        },),
        it({
          name: 'preserves an ESC[ sequence with an invalid terminator',
          fn: async () => {
            expect(stripAnsi('[31x',),).toBe('[31x',);
          },
        },),
        it({
          name: 'strips a multi-parameter sequence',
          fn: async () => {
            expect(stripAnsi('[1;31;42mtext[0m',),).toBe('text',);
          },
        },),
        it({
          name: 'leaves a long plain run unchanged',
          fn: async () => {
            const plain = 'a'.repeat(LONG_RUN,);
            expect(stripAnsi(plain,),).toBe(plain,);
          },
        },),
        it({
          name: 'strips a long digit-run sequence',
          fn: async () => {
            expect(stripAnsi(`[${'9'.repeat(LONG_RUN,)}mZ`,),).toBe('Z',);
          },
        },),
        it({
          name: 'preserves a long digit run with an invalid terminator',
          fn: async () => {
            const seq = `[${'9'.repeat(LONG_RUN,)}z`;
            expect(stripAnsi(seq,),).toBe(seq,);
          },
        },),
        it({
          name: 'strips many consecutive sequences',
          fn: async () => {
            expect(
              stripAnsi('[0m'.repeat(MANY_SEQUENCES,),),
            ).toBe('',);
          },
        },),
        it({
          name: 'strips many interleaved sequences preserving plain text',
          fn: async () => {
            expect(
              stripAnsi('a[31m'.repeat(MANY_SEQUENCES,),),
            )
              .toBe('a'.repeat(MANY_SEQUENCES,),);
          },
        },),
      ],
    },),

    //endregion stripAnsi edge cases

    //region extractRuleName

    describe({
      name: extractRuleName.name,
      children: [
        it({
          name: 'extracts from error diagnostic header',
          fn: async () => {
            expect(extractRuleName(
              '  x typescript-eslint(no-misused-promises): Promise-returning function provided to method that expects a function returning void.',
            ),)
              .toBe('no-misused-promises',);
          },
        },),
        it({
          name: 'extracts from warning diagnostic header',
          fn: async () => {
            expect(extractRuleName(
              '  ! eslint(no-magic-numbers): No magic number: 42.',
            ),)
              .toBe('no-magic-numbers',);
          },
        },),
        it({
          name: 'extracts from unicorn plugin format',
          fn: async () => {
            expect(extractRuleName(
              "  x eslint-plugin-unicorn(no-process-exit): Don't use `process.exit()`",
            ),)
              .toBe('no-process-exit',);
          },
        },),
        it({
          name: 'extracts from custom plugin format',
          fn: async () => {
            expect(extractRuleName(
              '  x no-restricted-syntax(no-arrow-function): Arrow functions are banned.',
            ),)
              .toBe('no-arrow-function',);
          },
        },),
        it({
          name: 'extracts with ANSI codes present',
          fn: async () => {
            expect(extractRuleName(
              '  \u001B[31mx\u001B[0m \u001B[33mtypescript-eslint(no-misused-promises)\u001B[0m: Promise-returning function.',
            ),)
              .toBe('no-misused-promises',);
          },
        },),
        it({
          name: 'returns null for context lines',
          fn: async () => {
            expect(extractRuleName('  92 |   const form = document.querySelector();',),)
              .toBe(NO_RULE,);
          },
        },),
        it({
          name: 'returns null for blank lines',
          fn: async () => {
            expect(extractRuleName('',),).toBe(NO_RULE,);
          },
        },),
        it({
          name: 'returns null for box-drawing lines',
          fn: async () => {
            expect(extractRuleName('   ,-[src/client.ts:93:30]',),).toBe(NO_RULE,);
          },
        },),
        it({
          name: 'returns null for help lines',
          fn: async () => {
            expect(extractRuleName('  help: Expected void return type.',),).toBe(NO_RULE,);
          },
        },),
      ],
    },),

    //endregion extractRuleName

    //region extractRuleName edge cases

    describe({
      name: 'extractRuleName edge cases',
      children: [
        it({
          name: 'returns null for all-whitespace input',
          fn: async () => {
            expect(extractRuleName('     ',),).toBe(NO_RULE,);
          },
        },),
        it({
          name: 'returns null when an x is not a header marker',
          fn: async () => {
            expect(extractRuleName('context line 42',),).toBe(NO_RULE,);
          },
        },),
        it({
          name: 'returns null when whitespace breaks the plugin name',
          fn: async () => {
            expect(extractRuleName('  x type script(no-misused-promises): msg',),)
              .toBe(NO_RULE,);
          },
        },),
        it({
          name: 'skips non-header markers before a valid header',
          fn: async () => {
            expect(extractRuleName(
              'xx x typescript-eslint(no-misused-promises): msg',
            ),)
              .toBe('no-misused-promises',);
          },
        },),
        it({
          name: 'returns null for a long run of bare markers',
          fn: async () => {
            expect(
              extractRuleName('x'.repeat(MANY_SEQUENCES,),),
            ).toBe(NO_RULE,);
          },
        },),
        it({
          name: 'returns null for a long non-marker run',
          fn: async () => {
            expect(
              extractRuleName('a'.repeat(LONG_RUN,),),
            ).toBe(NO_RULE,);
          },
        },),
        it({
          name: 'matches across a long whitespace gap',
          fn: async () => {
            expect(extractRuleName(`x${' '.repeat(LONG_RUN,)}p(rule): msg`,),)
              .toBe('rule',);
          },
        },),
        it({
          name: 'matches across a long plugin name',
          fn: async () => {
            expect(extractRuleName(`x ${'a'.repeat(LONG_RUN,)}(rule): msg`,),)
              .toBe('rule',);
          },
        },),
        it({
          name: 'matches a long rule name',
          fn: async () => {
            const rule = 'a'.repeat(LONG_RUN,);
            expect(extractRuleName(`x p(${rule}): msg`,),).toBe(rule,);
          },
        },),
      ],
    },),

    //endregion extractRuleName edge cases

    //region isHelpLine

    describe({
      name: isHelpLine.name,
      children: [
        it({
          name: 'detects help line',
          fn: async () => {
            expect(isHelpLine('  help: Expected void return type.',),).toBe(true,);
          },
        },),
        it({
          name: 'detects help line with ANSI codes',
          fn: async () => {
            expect(isHelpLine('  \u001B[36mhelp:\u001B[0m Expected void return type.',),)
              .toBe(true,);
          },
        },),
        it({
          name: 'rejects diagnostic header',
          fn: async () => {
            expect(isHelpLine('  x typescript-eslint(no-misused-promises): ...',),).toBe(
              false,
            );
          },
        },),
        it({
          name: 'rejects context line',
          fn: async () => {
            expect(isHelpLine('  92 |   const form = ...',),).toBe(false,);
          },
        },),
      ],
    },),

    //endregion isHelpLine

    //region formatGuidanceLine

    describe({
      name: formatGuidanceLine.name,
      children: [
        it({
          name: 'formats guidance with note prefix',
          fn: async () => {
            expect(formatGuidanceLine('Fix the issue.',),)
              .toBe('  note: Fix the issue.',);
          },
        },),
      ],
    },),

    //endregion formatGuidanceLine

    //region augmentOxlintOutput

    describe({
      name: augmentOxlintOutput.name,
      children: [
        it({
          name: 'injects guidance after help line for matched rule',
          fn: async () => {
            const input = [
              ...buildDiagnostic({
                rule: 'no-misused-promises',
                plugin: 'typescript-eslint',
                message:
                  'Promise-returning function provided to method that expects a function returning void.',
                file: 'src/client.ts',
                helpText: 'Expected void return type.',
              },),
              '',
            ]
              .join('\n',);

            const result = augmentOxlintOutput(input,);

            expect(result,).toContain(`  help: Expected void return type. ${getRuleGuidance('no-misused-promises',)}`,);
            expect(result,).not.toContain('note:',);
          },
        },),
        it({
          name: 'appends repository-safe guidance to non-null assertion help',
          fn: async () => {
            const helpText = 'Consider using the optional chain operator `?.` instead. `x!.y` is equivalent to `x.y` at runtime and will throw if `x` is `null` or `undefined`, but `x?.y` will return `undefined`.';
            const input = [
              ...buildDiagnostic({
                rule: 'no-non-null-assertion',
                plugin: 'typescript',
                message: 'Forbidden non-null assertion.',
                file: 'src/corpus-run/artifact-v2-read.unit.test.ts',
                helpText,
              },),
              '',
            ]
              .join('\n',);
            const guidance = getRuleGuidance('no-non-null-assertion',);

            const result = augmentOxlintOutput(input,);

            expect(guidance,).toContain('nonNullishOrThrow',);
            expect(guidance,).toContain('Do not use optional chaining',);
            expect(result,).toContain(`  help: ${helpText} ${guidance}`,);
          },
        },),
        it({
          name: 'injects repository-safe guidance for standalone non-null assertions',
          fn: async () => {
            const input = [
              ...buildDiagnostic({
                rule: 'no-non-null-assertion',
                plugin: 'typescript',
                message: 'Forbidden non-null assertion.',
                file: 'src/index.ts',
              },),
              '',
            ]
              .join('\n',);
            const guidance = getRuleGuidance('no-non-null-assertion',);

            const result = augmentOxlintOutput(input,);

            expect(result,).toContain(`  help: ${guidance}\n`,);
          },
        },),
        it({
          name: 'injects guidance for no-array-callback-reference after help line',
          fn: async () => {
            const input = [
              ...buildDiagnostic({
                rule: 'no-array-callback-reference',
                plugin: 'unicorn',
                message: 'Avoid passing a function reference directly to iterator methods.',
                file: 'src/index.node.ts',
                helpText:
                  'Wrap the function in an arrow function to explicitly pass only the element argument.',
              },),
              '',
            ]
              .join('\n',);

            const result = augmentOxlintOutput(input,);

            expect(result,).toContain(`  help: Wrap the function in an arrow function to explicitly pass only the element argument. ${getRuleGuidance('no-array-callback-reference',)}`,);
            expect(result,).not.toContain('note:',);
          },
        },),
        it({
          name: 'combines no-misused-spread guidance with help line',
          fn: async () => {
            const helpText = 'Consider using `Intl.Segmenter` for locale-aware string decomposition.';
            const input = [
              ...buildDiagnostic({
                rule: 'no-misused-spread',
                plugin: 'typescript',
                message: 'Using the spread operator on a string can mishandle special characters.',
                file: 'src/rules/tokenize.ts',
                helpText,
              },),
              '',
            ]
              .join('\n',);

            const result = augmentOxlintOutput(input,);

            expect(result,).toContain(`  help: ${helpText} ${getRuleGuidance('no-misused-spread',)}`,);
            expect(result,).not.toContain('note:',);
            expect(result.split('help:',).length - 1,).toBe(1,);
          },
        },),
        it({
          name: 'injects access help for node no-sync existsSync diagnostics',
          fn: async () => {
            const input = [
              ...buildDiagnostic({
                rule: 'no-sync',
                plugin: 'node',
                message: "Unexpected sync method: 'existsSync'.",
                file: 'src/cache.ts',
              },),
              '',
            ]
              .join('\n',);

            const result = augmentOxlintOutput(input,);

            expect(result,).toContain(`  help: ${getRuleGuidance('no-sync',)}`,);
            expect(result,).not.toContain('note:',);

            const lines = result.split('\n',);
            const helpLine = lines.findIndex(
              l => l.includes(getRuleGuidance('no-sync',),),
            );
            expect(lines[helpLine + 1],).toBe('',);
          },
        },),
        it({
          name: 'does not inject access help for other node no-sync methods',
          fn: async () => {
            const input = [
              ...buildDiagnostic({
                rule: 'no-sync',
                plugin: 'node',
                message: "Unexpected sync method: 'readFileSync'.",
                file: 'src/cache.ts',
              },),
              '',
            ]
              .join('\n',);

            const result = augmentOxlintOutput(input,);

            expect(result,).not.toContain(getRuleGuidance('no-sync',),);
            expect(result,).toBe(input,);
          },
        },),
        it({
          name: 'injects guidance before blank line when no help line exists',
          fn: async () => {
            const input = [
              ...buildDiagnostic({
                rule: 'no-misused-promises',
                plugin: 'typescript-eslint',
                message: 'Promise-returning function.',
                file: 'src/client.ts',
              },),
              '',
            ]
              .join('\n',);

            const result = augmentOxlintOutput(input,);

            expect(result,).toContain(`  help: ${getRuleGuidance('no-misused-promises',)}`,);

            // Help should appear before the trailing blank line
            const lines = result.split('\n',);
            const helpLine = lines.findIndex(
              l => l.includes(getRuleGuidance('no-misused-promises',),),
            );
            expect(lines[helpLine + 1],).toBe('',);
          },
        },),
        it({
          name: 'does not inject guidance for unmatched rules',
          fn: async () => {
            const input = [
              ...buildDiagnostic({
                rule: 'no-explicit-any',
                plugin: 'typescript-eslint',
                message: 'Unexpected any.',
                file: 'src/index.ts',
                helpText: 'Use unknown instead.',
              },),
              '',
            ]
              .join('\n',);

            const result = augmentOxlintOutput(input,);

            expect(result,).not.toContain('note:',);
            expect(result,).toBe(input,);
          },
        },),
        it({
          name: 'handles multiple diagnostics with mixed matching',
          fn: async () => {
            const input = [
              ...buildDiagnostic({
                rule: 'no-explicit-any',
                plugin: 'typescript-eslint',
                message: 'Unexpected any.',
                file: 'src/index.ts',
                helpText: 'Use unknown instead.',
              },),
              '',
              ...buildDiagnostic({
                rule: 'no-misused-promises',
                plugin: 'typescript-eslint',
                message: 'Promise-returning function.',
                file: 'src/client.ts',
                helpText: 'Expected void return type.',
              },),
              '',
              ...buildDiagnostic({
                rule: 'no-magic-numbers',
                plugin: 'eslint',
                message: 'No magic number: 42.',
                file: 'src/config.ts',
                helpText: 'Extract to a named constant.',
              },),
              '',
            ]
              .join('\n',);

            const result = augmentOxlintOutput(input,);

            expect(result,).not.toContain('note:',);
            expect(result,).toContain(`  help: Expected void return type. ${getRuleGuidance('no-misused-promises',)}`,);
          },
        },),
        it({
          name: 'handles empty output',
          fn: async () => {
            expect(augmentOxlintOutput('',),).toBe('',);
          },
        },),
        it({
          name: 'handles output with no diagnostics',
          fn: async () => {
            const input =
              'Finished in 42ms on 150 files with 300 rules using 8 threads.\n';

            expect(augmentOxlintOutput(input,),).toBe(input,);
          },
        },),
        it({
          name: 'handles trailing diagnostic without blank line',
          fn: async () => {
            const input = [
              ...buildDiagnostic({
                rule: 'no-misused-promises',
                plugin: 'typescript-eslint',
                message: 'Promise-returning function.',
                file: 'src/client.ts',
              },),
            ]
              .join('\n',);

            const result = augmentOxlintOutput(input,);

            expect(result,).toContain(`  help: ${getRuleGuidance('no-misused-promises',)}`,);
          },
        },),
        it({
          name: 'preserves ANSI codes in output while still matching',
          fn: async () => {
            const input = [
              '  \u001B[31mx\u001B[0m \u001B[33mtypescript-eslint(no-misused-promises)\u001B[0m: Promise-returning function.',
              '    ,-[src/client.ts:93:30]',
              '    `----',
              '  \u001B[36mhelp:\u001B[0m Expected void return type.',
              '',
            ]
              .join('\n',);

            const result = augmentOxlintOutput(input,);

            // Original ANSI codes preserved
            expect(result,).toContain('\u001B[31m',);
            expect(result,).toContain('\u001B[33m',);
            // Guidance injected
            expect(result,).toContain(getRuleGuidance('no-misused-promises',),);
            expect(result,).not.toContain('note:',);
          },
        },),
      ],
    },),
    //endregion augmentOxlintOutput
  ],
},);
