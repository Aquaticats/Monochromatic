import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  PROCESS_NAME_WAIT_BAN_REASON,
  waitsOnProcessName,
} from '@monochromatic-dev/agent-harness-shared-shell-command-analyzer';

await describe({
  name: 'process-name wait guard helpers',
  children: [
    describe({
      name: waitsOnProcessName.name,
      children: [
        it({
          name: 'detects until-negated pgrep in the condition list',
          fn: async function testUntilNegatedPgrep() {
            expect(waitsOnProcessName('until ! pgrep -f build; do sleep 1; done',),).toBe(true,);
          },
        },),
        it({
          name: 'detects while pgrep in the condition list',
          fn: async function testWhilePgrep() {
            expect(waitsOnProcessName('while pgrep --full build >/dev/null; do sleep 1; done',),)
              .toBe(true,);
          },
        },),
        it({
          name: 'detects pidof in the condition list',
          fn: async function testWhilePidof() {
            expect(waitsOnProcessName('while pidof build; do sleep 1; done',),).toBe(true,);
          },
        },),
        it({
          name: 'detects pgrep in the loop body',
          fn: async function testPgrepInBody() {
            expect(waitsOnProcessName('while true; do pgrep -f build || break; sleep 1; done',),)
              .toBe(true,);
          },
        },),
        it({
          name: 'detects pgrep in a nested loop after other commands',
          fn: async function testNestedLoop() {
            expect(
              waitsOnProcessName('cd x && for i in 1 2; do until ! pgrep build; do sleep 1; done; done',),
            )
              .toBe(true,);
          },
        },),
        it({
          name: 'detects pgrep loop inside a nested bash -c script',
          fn: async function testNestedScript() {
            expect(waitsOnProcessName("timeout 60 bash -c 'until ! pgrep -f build; do sleep 1; done'",),)
              .toBe(true,);
          },
        },),
        it({
          name: 'allows straight-line pgrep',
          fn: async function testStraightLinePgrep() {
            expect(waitsOnProcessName('pgrep --list-full --full build',),).toBe(false,);
          },
        },),
        it({
          name: 'allows pgrep inside a for loop',
          fn: async function testForLoopPgrep() {
            expect(waitsOnProcessName('for name in a b; do pgrep "$name"; done',),).toBe(false,);
          },
        },),
        it({
          name: 'allows pgrep after a finished loop',
          fn: async function testPgrepAfterLoop() {
            expect(waitsOnProcessName('while read -r line; do echo "$line"; done < in; pgrep x',),)
              .toBe(false,);
          },
        },),
        it({
          name: 'allows PID-based waiting loops',
          fn: async function testPidWait() {
            expect(waitsOnProcessName('while kill -0 "$pid"; do sleep 1; done',),).toBe(false,);
            expect(waitsOnProcessName('timeout 600 tail --pid=123 --follow /dev/null',),).toBe(false,);
          },
        },),
        it({
          name: 'allows prose mentioning pgrep inside a loop',
          fn: async function testProseInLoop() {
            expect(waitsOnProcessName('while read -r l; do echo "use pgrep later"; done < in',),)
              .toBe(false,);
          },
        },),
        it({
          name: 'allows pgrep named only in a function body outside any loop',
          fn: async function testFunctionBodyOutsideLoop() {
            expect(waitsOnProcessName('f(){ pgrep x; }; f',),).toBe(false,);
          },
        },),
        it({
          name: 'treats unparsable input as allowed',
          fn: async function testUnparsable() {
            expect(waitsOnProcessName('until ! pgrep x; do',),).toBe(false,);
          },
        },),
      ],
    },),
    describe({
      name: 'PROCESS_NAME_WAIT_BAN_REASON',
      children: [
        it({
          name: 'names the self-match cause and the PID-based replacements',
          fn: async function testBanReason() {
            expect(PROCESS_NAME_WAIT_BAN_REASON.includes('wrapper',),).toBe(true,);
            expect(PROCESS_NAME_WAIT_BAN_REASON.includes('tail --pid=<pid>',),).toBe(true,);
            expect(PROCESS_NAME_WAIT_BAN_REASON.includes('systemd-run --user --wait',),).toBe(true,);
          },
        },),
      ],
    },),
  ],
},);
