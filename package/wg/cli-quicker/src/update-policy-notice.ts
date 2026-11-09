import { sanitizeText, } from './escape-text.ts';

/**
 Failure kind of a refresh of the update-host rules that had problems.
 */
export const KIND_UPDATE_HOST_PROBLEMS = 'update-host problems';

/**
 Failure kind of a refresh of the update-host rules that failed outright.
 */
export const KIND_UPDATE_HOST_REFRESH_FAILED = 'update-host refresh failed';

/**
 Failure kind of a route monitor that failed and will be replaced.
 */
export const KIND_ROUTE_MONITOR_FAILED = 'route monitor failed';

/**
 Failure kind of a route watcher that stopped for good.
 */
export const KIND_ROUTE_WATCHER_STOPPED = 'route watcher stopped';

/**
 Failure kind of a route watcher that died without being stopped.
 */
export const KIND_ROUTE_WATCHER_DIED = 'route watcher died';

/**
 Words that mark every loud report of a policy that is not in force,
 in this package and in `wg-quicker-exempt`.
 */
export const POLICY_NOT_IN_FORCE_MARKER = 'POLICY NOT IN FORCE';

/**
 Name of the host-rule policy,
 used in every report about it.
 One rule set carries two lists,
 kept apart in the notice because each leaves the tunnel for its own reason:
 the update hosts and the model provider hosts.
 */
export const POLICY_CLEARNET_HOSTS = 'clearnet hosts: traffic to the update-host and provider-host lists stays outside the tunnel for every program';

/**
 Name of the process policy,
 which `wg-quicker-exempt` enforces and reports under the same name.
 A test compares this text with the Rust constant.
 */
export const POLICY_PROCESS_WATCH = 'process watch: update entry points, toolchain install roots and Pi stay outside the tunnel';

/**
 Name of the socket-marking policy,
 which `wg-quicker-exempt` enforces and reports under the same name.
 A test checks that the Rust sources use this text.
 */
export const POLICY_SOCKET_MARKING = 'socket marking of application and update cgroups';

/**
 Name of the policy that keeps the bypass table equal to the physical default route,
 which the route watcher of this package enforces.
 */
export const POLICY_BYPASS_TABLE = 'bypass table: marked traffic follows the physical default route';

/**
 Builds the notice printed on every start that sets up the tunnel or the watcher.

 The notice names exactly which traffic leaves outside the tunnel and why,
 what can imitate a match,
 and what the rules cannot tell apart,
 so anyone running `wg-quicker` knows before traffic flows.
 There is no switch to turn any of it off.

 @param hosts - The update-host list.

 @param providerHosts - The host list of the model providers Pi reaches.

 @returns Multi-line notice.

 @example
 ```ts
 updatePolicyNotice({ hosts: ['crates.io',], providerHosts: ['api.deepseek.com',] });
 ```
 */
export function updatePolicyNotice(
  {
    hosts,
    providerHosts,
  }: {
    readonly hosts: readonly string[];
    readonly providerHosts: readonly string[];
  },
): string {
  return [
    'POLICY NOTICE: by this repository\'s policy, and without any switch to turn it off,',
    'the following traffic leaves OUTSIDE the WireGuard tunnel while ExemptMark is configured:',
    '  1. Update entry points: mise toolchain tasks that fetch, mpm-shadow install, update and outdated, the mpm shadow,',
    '     and the TypeScript mpm-shadow launcher, together with every process they start.',
    '  2. Everything the toolchain installed, not only its updates: every process whose executable is under the toolchain',
    '     and toolchain-global directories of the monochromatic-dev data directory, together with every process it starts.',
    '     That is every script and dev server run by the toolchain\'s node, every build by its compilers,',
    '     and everything they fetch. The start notice of wg-quicker-exempt lists the tools installed on this machine now.',
    '  3. Pi: every process of a pi-<name>.service agent unit, and every node process whose first command-line word is pi,',
    '     together with every process they start. A unit is not moved: the user\'s service manager cannot move a service\'s',
    '     main process, so the socket mark the watcher attaches to that cgroup covers it and every process below it,',
    '     which is the same rule the terminals have.',
    '  4. Update hosts, for every program including browsers, as whole addresses:',
    `     ${hosts.join(', ',)}.`,
    '     A rule carries an address and not a site, so a shared CDN address sends every other site on it outside the tunnel too.',
    '     For example docs.python.org shares all its addresses with pypi.org,',
    '     and raw.githubusercontent.com and avatars.githubusercontent.com share half of theirs with the GitHub release-asset hosts.',
    '     Host rules use only answers from the tunnel\'s own DNS = servers, asked through the tunnel;',
    '     the tunnel\'s DNS servers and peer endpoints never get a rule.',
    '  5. Model provider hosts, for every program including browsers, as whole addresses:',
    `     ${providerHosts.join(', ',)}.`,
    '     These are the API hosts of Pi\'s deepseek, neuralwatt, opencode-go, qwen token plan and synthetic providers,',
    '     apart from the update hosts because they leave the tunnel for another reason: they carry the agents\' requests.',
    '     Their rules are installed with the update-host rules, from the same tunnel DNS answers, and carry whole addresses',
    '     the same way, so a shared CDN address serves those providers and every other site on it outside the tunnel.',
    'How a process is recognized, and what can imitate it: the executable\'s file name (mise, mpm-shadow, mpm, node)',
    'together with its command-line words, the tail of a toolchain script path, any executable named mpm,',
    'and any executable under the install roots, which are inside a home directory the user can write.',
    'Any program the user can run from such a path, or under such a name, leaves outside the tunnel too.',
    'Pi is matched by the first command-line word it sets for itself, together with the node file name,',
    'so `exec -a pi node` imitates it, and by the name of the unit it runs in, which is a name the user can choose.',
    'A toolchain folder that is a link is ignored: links are not followed, wherever they point,',
    'and each such root is reported loudly as not in force every time the watcher starts.',
    'Not covered: processes in another pid namespace (containers and sandboxes), processes of a login session,',
    'and the main process of a service whose cgroup the watcher does not mark; the watcher reports each one when it matches.',
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
  return `${POLICY_NOT_IN_FORCE_MARKER}: ${POLICY_PROCESS_WATCH}; ${POLICY_CLEARNET_HOSTS}. `
    + 'No ExemptMark is configured, so the bypass table that carries them does not exist and update and provider traffic uses the tunnel.';
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
  return `${POLICY_NOT_IN_FORCE_MARKER}: ${POLICY_PROCESS_WATCH}; ${POLICY_CLEARNET_HOSTS}. `
    + 'The config says Table = off, so wg-quicker installs no routing rule and no bypass table at all: '
    + 'it routes no update or provider traffic around the tunnel, and where any traffic goes is decided by your own routing.';
}

/**
 Builds the message of the error that stops a start whose config sets `ExemptMark`
 when the Rust companion cannot be found.

 The companion runs the process watch and marks the sockets of application and update cgroups.
 The owner chose that such a start fails and rolls back:
 "`up` fails and rolls back, as before this branch."
 Only root-controlled locations are searched,
 and the caller's own `PATH` is never searched,
 because it could name a different program to run as root.

 @param searched - Every location that was looked at.

 @returns The message naming the places searched and how to install the program.

 @example
 ```ts
 watcherMissingAbortMessage({ searched: ['/usr/bin/wg-quicker-exempt'] });
 ```
 */
export function watcherMissingAbortMessage({ searched, }: { readonly searched: readonly string[]; },): string {
  /**
   What was looked at,
   in words.
   */
  const where = searched.length === 0 ? 'no location' : searched.join(', ',);
  return `wg-quicker-exempt was not found in any root-controlled location (searched: ${where}), `
    + 'and this config sets ExemptMark, so the process watch and the socket marking cannot run. '
    + 'The start stops here and is rolled back: the tunnel is not left up, and traffic goes direct. '
    + 'Build the program with `mise run //package/wg/cli-quicker-exempt:build` '
    + 'or install it where sudo\'s secure_path finds it, then run `wg-quicker up` again. '
    + 'The caller\'s own PATH is never searched, because it could name a different program to run as root. '
    + 'A config without ExemptMark does not need the program.';
}

/**
 Builds the report for a stop where the Rust companion cannot be found.

 @param searched - Every location that was looked at.

 @returns One report saying that a watcher an earlier run started may still run.

 @example
 ```ts
 watcherMissingAtStopReport({ searched: [] });
 ```
 */
export function watcherMissingAtStopReport({ searched, }: { readonly searched: readonly string[]; },): string {
  return `wg-quicker-exempt was not found in any root-controlled location (searched: ${searched.length === 0 ? 'no location' : searched.join(', ',)}), `
    + 'so this run cannot stop a watcher an earlier run may have started; '
    + 'if one still runs, stop it with `wg-quicker-exempt watch-stop <interface>` once the program is available';
}

/**
 Builds the loud report for a policy that cannot be enforced.

 @param policy - Policy name.

 @param detail - What failed and what that leaves unenforced.
 Names a user chose, such as a host answer or a command message, are escaped here,
 because the line reaches a log, the journal and a terminal.

 @returns Single-line report.

 @example
 ```ts
 policyNotInForce({ policy: POLICY_CLEARNET_HOSTS, detail: 'DNS refresh failed' });
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
  return `${POLICY_NOT_IN_FORCE_MARKER}: ${policy}: ${sanitizeText({ text: detail, },)}`;
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
   The host list of the model providers Pi reaches.
   */
  readonly providerHosts: readonly string[];

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
 startNotices({ target: 'wg0', exemptMark: 100, tableOff: false, dnsServers: ['1.1.1.1'], hosts: [], providerHosts: [], recommendedMark: 100 });
 ```
 */
export function startNotices(input: StartNoticeInput,): readonly string[] {
  if (input.exemptMark === undefined)
    return [
      `Config for ${input.target} has no ExemptMark; Ghostty, Steam, Helium, Pale Moon, Firefox, Firefox Nightly, ChatGPT, Interpreter, Qure, OdyTTY and Pi will use the tunnel. `
        + `So will toolchain update runs. ${policiesNotInForceWithoutMark()} `
        + `Add \`ExemptMark = ${String(input.recommendedMark,)}\` under \`[Interface]\`, then bring this interface down and up again so application exemptions attach.`,
    ];
  if (input.tableOff)
    return [policiesNotInForceWithTableOff(),];
  /**
   Notices every start with an exemption mark prints.
   */
  const notices = [updatePolicyNotice({
    hosts: input.hosts,
    providerHosts: input.providerHosts,
  },),];
  if (input.dnsServers
    .length
    === 0)
    notices.push(policyNotInForce({
      policy: POLICY_CLEARNET_HOSTS,
      detail: `the config for ${input.target} sets no DNS = server, so no host rule will be installed and traffic to the update hosts and the model provider hosts uses the tunnel: `
        + 'host rules use only answers from the tunnel\'s own DNS = servers, asked through the tunnel. '
        + 'The process watch, the toolchain paths and the Pi coverage still run.',
    },),);
  return notices;
}
