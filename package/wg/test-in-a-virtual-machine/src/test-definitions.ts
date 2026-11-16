/**
 The four tests that need a machine of their own, as this runner drives them.

 Each definition names the mise task that runs it, whether the guest needs the
 Node workspace installed or the Rust crate built, and the commands that run
 inside the guest with the caller each was written for.

 @module
 */

/**
 The guest directory the checkout is unpacked into.
 */
export const GUEST_CHECKOUT_DIRECTORY = '/work';

/**
 The guest directory every phase's output is written to.
 */
export const GUEST_LOG_DIRECTORY = '/work/logs';

/**
 The guest file whose presence tells a test it runs inside a machine.
 */
export const GUEST_MARKER_PATH = '/run/wg-quicker-test-in-a-virtual-machine';

/**
 One command a test runs inside the guest.
 */
export type TestPhase = {
  /**
   Name of this phase, used for its log file and its heading in the output.
   */
  readonly id: string;

  /**
   Who runs it: the account that starts the tests, or the machine's root.
   */
  readonly caller: 'account' | 'root';

  /**
   Guest directory the command starts in.
   */
  readonly workingDirectory: string;

  /**
   Variables set for this command alone.
   */
  readonly environment: Readonly<Record<string, string>>;

  /**
   Program and arguments.
   */
  readonly words: readonly string[];
};

/**
 One test this package can run inside a machine.
 */
export type TestDefinition = {
  /**
   Identifier the command line names.
   */
  readonly id: string;

  /**
   One line saying what the test proves.
   */
  readonly description: string;

  /**
   The mise task that runs it, named in every refusal.
   */
  readonly miseTask: string;

  /**
   Whether the guest needs the Node workspace installed for the checkout.
   */
  readonly needsNodeWorkspace: boolean;

  /**
   Whether the guest needs the Rust crate built.
   */
  readonly needsRustCrate: boolean;

  /**
   The commands to run, in order, given what the guest holds after provisioning.
   *
   @param context - paths the provisioning found in the guest
   */
  readonly phases: (context: {
    readonly testBinaryPath: string;
  },) => readonly TestPhase[];
};

/**
 Path of the built wg-quicker bundle the integration tests drive.
 */
const WG_QUICKER_PACKAGE = `${GUEST_CHECKOUT_DIRECTORY}/package/wg/cli-quicker`;

/**
 Path of the Rust crate whose ignored tests need root or a user manager.
 */
export const GUEST_CRATE_DIRECTORY: string = `${GUEST_CHECKOUT_DIRECTORY}/package/wg/cli-quicker-exempt`;

/**
 The crate directory, spelled as the definitions below use it.
 */
const EXEMPT_PACKAGE = GUEST_CRATE_DIRECTORY;

/**
 Ignored tests that need an unprivileged caller with a user manager of its own.

 They create throwaway transient units in that manager, so they cannot run as
 root: a machine's root has no `user@0.service`, and the manager they must talk
 to is the account's.
 */
const UNPRIVILEGED_IGNORED_TESTS: readonly string[] = [
  'manager_probe_tests::the_manager_answers_each_part_of_a_call',
  'manager_probe_tests::the_manager_sends_signals_before_its_reply',
  'manager_probe_tests::a_call_after_a_big_reply_still_gets_its_own_reply',
  'leftover_sweep_tests::the_real_manager_sweeps_throwaway_scopes_and_spares_the_one_holding_a_process',
  'scope_adoption_tests::the_real_manager_creates_attaches_and_follows_limits',
  'sentinel_system_steps_tests::a_unit_that_stays_active_is_an_error_that_says_so',
];

/**
 Skip patterns that keep the root phase from running those six.

 `--skip` takes a substring, so each pattern is written to match only the tests
 named above.
 */
const UNPRIVILEGED_SKIP_PATTERNS: readonly string[] = [
  'manager_probe_tests',
  'leftover_sweep_tests::the_real_manager',
  'scope_adoption_tests::the_real_manager',
  'sentinel_system_steps_tests::a_unit_that_stays_active',
];

/**
 Environment the unprivileged phases need to reach the account's user manager.

 The manager answers on its private socket and its bus, both named by the
 account's runtime directory, which `runuser` does not set.
 */
const ACCOUNT_MANAGER_ENVIRONMENT: Readonly<Record<string, string>> = {
  DBUS_SESSION_BUS_ADDRESS: 'unix:path=/run/user/1000/bus',
  XDG_RUNTIME_DIR: '/run/user/1000',
};

/**
 Turns one ignored test name into the words that run exactly that test.

 @param binary - guest path of the test executable

 @param test - the ignored test to run

 @returns program and arguments
 */
function exactIgnoredTestWords(
  {
    binary,
    test,
  }: {
    readonly binary: string;
    readonly test: string;
  },
): readonly string[] {
  return [
    binary,
    '--exact',
    test,
    '--ignored',
    '--test-threads=1',
    '--nocapture',
  ];
}

/**
 Every test this package runs.
 */
export const TEST_DEFINITIONS: Readonly<Record<string, TestDefinition>> = {
  bypass: {
    id: 'bypass',
    description: 'bypass routing, its watcher and the built CLI in root network namespaces',
    miseTask: '//package/wg/cli-quicker:test:integration:bypass',
    needsNodeWorkspace: true,
    needsRustCrate: false,
    phases: function phases(): readonly TestPhase[] {
      return [
        {
          id: 'bypass',
          caller: 'account',
          workingDirectory: WG_QUICKER_PACKAGE,
          environment: {},
          words: [
            'node',
            'src/tunnel-bypass.integration.test.ts',
          ],
        },
      ];
    },
  },
  route: {
    id: 'route',
    description: 'endpoint routing, the built CLI and a transferring tunnel',
    miseTask: '//package/wg/cli-quicker:test:integration:route',
    needsNodeWorkspace: true,
    needsRustCrate: false,
    phases: function phases(): readonly TestPhase[] {
      return [
        {
          id: 'route',
          caller: 'account',
          workingDirectory: WG_QUICKER_PACKAGE,
          environment: {},
          words: [
            'node',
            'src/tunnel-route.integration.test.ts',
          ],
        },
      ];
    },
  },
  functional: {
    id: 'functional',
    description: 'every ignored test of the crate, each as the caller it was written for',
    miseTask: '//package/wg/cli-quicker-exempt:test:functional',
    needsNodeWorkspace: false,
    needsRustCrate: true,
    phases: function phases({ testBinaryPath, }: { readonly testBinaryPath: string; },): readonly TestPhase[] {
      return [
        // The unprivileged phases run first on purpose: they create the build
        // directory's scratch tree and the account's gate file, and a root
        // phase that ran first would leave both owned by root, where the
        // account can neither write nor lock them.
        ...UNPRIVILEGED_IGNORED_TESTS.map(function unprivilegedPhase(test,): TestPhase {
          return {
            id: `account-${test.split('::',)[1] ?? test}`,
            caller: 'account',
            workingDirectory: EXEMPT_PACKAGE,
            environment: {
              ...ACCOUNT_MANAGER_ENVIRONMENT,
              WGTEST_RUNNING_UNIT: 'dbus.service',
            },
            words: exactIgnoredTestWords({
              binary: testBinaryPath,
              test,
            },),
          };
        },),
        {
          id: 'root',
          caller: 'root',
          workingDirectory: EXEMPT_PACKAGE,
          environment: {},
          words: [
            testBinaryPath,
            '--ignored',
            '--test-threads=1',
            '--nocapture',
            ...UNPRIVILEGED_SKIP_PATTERNS.flatMap(function skipPattern(pattern,): readonly string[] {
              return [
                '--skip',
                pattern,
              ];
            },),
          ],
        },
      ];
    },
  },
  'functional-protocols': {
    id: 'functional-protocols',
    description: 'one root test loading every protocol hook on a throwaway cgroup',
    miseTask: '//package/wg/cli-quicker-exempt:test:functional:protocols',
    needsNodeWorkspace: false,
    needsRustCrate: true,
    phases: function phases({ testBinaryPath, }: { readonly testBinaryPath: string; },): readonly TestPhase[] {
      return [
        {
          id: 'protocols',
          caller: 'root',
          workingDirectory: EXEMPT_PACKAGE,
          environment: {},
          words: [
            testBinaryPath,
            '--exact',
            'pin_tests::all_protocol_hooks_mark_and_drop_cleanly',
            '--ignored',
            '--test-threads=1',
            '--nocapture',
          ],
        },
      ];
    },
  },
};
