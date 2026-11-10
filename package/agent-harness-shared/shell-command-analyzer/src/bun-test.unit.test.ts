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
        it({
          name: 'detects bun test behind wrapper commands',
          fn: async function testBunTestBehindWrappers() {
            expect(invokesBunTest('env bun test',),).toBe(true,);
            expect(invokesBunTest('timeout 10 bun test',),).toBe(true,);
            expect(invokesBunTest('nohup bun test',),).toBe(true,);
            expect(invokesBunTest('nice bun test',),).toBe(true,);
            expect(invokesBunTest('command bun test',),).toBe(true,);
          },
        },),
        it({
          name: 'detects bun test behind a wrapper chain that carries an assignment',
          fn: async function testBunTestBehindWrapperChain() {
            expect(invokesBunTest('env X=1 sudo nice -n 5 bun test',),).toBe(true,);
          },
        },),
        it({
          name: 'detects bun test launched through absolute paths',
          fn: async function testBunTestThroughAbsolutePaths() {
            expect(invokesBunTest('/usr/local/bin/bun test',),).toBe(true,);
            expect(invokesBunTest('/usr/bin/env bun test',),).toBe(true,);
            expect(invokesBunTest('/usr/bin/timeout 10 bun test',),).toBe(true,);
          },
        },),
        it({
          name: 'detects bun test after bun global flags',
          fn: async function testBunTestAfterGlobalFlags() {
            expect(invokesBunTest('bun --cwd /tmp test',),).toBe(true,);
            expect(invokesBunTest('bun -c bunfig.toml test',),).toBe(true,);
            expect(invokesBunTest('bun --smol test',),).toBe(true,);
          },
        },),
        it({
          name: 'detects bun test a wrapper joins into shell source',
          fn: async function testBunTestJoinedByWrapper() {
            expect(invokesBunTest('eval "bun test"',),).toBe(true,);
          },
        },),
        it({
          name: 'detects bun test a shell runs from its script option',
          fn: async function testBunTestInShellScript() {
            expect(invokesBunTest("bash -c 'bun test'",),).toBe(true,);
            expect(invokesBunTest("env bash -c 'bun test'",),).toBe(true,);
            expect(invokesBunTest("sh -c 'cd x && bun test'",),).toBe(true,);
            expect(invokesBunTest("/bin/bash -c 'bun test'",),).toBe(true,);
          },
        },),
        it({
          name: 'detects bun test in a command that fails to parse',
          fn: async function testBunTestInUnparsableCommand() {
            expect(invokesBunTest('bun test "unterminated',),).toBe(true,);
            expect(invokesBunTest('bun test $(echo ',),).toBe(true,);
            expect(invokesBunTest('bun --cwd /tmp test "unterminated',),).toBe(true,);
            expect(invokesBunTest('cd x && bun test "unterminated',),).toBe(true,);
            expect(invokesBunTest('echo $(bun test',),).toBe(true,);
            expect(invokesBunTest(String.raw`bun te\st "unterminated`,),).toBe(true,);
          },
        },),
        it({
          name: 'detects bun test behind a wrapper in a command that fails to parse',
          fn: async function testWrappedBunTestInUnparsableCommand() {
            expect(invokesBunTest('env bun test "unterminated',),).toBe(true,);
            expect(invokesBunTest('bash -c \'bun test"',),).toBe(true,);
          },
        },),
        it({
          name: 'allows prose containing bun test in a command that fails to parse',
          fn: async function testUnparsableProseAllowed() {
            expect(invokesBunTest('echo "bun test',),).toBe(false,);
            expect(invokesBunTest('printf bun test',),).toBe(false,);
          },
        },),
        it({
          name: 'allows a test word a known bun subcommand owns',
          fn: async function testTestWordUnderKnownSubcommand() {
            expect(invokesBunTest('env bun run test',),).toBe(false,);
            expect(invokesBunTest('bun install test',),).toBe(false,);
            expect(invokesBunTest('env bun run test "unterminated',),).toBe(false,);
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
