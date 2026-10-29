/**
 Shared type definitions for the vmsync CLI.
 
 @module
 */

//region Hypervisor detection

/**
 Supported hypervisor backends, auto-detected from the current platform.
 */
export type Hypervisor = 'kvm' | 'hyperv';

/**
 Supported disk image formats that vmsync manages.
 */
export type DiskFormat = 'qcow2' | 'raw' | 'vhdx';

//endregion Hypervisor detection

//region VM configuration: persisted as vmsync.jsonc

/**
 Boot settings shared across hypervisors.
 */
export type BootConfig = {
  /**
   Memory allocation, e.g. "4G".
   */
  readonly memory: string;
  /**
   Number of virtual CPUs.
   */
  readonly cpus: number;
};

/**
 Checksums for each managed disk format.
 */
export type DiskChecksums = {
  readonly qcow2: string;
  readonly vhdx: string;
};

/**
 Mutable sync state tracked between boots.
 */
export type SyncState = {
  /**
   Which hypervisor was used for the last boot.
   */
  lastBootHypervisor?: Hypervisor;
  /**
   ISO 8601 timestamp of the last boot.
   */
  lastBootAt?: string;
  /**
   True if the VM was booted but changes have not been synced to the other format.
   */
  synced: boolean;
  /**
   SHA-256 {@link DiskChecksums} recorded after the last sync.
   */
  checksums: DiskChecksums;
};

/**
 Top-level VM configuration persisted in vmsync.jsonc.
 */
export type VmsyncConfig = {
  /**
   Human-readable VM name (also the directory name).
   */
  readonly name: string;
  /**
   Original image path passed to `vmsync import`.
   */
  readonly importedFrom: string;
  /**
   ISO 8601 timestamp of import.
   */
  readonly importedAt: string;
  /**
   Disk size in bytes.
   */
  readonly diskSizeBytes: number;
  /**
   {@link BootConfig} settings.
   */
  readonly boot: BootConfig;
  /**
   {@link SyncState} managed by the CLI.
   */
  state: SyncState;
};

//endregion VM configuration

//region qemu-img info output

/**
 Relevant fields from `qemu-img info --output=json`.
 */
export type QemuImgInfo = {
  readonly filename: string;
  readonly format: string;
  readonly 'virtual-size': number;
  readonly 'actual-size'?: number;
  readonly 'backing-filename'?: string;
  readonly 'format-specific'?: unknown;
};

//endregion qemu-img info output

//region qemu-img map output

/**
 Single region from `qemu-img map --output=json`.
 Depth 0 means the data lives in the topmost overlay (= changed since snapshot).
 */
export type QemuMapRegion = {
  readonly start: number;
  readonly length: number;
  readonly depth: number;
  readonly present: boolean;
  readonly zero: boolean;
  readonly data: boolean;
  readonly offset: number;
  readonly filename: string;
};

//endregion qemu-img map output

//region Well-known file names inside a VM directory

/**
 Name of the KVM-ready disk image.
 */
export const QCOW2_FILENAME = 'base.qcow2';

/**
 Name of the Hyper-V-ready disk image.
 */
export const VHDX_FILENAME = 'base.vhdx';

/**
 Name of the transient overlay used during KVM boots.
 */
export const OVERLAY_FILENAME = 'overlay.qcow2';

/**
 Name of the per-VM configuration file.
 */
export const CONFIG_FILENAME = 'vmsync.jsonc';

//endregion Well-known file names
