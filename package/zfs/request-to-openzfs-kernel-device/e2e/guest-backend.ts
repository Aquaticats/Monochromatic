/**
 The one place this recipe takes the machine its guest lives on from, and what each backend offers in place of a libvirt
 domain.

 `mvm` itself resolves a backend from `--backend`, then from `MVM_BACKEND`, and falls back to `libvirt`
 (`package/mvm/cli/src/backend/registry.ts`, `resolveBackendKind`). This module reads the same two inputs the same way,
 so the recipe and every `mvm` it starts agree without a word being repeated at a call site, and so a run that says
 nothing keeps making the local guest it always made.

 What differs between the backends is not only a word on a command line. A libvirt guest is a domain this host owns:
 `mvm exec` reaches it through the QEMU guest agent, which runs as root, its files arrive in a virtiofs share mounted at
 `/mnt/shared`, and its own hostname is the name the run gave it. An OVHcloud instance is a machine reached over SSH as
 the account its image names (`package/mvm/cli/README.md`, "The login account is the image's own, never `root`"), its
 files arrive over `scp` at the path the caller names, its hostname is the instance's name with `mvm`'s prefix
 (`package/mvm/cli/src/backend/ovh/config.ts`, `validateOvhName`), and it has no share at all. A guest step that names
 any of those four facts itself is a step that only ever worked on one backend.

 Why the guest steps and not the drivers' arms only: the recipe is what makes a guest, prepares it and takes it away, so
 the facts above are the recipe's, and this file is where a second backend is answered.
 */

//region Kinds and the record

/**
 The backends this recipe can drive a guest on.
 Not every backend `mvm` knows: a backend belongs here once the recipe's steps run on it, and `mvm`'s own registry
 carries the rest (`package/mvm/cli/src/backend/registry.ts`).
 */
export type GuestBackendKind = 'libvirt' | 'ovh';

/**
 One backend, as every step of the recipe needs it.
 */
export type GuestBackend = {
  /**
   The kind this record answers for.
   */
  readonly kind: GuestBackendKind;
  /**
   The words that select this backend on every `mvm` command the recipe starts.
   */
  readonly mvmArgumentWords: readonly string[];
  /**
   The hostname a guest `mvm` made under `name` answers on.
   */
  readonly hostnameOf: (name: string) => string;
  /**
   What a guest command that must run as root is started with.
   Empty on a backend whose command already runs as root.
   */
  readonly rootWords: readonly string[];
  /**
   The path `mvm push` is given for one file, whose base name is what the guest reads.
   */
  readonly pushDestinationOf: (file: string) => string;
  /**
   The folder inside the guest a pushed file, and a log copied out of it, are readable at.
   */
  readonly pushedFolder: string;
  /**
   Whether this backend lets a step reboot its guest or cut its power.
   */
  readonly powerCycles: boolean;
  /**
   Whether the guest is made from an image this package bakes, whose kernel release is pinned here.
   A backend that boots the provider's own catalogue image answers its own release instead.
   */
  readonly imageIsBaked: boolean;
};

/**
 The environment variable `mvm` reads when no `--backend` word is given
 (`package/mvm/cli/src/backend/registry.ts`, `BACKEND_ENV_VAR`).
 */
export const GUEST_BACKEND_ENV_VAR = 'MVM_BACKEND';

/**
 The backend a run that says nothing gets, which is the one `mvm` itself falls back to.
 */
export const DEFAULT_GUEST_BACKEND_KIND = 'libvirt';

/**
 The home of the account an OVHcloud Ubuntu image logs in as.
 The image's own `user` field names it (`package/mvm/cli/README.md`, "The login account is the image's own"), and
 `/root` is not writable by that account, so a pushed file and a copied-out log live here.
 */
const OVH_ACCOUNT_HOME = '/home/ubuntu';

/**
 The path `mvm`'s own guest agent sees a pushed file at, and the guest sees the virtiofs share at
 (`package/mvm/cli/src/config.ts`, `GUEST_MOUNT_POINT`).
 */
const LIBVIRT_SHARED_FOLDER = '/mnt/shared';

/**
 Every backend this recipe can drive, keyed by kind.
 */
export const GUEST_BACKENDS: Record<GuestBackendKind, GuestBackend> = {
  libvirt: {
    kind: 'libvirt',
    mvmArgumentWords: ['--backend', 'libvirt',],
    // `mvm`'s cloud-init seed sets the guest's hostname to the name the run gave it.
    hostnameOf: (name: string,): string => name,
    // The guest agent runs as root, so a step that must be root asks for nothing.
    rootWords: [],
    // The share keeps only the file name, so the word names one file and no directory.
    pushDestinationOf: (file: string,): string => `/shared/${file}`,
    pushedFolder: LIBVIRT_SHARED_FOLDER,
    powerCycles: true,
    imageIsBaked: true,
  },
  ovh: {
    kind: 'ovh',
    mvmArgumentWords: ['--backend', 'ovh',],
    // OVHcloud turns an instance name into its hostname, and mvm names the instance `mvm-<name>`.
    hostnameOf: (name: string,): string => `mvm-${name}`,
    // The command runs over SSH as the image's own account, which the Ubuntu image lets sudo without a password.
    rootWords: ['sudo', '--non-interactive', '--',],
    pushDestinationOf: (file: string,): string => `${OVH_ACCOUNT_HOME}/${file}`,
    pushedFolder: OVH_ACCOUNT_HOME,
    // `mvm` never stops, pauses, shelves or resumes an OVHcloud instance (`package/mvm/cli/README.md`,
    // "OVHcloud Public Cloud backend"), so nothing can power-cycle one.
    powerCycles: false,
    // The guest boots the catalogue image the run names, whose kernel OVHcloud moves without asking.
    imageIsBaked: false,
  },
};

//endregion Kinds and the record

//region Resolution

/**
 Whether `value` names a backend this recipe can drive.

 @param value - candidate word

 @returns whether the word names one of {@link GUEST_BACKENDS}

 @example
 ```ts
 isGuestBackendKind('ovh'); // true
 isGuestBackendKind('hetzner'); // false
 ```
 */
export function isGuestBackendKind(value: string,): value is GuestBackendKind {
  return Object.hasOwn(
    GUEST_BACKENDS,
    value,
  );
}

/**
 The backend a run's own word, its environment and the default resolve to.

 The precedence is `mvm`'s own: an explicit `--backend` word wins over {@link GUEST_BACKEND_ENV_VAR}, an empty word
 falls through to the variable, and a run that names neither gets {@link DEFAULT_GUEST_BACKEND_KIND}. A word naming a
 backend this recipe cannot drive refuses here, by name, rather than at the first step that reaches the guest.

 @param raw - the word the run gave `--backend`, or `undefined` when the run gave none

 @param env - the environment to read the variable from

 @returns the backend every step of the run drives

 @throws Error naming the word and the backends this recipe can drive

 @example
 ```ts
 resolveGuestBackend({ raw: undefined, env: { MVM_BACKEND: 'ovh' } }).kind; // 'ovh'
 resolveGuestBackend({ raw: undefined, env: {} }).kind; // 'libvirt'
 ```
 */
export function resolveGuestBackend({
  raw,
  env,
}: {
  readonly raw?: string | undefined;
  readonly env: Readonly<Record<string, string | undefined>>;
},): GuestBackend {
  /**
   The word that decides: the run's own when it gave one, and otherwise the environment's.
   */
  const word = ((raw === undefined) || (raw === '')) ? env[GUEST_BACKEND_ENV_VAR] : raw;
  if ((word === undefined) || (word === '')) {
    return GUEST_BACKENDS[DEFAULT_GUEST_BACKEND_KIND];
  }
  if (!isGuestBackendKind(word,)) {
    throw new Error(
      `unknown backend '${word}': the guest recipe drives ${Object.keys(GUEST_BACKENDS,).join(', ',)}, and ${GUEST_BACKEND_ENV_VAR} or --backend must name one of those`,
    );
  }
  return GUEST_BACKENDS[word];
}

//endregion Resolution

//region Steps that need a power cycle

/**
 The steps whose subject is the guest's own reboot or its power cut, or which run those steps.
 A backend that cannot stop and start its guest can neither, so these refuse before a guest is made.
 */
export const POWER_CYCLE_STEPS: readonly string[] = [
  'zfs-add',
  'zfs-add-boot-before',
  'zfs-add-boot-reboot',
  'zfs-add-boot-plant',
  'zfs-add-boot-after',
  'zfs-add-power-before',
  'zfs-add-power-after',
];

/**
 Refuse a step whose subject is a reboot or a power cut on a backend that cannot power-cycle its guest.

 Said before anything is created: the step names itself, the backend and what stands in the way, so a run that cannot
 run it never leaves a machine behind.

 @param backend - the backend this run resolved

 @param step - the step the run named

 @throws Error naming the step, the backend and the reason

 @example
 ```ts
 refuseAStepThatNeedsAPowerCycle({ backend: GUEST_BACKENDS.ovh, step: 'zfs-add-boot-reboot' }); // throws
 refuseAStepThatNeedsAPowerCycle({ backend: GUEST_BACKENDS.ovh, step: 'zfs-attach-arm' }); // returns
 ```
 */
export function refuseAStepThatNeedsAPowerCycle({
  backend,
  step,
}: {
  readonly backend: GuestBackend;
  readonly step: string;
}): void {
  if (!POWER_CYCLE_STEPS.includes(step,) || backend.powerCycles) {
    return;
  }
  throw new Error(
    `step '${step}' reboots the guest or cuts its power, and backend '${backend.kind}' cannot power-cycle a machine: mvm never stops, pauses, shelves or resumes an instance on it, so a guest that did not come back could be neither reset nor inspected`,
  );
}

//endregion Steps that need a power cycle
