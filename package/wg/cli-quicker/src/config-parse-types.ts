import type { AllowedFromFiles, } from './config-types.ts';

/**
 Mutable state gathered while walking config lines.
 */
export type ParseAcc = {
  /**
   Accumulated `Address` values.
   */
  addresses: string[];

  /**
   Accumulated DNS server literals.
   */
  dns: string[];

  /**
   Accumulated DNS search domains.
   */
  dnsSearch: string[];

  /**
   Accumulated explicit MTU.
   */
  mtu?: number;

  /**
   Accumulated `Table` value.
   */
  table?: string;

  /**
   Accumulated `ExemptMark` value.
   */
  exemptMark?: number;

  /**
   Accumulated peer endpoint UDP ports.
   */
  endpointPorts: number[];

  /**
   Accumulated peer-scoped `AllowedIPsFromFiles` directives.
   */
  allowedFromFiles: AllowedFromFiles[];

  /**
   Accumulated `PreUp` hooks.
   */
  preUp: string[];

  /**
   Accumulated `PostUp` hooks.
   */
  postUp: string[];

  /**
   Accumulated `PreDown` hooks.
   */
  preDown: string[];

  /**
   Accumulated `PostDown` hooks.
   */
  postDown: string[];

  /**
   Raw lines forwarded to `wg addconf`.
   */
  wgLines: string[];

  /**
   Whether cursor is inside `[Interface]` section.
   */
  inInterface: boolean;

  /**
   Whether cursor is inside `[Peer]` section.
   */
  inPeer: boolean;

  /**
   Zero-based index of current or most recently opened peer.
   */
  peerIndex: number;

  /**
   Whether current peer already contains literal `AllowedIPs`.
   */
  peerHasAllowedIps: boolean;

  /**
   Whether current peer already contains `AllowedIPsFromFiles`.
   */
  peerHasAllowedFromFiles: boolean;
};
