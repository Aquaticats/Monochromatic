/**
 Regression coverage for the installed `unbash` syntax tree.
 
 Cases were written against the unbash 5 tree,
 and the file name records that origin;
 they stay valid for unbash 6,
 where `TestLogical` is n-ary and `Time`/`Negation` are pipeline prefixes.
 
 @module
 */

import { analyzeShellCommand, } from '@monochromatic-dev/agent-harness-shared-shell-command-analyzer';
import { describe, expect, it, } from '@monochromatic-dev/test-harness-core/ts';

//region Fixtures

/**
 One shell form whose hidden command must reach the analysis.

The marker is the argument the hidden command carries,
so a reported command can be traced back to the form that held it.
 */
type SubstitutionForm = {
  /**
   Shell source handed to the analyzer; it is never executed.
   */
  readonly source: string;
  /**
   Argument the nested command carries.
   */
  readonly marker: string;
  /**
   Whether the nesting holds a process substitution instead of a command substitution.
   */
  readonly processSubstitution: boolean;
};

/**
 Word and expression forms that can hold a nested command, keyed by where the
 substitution sits.
 */
const NESTED_SUBSTITUTION_FORMS: readonly SubstitutionForm[] = [
  { source: 'echo {a,$(touch marker)}', marker: 'marker', processSubstitution: false, },
  { source: 'echo {a,{b,$(touch marker)}}', marker: 'marker', processSubstitution: false, },
  { source: 'echo @(a|$(touch marker))', marker: 'marker', processSubstitution: false, },
  { source: 'echo !(a|$(touch marker))', marker: 'marker', processSubstitution: false, },
  { source: 'echo *(a|$(touch marker))', marker: 'marker', processSubstitution: false, },
  { source: 'echo +(a|$(touch marker))', marker: 'marker', processSubstitution: false, },
  { source: 'echo ?(a|$(touch marker))', marker: 'marker', processSubstitution: false, },
  { source: 'echo [$(touch marker)]', marker: 'marker', processSubstitution: false, },
  { source: 'echo $(( 1$(touch marker) ))', marker: 'marker', processSubstitution: false, },
  { source: 'echo $(( ${value:-$(touch marker)} ))', marker: 'marker', processSubstitution: false, },
  { source: 'echo $(( 1@(a|$(touch marker)) ))', marker: 'marker', processSubstitution: false, },
  { source: 'echo ${value:-{x,$(touch marker)}}', marker: 'marker', processSubstitution: false, },
  { source: 'echo {a,@(b|$(touch marker))}', marker: 'marker', processSubstitution: false, },
  { source: 'echo hi > {a,$(touch marker)}', marker: 'marker', processSubstitution: false, },
  { source: 'cat <<< {a,$(touch marker)}', marker: 'marker', processSubstitution: false, },
  { source: 'value={a,$(touch marker)} printf hi', marker: 'marker', processSubstitution: false, },
  { source: 'value=({a,$(touch marker)}) printf hi', marker: 'marker', processSubstitution: false, },
  { source: 'for name in {a,$(touch marker)}; do printf hi; done', marker: 'marker', processSubstitution: false, },
  { source: 'case x in {a,$(touch marker)}) printf hi;; esac', marker: 'marker', processSubstitution: false, },
  { source: 'cat <<EOF\n{a,$(touch marker)}\nEOF', marker: 'marker', processSubstitution: false, },
  { source: 'echo {a,<(touch marker)}', marker: 'marker', processSubstitution: true, },
  { source: 'echo @(a|<(touch marker))', marker: 'marker', processSubstitution: true, },
];

/**
 Word forms the shell grammar keeps literal, so no command can hide in them.
 */
const COMMAND_FREE_FORMS: readonly { readonly source: string; readonly reason: string }[] = [
  { source: 'echo \'$(touch marker)\'', reason: 'single quotes suppress every expansion', },
  { source: 'echo $\'$(touch marker)\'', reason: 'ANSI-C quoting only decodes backslash escapes', },
  { source: 'echo *', reason: 'a star glob part carries no child list', },
  { source: 'echo a[bc]d', reason: 'a bracket glob over literals carries no expansion', },
  { source: 'cat <<\'EOF\'\n$(touch marker)\nEOF', reason: 'a quoted heredoc delimiter makes the body literal', },
];

//endregion Fixtures

await describe({
  name: '',
  children: [
    describe({
      name: analyzeShellCommand.name,
      children: [
        ...[
          { source: 'time -p printf timed', name: 'printf', },
          { source: '! printf negated', name: 'printf', },
          { source: '{ printf grouped; } > output.txt', name: 'printf', },
          { source: 'coproc worker { printf child; } > output.txt', name: 'printf', },
          { source: 'for value; do printf "$value"; done', name: 'printf', },
          { source: 'select value; do printf "$value"; done', name: 'printf', },
        ].map(function wrappedCommand({ source, name, },) {
          return it({
            name: `retains commands in ${source}`,
            fn: async function testWrappedCommand() {
              const result = analyzeShellCommand(source,);
              expect(result.parsed,).toBe(true,);
              expect(result.commands.map(function commandName(command,) {
                return command.name;
              },),).toContain(name,);
              if (source.includes('> output.txt',))
                expect(result.commands.flatMap(function targets(command,) {
                  return command.redirectTargets;
                },),).toEqual(['output.txt',],);
            },
          },);
        },),
        it({
          name: 'traverses every operand of a two-way TestLogical expression',
          fn: async function testTwoWayTestLogical() {
            const result = analyzeShellCommand('[[ -n $(printf logical_left_value) && -n $(printf logical_right_value) ]]',);
            expect(result.parsed,).toBe(true,);
            expect(result.hasCommandSubstitution,).toBe(true,);
            const nestedArguments = result.commands.flatMap(function commandArguments(command,) {
              return (command.name === 'printf') ? command.args : [];
            },);
            expect(nestedArguments,).toContain('logical_left_value',);
            expect(nestedArguments,).toContain('logical_right_value',);
          },
        },),
        it({
          name: 'traverses every operand of a three-way TestLogical expression',
          fn: async function testThreeWayTestLogical() {
            const result = analyzeShellCommand('[[ -n $(printf logical_first_value) && -n $(printf logical_second_value) || -n $(printf logical_third_value) ]]',);
            expect(result.parsed,).toBe(true,);
            expect(result.hasCommandSubstitution,).toBe(true,);
            const nestedArguments = result.commands.flatMap(function commandArguments(command,) {
              return (command.name === 'printf') ? command.args : [];
            },);
            expect(nestedArguments,).toContain('logical_first_value',);
            expect(nestedArguments,).toContain('logical_second_value',);
            expect(nestedArguments,).toContain('logical_third_value',);
          },
        },),
        it({
          name: 'keeps redirections separate from mixed command prefixes and suffixes',
          fn: async function testMixedCommandItems() {
            const result = analyzeShellCommand('> before.txt FIRST="one two" printf alpha 2> after.txt beta',);
            const [command,] = result.commands;
            expect(result.parsed,).toBe(true,);
            expect(command?.name,).toBe('printf',);
            expect(command?.args,).toEqual(['alpha', 'beta',],);
            expect(command?.envAssignments,).toEqual([{ name: 'FIRST', value: 'one two', },],);
            expect(command?.redirectTargets,).toEqual(['before.txt', 'after.txt',],);
            expect(command?.redirects[1]?.fileDescriptor,).toBe(2,);
          },
        },),
        it({
          name: 'preserves numeric and variable redirect descriptors',
          fn: async function testRedirectDescriptors() {
            const result = analyzeShellCommand('exec {output}> out.txt 2>&1',);
            expect(result.parsed,).toBe(true,);
            expect(result.commands[0]?.redirects,).toEqual([
              { operator: '>', fileDescriptorVariable: 'output', target: 'out.txt', targetSourceText: 'out.txt', kind: 'file', writesFile: true, readsFile: false, },
              { operator: '>&', fileDescriptor: 2, target: '1', targetSourceText: '1', kind: 'fileDescriptor', writesFile: false, readsFile: false, },
            ],);
          },
        },),
        it({
          name: 'preserves declaration assignment values and source spelling',
          fn: async function testDeclarationAssignments() {
            const result = analyzeShellCommand('export VALUE="one two" > output.txt',);
            expect(result.parsed,).toBe(true,);
            expect(result.commands[0]?.args,).toEqual(['VALUE=one two',],);
            expect(result.commands[0]?.argSources,).toEqual([
              { value: 'VALUE=one two', sourceText: 'VALUE="one two"', },
            ],);
          },
        },),
        ...[
          { source: 'declare VALUES=("one two" three)', value: 'VALUES=(one two three)', },
          { source: 'declare VALUES[2]+="more text"', value: 'VALUES[2]+=more text', },
          { source: 'declare VALUES=()', value: 'VALUES=()', },
        ].map(function assignmentArgument({ source, value, },) {
          return it({
            name: `preserves assignment argument ${source}`,
            fn: async function testAssignmentArgument() {
              const result = analyzeShellCommand(source,);
              expect(result.parsed,).toBe(true,);
              expect(result.commands[0]?.args,).toEqual([value,],);
              expect(result.commands[0]?.argSources[0]?.sourceText,).toBe(source.slice('declare '.length,),);
            },
          },);
        },),
        ...[
          `printf "\${value@Q}"`,
          `printf "\${!prefix@}"`,
          `printf "\${value^^}"`,
        ].map(function parameterOperation(source,) {
          return it({
            name: `keeps parameter operation ${source} free of invented commands`,
            fn: async function testParameterOperation() {
              const result = analyzeShellCommand(source,);
              expect(result.parsed,).toBe(true,);
              expect(result.hasCommandSubstitution,).toBe(false,);
              expect(result.commands.map(function commandName(command,) {
                return command.name;
              },),).toEqual(['printf',],);
            },
          },);
        },),
        ...[
          'values=($(printf array_value))',
          'values[$(printf array_index)]=literal',
          'declare values=($(printf declaration_value))',
          `printf "\${values[$(printf parameter_index)]}"`,
          `printf "\${value:-$(printf default_value)}"`,
          `printf "\${value#$(printf removal_pattern)}"`,
          `printf "\${value^$(printf case_pattern)}"`,
          `printf "\${value/$(printf replace_pattern)/$(printf replacement)}"`,
          `printf "\${value:$(printf slice_offset):$(printf slice_length)}"`,
          'cat <<< "$(printf here_string)"',
          'cat <<EOF\n$(printf here_body)\nEOF',
        ].map(function nestedExpression(source,) {
          return it({
            name: `traverses nested commands in ${source}`,
            fn: async function testNestedExpression() {
              const result = analyzeShellCommand(source,);
              expect(result.parsed,).toBe(true,);
              expect(result.hasCommandSubstitution,).toBe(true,);
              expect(result.commands.some(function hasNestedCommand(command,) {
                return (command.name === 'printf')
                  && command.args.some(function hasMarker(argument,) {
                    return argument.endsWith('_value',) || argument.endsWith('_index',)
                      || argument.endsWith('_pattern',) || (argument === 'replacement')
                      || argument.startsWith('slice_',) || argument.startsWith('here_',);
                  },);
              },),).toBe(true,);
            },
          },);
        },),
        it({
          name: 'does not execute quoted heredoc content during analysis',
          fn: async function testQuotedHeredoc() {
            const result = analyzeShellCommand("cat <<'EOF'\n$(printf literal_body)\nEOF",);
            expect(result.parsed,).toBe(true,);
            expect(result.hasHeredoc,).toBe(true,);
            expect(result.hasCommandSubstitution,).toBe(false,);
            expect(result.commands.map(function commandName(command,) {
              return command.name;
            },),).toEqual(['cat',],);
          },
        },),
        it({
          name: 'keeps redirected function bodies in function-definition context',
          fn: async function testFunctionRedirect() {
            const result = analyzeShellCommand('saved() { printf body; } > output.txt',);
            expect(result.parsed,).toBe(true,);
            expect(result.commands.every(function belongsToFunction(command,) {
              return command.context.kind === 'functionDefinition';
            },),).toBe(true,);
            expect(result.commands.flatMap(function targets(command,) {
              return command.redirectTargets;
            },),).toEqual(['output.txt',],);
          },
        },),
      ],
    },),
    describe({
      name: 'nested command substitution',
      children: [
        ...NESTED_SUBSTITUTION_FORMS.map(function substitutionForm(form,) {
          return it({
            name: `reports the nested command in ${form.source}`,
            fn: async function testNestedSubstitutionForm() {
              const result = analyzeShellCommand(form.source,);
              expect(result.parsed,).toBe(true,);
              expect(result.hasCommandSubstitution,).toBe(!form.processSubstitution,);
              expect(result.hasProcessSubstitution,).toBe(form.processSubstitution,);
              expect(result.notUnderstoodNodeKinds,).toEqual([],);
              expect(result.executedCommands.filter(function carriesMarker(command,) {
                return command.args.includes(form.marker,);
              },).map(function markerCommandName(command,) {
                return command.name;
              },),).toEqual(['touch',],);
            },
          },);
        },),
        ...COMMAND_FREE_FORMS.map(function commandFreeForm(form,) {
          return it({
            name: `reports no command in ${form.source} because ${form.reason}`,
            fn: async function testCommandFreeForm() {
              const result = analyzeShellCommand(form.source,);
              expect(result.parsed,).toBe(true,);
              expect(result.hasCommandSubstitution,).toBe(false,);
              expect(result.hasProcessSubstitution,).toBe(false,);
              expect(result.notUnderstoodNodeKinds,).toEqual([],);
              expect(result.commands.some(function isTouch(command,) {
                return command.name === 'touch';
              },),).toBe(false,);
            },
          },);
        },),
        it({
          name: 'names a parameter operation the analyzer cannot read',
          fn: async function testUnreadParameterOperation() {
            const result = analyzeShellCommand('echo ${value;text}',);
            expect(result.parsed,).toBe(true,);
            expect(result.notUnderstoodNodeKinds,).toEqual(['Unknown',],);
          },
        },),
        it({
          name: 'names no unread node kind for a readable parameter default',
          fn: async function testReadParameterOperation() {
            const result = analyzeShellCommand('echo ${value:-text}',);
            expect(result.parsed,).toBe(true,);
            expect(result.notUnderstoodNodeKinds,).toEqual([],);
          },
        },),
      ],
    },),
  ],
},);
