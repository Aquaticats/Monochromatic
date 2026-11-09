import {
  chmod,
  mkdtemp,
  readFile,
  rm,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';

import {
  escapeMarkup,
  sanitizeText,
} from '../dist/final/node/escape-text.mjs';
import {
  createRepeatLimit,
  MAX_CAUSES_PER_KIND,
  MAX_CAUSE_CHARS,
  MAX_EXTRA_CAUSES_PER_INTERVAL,
  MAX_REMEMBERED_KINDS,
  normalizeCause,
  withHeldRepeats,
} from '../dist/final/node/failure-repeat-limit.mjs';
import { resolveNotificationUser, } from '../dist/final/node/notification-user.mjs';
import {
  POLICY_CLEARNET_HOSTS,
  policyNotInForce,
} from '../dist/final/node/update-policy-notice.mjs';
import {
  createFailureDelivery,
  FAILURE_JOURNAL_INTERVAL_MS,
  FAILURE_NOTIFICATION_INTERVAL_MS,
  journalCommand,
  NOTIFICATION_EXPIRY_MS,
  notificationArguments,
  notificationCommand,
  parseNotificationId,
  runDeliveryCommandWithin,
  SECURE_PATH,
  type DeliveryCommand,
} from '../dist/final/node/runtime-failure-report.mjs';

/**
 A user to notify in the delivery tests.
 */
const USER = { uid: 1_000, gid: 1_001, } as const;

/**
 Builds a delivery over a fake runner and a fake clock.

 @param lookupFound - Whether a user to notify exists.

 @param failing - Command names whose run rejects.

 @returns The delivery,
 the commands it ran,
 the channel failures it reported,
 and a setter for the clock.

 @example
 ```ts
 const fixture = delivery({ lookupFound: true, failing: [] });
 ```
 */
function delivery(
  {
    lookupFound,
    failing,
  }: {
    readonly lookupFound: boolean;
    readonly failing: readonly string[];
  },
) {
  /**
   Commands the fake runner ran.
   */
  const ran: DeliveryCommand[] = [];
  /**
   Channel failures reported.
   */
  const problems: string[] = [];
  /**
   Clock value in milliseconds.
   */
  const clock = { now: 0, };
  return {
    ran,
    problems,
    clock,
    deliver: createFailureDelivery({
      lookup: lookupFound
        ? { found: true, user: USER, }
        : { found: false, reason: 'no user to notify: SUDO_UID is not set', },
      run: (call,) => {
        ran.push(call,);
        if (failing.includes(call.command,))
          return Promise.reject(new Error(`${call.command} refused`,),);
        return Promise.resolve(call.command === 'setpriv' ? `u ${String(100 + ran.length,)}\n` : '',);
      },
      now: () => clock.now,
      asRoot: true,
      reportChannelFailure: (message,) => {
        problems.push(message,);
      },
    },),
  };
}

await describe({
  name: resolveNotificationUser.name,
  children: [
    it({
      name: 'names the sudo caller and refuses a missing or malformed id, saying which one',
      fn: async () => {
        expect(resolveNotificationUser({ environment: { SUDO_UID: '1000', SUDO_GID: '1001', }, },),).toEqual({
          found: true,
          user: { uid: 1_000, gid: 1_001, },
        },);
        expect(resolveNotificationUser({ environment: {}, },),).toEqual({
          found: false,
          reason: 'no user to notify: SUDO_UID is not set',
        },);
        expect(resolveNotificationUser({ environment: { SUDO_UID: '1000', }, },),).toEqual({
          found: false,
          reason: 'no user to notify: SUDO_GID is not set',
        },);
        expect(resolveNotificationUser({ environment: { SUDO_UID: '1e3', SUDO_GID: '1', }, },),).toEqual({
          found: false,
          reason: 'no user to notify: SUDO_UID is not a decimal id: 1e3',
        },);
        expect(resolveNotificationUser({ environment: { SUDO_UID: '1', SUDO_GID: '', }, },),).toEqual({
          found: false,
          reason: 'no user to notify: SUDO_GID is not a decimal id: ',
        },);
      },
    },),
  ],
},);

await describe({
  name: 'delivery commands',
  children: [
    it({
      name: 'write the journal line at error level under the wg-quicker identifier',
      fn: async () => {
        expect(journalCommand({ line: 'POLICY NOT IN FORCE: x', },),).toEqual({
          command: 'systemd-cat',
          args: ['--identifier=wg-quicker', '--priority=err',],
          input: 'POLICY NOT IN FORCE: x\n',
        },);
      },
    },),
    it({
      name: 'drop root to the user with a cleared environment, and call the client directly when not root',
      fn: async () => {
        /**
         Command of a root watcher.
         */
        const root = notificationCommand({ user: USER, line: 'text', asRoot: true, },);
        expect(root.command,).toBe('setpriv',);
        expect(root.args.slice(0, 7,),).toEqual([
          '--reuid',
          '1000',
          '--regid',
          '1001',
          '--clear-groups',
          '--no-new-privs',
          'env',
        ],);
        expect(root.args.slice(7, 10,),).toEqual([
          '--ignore-environment',
          `PATH=${SECURE_PATH}`,
          'XDG_RUNTIME_DIR=/run/user/1000',
        ],);
        expect(root.args[10],).toBe('busctl',);
        expect(root.args.slice(11,),).toEqual(notificationArguments({
          summary: 'wg-quicker: update traffic policy not in force',
          body: 'text',
          replacesId: 0,
          expiryMs: NOTIFICATION_EXPIRY_MS,
        },),);
        /**
         Command of a watcher that already runs as the user.
         */
        const direct = notificationCommand({ user: USER, line: 'text', asRoot: false, },);
        expect(direct.command,).toBe('env',);
        expect(direct.args.slice(0, 3,),).toEqual(['--ignore-environment', `PATH=${SECURE_PATH}`, 'XDG_RUNTIME_DIR=/run/user/1000',],);
      },
    },),
    it({
      name: 'follow the Notify signature, replace an earlier notification by id, expire, and escape the body markup',
      fn: async () => {
        /**
         Arguments for one notification.
         */
        const args = notificationArguments({
          summary: 'summary text',
          body: 'body <b>text</b> & more',
          replacesId: 42,
          expiryMs: 600_000,
        },);
        expect(args.slice(0, 2,),).toEqual(['--user', 'call',],);
        expect(args[2],).toBe('org.freedesktop.Notifications',);
        expect(args[5],).toBe('Notify',);
        expect(args[6],).toBe('susssasa{sv}i',);
        expect(args.slice(7,),).toEqual([
          'wg-quicker',
          '42',
          'dialog-warning',
          'summary text',
          'body &lt;b&gt;text&lt;/b&gt; &amp; more',
          '0',
          '0',
          '600000',
        ],);
        expect(NOTIFICATION_EXPIRY_MS,).toBe(FAILURE_NOTIFICATION_INTERVAL_MS,);
      },
    },),
    it({
      name: 'neither the journal line nor the notification body can carry a control character from a user chosen name',
      fn: async () => {
        /**
         A host answer or command message that tries to repaint a terminal and start a second line.
         */
        const hostile = 'name\u001B[2J\u001B]0;pwned\u0007\nPOLICY NOT IN FORCE: fake\u202E';
        /**
         The journal input.
         */
        const journal = journalCommand({ line: hostile, },).input ?? '';
        expect(journal.slice(0, -1,),).toBe(sanitizeText({ text: journal.slice(0, -1,), },),);
        expect(journal,).not.toContain('\u001B',);
        expect(journal,).not.toContain('\u202E',);
        expect(journal,).toContain(String.raw`\u{001b}[2J`,);
        expect(journal.split('\n',),).toHaveLength(2,);
        /**
         The notification body.
         */
        const body = notificationArguments({ summary: 's', body: hostile, replacesId: 0, expiryMs: 1, },)[11] ?? '';
        expect(body,).toBe(escapeMarkup({ text: sanitizeText({ text: body, },), },),);
        expect(body,).not.toContain('\n',);
        expect(body,).not.toContain('\u001B',);
        expect(body,).toContain(String.raw`\u{000a}`,);
      },
    },),
    it({
      name: 'read the notification id out of what the client printed',
      fn: async () => {
        expect(parseNotificationId({ output: 'u 42\n', },),).toBe(42,);
        expect(parseNotificationId({ output: 'u 7', },),).toBe(7,);
        expect(parseNotificationId({ output: 's "x"', },),).toBe(0,);
        expect(parseNotificationId({ output: '', },),).toBe(0,);
        expect(parseNotificationId({ output: 'u many', },),).toBe(0,);
        expect(parseNotificationId({ output: 'u -3', },),).toBe(0,);
      },
    },),
  ],
},);

await describe({
  name: createRepeatLimit.name,
  children: [
    it({
      name: 'holds back repeats inside the interval, counts them, and reports the count at the next delivery',
      fn: async () => {
        /**
         Limit of one minute.
         */
        const limit = createRepeatLimit({ intervalMs: 60_000, },);
        expect(limit({ key: 'a', cause: 'c', now: 0, },),).toEqual({ deliver: true, held: 0, },);
        expect(limit({ key: 'a', cause: 'c', now: 10_000, },),).toEqual({ deliver: false, },);
        expect(limit({ key: 'a', cause: 'c', now: 59_999, },),).toEqual({ deliver: false, },);
        expect(limit({ key: 'b', cause: 'c', now: 59_999, },),).toEqual({ deliver: true, held: 0, },);
        expect(limit({ key: 'a', cause: 'c', now: 60_000, },),).toEqual({ deliver: true, held: 2, },);
        expect(limit({ key: 'a', cause: 'c', now: 120_000, },),).toEqual({ deliver: true, held: 0, },);
      },
    },),
    it({
      name: 'delivers the first distinct causes of a kind in each interval, holds back the rest and counts them',
      fn: async () => {
        /**
         Limit of one minute.
         */
        const limit = createRepeatLimit({ intervalMs: 60_000, },);
        /**
         First distinct causes of one kind, one per slot the limit delivers.
         */
        const causes = Array.from({ length: MAX_CAUSES_PER_KIND, }, function causeAt(_unused, index,) {
          return `cause ${String(index,)}`;
        },);
        for (const cause of causes)
          expect(limit({ key: 'skipped', cause, now: 0, },),).toEqual({ deliver: true, held: 0, },);
        expect(limit({ key: 'skipped', cause: 'epsilon', now: 1, },),).toEqual({ deliver: false, },);
        expect(limit({ key: 'skipped', cause: causes[0] ?? '', now: 2, },),).toEqual({ deliver: false, },);
        expect(limit({ key: 'skipped', cause: 'zeta', now: 3, },),).toEqual({ deliver: false, },);
        expect(limit({ key: 'skipped', cause: 'epsilon', now: 60_000, },),).toEqual({ deliver: true, held: 3, },);
      },
    },),
    it({
      name: 'shares a cap on extra causes over all kinds and always delivers the first cause of a kind',
      fn: async () => {
        /**
         Limit of one minute.
         */
        const limit = createRepeatLimit({ intervalMs: 60_000, },);
        /**
         Deliveries over twenty kinds of four causes each.
         */
        let delivered = 0;
        for (let kind = 0; kind < 20; kind += 1)
          for (const cause of ['a', 'b', 'c', 'd',])
            if (limit({ key: `kind ${String(kind,)}`, cause, now: 0, },).deliver)
              delivered += 1;
        expect(delivered,).toBe(20 + MAX_EXTRA_CAUSES_PER_INTERVAL,);
        expect(limit({ key: 'a new kind', cause: 'a', now: 1, },),).toEqual({ deliver: true, held: 0, },);
      },
    },),
    it({
      name: 'forgets expired kinds first when too many are remembered, so memory stays bounded',
      fn: async () => {
        /**
         Limit that remembers up to the bound.
         */
        const limit = createRepeatLimit({ intervalMs: 1_000, },);
        for (let index = 0; index < (MAX_REMEMBERED_KINDS * 2); index += 1)
          limit({ key: `kind ${String(index,)}`, cause: 'c', now: index, },);
        expect(limit({ key: 'kind 0', cause: 'c', now: 5_000, },),).toEqual({ deliver: true, held: 0, },);
      },
    },),
    it({
      name: 'forgets the oldest kind when none has expired, and still holds back the newest',
      fn: async () => {
        /**
         Limit whose interval outlasts the test.
         */
        const limit = createRepeatLimit({ intervalMs: 1_000_000, },);
        for (let index = 0; index < (MAX_REMEMBERED_KINDS + 10); index += 1)
          limit({ key: `kind ${String(index,)}`, cause: 'c', now: index, },);
        expect(limit({ key: 'kind 0', cause: 'c', now: 1_000, },),).toEqual({ deliver: true, held: 0, },);
        expect(limit({ key: `kind ${String(MAX_REMEMBERED_KINDS + 9,)}`, cause: 'c', now: 1_001, },),).toEqual({ deliver: false, },);
      },
    },),
  ],
},);

await describe({
  name: normalizeCause.name,
  children: [
    it({
      name: 'makes numbers alike, keeps reasons, and keeps the first characters only',
      fn: async () => {
        expect(normalizeCause({ line: 'pid 12 failed (attempt 3) in 30s', },),).toBe('pid # failed (attempt #) in #s',);
        expect(normalizeCause({ line: 'pid 12 failed', },),).toBe(normalizeCause({ line: 'pid 99999 failed', },),);
        expect(normalizeCause({ line: 'pid 12 failed: boom', },),).not.toBe(normalizeCause({ line: 'pid 12 failed: other', },),);
        expect(normalizeCause({ line: 'x'.repeat(5_000,), },),).toHaveLength(MAX_CAUSE_CHARS,);
        expect(normalizeCause({ line: 'word '.repeat(5_000,), },),).toHaveLength(MAX_CAUSE_CHARS,);
        expect(normalizeCause({ line: '', },),).toBe('',);
      },
    },),
    it({
      name: 'makes absolute paths and unit names, the names a user chose, alike and keeps their punctuation',
      fn: async () => {
        /**
         The same reason for two cgroups with different names.
         */
        const first = normalizeCause({ line: 'its cgroup /user.slice/user-1000.slice/user@1000.service/app.slice/app-a.scope is outside', },);
        const second = normalizeCause({ line: 'its cgroup /user.slice/user-1000.slice/user@1000.service/app.slice/app-completely-other.scope is outside', },);
        expect(first,).toBe(second,);
        expect(first,).toBe('its cgroup <name> is outside',);
        expect(normalizeCause({ line: 'the service app-x@1.service, and more', },),).toBe(normalizeCause({ line: 'the service app-other@22.service, and more', },),);
        expect(normalizeCause({ line: '(/a/b/c), then /d/e.', },),).toBe('(<name>), then <name>.',);
        expect(normalizeCause({ line: 'its cgroup /a is outside', },),).not.toBe(normalizeCause({ line: 'its cgroup /a is inside', },),);
        expect(normalizeCause({ line: 'a word that merely ends .service-like', },),).toBe('a word that merely ends .service-like',);
        expect(normalizeCause({ line: 'the .service suffix alone', },),).toBe('the .service suffix alone',);
      },
    },),
    it({
      name: 'keeps two different errors behind the fixed phrase of a loud line apart, and the same error for another pid together',
      fn: async () => {
        /**
         Loud lines for two different errors of one kind.
         */
        const denied = policyNotInForce({ policy: POLICY_CLEARNET_HOSTS, detail: '3 problem(s); traffic to the affected hosts uses the tunnel until the next refresh in 30 s: github.com answered Access denied', },);
        const exists = policyNotInForce({ policy: POLICY_CLEARNET_HOSTS, detail: '3 problem(s); traffic to the affected hosts uses the tunnel until the next refresh in 30 s: github.com answered Unit already exists', },);
        const other = policyNotInForce({ policy: POLICY_CLEARNET_HOSTS, detail: '7 problem(s); traffic to the affected hosts uses the tunnel until the next refresh in 45 s: github.com answered Access denied', },);
        expect(normalizeCause({ line: denied, },),).not.toBe(normalizeCause({ line: exists, },),);
        expect(normalizeCause({ line: denied, },),).toBe(normalizeCause({ line: other, },),);
        expect(denied.indexOf('github.com',),).toBeGreaterThan(100,);
      },
    },),
  ],
},);

await describe({
  name: 'held repeats and escaping',
  children: [
    it({
      name: 'word the held-back count',
      fn: async () => {
        expect(withHeldRepeats({ line: 'x', held: 0, },),).toBe('x',);
        expect(withHeldRepeats({ line: 'x', held: 3, },),).toBe('x (repeated 3 more time(s) since the last report)',);
      },
    },),
    it({
      name: 'escape control characters, line breaks and direction marks, and keep ordinary text',
      fn: async () => {
        expect(sanitizeText({ text: 'plain text, 100% fine: é 日本', },),).toBe('plain text, 100% fine: é 日本',);
        expect(sanitizeText({ text: 'a\nb', },),).toBe(String.raw`a\u{000a}b`,);
        expect(sanitizeText({ text: '\u001B[31m', },),).toBe(String.raw`\u{001b}[31m`,);
        expect(sanitizeText({ text: 'x\u202Ey', },),).toBe(String.raw`x\u{202e}y`,);
        expect(sanitizeText({ text: '\u0085', },),).toBe(String.raw`\u{0085}`,);
        expect(sanitizeText({ text: 'a\u007Fb', },),).toBe(String.raw`a\u{007f}b`,);
        expect(sanitizeText({ text: '\uFEFF', },),).toBe(String.raw`\u{feff}`,);
        expect(sanitizeText({ text: 'a\u2028b\u2029c', },),).toBe(String.raw`a\u{2028}b\u{2029}c`,);
        expect(sanitizeText({ text: 'tag\u{E0041}\u{E007F}x', },),).toBe(String.raw`tag\u{e0041}\u{e007f}x`,);
        expect(sanitizeText({ text: '\u{E0080}\u2027\u202F', },),).toBe('\u{E0080}\u2027\u202F',);
      },
    },),
    it({
      name: 'escape the markup characters of a notification body',
      fn: async () => {
        expect(escapeMarkup({ text: '<b>"a" & \'b\'</b>', },),).toBe('&lt;b&gt;&quot;a&quot; &amp; &apos;b&apos;&lt;/b&gt;',);
        expect(escapeMarkup({ text: 'plain', },),).toBe('plain',);
      },
    },),
  ],
},);

await describe({
  name: createFailureDelivery.name,
  children: [
    it({
      name: 'writes a new failure to the journal and the desktop',
      fn: async () => {
        const { deliver, ran, problems, } = delivery({ lookupFound: true, failing: [], },);
        await deliver({ kind: 'update-host problems', line: 'POLICY NOT IN FORCE: update hosts: x', },);
        expect(ran.map((call,) => call.command,),).toEqual(['systemd-cat', 'setpriv',],);
        expect(ran[0]?.input,).toBe('POLICY NOT IN FORCE: update hosts: x\n',);
        expect(problems,).toEqual([],);
      },
    },),
    it({
      name: 'limits a kind to the journal once a minute and the desktop once in ten minutes, and counts what it held back',
      fn: async () => {
        const { deliver, ran, clock, } = delivery({ lookupFound: true, failing: [], },);
        await deliver({ kind: 'refresh failed', line: 'refresh 1 failed: boom', },);
        clock.now = 30_000;
        await deliver({ kind: 'refresh failed', line: 'refresh 2 failed: boom', },);
        expect(ran,).toHaveLength(2,);
        clock.now = FAILURE_JOURNAL_INTERVAL_MS + 1;
        await deliver({ kind: 'refresh failed', line: 'refresh 3 failed: boom', },);
        expect(ran.map((call,) => call.command,),).toEqual(['systemd-cat', 'setpriv', 'systemd-cat',],);
        expect(ran[2]?.input,).toBe('refresh 3 failed: boom (repeated 1 more time(s) since the last report)\n',);
        clock.now = FAILURE_NOTIFICATION_INTERVAL_MS + 1;
        await deliver({ kind: 'refresh failed', line: 'refresh 4 failed: boom', },);
        expect(ran.map((call,) => call.command,),).toEqual(['systemd-cat', 'setpriv', 'systemd-cat', 'systemd-cat', 'setpriv',],);
        expect(ran[4]?.args.at(-5),).toBe('wg-quicker: update traffic policy not in force',);
        expect(ran[4]?.args.at(-4),).toBe('refresh 4 failed: boom (repeated 2 more time(s) since the last report)',);
      },
    },),
    it({
      name: 'delivers only the first distinct causes when names in the text differ, and delivers one when they differ only in digits',
      fn: async () => {
        const { deliver, ran, clock, } = delivery({ lookupFound: true, failing: [], },);
        await Promise.all(Array.from({ length: 200, }, (_unused, index,) => deliver({
          kind: 'update-host problems',
          line: `POLICY NOT IN FORCE: host-${String(index,)}.example failed ${'x'.repeat(index,)}`,
        },),),);
        expect(clock.now,).toBe(0,);
        expect(ran,).toHaveLength(2 * MAX_CAUSES_PER_KIND,);
        /**
         Deliveries whose lines differ only in digits.
         */
        const digits = delivery({ lookupFound: true, failing: [], },);
        await Promise.all(Array.from({ length: 200, }, (_unused, index,) => digits.deliver({
          kind: 'update-host problems',
          line: `POLICY NOT IN FORCE: host-${String(index,)}.example failed`,
        },),),);
        expect(digits.ran,).toHaveLength(2,);
      },
    },),
    it({
      name: 'delivers a different kind at once',
      fn: async () => {
        const { deliver, ran, clock, } = delivery({ lookupFound: true, failing: [], },);
        await deliver({ kind: 'route monitor failed', line: 'the route monitor failed', },);
        clock.now = 1;
        await deliver({ kind: 'update-host refresh failed', line: 'the update-host refresh failed', },);
        expect(ran,).toHaveLength(4,);
      },
    },),
    it({
      name: 'replaces the notification of the same kind by the id the client returned, and only that kind',
      fn: async () => {
        const { deliver, ran, clock, } = delivery({ lookupFound: true, failing: [], },);
        await deliver({ kind: 'a', line: 'a failed 1', },);
        await deliver({ kind: 'b', line: 'b failed 1', },);
        clock.now = FAILURE_NOTIFICATION_INTERVAL_MS + 1;
        await deliver({ kind: 'a', line: 'a failed 2', },);
        /**
         The notification calls in order.
         */
        const notifications = ran.filter((call,) => call.command === 'setpriv',);
        expect(notifications.map((call,) => call.args.at(-7,),),).toEqual(['0', '0', '102',],);
      },
    },),
    it({
      name: 'delivers distinct causes of one kind to both channels, and a notification replaces only the one of its own cause',
      fn: async () => {
        const { deliver, ran, clock, } = delivery({ lookupFound: true, failing: [], },);
        await deliver({ kind: 'move skipped', line: 'POLICY NOT IN FORCE: pid 11 is in a login session scope', },);
        await deliver({ kind: 'move skipped', line: 'POLICY NOT IN FORCE: pid 12 runs in another pid namespace', },);
        await deliver({ kind: 'move skipped', line: 'POLICY NOT IN FORCE: pid 13 is the main process of a service', },);
        expect(ran.map((call,) => call.command,),).toEqual(['systemd-cat', 'setpriv', 'systemd-cat', 'setpriv', 'systemd-cat', 'setpriv',],);
        await deliver({ kind: 'move skipped', line: 'POLICY NOT IN FORCE: pid 14 is in a login session scope', },);
        expect(ran,).toHaveLength(6,);
        clock.now = FAILURE_NOTIFICATION_INTERVAL_MS + 1;
        await deliver({ kind: 'move skipped', line: 'POLICY NOT IN FORCE: pid 15 is in a login session scope', },);
        await deliver({ kind: 'move skipped', line: 'POLICY NOT IN FORCE: pid 16 runs in another pid namespace', },);
        /**
         The notification calls in order,
         with the id each replaces.
         */
        const notifications = ran.filter((call,) => call.command === 'setpriv',);
        expect(notifications.map((call,) => call.args.at(-7,),),).toEqual(['0', '0', '0', '102', '104',],);
      },
    },),
    it({
      name: 'names a failing journal and still tries the desktop, and the reverse',
      fn: async () => {
        /**
         Delivery whose journal command fails.
         */
        const journalDown = delivery({ lookupFound: true, failing: ['systemd-cat',], },);
        await journalDown.deliver({ kind: 'k', line: 'POLICY NOT IN FORCE: x', },);
        expect(journalDown.ran.map((call,) => call.command,),).toEqual(['systemd-cat', 'setpriv',],);
        expect(journalDown.problems,).toHaveLength(1,);
        expect(journalDown.problems[0],).toContain('journal: Error: systemd-cat refused',);
        expect(journalDown.problems[0],).toContain('report was: POLICY NOT IN FORCE: x',);
        /**
         Delivery whose desktop command fails.
         */
        const desktopDown = delivery({ lookupFound: true, failing: ['setpriv',], },);
        await desktopDown.deliver({ kind: 'k', line: 'POLICY NOT IN FORCE: y', },);
        expect(desktopDown.problems[0],).toContain('desktop notification: Error: setpriv refused',);
      },
    },),
    it({
      name: 'journals a failure and names the missing user when there is nobody to notify',
      fn: async () => {
        const { deliver, ran, problems, } = delivery({ lookupFound: false, failing: [], },);
        await deliver({ kind: 'k', line: 'POLICY NOT IN FORCE: z', },);
        expect(ran.map((call,) => call.command,),).toEqual(['systemd-cat',],);
        expect(problems,).toHaveLength(1,);
        expect(problems[0],).toContain('desktop notification: no user to notify: SUDO_UID is not set',);
      },
    },),
  ],
},);

await describe({
  name: runDeliveryCommandWithin.name,
  children: [
    it({
      name: 'pipes the input, clears the environment, rejects a nonzero exit and kills a command past its timeout',
      fn: async () => {
        /**
         Scratch directory for the fake client.
         */
        const directory = await mkdtemp(join(scratchRoot(), 'wgv-run-',),);
        await using cleanup = {
          async [Symbol.asyncDispose](): Promise<void> {
            await rm(directory, { force: true, recursive: true, },);
          },
        };
        /**
         Files the fake client writes.
         */
        const stdinFile = join(directory, 'stdin',);
        const envFile = join(directory, 'env',);
        /**
         Fake client recording its standard in and environment.
         */
        const recorder = join(directory, 'recorder',);
        await writeFile(recorder, `#!/bin/sh\n/usr/bin/cat > '${stdinFile}'\n/usr/bin/env > '${envFile}'\necho 'u 42'\n`,);
        await chmod(recorder, 0o700,);
        expect(await runDeliveryCommandWithin({ call: { command: recorder, args: [], input: 'one line\n', }, timeoutMs: 5_000, },),).toBe('u 42\n',);
        expect(await readFile(stdinFile, 'utf8',),).toBe('one line\n',);
        /**
         Names in the environment of the child:
         the secure path we set,
         and the three variables `/bin/sh` adds by itself.
         */
        const names = (await readFile(envFile, 'utf8',)).trim()
          .split('\n',)
          .map((line,) => line.split('=',)
            .at(0,) ?? '',)
          .toSorted((left, right,) => left.localeCompare(right,),);
        expect(names,).toEqual(['_', 'PATH', 'PWD', 'SHLVL',],);
        expect(await readFile(envFile, 'utf8',),).toContain(`PATH=${SECURE_PATH}`,);
        /**
         Fake client that fails.
         */
        const failing = join(directory, 'failing',);
        await writeFile(failing, '#!/bin/sh\necho boom >&2\nexit 3\n',);
        await chmod(failing, 0o700,);
        await expect(runDeliveryCommandWithin({ call: { command: failing, args: [], }, timeoutMs: 5_000, },),).rejects.toThrow();
        /**
         Fake client that outlives its timeout.
         */
        const hanging = join(directory, 'hanging',);
        await writeFile(hanging, '#!/bin/sh\nexec /usr/bin/sleep 30\n',);
        await chmod(hanging, 0o700,);
        /**
         Moment the hanging run began.
         */
        const started = performance.now();
        await expect(runDeliveryCommandWithin({ call: { command: hanging, args: [], }, timeoutMs: 200, },),).rejects.toThrow();
        expect(performance.now() - started,).toBeLessThan(10_000,);
        await expect(runDeliveryCommandWithin({ call: { command: join(directory, 'absent',), args: [], }, timeoutMs: 200, },),).rejects.toThrow();
      },
    },),
  ],
},);
