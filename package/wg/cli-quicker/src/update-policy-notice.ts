/**
 Name of the update-host policy,
 used in every report about it.
 */
export const POLICY_UPDATE_HOSTS = 'update hosts: traffic to the update-host list stays outside the tunnel for every program';

/**
 Name of the process policy,
 which `wg-quicker-exempt` enforces and reports under the same name.
 */
export const POLICY_PROCESS_WATCH = 'process watch: update entry points and toolchain install roots stay outside the tunnel';

/**
 Builds the notice printed on every start that sets up the tunnel or the watcher.

 The notice names exactly which traffic leaves outside the tunnel and why,
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
    '  2. Toolchain install roots: every process whose executable is under the toolchain and toolchain-global',
    '     directories of the monochromatic-dev data directory, together with every process it starts.',
    `  3. Update hosts, for every program including browsers: ${hosts.join(', ',)}.`,
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
  return `POLICY NOT IN FORCE: ${POLICY_PROCESS_WATCH}; ${POLICY_UPDATE_HOSTS}. `
    + 'No ExemptMark is configured, so the bypass table that carries them does not exist and update traffic uses the tunnel.';
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
  return `POLICY NOT IN FORCE: ${policy}: ${detail}`;
}
