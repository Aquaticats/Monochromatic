import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  BUN_TEST_BAN_REASON,
  invokesBunTest,
} from '@monochromatic-dev/agent-harness-shared-shell-command-analyzer';

await describe({
  name: 'bun test guard helpers',
  children: [
    describe({
      name: invokesBunTest.name,
      children: [
        it({
          name: 'detects direct bun test invocation',
          fn: async function testDirectBunTest() {
            expect(invokesBunTest('bun test',),).toBe(true,);
          },
        },),
        it({
          name: 'detects bun test after command separator',
          fn: async function testBunTestAfterSeparator() {
            expect(invokesBunTest('cd x && bun test foo.ts',),).toBe(true,);
          },
        },),
        it({
          name: 'detects bun test in subshell segment',
          fn: async function testBunTestInSubshell() {
            expect(invokesBunTest('(bun test)',),).toBe(true,);
          },
        },),
        it({
          name: 'blocks bun test hidden inside function body',
          fn: async function testBunTestInFunctionBody() {
            expect(invokesBunTest('f(){ bun test; }; f',),).toBe(true,);
          },
        },),
        it({
          name: 'allows prose containing bun test inside echo',
          fn: async function testEchoAllowsBunTestText() {
            expect(invokesBunTest('echo "use bun test here"',),).toBe(false,);
          },
        },),
        it({
          name: 'allows bun run test script command',
          fn: async function testBunRunTestAllowed() {
            expect(invokesBunTest('bun run test',),).toBe(false,);
          },
        },),
        it({
          name: 'allows longer test-prefixed words',
          fn: async function testLongerWordsAllowed() {
            expect(invokesBunTest('bun tests',),).toBe(false,);
            expect(invokesBunTest('bun test_runner',),).toBe(false,);
          },
        },),
      ],
    },),
    describe({
      name: 'BUN_TEST_BAN_REASON',
      children: [
        it({
          name: 'points users at mise test tasks and node file runs',
          fn: async function testBanReason() {
            expect(BUN_TEST_BAN_REASON.includes('mise run //package/<path>:test:unit',),)
              .toBe(true,);
            expect(BUN_TEST_BAN_REASON.includes('node <file>',),).toBe(true,);
          },
        },),
      ],
    },),
  ],
},);
