import { PolicyRoutingConflictError, } from './errors.ts';
import { run, } from './runner.ts';

/**
 Address-family flags inspected for conflicting rules.
 */
const POLICY_FAMILIES = [
  '-4',
  '-6',
] as const;

/**
 Packet mark installed by IVPN Desktop split tunneling.
 */
const IVPN_SPLIT_MARK = 51_820;

/**
 Numeric IVPN Desktop split-tunnel route table.
 */
const IVPN_SPLIT_TABLE_ID = 17;

/**
 Named IVPN Desktop split-tunnel route table.
 */
const IVPN_SPLIT_TABLE_NAME = 'ivpn-exclude-tbl';

/**
 Untrusted shape returned by `ip -json rule show`.
 */
type PolicyRule = {
  /**
   Optional packet mark selector.
   */
  readonly fwmark?: unknown;

  /**
   Optional route-table selector.
   */
  readonly table?: unknown;
};

/**
 Reports whether unknown JSON value is IVPN Desktop's split-tunnel rule.
 
 @param value - Parsed item from `ip` JSON output.
 
 @returns Whether rule selects IVPN table 17 for mark `0xca6c`.
 
 @internal
 
 @example
 ```ts
 isIvpnSplitRule({ fwmark: '0xca6c', table: '17' });
 ```
 */
export function isIvpnSplitRule(value: unknown,): value is PolicyRule {
  if (((typeof value) !== 'object') || (value === null))
    return false;
  /**
   Object narrowed enough for optional routing fields.
   */
  const rule = value as PolicyRule;
  /**
   Numeric packet mark normalized from current iproute2 JSON forms.
   */
  const mark = (typeof rule.fwmark) === 'number'
    ? rule.fwmark
    : (typeof rule.fwmark) === 'string'
      ? Number(rule.fwmark,)
      : Number.NaN;
  /**
   Whether table uses numeric, rendered numeric, or configured name form.
   */
  const matchesTable = (rule.table === IVPN_SPLIT_TABLE_ID)
    || (rule.table === String(IVPN_SPLIT_TABLE_ID,))
    || (rule.table === IVPN_SPLIT_TABLE_NAME);
  return (mark === IVPN_SPLIT_MARK)
    && matchesTable;
}

/**
 Rejects IVPN Desktop split tunneling before it can override tunnel routes.
 
 IVPN inverse mode marks ordinary application packets for its physical table,
 so route lookups appear correct while real connections bypass WireGuard.
 Detecting its exact rule is fail-closed and does not mutate IVPN preferences.
 
 @throws {@link PolicyRoutingConflictError} when either family carries IVPN's rule.
 
 @example
 ```ts
 await assertNoPolicyRoutingConflict();
 ```
 */
export async function assertNoPolicyRoutingConflict(): Promise<void> {
  /**
   Per-family conflict results read concurrently before network mutation.
   */
  const conflicts = await Promise.all(POLICY_FAMILIES.map(async function readRules(
    family,
  ): Promise<boolean> {
    /**
     JSON rule output for selected family.
     */
    const { stdout, } = await run({
      command: 'ip',
      args: [
        family,
        '-json',
        'rule',
        'show',
      ],
    },);
    /**
     Untrusted parsed rule listing.
     */
    const listing: unknown = JSON.parse(stdout,);
    return Array.isArray(listing,)
      && listing.some(isIvpnSplitRule,);
  },),);
  /**
   Whether any family contains exact IVPN split-tunnel selector.
   */
  const conflict = conflicts.includes(true,);
  if (!conflict)
    return;
  throw new PolicyRoutingConflictError(
    'IVPN Desktop split tunneling is active (fwmark 0xca6c, table 17) and would bypass this WireGuard tunnel. Disable IVPN split tunneling with `ivpn splittun -off` before running `wg-quicker up`.',
  );
}
