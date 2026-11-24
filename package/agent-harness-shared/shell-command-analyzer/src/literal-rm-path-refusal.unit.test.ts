/**
 Tests for the refusal text of the literal-`rm`-path guard.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  LITERAL_RM_PATH_BAN_REASON,
  literalRmPathRefusal,
} from '@monochromatic-dev/agent-harness-shared-shell-command-analyzer';

//region Fixtures

/**
 One finding read from a command string, which names no file.
 */
const COMMAND_FINDING = {
  command: 'rm -rf "$d"',
  word: '"$d"',
};

/**
 One finding read from a script file, which names the file and the line.
 */
const SCRIPT_FINDING = {
  command: 'rm -f "$d/$n"',
  word: '"$d/$n"',
  file: '/repo/clean.sh',
  line: 3,
};

//endregion Fixtures

await describe({
  name: 'literal rm path refusal',
  children: [
    describe({
      name: LITERAL_RM_PATH_BAN_REASON,
      children: [
        it({
          name: 'names the rule and the reason an unset variable widens a deletion',
          fn: async function testRuleReason() {
            expect(LITERAL_RM_PATH_BAN_REASON,).toContain('`$HOME`',);
            expect(LITERAL_RM_PATH_BAN_REASON,).toContain('unset or empty variable',);
          },
        },),
      ],
    },),
    describe({
      name: literalRmPathRefusal.name,
      children: [
        it({
          name: 'names the command and the offending word',
          fn: async function testCommandAndWord() {
            /**
             Refusal text for one finding read from a command string.
             */
            const refusal = literalRmPathRefusal({ findings: [COMMAND_FINDING,], },);
            expect(refusal,).toContain('`rm -rf "$d"`',);
            expect(refusal,).toContain('(word `"$d"`)',);
          },
        },),
        it({
          name: 'names the file and line of a finding read from a script',
          fn: async function testFileAndLine() {
            expect(
              literalRmPathRefusal({ findings: [SCRIPT_FINDING,], },),
            ).toContain('/repo/clean.sh line 3: `rm -f "$d/$n"`',);
          },
        },),
        it({
          name: 'says to write each path literally in one rm command',
          fn: async function testRemedy() {
            expect(
              literalRmPathRefusal({ findings: [COMMAND_FINDING,], },),
            ).toContain('`rm --recursive /home/user/temp/agent/<folder>/scratch`',);
          },
        },),
        it({
          name: 'says not to re-express the deletion through another form',
          fn: async function testNoReexpression() {
            /**
             Refusal text, whose guidance covers every surface a deletion can move to.
             */
            const refusal = literalRmPathRefusal({ findings: [COMMAND_FINDING,], },);
            for (const form of ['script', 'heredoc', 'another tool', 'code mode', 'another program',]) {
              expect(refusal,).toContain(form,);
            }
          },
        },),
        it({
          name: 'summarizes the findings past the listed count',
          fn: async function testBoundedList() {
            /**
             More findings than one refusal spells out.
             */
            const findings = Array.from(
              { length: 8, },
              function findingAt(_, index,) {
                return {
                  command: `rm -rf "$d${String(index,)}"`,
                  word: `"$d${String(index,)}"`,
                };
              },
            );
            /**
             Refusal text for those findings.
             */
            const refusal = literalRmPathRefusal({ findings, },);
            expect(refusal,).toContain('and 3 more',);
            expect(refusal,).not.toContain('"$d7"',);
          },
        },),
      ],
    },),
  ],
},);
