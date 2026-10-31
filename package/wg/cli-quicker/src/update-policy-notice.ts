/**
 Words that mark every loud report of a policy that is not in force,
 in this package and in `wg-quicker-exempt`.
 */
export const POLICY_NOT_IN_FORCE_MARKER = 'POLICY NOT IN FORCE';

/**
 Name of the update-host policy,
 used in every report about it.
 */
export const POLICY_UPDATE_HOSTS = 'update hosts: traffic to the update-host list stays outside the tunnel for every program';

/**
 Name of the process policy,
 which `wg-quicker-exempt` enforces and reports under the same name.
 A test compares this text with the Rust constant.
 */
export const POLICY_PROCESS_WATCH = 'process watch: update entry points and toolchain install roots stay outside the tunnel';

/**
 Builds the notice printed on every start that sets up the tunnel or the watcher.

 The notice names exactly which traffic leaves outside the tunnel and why,
 what can imitate a match,
 and what the rules cannot tell apart,
 so anyone running `wg-quicker` knows before traffic flows.
 There is no switch to turn any of it off.

 @param hosts - The update-host list.

 @returns Multi-line notice.

 @example
 ```ts
 updatePolicyNotice({ hosts: ['crates.io',] });
 ```
 */
export function updatePolicyNotice({ hosts, }: { readonly hosts: readonly string[]; },): string {
  return [
    'POLICY NOTICE: by this repository\'s policy, and without any switch to turn it off,',
    'the following traffic leaves OUTSIDE the WireGuard tunnel while ExemptMark is configured:',
    '  1. Update entry points: mise toolchain tasks that fetch, mpm-shadow install, update and outdated, the mpm shadow,',
    '     and the TypeScript toolchain sync and update scripts, together with every process they start.',
    '  2. Everything the toolchain installed, not only its updates: every process whose executable is under the toolchain',
    '     and toolchain-global directories of the monochromatic-dev data directory, together with every process it starts.',
    '     That is every script and dev server run by the toolchain\'s node, every build by its compilers,',
    '     and everything they fetch. The installed tools include node, bun, pnpm, uv, zig, cmake, rust, caddy, sops and age.',
    '  3. Update hosts, for every program including browsers, as whole addresses:',
    `     ${hosts.join(', ',)}.`,
    '     A rule carries an address and not a site, so a shared CDN address sends every other site on it outside the tunnel too.',
    '     For example docs.python.org shares all its addresses with pypi.org,',
    '     and raw.githubusercontent.com and avatars.githubusercontent.com share half of theirs with the GitHub release-asset hosts.',
    '     Host rules use only answers from the tunnel\'s own DNS = servers, asked through the tunnel;',
    '     the tunnel\'s DNS servers and peer endpoints never get a rule.',
    'How a process is recognized, and what can imitate it: the executable\'s file name (mise, mpm-shadow, mpm, node)',
    'together with its command-line words, the tail of a toolchain script path, any executable named mpm,',
    'and any executable under the install roots, which are inside a home directory the user can write.',
    'Any program the user can run from such a path, or under such a name, leaves outside the tunnel too.',
    'Not covered: processes in another pid namespace (containers and sandboxes), and processes the user\'s service manager',
    'cannot move (login sessions and the main process of a service); the watcher reports each one when it matches.',
    'Why: the repository keeps toolchain updates on the physical network so they are fast and do not depend on a tunnel exit.',
    'Anyone who must not reach these hosts or run these programs outside the tunnel must not use this tool.',
  ].join('\n',);
}

/**
 Notice for a start where no exemption mark is configured,
 so none of the update policies can be enforced.

 @returns One-line report naming both policies.

 @example
 ```ts
 policiesNotInForceWithoutMark();
 ```
 */
export function policiesNotInForceWithoutMark(): string {
  return `${POLICY_NOT_IN_FORCE_MARKER}: ${POLICY_PROCESS_WATCH}; ${POLICY_UPDATE_HOSTS}. `
    + 'No ExemptMark is configured, so the bypass table that carries them does not exist and update traffic uses the tunnel.';
}

/**
 Report for a start whose config says `Table = off`.

 With `Table = off` wg-quicker installs no routing rule and no bypass table,
 so no update traffic is routed around the tunnel by it,
 and no traffic is routed into the tunnel by it either.

 @returns One-line report naming both policies.

 @example
 ```ts
 policiesNotInForceWithTableOff();
 ```
 */
export function policiesNotInForceWithTableOff(): string {
  return `${POLICY_NOT_IN_FORCE_MARKER}: ${POLICY_PROCESS_WATCH}; ${POLICY_UPDATE_HOSTS}. `
    + 'The config says Table = off, so wg-quicker installs no routing rule and no bypass table at all: '
    + 'it routes no update traffic around the tunnel, and where any traffic goes is decided by your own routing.';
}

/**
 Builds the loud report for a policy that cannot be enforced.

 @param policy - Policy name.

 @param detail - What failed and what that leaves unenforced.

 @returns Single-line report.

 @example
 ```ts
 policyNotInForce({ policy: POLICY_UPDATE_HOSTS, detail: 'DNS refresh failed' });
 ```
 */
export function policyNotInForce(
  {
    policy,
    detail,
  }: {
    readonly policy: string;
    readonly detail: string;
  },
): string {
  return `${POLICY_NOT_IN_FORCE_MARKER}: ${policy}: ${detail}`;
}

/**
 What decides which notices a start prints.
 */
export type StartNoticeInput = {
  /**
   Interface name or config path the user gave.
   */
  readonly target: string;

  /**
   `ExemptMark` of the config,
   absent when none is configured.
   */
  readonly exemptMark?: number;

  /**
   Whether the config says `Table = off`.
   */
  readonly tableOff: boolean;

  /**
   `DNS =` servers of the config that are IP literals.
   */
  readonly dnsServers: readonly string[];

  /**
   The update-host list.
   */
  readonly hosts: readonly string[];

  /**
   Mark the user should add to the config when none is set.
   */
  readonly recommendedMark: number;
};

/**
 Builds every line a start prints, in order.

 Every start prints something:
 the notice of what leaves outside the tunnel when the policies run,
 or the loud report naming the policy that cannot run and why.
 A config with no `DNS =` additionally gets the loud report that no host rule will exist.

 @param input - Config facts.

 @returns Lines for the warning log.

 @example
 ```ts
 startNotices({ target: 'wg0', exemptMark: 100, tableOff: false, dnsServers: ['1.1.1.1'], hosts: [], recommendedMark: 100 });
 ```
 */
export function startNotices(input: StartNoticeInput,): readonly string[] {
  if (input.exemptMark === undefined)
    return [
      `Config for ${input.target} has no ExemptMark; Ghostty, Steam, Helium, Pale Moon, Firefox Nightly, ChatGPT, Interpreter, and Qure will use the tunnel. `
        + `So will toolchain sync and update runs. ${policiesNotInForceWithoutMark()} `
        + `Add \`ExemptMark = ${String(input.recommendedMark,)}\` under \`[Interface]\`, then bring this interface down and up again so application exemptions attach.`,
    ];
  if (input.tableOff)
    return [policiesNotInForceWithTableOff(),];
  /**
   Notices every start with an exemption mark prints.
   */
  const notices = [updatePolicyNotice({ hosts: input.hosts, },),];
  if (input.dnsServers
    .length
    === 0)
    notices.push(policyNotInForce({
      policy: POLICY_UPDATE_HOSTS,
      detail: `the config for ${input.target} sets no DNS = server, so no update-host rule will be installed and traffic to the update hosts uses the tunnel: `
        + 'host rules use only answers from the tunnel\'s own DNS = servers, asked through the tunnel. '
        + 'The process watch and the toolchain paths still run.',
    },),);
  return notices;
}
