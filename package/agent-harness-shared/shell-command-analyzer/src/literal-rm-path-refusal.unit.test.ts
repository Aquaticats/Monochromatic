/**
 Tests for the refusal text of the literal-path deletion guard.
 
 Every check the reader can fire gets its own case,
 so a refusal that stops naming one is caught here.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  CHECK_PHRASES,
  LITERAL_RM_PATH_BAN_REASON,
  literalRmPathRefusal,
  type LiteralRmPathCheck,
  type LiteralRmPathFinding,
} from '@monochromatic-dev/agent-harness-shared-shell-command-analyzer';

//region Fixtures

/**
 One finding read from a command string, which names no file.
 */
const COMMAND_FINDING: LiteralRmPathFinding = {
  command: 'rm -rf "$d"',
  word: '"$d"',
  check: 'shell-computes-the-path',
};

/**
 One finding read from a script file, which names the file and the line.
 */
const SCRIPT_FINDING: LiteralRmPathFinding = {
  command: 'rm -f "$d/$n"',
  word: '"$d/$n"',
  check: 'shell-computes-the-path',
  file: '/repo/clean.sh',
  line: 3,
};

/**
 Every check the reader can fire, with a finding that carries it.
 */
const FINDING_PER_CHECK: readonly LiteralRmPathFinding[] = [
  {
    command: 'printf \'%s\' build | xargs rm -rf',
    word: 'rm',
    check: 'another-program-supplies-the-paths',
  },
  {
    command: 'find /home/user/temp/agent/x -delete',
    word: '-delete',
    check: 'find-deletes-the-paths-it-found',
  },
  COMMAND_FINDING,
  {
    command: 'rmdir -p /home',
    word: '/home',
    check: 'target-collapses-to-a-broader-path',
    detail: 'a top-level folder',
  },
];

//endregion Fixtures

await describe({
  name: 'literal rm path refusal',
  children: [
    describe({
      name: LITERAL_RM_PATH_BAN_REASON,
      children: [
        it({
          name: 'names the rule, every deleting program, and the reason a value widens a deletion',
          fn: async function testRuleReason() {
            expect(LITERAL_RM_PATH_BAN_REASON,).toContain('`$HOME`',);
            expect(LITERAL_RM_PATH_BAN_REASON,).toContain('`rmdir`',);
            expect(LITERAL_RM_PATH_BAN_REASON,).toContain('`unlink`',);
            expect(LITERAL_RM_PATH_BAN_REASON,).toContain('`shred`',);
            expect(LITERAL_RM_PATH_BAN_REASON,).toContain('`find -delete`',);
            expect(LITERAL_RM_PATH_BAN_REASON,).toContain('unset or empty value',);
            expect(LITERAL_RM_PATH_BAN_REASON,).toContain('collapses to',);
          },
        },),
      ],
    },),
    describe({
      name: 'CHECK_PHRASES',
      children: [
        it({
          name: 'names every check the reader can fire',
          fn: async function testEveryCheckNamed() {
            for (const finding of FINDING_PER_CHECK) {
              expect(CHECK_PHRASES.has(finding.check,),).toBe(true,);
            }
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
            expect(refusal,).toContain('(word `"$d"`;',);
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
          name: 'names which check fired and why, for every check',
          fn: async function testCheckNamed() {
            for (const finding of FINDING_PER_CHECK) {
              /**
               Refusal text for one finding.
               */
              const refusal = literalRmPathRefusal({ findings: [finding,], },);
              /**
               Phrase the refusal prints for this check, with its reading when it has one.
               */
              const expectedReason = (finding.detail === undefined)
                ? CHECK_PHRASES.get(finding.check,)
                : `${CHECK_PHRASES.get(finding.check,)}: ${finding.detail}`;
              expect(refusal,).toContain(`(word \`${finding.word}\`; ${expectedReason})`,);
            }
          },
        },),
        it({
          name: 'says to write each path literally in one deletion command',
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
            const findings: readonly LiteralRmPathFinding[] = Array.from(
              { length: 8, },
              function findingAt(_, index,): LiteralRmPathFinding {
                return {
                  command: `rm -rf "$d${String(index,)}"`,
                  word: `"$d${String(index,)}"`,
                  check: 'shell-computes-the-path',
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
        it({
          name: 'prints a check it has no phrase for as the check itself',
          fn: async function testUnknownCheckPhrase() {
            /**
             A check the phrase table does not carry, which the reader cannot fire.
             */
            const unexpectedCheck = 'a-check-nobody-tabled' as LiteralRmPathCheck;
            expect(
              literalRmPathRefusal({
                findings: [{
                  command: 'rm -rf "$d"',
                  word: '"$d"',
                  check: unexpectedCheck,
                },],
              },),
            ).toContain('(word `"$d"`; a-check-nobody-tabled)',);
          },
        },),
      ],
    },),
  ],
},);
