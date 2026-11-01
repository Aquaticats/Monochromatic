/**
 Backend abstraction shared by every mvm provider.
 
 A {@link Backend} is a record of the VM operations the CLI and MCP server
 dispatch to. The local libvirt path and the Hetzner Cloud path each export
 one. Method shapes match the existing standalone functions exactly, so the
 libvirt backend is a pure re-wiring of them.
 
 @module
 */

import type { ExecResult, } from '../exec.ts';
import type { VmInfo, } from '../list.ts';

//region Backend kind

/**
 Discriminant naming each supported provider.
 Extended as backends land (Hyper-V, Apple Virtualization, more clouds).
 
 @example
 ```ts
 const kind: BackendKind = 'hetzner';
 ```
 */
export type BackendKind = 'hetzner' | 'libvirt';

//endregion Backend kind

//region Backend operations

/**
 Record of the VM operations a provider implements.
 Every member mirrors the signature of the corresponding standalone function
 in this package; {@link create} additionally accepts cloud-only `serverType` and
 `location` hints that the libvirt backend ignores.
 
 @example
 ```ts
 const backend: Backend = libvirtBackend;
 await backend.create({ name: 'dev-01' });
 ```
 */
export type Backend = {
  /**
   Creates and starts a VM.
   
   @param args - Name, optional image, and cloud-only server type / location
   */
  readonly create: (args: {
    readonly name: string;
    readonly image?: string;
    readonly serverType?: string;
    readonly location?: string;
  },) => Promise<void>;

  /**
   Clones an existing VM into a new one.
   
   @param args - Source VM name and destination VM name
   */
  readonly clone: (args: {
    readonly destination: string;
    readonly source: string;
  },) => Promise<void>;

  /**
   Destroys a single VM by name.
   
   @param args - VM name without the mvm- prefix
   */
  readonly destroy: (args: { readonly name: string; },) => Promise<void>;

  /**
   Destroys every VM this tool manages.
   */
  readonly destroyAll: () => Promise<void>;

  /**
   Lists managed VMs with their state.
   */
  readonly list: () => Promise<readonly VmInfo[]>;

  /**
   Runs a command inside a named VM.
   
   @param args - Command to run and target VM name
   */
  readonly exec: (args: {
    readonly command: string;
    readonly name: string;
  },) => Promise<ExecResult>;

  /**
   Creates an ephemeral VM, runs a command, then destroys it.
   
   @param args - Command to run and optional source VM to clone from
   */
  readonly run: (args: {
    readonly command: string;
    readonly from?: string;
  },) => Promise<ExecResult>;

  /**
   Copies a file from the host into a VM.
   
   @param args - VM name, host source path, and guest destination path
   */
  readonly pushFile: (args: {
    readonly name: string;
    readonly hostPath: string;
    readonly guestPath: string;
  },) => Promise<string>;

  /**
   Copies a file out of a VM to the host.
   
   @param args - VM name and guest source path
   */
  readonly pullFile: (args: {
    readonly name: string;
    readonly guestPath: string;
  },) => Promise<Buffer>;

  /**
   Refreshes provider-managed images or templates.
   */
  readonly update: () => Promise<void>;

  /**
   Opens an interactive session to a VM.
   
   @param args - VM name without the mvm- prefix
   */
  readonly shell: (args: { readonly name: string; },) => Promise<void>;
};

//endregion Backend operations

//region Backend metadata

/**
 Static metadata describing where a backend can run.
 `platforms` is `'all'` for cloud backends reachable from any OS, or a list of
 supported {@link NodeJS.Platform} values for host-hypervisor backends.
 
 @example
 ```ts
 const meta: BackendMeta = { platforms: ['linux'], description: 'local KVM' };
 ```
 */
export type BackendMeta = {
  /**
   Supported platforms, or `'all'` when any platform can use the backend.
   */
  readonly platforms: readonly NodeJS.Platform[] | 'all';
  /**
   One-line human description for help and diagnostics.
   */
  readonly description: string;
};

//endregion Backend metadata
