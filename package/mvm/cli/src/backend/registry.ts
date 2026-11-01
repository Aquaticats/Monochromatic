/**
 Static backend registry, selection, and platform guard.
 
 A plain `Record` maps each {@link BackendKind} to its metadata and a lazy
 loader. Lazy `import()` keeps platform-specific backend modules (and their
 dependencies) off the import graph until selected. Selection is resolved at
 call time, not through any dynamic discovery or plugin system.
 
 @module
 */

import type {
  Backend,
  BackendKind,
  BackendMeta,
} from './types.ts';

//region Registry

/**
 One registry entry: static metadata plus a lazy module loader.
 
 @example
 ```ts
 const libvirt: BackendEntry = BACKENDS.libvirt;
 ```
 */
type BackendEntry = {
  /**
   Where the backend can run and what it is.
   */
  readonly meta: BackendMeta;
  /**
   Dynamically imports and returns the backend implementation.
   */
  readonly load: () => Promise<Backend>;
};

/**
 Default backend when neither flag nor env selects one.
 Libvirt matches today's behaviour and is the only host-local backend.
 */
export const DEFAULT_BACKEND_KIND: BackendKind = 'libvirt';

/**
 Environment variable that sets the default backend when `--backend` is absent.
 */
export const BACKEND_ENV_VAR = 'MVM_BACKEND';

/**
 Registry of every supported backend, keyed by {@link BackendKind}.
 The `load` thunks `import()` lazily so a backend's module never loads on a
 platform that cannot use it.
 
 @example
 ```ts
 const backend = await BACKENDS.hetzner.load();
 ```
 */
export const BACKENDS: Record<BackendKind, BackendEntry> = {
  hetzner: {
    meta: {
      description: 'Hetzner Cloud servers provisioned over the HTTP API (any platform; needs HCLOUD_TOKEN and an OpenSSH client)',
      platforms: 'all',
    },
    load: async function loadHetzner(): Promise<Backend> {
      return (await import('./hetzner/index.ts')).hetznerBackend;
    },
  },
  libvirt: {
    meta: {
      description: 'Local QEMU/KVM virtual machines via libvirt (Linux only)',
      platforms: ['linux',],
    },
    load: async function loadLibvirt(): Promise<Backend> {
      return (await import('./libvirt/index.ts')).libvirtBackend;
    },
  },
};

//endregion Registry

//region Kind resolution

/**
 Type guard: whether `value` names a registered backend.
 
 @param value - candidate backend name
 
 @returns whether `value` is a {@link BackendKind}
 
 @example
 ```ts
 isKnownBackendKind('hetzner'); // true
 isKnownBackendKind('aws');     // false
 ```
 */
export function isKnownBackendKind(value: string,): value is BackendKind {
  return Object.hasOwn(
    BACKENDS,
    value,
  );
}

/**
 Resolves the backend kind from an explicit value, the environment, or the
 default. An explicit but unknown value is an error so typos do not silently
 fall back to libvirt.
 
 @param raw - value parsed from `--backend`, or `undefined` when not passed
 
 @returns resolved backend kind
 
 @throws Error when `raw` (or the {@link BACKEND_ENV_VAR} env) names an unknown backend
 
 @example
 ```ts
 resolveBackendKind('hetzner'); // 'hetzner'
 resolveBackendKind(undefined); // 'libvirt' (or MVM_BACKEND when set)
 ```
 */
export function resolveBackendKind(raw?: string,): BackendKind {
  /**
   Explicit flag wins over env; env wins over the built-in default. An empty
   `raw` (no flag passed) falls through to the env rather than forcing default.
   */
  const candidate = ((raw === undefined) || (raw === ''))
    ? process.env[BACKEND_ENV_VAR]
    : raw;
  if ((candidate === undefined) || (candidate === '')) {
    return DEFAULT_BACKEND_KIND;
  }
  if (!isKnownBackendKind(candidate,)) {
    throw new Error(
      `unknown backend "${candidate}". Available backends: ${
        Object.keys(BACKENDS,)
          .join(', ',)
      }`,
    );
  }
  return candidate;
}

//endregion Kind resolution

//region Selection and platform guard

/**
 Pure predicate: whether `kind` supports `platform`.
 Split out from {@link selectBackend} so the guard is testable without
 touching the real `process.platform`.
 
 @param kind - backend to check
 
 @param platform - platform to test against (e.g. `process.platform`)
 
 @returns whether the backend runs on that platform
 
 @example
 ```ts
 isBackendAvailable('libvirt', 'linux'); // true
 isBackendAvailable('libvirt', 'win32'); // false
 isBackendAvailable('hetzner', 'win32'); // true ('all')
 ```
 */
export function isBackendAvailable(
  {
    kind,
    platform,
  }: {
    readonly kind: BackendKind;
    readonly platform: NodeJS.Platform;
  },
): boolean {
  /**
   Supported-platform descriptor for `kind`; `'all'` short-circuits the check.
   */
  const { platforms, } = BACKENDS[kind]
    .meta;
  return (platforms === 'all') || platforms.includes(platform,);
}

/**
 Validates that `kind` runs on the current platform, then lazily loads and
 returns its backend. The platform guard throws before any provisioning work.
 
 @param kind - resolved backend kind
 
 @returns loaded backend implementation
 
 @throws Error when the current platform does not support `kind`
 
 @example
 ```ts
 const backend = await selectBackend(resolveBackendKind(rawBackend));
 await backend.list();
 ```
 */
export function selectBackend(kind: BackendKind,): Promise<Backend> {
  if (!isBackendAvailable({
    kind,
    platform: process.platform,
  },)) {
    /**
     Human-readable supported-platform list for the error message.
     */
    const { platforms, } = BACKENDS[kind]
      .meta;
    /**
     Rendered platform set; `'all'` never reaches here, so it is always a list.
     */
    const supported = (platforms === 'all') ? 'all' : platforms.join(', ',);
    throw new Error(
      `backend "${kind}" is not available on ${process.platform} (supported: ${supported})`,
    );
  }
  return BACKENDS[kind]
    .load();
}

//endregion Selection and platform guard
