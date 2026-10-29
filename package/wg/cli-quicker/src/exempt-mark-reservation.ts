import dedent from 'string-dedent';

import { ConfigError, } from './errors.ts';

/**
 One packet-mark bit range a different subsystem already matches on.
 */
export type ReservedMarkMask = {
  /**
   Bits the owning subsystem tests in its own rules.
   */
  readonly mask: number;

  /**
   Subsystem installing rules that test these bits.
   */
  readonly owner: string;

  /**
   Runtime behaviour a shared bit hands to packets this tunnel marked.
   */
  readonly consequence: string;
};

/**
 `ExemptMark` value sharing no bit with any entry of {@link RESERVED_MARK_MASKS}.

 Every set bit stays below `0x100`, under the ranges container, cluster, and
 mesh networking assign to themselves.
 */
export const RECOMMENDED_EXEMPT_MARK = 100;

/**
 Mark bits another subsystem on this host already acts on.

 An exempt mark sharing any of these bits reaches that subsystem's rules
 indistinguishable from a packet it marked, so entries list a whole mask rather
 than one value. Entries carry a rule read from installed source, not a
 convention borrowed from a subsystem nobody here runs;
 `doc/troubleshooting/netavark-masquerade-mask-exempt-mark.md` holds the trace.

 @example
 ```ts
 RESERVED_MARK_MASKS.map(({ mask }) => mask);
 ```
 */
export const RESERVED_MARK_MASKS: readonly ReservedMarkMask[] = [
  {
    mask: 0x20_00,
    owner: 'netavark, the podman network backend (`MASK`, `src/firewall/nft.rs:32`)',
    consequence:
      'its `table inet netavark` postrouting rule `meta mark & 0x2000 == 0x2000 masquerade` carries no interface or address restriction, so it source-NATs every matching packet, IPv4 loopback included, and a server bound to `127.0.0.1` then reads the outbound interface address as its peer',
  },
] as const;

/**
 Renders a mark the way `nft` and `ip rule` print one.

 @param mark - Unsigned packet mark.

 @returns Lowercase hexadecimal text with `0x` prefix.

 @example
 ```ts
 formatMark({ mark: 8888 }); // '0x22b8'
 ```
 */
export function formatMark(
  { mark, }: { readonly mark: number; },
): string {
  return `0x${mark.toString(16,)}`;
}

/**
 Reports reserved masks sharing at least one bit with a mark.

 @param mark - Candidate exempt mark.

 @returns Fresh list of overlapping reservations, empty when the mark is free.

 @example
 ```ts
 reservedMarkCollisions({ mark: 8888 });
 ```
 */
export function reservedMarkCollisions(
  { mark, }: { readonly mark: number; },
): readonly ReservedMarkMask[] {
  return RESERVED_MARK_MASKS.filter(function sharesBits(reserved,) {
    return (mark & reserved.mask) !== 0;
  },);
}

/**
 Rejects an exempt mark whose bits another subsystem already claims.

 @param key - Config key naming the rejected value in the diagnostic.

 @param mark - Parsed positive mark.

 @throws {@link ConfigError} when the mark shares bits with a reserved mask.

 @example
 ```ts
 assertUnreservedExemptMark({ key: 'ExemptMark', mark: 8888 });
 ```
 */
export function assertUnreservedExemptMark(
  {
    key,
    mark,
  }: {
    readonly key: string;
    readonly mark: number;
  },
): void {
  /**
   Reservations this mark would trigger.
   */
  const collisions = reservedMarkCollisions({ mark, },);
  if (collisions.length === 0)
    return;
  /**
   One sentence per overlapping reservation.
   */
  const reasons = collisions.map(function describe(reserved,) {
    return `Bit range ${formatMark({ mark: reserved.mask, },)} belongs to ${reserved.owner}: ${reserved.consequence}.`;
  },);
  throw new ConfigError(dedent`
    Invalid \`${key}' value \`${String(mark,)}' (${formatMark({ mark, },)}): it shares bits with a packet mask another subsystem matches on.

    ${reasons.join('\n',)}

    Pick a mark sharing no bit with those masks, such as \`${key} = ${String(RECOMMENDED_EXEMPT_MARK,)}' (${formatMark({ mark: RECOMMENDED_EXEMPT_MARK, },)}), then bring this interface down and up again so the watcher and policy rule carry the new mark. Keeping the current value requires removing the other subsystem instead: the collision is in its rules, not in this config file.
  `,);
}
