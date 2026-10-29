import type { AllowedFromFilesPaths, } from './config-parse-values.ts';

/**
 Peer-associated address-set source files and insertion location.
 */
export type AllowedFromFiles = AllowedFromFilesPaths & {
  /**
   Zero-based peer number containing directive.
   */
  readonly peerIndex: number;

  /**
   Forwarded-config line index where generated `AllowedIPs` belongs.
   */
  readonly wgLineIndex: number;
};

/**
 Parsed WireGuard config subset needed by tunnel lifecycle.
 */
export type WireguardConfig = {
  /**
   Interface name derived from config filename.
   */
  readonly interfaceName: string;

  /**
   `Address` values from `[Interface]`.
   */
  readonly addresses: readonly string[];

  /**
   `DNS` values that are IP literals.
   */
  readonly dns: readonly string[];

  /**
   `DNS` values that are search domains.
   */
  readonly dnsSearch: readonly string[];

  /**
   Explicit `MTU` when present.
   */
  readonly mtu?: number;

  /**
   `Table` value:
   `off`,
   numeric table,
   or absent for automatic.
   */
  readonly table?: string;

  /**
   Socket mark whose traffic bypasses tunnel.
   */
  readonly exemptMark?: number;

  /**
   Distinct UDP destination ports from peer `Endpoint` values.
   */
  readonly endpointPorts: readonly number[];

  /**
   Peer-scoped source-file directives awaiting optional expansion.
   */
  readonly allowedFromFiles: readonly AllowedFromFiles[];

  /**
   `PreUp` hooks in declaration order.
   */
  readonly preUp: readonly string[];

  /**
   `PostUp` hooks in declaration order.
   */
  readonly postUp: readonly string[];

  /**
   `PreDown` hooks in declaration order.
   */
  readonly preDown: readonly string[];

  /**
   `PostDown` hooks in declaration order.
   */
  readonly postDown: readonly string[];

  /**
   Reconstructed native config passed to `wg addconf`.
   */
  readonly wgConfig: string;
};
