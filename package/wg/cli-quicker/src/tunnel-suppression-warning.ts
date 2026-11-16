/**
 The main-table suppression rules of tunnels an earlier build set up, and their removal.

 A `suppress_prefixlength 0` rule names no interface and no table, so it has no identity
 of its own; the tunnels this program starts now carry an explicit preference recorded in
 their claim, and this module is what a tunnel without a claim is left with:
 a tagged rule is removed by shape, and an untagged one is reported, never deleted.

 @module
 */

import { tagged, } from '@monochromatic-dev/logger-core/ts';

import {
  runAllowingFailure,
} from './runner.ts';
import { isAbsentDelete, } from './ip-delete.ts';
import { POLICY_RULE_PROTOCOL, } from './tunnel-rule-protocol.ts';

/**
 Module logger for the suppression rules of earlier builds.
 */
const l = tagged({ tag: 'tunnel-suppression-warning', },);

/**
 The address family a rule listing belongs to.
 */
type Proto = '-4' | '-6';

/**
 Removes the main-table suppression of one family,
 and only the rule this program created.

 A rule without the tag is left in place,
 because it cannot be told apart from one an administrator or another tunnel installed:
 the shape names no interface and no table,
 and `ip rule delete table main suppress_prefixlength 0` removes the first rule of that shape,
 which is whichever one has the lowest preference,
 not necessarily this program's.
 A tunnel an earlier build set up therefore leaves its untagged suppression behind,
 and the warning names the exact command that removes it.

 @param proto - Address family whose suppression is removed.

 @param automaticRouting - Whether the interface carries the fwmark that automatic routing sets,
 which is what would make an untagged main-table suppression an earlier build's leftover.

 @example
 ```ts
 await removeSuppressRule({ proto: '-4', automaticRouting: true });
 ```
 */
export async function removeSuppressRule(
  {
    proto,
    automaticRouting,
  }: {
    readonly proto: Proto;
    readonly automaticRouting: boolean;
  },
): Promise<void> {
  /**
   What the exact, protocol-tagged delete ended with.
   */
  const deletion = await runAllowingFailure({
    command: 'ip',
    args: [
      proto,
      'rule',
      'delete',
      'table',
      'main',
      'suppress_prefixlength',
      '0',
      'protocol',
      String(POLICY_RULE_PROTOCOL,),
    ],
  },);
  if (deletion.exitCode === 0)
    return;
  if (!isAbsentDelete({ result: deletion, },)) {
    /**
     What the delete said when it failed.
     */
    const refusal = deletion.stderr
      .trim();
    l.warn(`could not remove the ${proto} main-table suppression: ${refusal}`,);
    return;
  }
  if (!automaticRouting)
    return;
  /**
   What the probe of the family's rules ended with.
   */
  const shown = await runAllowingFailure({
    command: 'ip',
    args: [
      proto,
      'rule',
      'show',
    ],
  },);
  if (shown.exitCode !== 0) {
    /**
     What the rule listing said when it failed.
     */
    const refusal = shown.stderr
      .trim();
    l.debug(`could not list the ${proto} rules to look for a suppression without the tag: ${refusal}`,);
    return;
  }
  /**
   Every suppression of the family that carries no tag.
   */
  for (const rule of untaggedSuppressionRules({ listed: shown.stdout, },)) {
    l.warn(untaggedSuppressionWarning({
      proto,
      rule,
    },),);
  }
}

/**
 Finds the main-table suppressions of one family that carry no protocol tag,
 among the lines `ip -4 rule show` or `ip -6 rule show` printed.

 A tagged rule prints the tag,
 so a line that names the suppression and no tag is a rule this program never created.
 Nothing is deleted on this finding:
 the shape names no interface and no table,
 so the line is reported to the person who can tell whose rule it is.

 @param listed - Everything the rule listing printed.

 @returns Every untagged suppression line the listing printed, in its order.

 @example
 ```ts
 untaggedSuppressionRule({ listed: '32765: from all lookup main suppress_prefixlength 0\n' });
 ```
 */
export function untaggedSuppressionRules(
  { listed, }: { readonly listed: string; },
): readonly string[] {
  /**
   Lines of untagged suppressions, in the order the listing printed them.
   */
  const found: string[] = [];
  for (const line of listed.split('\n',)) {
    /**
     One rule line without the whitespace around it.
     */
    const trimmed = line.trim();
    /**
     Words of that rule line.
     */
    /* oxlint-disable-next-line no-restricted-syntax/no-regex -- One line of `ip rule show` output is bounded by a family's rule count, and its fields are separated by runs of whitespace a string split cannot express. */
    const words = trimmed.split(/\s+/u,);
    if (!words.includes('suppress_prefixlength',))
      continue;
    if (words.includes('proto',) || words.includes('protocol',))
      continue;
    found.push(trimmed,);
  }
  return found;
}

/**
 Explains an untagged main-table suppression that was left in place,
 and names every command that removes it.

 Deleted only by a person who knows whose rule it is:
 an earlier build of this program for an interface with automatic routing left one,
 and an administrator or another tunnel can leave the same shape,
 so removing it by shape could take routing another owner depends on.

 @param proto - Address family the rule belongs to.

 @param rule - The line the rule listing printed for it.

 @returns The warning text.

 @example
 ```ts
 untaggedSuppressionWarning({ proto: '-4', rule: '32765: from all lookup main suppress_prefixlength 0' });
 ```
 */
export function untaggedSuppressionWarning(
  {
    proto,
    rule,
  }: {
    readonly proto: Proto;
    readonly rule: string;
  },
): string {
  return `the ${proto} rule \`${rule}\` suppresses the main table without this program's protocol tag ${String(POLICY_RULE_PROTOCOL,)}, so this program did not create it and left it in place: an earlier build of this program for an interface with automatic routing left the same shape, and so can an administrator or another tunnel. If it is a leftover of an earlier build, remove it with \`sudo ip ${proto} rule delete table main suppress_prefixlength 0\`; if another owner depends on it, keep it.`;
}
