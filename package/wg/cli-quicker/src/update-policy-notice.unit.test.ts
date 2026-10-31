import { readFile, } from 'node:fs/promises';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { UPDATE_HOSTS, } from '../dist/final/node/update-hosts.data.mjs';
import {
  POLICY_NOT_IN_FORCE_MARKER,
  POLICY_PROCESS_WATCH,
  POLICY_UPDATE_HOSTS,
  policiesNotInForceWithoutMark,
  policyNotInForce,
  startNotices,
  updatePolicyNotice,
} from '../dist/final/node/update-policy-notice.mjs';

/**
 Directory of the Rust companion's sources.
 */
const COMPANION_SOURCE_DIRECTORY = join(import.meta.dirname, '..', '..', 'cli-quicker-exempt', 'src',);

/**
 Config facts the start notices depend on.
 */
type Facts = Parameters<typeof startNotices>[0];

/**
 Builds the facts of a start.

 @param exemptMark - `ExemptMark` of the config,
 absent when none is set.

 @param tableOff - Whether the config says `Table = off`.

 @param dnsServers - `DNS =` servers of the config.

 @returns Facts for `startNotices`.

 @example
 ```ts
 facts({ exemptMark: 100, tableOff: false, dnsServers: ['1.1.1.1'] });
 ```
 */
function facts(
  {
    exemptMark,
    tableOff,
    dnsServers,
  }: {
    readonly exemptMark?: number;
    readonly tableOff: boolean;
    readonly dnsServers: readonly string[];
  },
): Facts {
  return {
    target: 'wg0',
    ...(exemptMark === undefined ? {} : { exemptMark, }),
    tableOff,
    dnsServers,
    hosts: UPDATE_HOSTS,
    recommendedMark: 100,
  };
}

/**
 Config facts of a normal start with a mark and one DNS server.
 */
const NORMAL_START: Facts = facts({ exemptMark: 100, tableOff: false, dnsServers: ['1.1.1.1',], },);

/**
 Reads the Rust source text that defines the words of the process policy.

 @returns Text of the Rust report module.

 @example
 ```ts
 await readCompanionReport();
 ```
 */
async function readCompanionReport(): Promise<string> {
  return await readFile(join(COMPANION_SOURCE_DIRECTORY, 'update_report.rs',), 'utf8',);
}

await describe({
  name: updatePolicyNotice.name,
  children: [
    it({
      name: 'names the traffic that leaves, every update host, the reason, and that nothing turns it off',
      fn: async () => {
        /**
         Notice printed on every start.
         */
        const notice = updatePolicyNotice({ hosts: UPDATE_HOSTS, },);
        for (
          const phrase of [
            'POLICY NOTICE',
            'without any switch',
            'OUTSIDE the WireGuard tunnel',
            'Update entry points',
            'mpm-shadow install, update and outdated',
            'Update hosts, for every program including browsers',
            'Why:',
            'must not use this tool',
          ]
        )
          expect(notice,).toContain(phrase,);
        for (const host of UPDATE_HOSTS)
          expect(notice,).toContain(host,);
      },
    },),
    it({
      name: 'says that the toolchain root rule covers every installed program and not only updates',
      fn: async () => {
        /**
         Notice printed on every start.
         */
        const notice = updatePolicyNotice({ hosts: UPDATE_HOSTS, },);
        for (
          const phrase of [
            'Everything the toolchain installed, not only its updates',
            'every script and dev server run by the toolchain\'s node',
            'node, bun, pnpm, uv, zig, cmake, rust, caddy, sops and age',
          ]
        )
          expect(notice,).toContain(phrase,);
      },
    },),
    it({
      name: 'says that a rule carries a whole shared address, with measured examples',
      fn: async () => {
        /**
         Notice printed on every start.
         */
        const notice = updatePolicyNotice({ hosts: UPDATE_HOSTS, },);
        for (
          const phrase of [
            'whole addresses',
            'a shared CDN address sends every other site on it outside the tunnel too',
            'docs.python.org shares all its addresses with pypi.org',
            'avatars.githubusercontent.com share half of theirs with the GitHub release-asset hosts',
          ]
        )
          expect(notice,).toContain(phrase,);
      },
    },),
    it({
      name: 'names every match that can be imitated and what is not covered',
      fn: async () => {
        /**
         Notice printed on every start.
         */
        const notice = updatePolicyNotice({ hosts: UPDATE_HOSTS, },);
        for (
          const phrase of [
            'what can imitate it',
            'the executable\'s file name (mise, mpm-shadow, mpm, node)',
            'the tail of a toolchain script path',
            'any executable named mpm',
            'inside a home directory the user can write',
            'Any program the user can run from such a path, or under such a name, leaves outside the tunnel too',
            'another pid namespace',
          ]
        )
          expect(notice,).toContain(phrase,);
      },
    },),
    it({
      name: 'states the same imitation and coverage facts as the watcher notice',
      fn: async () => {
        /**
         Rust source holding the watcher notice.
         */
        const companion = await readCompanionReport();
        /**
         Notice printed by `up`.
         */
        const notice = updatePolicyNotice({ hosts: UPDATE_HOSTS, },);
        for (
          const phrase of [
            'the executable\'s file name (mise, mpm-shadow, mpm, node)',
            'any executable named mpm',
            'another pid namespace',
            'shared CDN address',
            'without any switch',
          ]
        ) {
          expect(notice,).toContain(phrase,);
          expect(companion,).toContain(phrase,);
        }
      },
    },),
  ],
},);

await describe({
  name: startNotices.name,
  children: [
    it({
      name: 'prints the notice of what leaves the tunnel for a start with a mark and a DNS server, and nothing else',
      fn: async () => {
        /**
         Lines the start prints.
         */
        const notices = startNotices(NORMAL_START,);
        expect(notices,).toEqual([updatePolicyNotice({ hosts: UPDATE_HOSTS, },),],);
      },
    },),
    it({
      name: 'adds the loud report that no host rule will exist when the config sets no DNS server',
      fn: async () => {
        /**
         Lines a start without DNS prints.
         */
        const notices = startNotices(facts({ exemptMark: 100, tableOff: false, dnsServers: [], },),);
        expect(notices,).toHaveLength(2,);
        expect(notices[0],).toBe(updatePolicyNotice({ hosts: UPDATE_HOSTS, },),);
        expect(notices[1],).toContain(POLICY_NOT_IN_FORCE_MARKER,);
        expect(notices[1],).toContain(POLICY_UPDATE_HOSTS,);
        expect(notices[1],).toContain('sets no DNS = server',);
        expect(notices[1],).toContain('no update-host rule will be installed',);
        expect(notices[1],).toContain('The process watch and the toolchain paths still run',);
      },
    },),
    it({
      name: 'reports both policies as not in force, and says traffic uses the tunnel, for a config without a mark',
      fn: async () => {
        /**
         Lines a start without a mark prints.
         */
        const notices = startNotices(facts({ tableOff: false, dnsServers: ['1.1.1.1',], },),);
        expect(notices,).toHaveLength(1,);
        expect(notices[0],).toContain('will use the tunnel.',);
        expect(notices[0],).toContain(policiesNotInForceWithoutMark(),);
        expect(notices[0],).toContain('ExemptMark = 100',);
        expect(notices[0],).not.toContain('leaves OUTSIDE',);
      },
    },),
    it({
      name: 'does not claim that traffic leaves the tunnel when the config says Table = off',
      fn: async () => {
        /**
         Lines a start with Table = off prints.
         */
        const notices = startNotices(facts({ exemptMark: 100, tableOff: true, dnsServers: ['1.1.1.1',], },),);
        expect(notices,).toHaveLength(1,);
        expect(notices[0],).toContain(POLICY_NOT_IN_FORCE_MARKER,);
        expect(notices[0],).toContain(POLICY_PROCESS_WATCH,);
        expect(notices[0],).toContain(POLICY_UPDATE_HOSTS,);
        expect(notices[0],).toContain('Table = off',);
        expect(notices[0],).toContain('installs no routing rule and no bypass table',);
        expect(notices[0],).not.toContain('leaves OUTSIDE',);
      },
    },),
    it({
      name: 'prints something on every combination of config facts, so no start is silent',
      fn: async () => {
        for (const marked of [false, true,]) {
          for (const tableOff of [false, true,]) {
            for (const dnsServers of [[], ['1.1.1.1',],]) {
              expect(startNotices(facts({
                ...(marked ? { exemptMark: 100, } : {}),
                tableOff,
                dnsServers,
              },),).length,).toBeGreaterThan(0,);
            }
          }
        }
      },
    },),
  ],
},);

await describe({
  name: policiesNotInForceWithoutMark.name,
  children: [
    it({
      name: 'names both policies and the reason they cannot be enforced',
      fn: async () => {
        /**
         Report for a start without an exemption mark.
         */
        const report = policiesNotInForceWithoutMark();
        expect(report,).toContain('POLICY NOT IN FORCE',);
        expect(report,).toContain(POLICY_PROCESS_WATCH,);
        expect(report,).toContain(POLICY_UPDATE_HOSTS,);
        expect(report,).toContain('No ExemptMark is configured',);
        expect(policyNotInForce({ policy: POLICY_UPDATE_HOSTS, detail: 'DNS refresh failed', },),).toBe(
          `POLICY NOT IN FORCE: ${POLICY_UPDATE_HOSTS}: DNS refresh failed`,
        );
      },
    },),
  ],
},);

await describe({
  name: 'policy words shared with wg-quicker-exempt',
  children: [
    it({
      name: 'uses the same policy name and the same loud marker in the TypeScript and Rust packages',
      fn: async () => {
        /**
         Rust source holding the policy words.
         */
        const companion = await readCompanionReport();
        expect(companion,).toContain(`pub const POLICY_PROCESS_WATCH: &str = "${POLICY_PROCESS_WATCH}";`,);
        expect(companion,).toContain(`pub const POLICY_NOT_IN_FORCE_MARKER: &str = "${POLICY_NOT_IN_FORCE_MARKER}";`,);
      },
    },),
  ],
},);
