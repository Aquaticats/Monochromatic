import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  BLOCKING_TIMEOUT_ENABLED_ENV,
  BLOCKING_TIMEOUT_MS,
  canMoveToBackground,
  clampedTimeout,
  DISABLE_BACKGROUND_TASKS_ENV,
  disabledBackgroundTasksWarning,
  isBlockingTimeoutEnabled,
  isTimeoutPolicyCall,
  SLEEP_PREFIX,
  startsWithRedirection,
  startsWithSleepWord,
  withSleepPrefix,
} from './blocking-timeout.ts';

const DISABLED_INPUTS = ['0', 'false', 'off',] as const;
const ENABLED_INPUTS = [undefined, '', '1', 'FALSE', 'Off',] as const;

/**
 Caller timeout main sessions sent most often among long waits (58 minutes),
 per `doc/planning/claude-code-bash-caller-timeout-cap.md`.
 */
const OBSERVED_LONG_CALLER_TIMEOUT_MS = 3_480_000;

const KEPT_TIMEOUTS = [1, BLOCKING_TIMEOUT_MS / 2, BLOCKING_TIMEOUT_MS,] as const;
const CLAMPED_TIMEOUTS = [
  undefined,
  0,
  -BLOCKING_TIMEOUT_MS,
  Number.NaN,
  BLOCKING_TIMEOUT_MS + 1,
  OBSERVED_LONG_CALLER_TIMEOUT_MS,
] as const;

const LEADING_SLEEP_COMMANDS = [
  'sleep',
  'sleep 5',
  'sleep 5 && make',
  'sleep 5;printf x',
  'sleep&&x',
  'sleep|cat',
  'sleep>out',
  'sleep<in',
  'sleep\t2',
  '  sleep 5',
  '\nsleep 5',
  '# note\nsleep 5',
  '# a\n  # b\n\nsleep 1 > out',
  '\\\nsleep 5 && x',
  'sleep\r',
  'sleep(',
  'sleep)',
] as const;
const REDIRECTION_LED_COMMANDS = [
  '> log\nsleep 5 && x',
  '< in; sleep 5',
  '>> log',
  '2> err\nsleep 5',
  '&> all\nsleep 1',
  '  # note\n> log\nsleep 5',
] as const;
const NOT_REDIRECTION_LED_COMMANDS = [
  '',
  'make > log',
  'sleep 5',
  '2',
  'x>y',
  '&& sleep',
] as const;
const DISABLING_VALUES = ['1', 'true', 'TRUE', ' yes ', 'On',] as const;
const NON_DISABLING_VALUES = ['0', 'false', 'no', '', 'off',] as const;
const OTHER_COMMANDS = [
  '',
  'sleeper 5',
  'sleep5',
  'printf sleep',
  'X=1 sleep 5',
  'true && sleep 5',
  '(sleep 5)',
  '{ sleep 5; }',
  '# only a comment',
  '#sleep 5',
  '# note sleep 5',
] as const;

function policyCall(
  overrides: Readonly<{
    agentId?: string;
    env?: Readonly<NodeJS.ProcessEnv>;
    runInBackground?: boolean;
  }>,
): boolean {
  return isTimeoutPolicyCall({
    env: {},
    ...overrides,
  },);
}

await describe({
  name: '',
  children: [
    describe({
      name: 'blocking timeout constants',
      children: [
        it({
          name: 'names the enabled environment variable',
          fn: async () => {
            expect(BLOCKING_TIMEOUT_ENABLED_ENV,).toBe('MONOCHROMATIC_BASH_BLOCKING_TIMEOUT_ENABLED',);
          },
        },),
        it({
          name: 'names the native background-disable variable',
          fn: async () => {
            expect(DISABLE_BACKGROUND_TASKS_ENV,).toBe('CLAUDE_CODE_DISABLE_BACKGROUND_TASKS',);
          },
        },),
        it({
          name: 'uses the owner-selected one-second default',
          fn: async () => {
            expect(BLOCKING_TIMEOUT_MS,).toBe(1_000,);
          },
        },),
        it({
          name: 'prefixes with a no-op whose first word is not sleep',
          fn: async () => {
            expect(SLEEP_PREFIX,).toBe('true && ',);
            expect(startsWithSleepWord(SLEEP_PREFIX,),).toBe(false,);
          },
        },),
        it({
          name: 'warning names the conflict, remedies, and bare-mode limit',
          fn: async () => {
            expect(disabledBackgroundTasksWarning,).toContain(BLOCKING_TIMEOUT_ENABLED_ENV,);
            expect(disabledBackgroundTasksWarning,).toContain('stop instead of moving to the background',);
            expect(disabledBackgroundTasksWarning,).toContain('Bare mode',);
          },
        },),
      ],
    },),
    describe({
      name: isBlockingTimeoutEnabled.name,
      children: [
        ...DISABLED_INPUTS.map(function disabledValue(value,) {
          return it({
            name: `disables for exact lowercase ${value}`,
            fn: async () => {
              expect(isBlockingTimeoutEnabled({ [BLOCKING_TIMEOUT_ENABLED_ENV]: value, },),).toBe(false,);
            },
          },);
        },),
        ...ENABLED_INPUTS.map(function enabledValue(value,) {
          return it({
            name: `enables for ${value === undefined ? 'an unset variable' : JSON.stringify(value)}`,
            fn: async () => {
              expect(isBlockingTimeoutEnabled({ [BLOCKING_TIMEOUT_ENABLED_ENV]: value, },),).toBe(true,);
            },
          },);
        },),
      ],
    },),
    describe({
      name: isTimeoutPolicyCall.name,
      children: [
        it({
          name: 'covers a main-agent foreground call',
          fn: async () => {
            expect(policyCall({},),).toBe(true,);
          },
        },),
        it({
          name: 'covers an explicit foreground call',
          fn: async () => {
            expect(policyCall({ runInBackground: false, },),).toBe(true,);
          },
        },),
        it({
          name: 'excludes a native subagent',
          fn: async () => {
            expect(policyCall({ agentId: 'agent-1', },),).toBe(false,);
          },
        },),
        it({
          name: 'excludes an explicit background call',
          fn: async () => {
            expect(policyCall({ runInBackground: true, },),).toBe(false,);
          },
        },),
        it({
          name: 'excludes every call when the policy is disabled',
          fn: async () => {
            expect(policyCall({ env: { [BLOCKING_TIMEOUT_ENABLED_ENV]: 'off', }, },),).toBe(false,);
          },
        },),
        it({
          name: 'ignores the removed legacy variable',
          fn: async () => {
            expect(policyCall({ env: { MONOCHROMATIC_BASH_FORCE_BACKGROUND: 'off', }, },),).toBe(true,);
          },
        },),
      ],
    },),
    describe({
      name: canMoveToBackground.name,
      children: [
        it({
          name: 'allows moves when the variable is unset',
          fn: async () => {
            expect(canMoveToBackground({},),).toBe(true,);
          },
        },),
        ...NON_DISABLING_VALUES.map(function nonDisabling(value,) {
          return it({
            name: `allows moves for ${JSON.stringify(value,)}, which Claude Code reads as false`,
            fn: async () => {
              expect(canMoveToBackground({ [DISABLE_BACKGROUND_TASKS_ENV]: value, },),).toBe(true,);
            },
          },);
        },),
        ...DISABLING_VALUES.map(function disabling(value,) {
          return it({
            name: `refuses moves for ${JSON.stringify(value,)}, which Claude Code reads as true`,
            fn: async () => {
              expect(canMoveToBackground({ [DISABLE_BACKGROUND_TASKS_ENV]: value, },),).toBe(false,);
            },
          },);
        },),
      ],
    },),
    describe({
      name: clampedTimeout.name,
      children: [
        ...KEPT_TIMEOUTS.map(function keptTimeout(timeout,) {
          return it({
            name: `keeps caller timeout ${timeout}, which already moves no later than the default`,
            fn: async () => {
              expect(clampedTimeout({ env: {}, timeout, },),).toBe(timeout,);
            },
          },);
        },),
        ...CLAMPED_TIMEOUTS.map(function clamped(timeout,) {
          return it({
            name: `replaces caller timeout ${String(timeout,)} with the default`,
            fn: async () => {
              expect(clampedTimeout({ env: {}, ...(timeout === undefined ? {} : { timeout, }), },),)
                .toBe(BLOCKING_TIMEOUT_MS,);
            },
          },);
        },),
        it({
          name: 'keeps every caller timeout when native background tasks are disabled',
          fn: async () => {
            const env = { [DISABLE_BACKGROUND_TASKS_ENV]: '1', };
            expect(clampedTimeout({ env, timeout: OBSERVED_LONG_CALLER_TIMEOUT_MS, },),)
              .toBe(OBSERVED_LONG_CALLER_TIMEOUT_MS,);
            expect(clampedTimeout({ env, timeout: 0, },),).toBe(0,);
          },
        },),
        it({
          name: 'still supplies the default to an absent timeout when native background tasks are disabled',
          fn: async () => {
            expect(clampedTimeout({ env: { [DISABLE_BACKGROUND_TASKS_ENV]: '1', }, },),).toBe(BLOCKING_TIMEOUT_MS,);
          },
        },),
      ],
    },),
    describe({
      name: startsWithSleepWord.name,
      children: [
        ...LEADING_SLEEP_COMMANDS.map(function leadingSleep(command,) {
          return it({
            name: `sees a leading sleep in ${JSON.stringify(command,)}`,
            fn: async () => {
              expect(startsWithSleepWord(command,),).toBe(true,);
            },
          },);
        },),
        ...OTHER_COMMANDS.map(function otherCommand(command,) {
          return it({
            name: `sees no leading sleep in ${JSON.stringify(command,)}`,
            fn: async () => {
              expect(startsWithSleepWord(command,),).toBe(false,);
            },
          },);
        },),
      ],
    },),
    describe({
      name: startsWithRedirection.name,
      children: [
        ...REDIRECTION_LED_COMMANDS.map(function redirectionLed(command,) {
          return it({
            name: `sees a leading redirection in ${JSON.stringify(command,)}`,
            fn: async () => {
              expect(startsWithRedirection(command,),).toBe(true,);
            },
          },);
        },),
        ...NOT_REDIRECTION_LED_COMMANDS.map(function notRedirectionLed(command,) {
          return it({
            name: `sees no leading redirection in ${JSON.stringify(command,)}`,
            fn: async () => {
              expect(startsWithRedirection(command,),).toBe(false,);
            },
          },);
        },),
      ],
    },),
    describe({
      name: withSleepPrefix.name,
      children: [
        ...REDIRECTION_LED_COMMANDS.map(function prefixedRedirection(command,) {
          return it({
            name: `prefixes redirection-led ${JSON.stringify(command,)} and keeps its text`,
            fn: async () => {
              expect(withSleepPrefix(command,),).toBe(`${SLEEP_PREFIX}${command}`,);
            },
          },);
        },),
        ...LEADING_SLEEP_COMMANDS.map(function prefixed(command,) {
          return it({
            name: `prefixes ${JSON.stringify(command,)} and keeps its text`,
            fn: async () => {
              expect(withSleepPrefix(command,),).toBe(`${SLEEP_PREFIX}${command}`,);
            },
          },);
        },),
        ...OTHER_COMMANDS.map(function unprefixed(command,) {
          return it({
            name: `leaves ${JSON.stringify(command,)} unchanged`,
            fn: async () => {
              expect(withSleepPrefix(command,),).toBe(command,);
            },
          },);
        },),
        it({
          name: 'keeps quotes, operators, and newlines of the original text',
          fn: async () => {
            const command = 'sleep 1\nprintf \'a && b; c | d\' > "out file" 2>&1 &';
            expect(withSleepPrefix(command,),).toBe(`${SLEEP_PREFIX}${command}`,);
          },
        },),
        it({
          name: 'adds the prefix once',
          fn: async () => {
            const once = withSleepPrefix('sleep 5 > out',);
            expect(withSleepPrefix(once,),).toBe(once,);
          },
        },),
      ],
    },),
  ],
},);
