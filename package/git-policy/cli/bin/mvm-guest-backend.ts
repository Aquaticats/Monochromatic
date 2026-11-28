/**
 The one place a guest driver takes the machine its guest lives on from, and what each backend offers in place of a
 libvirt domain.

 `mvm` resolves a backend from `--backend`, then from `MVM_BACKEND`, and falls back to `libvirt`
 (`package/mvm/cli/src/backend/registry.ts`, `resolveBackendKind`). This module reads the same variable the same way, so
 a driver and every `mvm` it starts agree without a word being repeated at a call site, and a driver that says nothing
 keeps driving the local guest it has always driven.

 What a driver needs to know is not the word alone. A libvirt guest is a domain this host owns: `mvm exec` reaches it
 through the QEMU guest agent, which runs as root, `mvm push` lands a file in a virtiofs share the guest mounts at
 `/mnt/shared` under the file's own name, and the guest's hostname is the name the run gave it. An OVHcloud instance is a
 machine reached over SSH as the account its image names (`package/mvm/cli/README.md`, "The login account is the image's
 own, never `root`"), `mvm push` writes the absolute path it is given over `scp`, its hostname is the instance's name with
 `mvm`'s prefix (`package/mvm/cli/src/backend/ovh/config.ts`, `validateOvhName`), and it has no share at all.

 The arms are the one thing this module still refuses on a second backend: every arm script this package runs is a shell
 script that reads and writes `/mnt/shared` itself, which is the libvirt share, so an arm step says which backend it
 cannot run on and refuses before it creates a guest ({@link refuseAStepWhoseArmsCannotRun}). A driver's `create`,
 `prepare` and `destroy` steps name no such path, and run on either backend.
 */

//region Kinds and the record

/**
 The backends a guest driver can drive a machine on.
 Not every backend `mvm` knows: a kind belongs here once this package's steps run on it.
 */
export type GuestBackendKind = 'libvirt' | 'ovh';

/**
 One backend, as every guest step of this package needs it.
 */
export type GuestBackend = {
  /**
   The kind this record answers for.
   */
  readonly kind: GuestBackendKind;
  /**
   The words that select this backend on every `mvm` a driver starts.
   */
  readonly mvmArgumentWords: readonly string[];
  /**
   The hostname a guest `mvm` made under `name` answers on.
   */
  readonly hostnameOf: (name: string) => string;
  /**
   What a guest command that must run as root is started with, empty on a backend whose command already runs as root.
   */
  readonly rootWords: readonly string[];
  /**
   The folder inside the guest a pushed file, and a log copied out of it, are readable at.
   */
  readonly guestSharedFolder: string;
  /**
   The path `mvm push` is given for one file, whose base name is what the guest reads.
   */
  readonly pushDestinationOf: (file: string) => string;
  /**
   Whether this package's arm scripts run on this backend.
   */
  readonly runsTheArms: boolean;
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
 `/root` is not writable by that account, so a pushed file and a copied-out log live here.
 */
const OVH_ACCOUNT_HOME = '/home/ubuntu';

/**
 Every backend a guest driver can drive, keyed by kind.
 */
export const GUEST_BACKENDS: Record<GuestBackendKind, GuestBackend> = {
  libvirt: {
    kind: 'libvirt',
    mvmArgumentWords: [
      '--backend',
      'libvirt',
    ],
    // `mvm`'s cloud-init seed sets the guest's hostname to the name the run gave it.
    hostnameOf(name: string,): string {
      return name;
    },
    // The guest agent runs as root, so a step that must be root asks for nothing.
    rootWords: [],
    guestSharedFolder: '/mnt/shared',
    // The share keeps only the file name, so the word names one file and no directory.
    pushDestinationOf(file: string,): string {
      return `/shared/${file}`;
    },
    runsTheArms: true,
  },
  ovh: {
    kind: 'ovh',
    mvmArgumentWords: [
      '--backend',
      'ovh',
    ],
    // OVHcloud turns an instance name into its hostname, and mvm names the instance `mvm-<name>`.
    hostnameOf(name: string,): string {
      return `mvm-${name}`;
    },
    // The command runs over SSH as the image's own account, which the Ubuntu image lets sudo without a password.
    rootWords: [
      'sudo',
      '--non-interactive',
      '--',
    ],
    guestSharedFolder: OVH_ACCOUNT_HOME,
    pushDestinationOf(file: string,): string {
      return `${OVH_ACCOUNT_HOME}/${file}`;
    },
    runsTheArms: false,
  },
};

//endregion Kinds and the record

//region Resolution

/**
 Whether `value` names a backend a guest driver can drive.

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
 backend this package cannot drive refuses here, by name, rather than at the first step that reaches a guest.

 @param raw - the word the run gave `--backend`, absent when the run gave none

 @param env - the environment to read the variable from

 @returns the backend every step of the run drives

 @throws Error naming the word and the backends a driver can drive

 @example
 ```ts
 resolveGuestBackend({ env: { MVM_BACKEND: 'ovh' } }).kind; // 'ovh'
 resolveGuestBackend({ env: {} }).kind; // 'libvirt'
 ```
 */
export function resolveGuestBackend({
  raw,
  env,
}: {
  readonly raw?: string;
  readonly env: Readonly<NodeJS.ProcessEnv>;
},): GuestBackend {
  /**
   The word that decides: the run's own when it gave one, and otherwise the environment's.
   */
  const word = ((raw === undefined) || (raw === '')) ? env[GUEST_BACKEND_ENV_VAR] : raw;
  if ((word === undefined) || (word === '')) {
    return GUEST_BACKENDS[DEFAULT_GUEST_BACKEND_KIND];
  }
  if (!isGuestBackendKind(word,)) {
    /**
     The kinds a driver can drive, as one refusal lists them.
     */
    const kinds = Object.keys(GUEST_BACKENDS,)
      .join(', ',);
    throw new Error(
      `unknown backend '${word}': a guest driver drives ${kinds}, and ${GUEST_BACKEND_ENV_VAR} or --backend must name one of those`,
    );
  }
  return GUEST_BACKENDS[word];
}

/**
 The backend this process runs under, read once from {@link GUEST_BACKEND_ENV_VAR}.

 A driver's helpers call this where they build a command, so the environment is the only thing that decides, and no step
 carries a backend of its own.

 @returns the backend of this process

 @example
 ```ts
 guestBackend().mvmArgumentWords; // ['--backend', 'libvirt']
 ```
 */
export function guestBackend(): GuestBackend {
  return resolveGuestBackend({ env: process.env, },);
}

//endregion Resolution

//region Steps whose arms cannot run

/**
 The steps a guest driver runs without an arm in the guest: the recipe's own create, prepare and destroy.

 Every other step of every driver runs an arm, so a driver hands this list to
 {@link refuseAStepWhoseArmsCannotRun} rather than naming its own, and a step a driver adds later is refused on a backend
 without the arms until it is written to run there.
 */
export const STEPS_THAT_RUN_NO_ARM: readonly string[] = [
  'create',
  'prepare',
  'destroy',
];

/**
 Refuse a step whose arms run in a backend this package's arm scripts cannot reach.

 The arms are shell scripts that read and write the guest's `/mnt/shared` folder, which is the libvirt share, so a
 backend without that share can make and prepare a guest but not run an arm in it. Said before the step runs anything,
 so a step that cannot finish never leaves a machine behind.

 @param backend - the backend this run resolved

 @param step - the step the run named

 @param stepsThatRunNoArm - the steps of this driver that run no arm in the guest

 @throws Error naming the step, the backend and the path the arms need

 @example
 ```ts
 refuseAStepWhoseArmsCannotRun({ backend, step: 'run', stepsThatRunNoArm: STEPS_THAT_RUN_NO_ARM });
 ```
 */
export function refuseAStepWhoseArmsCannotRun({
  backend,
  step,
  stepsThatRunNoArm,
}: {
  readonly backend: GuestBackend;
  readonly step: string;
  readonly stepsThatRunNoArm: readonly string[];
},): void {
  if (backend.runsTheArms || stepsThatRunNoArm.includes(step,)) {
    return;
  }
  throw new Error(
    `step '${step}' runs an arm in the guest, and this package's arm scripts read and write the guest's /mnt/shared folder, which a '${backend.kind}' guest does not have: create, prepare and destroy run on '${backend.kind}', and an arm needs a libvirt guest`,
  );
}

//endregion Steps whose arms cannot run
