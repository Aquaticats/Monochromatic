/**
 The seccomp profile the gate's containers are given, and the arguments that give it to them.

 Podman applies the containers-common default seccomp profile, whose rule for a system call it does not list is
 `SCMP_ACT_ERRNO` with errno 38 (`ENOSYS`). That profile allows `mount`, `mount_setattr` and `move_mount` but not
 `statmount` or `listmount`, so in a gate container `statmount(2)` answers `ENOSYS` exactly as an older kernel would,
 and the sweep's control then reads the live table instead of the kernel's answer
 (`doc/troubleshooting/podman-default-seccomp-blocks-statmount-in-a-container.md`).

 The gate is here to test the product where the product runs, so its containers get the host's own profile with those two
 calls added. Deriving it from what podman reports keeps every other rule the host applies, including the rules a newer
 containers-common adds, which a copy committed here would drift away from.
 */
import {
  copyFile,
  readFile,
  writeFile,
} from 'node:fs/promises';
import {
  join,
  resolve,
} from 'node:path';

import {
  NativeVerificationError,
  runCommand,
} from './native-verification-process.ts';

/**
 The calls the standing-mount sweep asks the kernel by a mount's number.

 `statmount(2)` is the call the sweep makes; `listmount(2)` is the sibling call of the same uapi, and the profile carries
 both so a control or a later step that lists mounts is not denied where the product may run.
 */
export const theCallsTheMountSweepAsks: readonly string[] = [
  'listmount',
  'statmount',
];

/**
 The variable a run's containers carry when their seccomp profile allows those calls.

 The controls read it (`ALLOWED_IN_CONTAINERS_VARIABLE` in
 `src/native/worktree_clone_mount_handler/standing_mount_source_call_refusal.rs`): a run that states the allowance must
 see the call answer, and every other run prints one line naming the control that did not run.
 */
export const statmountAllowedVariable = 'CLI_GIT_STATMOUNT_ALLOWED_IN_CONTAINERS';

/**
 The two calls as one line names them.

 It is derived from {@link theCallsTheMountSweepAsks} rather than written out, so a call added to that list reaches every
 line this module prints.
 */
const theCallsAsOneLineNamesThem: string = theCallsTheMountSweepAsks
  .join(' and ',);

/**
 Milliseconds the two short podman calls this module makes may take.

 `podman info` is a read of the host's own store and files and answers in well under a second on every recorded gate;
 the bound is here so a store another session holds cannot hold the gate open.
 */
const podmanQueryDeadlineMilliseconds = 60_000;

/**
 The name this file is, as a refusal points a reader at it.
 */
const thisFile = 'bin/native-gate-syscall-profile.ts';

/**
 One rule of a seccomp profile: the calls it names and what it does with them.
 */
type ProfileRule = {
  /**
   What the rule does with the calls it names, such as `SCMP_ACT_ALLOW`.
   */
  readonly action?: string;
  /**
   The calls it names.
   */
  readonly names?: readonly string[];
  /**
   The capabilities the rule applies to, empty or absent when it always applies.
   */
  readonly includes?: Readonly<Record<string, unknown>>;
  /**
   The capabilities the rule skips, empty or absent when it always applies.
   */
  readonly excludes?: Readonly<Record<string, unknown>>;
};

/**
 One parsed seccomp profile, holding the rules it applies.
 */
type ProfileText = {
  /**
   The rules, in the order the profile applies them.
   */
  readonly syscalls?: readonly ProfileRule[];
};

/**
 What podman reports about the seccomp filter it applies.
 */
type ReportedSecurity = {
  /**
   Whether podman installs a filter at all.
   */
  readonly seccompEnabled?: boolean;
  /**
   The profile file podman applies, as it reports it, empty when it applies none.
   */
  readonly seccompProfilePath?: string;
};

/**
 What reading a piece of JSON answered.
 */
type ParsedJson = {
  /**
   Whether the text was JSON this reader could parse.
   */
  readonly kind: 'read' | 'unread';
  /**
   The value the text held, present when the text was JSON.
   */
  readonly value?: unknown;
  /**
   What the parser said, present when the text was not JSON.
   */
  readonly said?: string;
};

/**
 A seccomp profile with the sweep's two calls added to the rule that allows the mount family.
 */
export type DerivedSeccompProfile = {
  /**
   The profile's text, ready to be written as the file podman is given.
   */
  readonly text: string;
  /**
   The calls this derivation had to add, in the profile's order, empty when the profile already allowed all of them.
   */
  readonly added: readonly string[];
};

/**
 The seccomp profile the gate's containers are given, and how they are told about it.
 */
export type GateSyscallProfile = {
  /**
   The `podman run` arguments that pass the profile and state it.
   */
  readonly arguments: readonly string[];
  /**
   The lines the gate prints, one per fact a person needs to read the run's log.
   */
  readonly lines: readonly string[];
};

/**
 Read a piece of JSON without throwing, so a caller can refuse in its own words.

 @param text - the text to parse

 @returns the value, or what the parser said

 @example
 ```ts
 parsedJson('{"syscalls":[]}');
 ```
 */
function parsedJson(text: string,): ParsedJson {
  try {
    return {
      kind: 'read',
      value: JSON.parse(text,),
    };
  }
  catch (error) {
    return {
      kind: 'unread',
      said: String(error,),
    };
  }
}

/**
 Whether a parsed JSON value is an object this module can read a named field from.

 @param value - a value `JSON.parse` produced

 @returns true when the value is a JSON object

 @example
 ```ts
 isRecord(JSON.parse('{}')); // true
 ```
 */
function isRecord(value: unknown,): value is Record<string, unknown> {
  return ((typeof value) === 'object') && (value !== null);
}

/**
 Whether a parsed JSON value is a seccomp rule this module may read.

 @param value - one element of a profile's `syscalls` list

 @returns true when every field the derivation reads has its documented type

 @example
 ```ts
 isProfileRule({ action: 'SCMP_ACT_ALLOW', names: ['mount'] }); // true
 ```
 */
function isProfileRule(value: unknown,): value is ProfileRule {
  if (!isRecord(value,))
    return false;
  /**
   The rule's action, which is a word when it is there.
   */
  const { action, } = value;
  if ((action !== undefined) && ((typeof action) !== 'string'))
    return false;
  /**
   The rule's call names, which are words when they are there.
   */
  const { names, } = value;
  /**
   Whether the names, where they are there, are all words.
   */
  const namesAreWords: boolean = Array.isArray(names,) && names.every(function isText(name,) {
    return (typeof name) === 'string';
  },);
  if ((names !== undefined) && (!namesAreWords))
    return false;
  return [
    'includes',
    'excludes',
  ].every(function isRecordOrAbsent(field,) {
    /**
     The rule's field of this name, which is a record when it is there.
     */
    const narrowing = value[field];
    return (narrowing === undefined) || isRecord(narrowing,);
  },);
}

/**
 Whether `names` holds `mount`, which is what makes a rule the mount family's.

 @param rule - the rule to read

 @returns true when the rule names the call that marks its family

 @example
 ```ts
 namesTheMountCall({ rule: { action: 'SCMP_ACT_ALLOW', names: ['mount'] } }); // true
 ```
 */
function namesTheMountCall({ rule }: { readonly rule: ProfileRule; },): boolean {
  /**
   The calls the rule names, which is an empty list when it names none.
   */
  const held: readonly string[] = rule.names ?? [];
  return held.includes('mount',);
}

/**
 Whether a parsed JSON value is a seccomp profile this module may derive from.

 @param value - a value `JSON.parse` produced from a profile file

 @returns true when the value holds rules, each of a shape this module reads

 @example
 ```ts
 isProfileText({ syscalls: [] }); // true
 ```
 */
function isProfileText(value: unknown,): value is ProfileText {
  if (!isRecord(value,))
    return false;
  /**
   The profile's rules, which are a list of rules when they are there.
   */
  const { syscalls, } = value;
  if (syscalls === undefined)
    return true;
  return Array.isArray(syscalls,) && syscalls.every(isProfileRule,);
}

/**
 Whether a parsed JSON value is podman's report about the filter it applies.

 @param value - a value `JSON.parse` produced from podman's own output

 @returns true when the two fields the gate reads have their documented types

 @example
 ```ts
 isReportedSecurity({ seccompEnabled: true, seccompProfilePath: '/usr/share/containers/seccomp.json' }); // true
 ```
 */
function isReportedSecurity(value: unknown,): value is ReportedSecurity {
  if (!isRecord(value,))
    return false;
  /**
   Whether podman installs a filter, which is a truth value when it is reported.
   */
  const { seccompEnabled, } = value;
  if ((seccompEnabled !== undefined) && ((typeof seccompEnabled) !== 'boolean'))
    return false;
  /**
   The profile file podman applies, which is a path when it is reported.
   */
  const { seccompProfilePath, } = value;
  return (seccompProfilePath === undefined) || ((typeof seccompProfilePath) === 'string');
}

/**
 Whether one rule's `includes` and `excludes` leave it always applied.

 A rule narrowed by a capability would allow an added call only in a container holding that capability, which a gate
 container does not, so such a rule cannot carry the sweep's calls.

 @param rule - the rule to judge

 @returns true when nothing narrows the rule

 @example
 ```ts
 unconditionalRule({ rule: { action: 'SCMP_ACT_ALLOW', names: ['mount'] } }); // true
 ```
 */
function unconditionalRule({ rule }: { readonly rule: ProfileRule; },): boolean {
  /**
   The capabilities the rule requires, which an absent list leaves empty.
   */
  const required: readonly string[] = Object.keys(rule.includes ?? {},);
  /**
   The capabilities the rule skips, which an absent list leaves empty.
   */
  const skipped: readonly string[] = Object.keys(rule.excludes ?? {},);
  return (required.length === 0) && (skipped.length === 0);
}

/**
 The calls a rule names with `name` among them.

 The allow list of a containers-common profile is one rule naming several hundred calls in alphabetical order, so a name
 added at the end would be the one line of the file that reads out of order.

 @param names - the calls the rule already names

 @param name - the call to add

 @returns a new list holding every name once, in alphabetical order

 @example
 ```ts
 inAlphabeticalOrder({ names: ['mount', 'move_mount'], name: 'listmount' }); // ['listmount', 'mount', 'move_mount']
 ```
 */
function inAlphabeticalOrder({
  names,
  name,
}: {
  readonly names: readonly string[];
  readonly name: string;
},): readonly string[] {
  /**
   The names with this one among them.
   */
  const together: readonly string[] = [
    ...names,
    name,
  ];
  return together.toSorted(function inOrder(
    one: string,
    two: string,
  ) {
    return one.localeCompare(
      two,
      'en',
    );
  },);
}

/**
 The rule of `profileText` the sweep's calls are added to: the one that allows a whole family of mount calls.

 An allow rule that names `mount` is that family's rule, and only an unconditional one can carry the added calls.

 @param profileText - the profile as parsed from its file

 @param profilePath - the file it was read from, named in the refusal

 @returns the rule to add the calls to

 @example
 ```ts
 const rule = mountFamilyRule({ profileText, profilePath: '/usr/share/containers/seccomp.json' });
 ```
 */
function mountFamilyRule({
  profileText,
  profilePath,
}: {
  readonly profileText: ProfileText;
  readonly profilePath: string;
},): ProfileRule {
  /**
   The rules the profile holds, which is an empty list when it names none.
   */
  const rules: readonly ProfileRule[] = profileText.syscalls ?? [];
  /**
   The rule that allows the mount family, when the profile holds one.
   */
  const rule = rules.find(function allowsTheMountFamily(candidate,) {
    /**
     Whether the rule allows the mount family and nothing narrows it to a capability.
     */
    const allows: boolean = (candidate.action === 'SCMP_ACT_ALLOW') && namesTheMountCall({ rule: candidate, },);
    return allows && unconditionalRule({ rule: candidate, },);
  },);
  if (rule === undefined)
    throw new NativeVerificationError(
      `The seccomp profile ${profilePath} holds no unconditional SCMP_ACT_ALLOW rule that names mount, so this gate cannot tell where to allow ${theCallsAsOneLineNamesThem}; give the host's podman a containers-common profile, or teach ${thisFile} where the mount family's rule is.`,
    );
  return rule;
}

/**
 A parsed profile with one rule's names replaced.

 @param parsed - the host's profile

 @param replaced - the rule whose names change

 @param names - the names that rule then holds

 @returns the profile object to write, every other rule as parsed

 @example
 ```ts
 addedToRule({ parsed, replaced: rule, names: ['listmount', 'mount', 'statmount'] });
 ```
 */
function addedToRule({
  parsed,
  replaced,
  names,
}: {
  readonly parsed: ProfileText;
  readonly replaced: ProfileRule;
  readonly names: readonly string[];
},): ProfileText {
  /**
   The rules the profile holds, which is an empty list when it names none.
   */
  const rules: readonly ProfileRule[] = parsed.syscalls ?? [];
  /**
   Every rule, with the mount family's replaced and every other one kept as the host had it.
   */
  const syscalls: readonly ProfileRule[] = rules.map(function withNames(candidate,) {
    if (candidate !== replaced)
      return candidate;
    return {
      ...candidate,
      names,
    };
  },);
  return {
    ...parsed,
    syscalls,
  };
}

/**
 Parse a profile's text, refusing in words that name the file.

 @param profileText - the profile as read from disk

 @param profilePath - the file it came from

 @returns the profile as data

 @example
 ```ts
 parsedProfile({ profileText: await readFile(host, 'utf8'), profilePath: host });
 ```
 */
function parsedProfile({
  profileText,
  profilePath,
}: {
  readonly profileText: string;
  readonly profilePath: string;
},): ProfileText {
  /**
   What reading the file's text answered.
   */
  const read: ParsedJson = parsedJson(profileText,);
  if (read.kind === 'unread')
    throw new NativeVerificationError(`The seccomp profile ${profilePath} is not JSON this gate can read: ${read.said ?? ''}`);
  if (!isProfileText(read.value,))
    throw new NativeVerificationError(`The seccomp profile ${profilePath} is not a seccomp profile this gate can read`);
  return read.value;
}

/**
 The profile `profileText` with the sweep's calls added to its mount family's rule.

 @param profileText - the profile as read from disk

 @param profilePath - the file it came from, named in every refusal

 @returns the profile's text and the calls that had to be added

 @example
 ```ts
 const derived = derivedSeccompProfile({ profileText: await readFile(host, 'utf8'), profilePath: host });
 ```
 */
export function derivedSeccompProfile({
  profileText,
  profilePath,
}: {
  readonly profileText: string;
  readonly profilePath: string;
},): DerivedSeccompProfile {
  /**
   The host's own profile, as data.
   */
  const parsed: ProfileText = parsedProfile({
    profileText,
    profilePath,
  },);
  /**
   The rule the two calls are added to.
   */
  const rule: ProfileRule = mountFamilyRule({
    profileText: parsed,
    profilePath,
  },);
  /**
   The calls that rule lacks.
   */
  const missing: readonly string[] = theCallsTheMountSweepAsks.filter(function lacks(call,) {
    /**
     The calls the rule holds, which is an empty list when it names none.
     */
    const held: readonly string[] = rule.names ?? [];
    return !held.includes(call,);
  },);
  if (missing.length === 0)
    return {
      text: profileText,
      added: [],
    };
  /**
   The names the rule holds, which is an empty list when it names none.
   */
  const held: readonly string[] = rule.names ?? [];
  /**
   Every name the rule holds, with the missing calls in their alphabetical places.
   */
  const names: readonly string[] = missing.reduce(
    function withCall(
      soFar,
      call,
    ) {
      return inAlphabeticalOrder({
        names: soFar,
        name: call,
      },);
    },
    held,
  );
  return {
    text: `${JSON.stringify(
      addedToRule({
        parsed,
        replaced: rule,
        names,
      },),
      undefined,
      '\t',
    )}\n`,
    added: missing,
  };
}

/**
 What podman reports about the seccomp filter it applies.

 @param environment - environment the podman client runs in

 @returns the report, as podman printed it

 @example
 ```ts
 const reported = await reportedSecurity({ environment: process.env });
 ```
 */
async function reportedSecurity({ environment }: { readonly environment: NodeJS.ProcessEnv; },): Promise<ReportedSecurity> {
  /**
   The client's report, captured so it can be parsed.
   */
  const reported = await runCommand({
    command: 'podman',
    args: [
      'info',
      '--format',
      '{{json .Host.Security}}',
    ],
    capture: true,
    deadlineMilliseconds: podmanQueryDeadlineMilliseconds,
    env: environment,
  },);
  /**
   What the client printed, without the newline `podman` ends it with.
   */
  const printed: string = reported.stdout
    .trim();
  /**
   What reading that text answered.
   */
  const read: ParsedJson = parsedJson(printed,);
  if (read.kind === 'unread')
    throw new NativeVerificationError(`podman reported its security settings in a form this gate cannot read: ${printed} (${read.said ?? ''})`);
  if (!isReportedSecurity(read.value,))
    throw new NativeVerificationError(`podman reported its security settings in a form this gate cannot read: ${printed}`);
  return read.value;
}

/**
 The seccomp profile that lets the gate's containers make the calls the sweep makes, written beside the run's evidence.

 @param evidenceDirectory - the directory this run keeps its evidence in, which receives the derived profile and a copy
   of the host's own

 @param environment - environment the podman client runs in, which defaults to this process's

 @returns the `podman run` arguments that pass the profile and state it, and the lines to print

 @example
 ```ts
 const profile = await syscallProfileForTheGateContainers({ evidenceDirectory: evidence });
 const limits = [...baseLimits, ...profile.arguments];
 ```
 */
export async function syscallProfileForTheGateContainers({
  evidenceDirectory,
  environment = process.env,
}: {
  readonly evidenceDirectory: string;
  readonly environment?: NodeJS.ProcessEnv;
},): Promise<GateSyscallProfile> {
  /**
   What podman reported about the filter it applies.
   */
  const reported: ReportedSecurity = await reportedSecurity({ environment, },);
  /**
   The statement that the containers allow the calls, which every arm carries.
   */
  const statement: readonly string[] = [
    '--env',
    `${statmountAllowedVariable}=1`,
  ];
  if (reported.seccompEnabled !== true) {
    return {
      arguments: statement,
      lines: [`This host's podman applies no seccomp profile, so the gate's containers already allow ${theCallsAsOneLineNamesThem}.`],
    };
  }
  /**
   The profile podman applies, which it reported.
   */
  const hostProfilePath: string = reported.seccompProfilePath ?? '';
  if (hostProfilePath === '')
    throw new NativeVerificationError(
      `podman reports that it applies a seccomp filter and names no profile file, so this gate cannot derive one that allows ${theCallsAsOneLineNamesThem}; state the profile in the host's containers.conf, or teach ${thisFile} where podman keeps it.`,
    );
  /**
   The host's own profile, as it is on disk.
   */
  const hostProfileText: string = await readFile(
    hostProfilePath,
    'utf8',
  );
  /**
   The host's profile with the sweep's calls added.
   */
  const derived: DerivedSeccompProfile = derivedSeccompProfile({
    profileText: hostProfileText,
    profilePath: hostProfilePath,
  },);
  // The host's own profile is copied beside the derived one, so the delta the gate's containers ran under stays auditable
  // from the run's evidence after the host's containers-common moved on.
  await copyFile(
    hostProfilePath,
    join(
      evidenceDirectory,
      'host-seccomp.json',
    ),
  );
  /**
   The calls the derivation had to add, which is none when the host already allowed them.
   */
  const added: readonly string[] = derived.added;
  if (added.length === 0) {
    return {
      arguments: [
        '--security-opt',
        `seccomp=${resolve(hostProfilePath,)}`,
        ...statement,
      ],
      lines: [`The host's seccomp profile ${hostProfilePath} already allows ${theCallsAsOneLineNamesThem}, so the gate's containers are given it as it is.`],
    };
  }
  /**
   The file the gate's containers are given.
   */
  const profilePath: string = resolve(
    join(
      evidenceDirectory,
      'gate-container-seccomp.json',
    ),
  );
  await writeFile(
    profilePath,
    derived.text,
  );
  /**
   The calls this derivation added, as one line names them.
   */
  const addedAsOneLineNamesThem: string = added
    .join(' and ',);
  return {
    arguments: [
      '--security-opt',
      `seccomp=${profilePath}`,
      ...statement,
    ],
    lines: [
      `The gate's containers allow ${theCallsAsOneLineNamesThem} through ${profilePath}, which is the host's seccomp profile ${hostProfilePath} with ${addedAsOneLineNamesThem} added to the rule that names mount.`,
    ],
  };
}
