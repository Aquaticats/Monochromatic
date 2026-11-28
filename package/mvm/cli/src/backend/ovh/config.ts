/**
 OVHcloud backend configuration: the five names, the endpoint, regions, flavors, and names.

 Every value comes from the environment, as the Hetzner backend reads
 `HCLOUD_TOKEN` from it. The five key names are the ones OVHcloud's own clients
 read (`OVH_ENDPOINT`, `OVH_APPLICATION_KEY`, `OVH_APPLICATION_SECRET`,
 `OVH_CONSUMER_KEY`; `README.rst` of `github.com/ovh/python-ovh`, lines 402-403
 of the clone read on 2026-11-28) plus this tool's own name for the project,
 `MVM_OVH_PROJECT_ID`.

 No function here reads a key's value into a message, a log line or an error;
 a missing name is reported by its name alone.

 @module
 */

import { join, } from 'node:path';

import {
  DATA_DIR,
  VM_PREFIX,
} from '../../config.ts';
import { parseToolCommand, } from '../../libvirt-tools.ts';

//region Key names

/**
 Environment variable naming the API host: `ovh-ca`, `ovh-eu` or `ovh-us`.
 */
export const ENDPOINT_ENV = 'OVH_ENDPOINT';

/**
 Environment variable holding the application key, the `AK` of OVHcloud's guide.
 */
export const APPLICATION_KEY_ENV = 'OVH_APPLICATION_KEY';

/**
 Environment variable holding the application secret, the `AS` of OVHcloud's guide.
 */
export const APPLICATION_SECRET_ENV = 'OVH_APPLICATION_SECRET';

/**
 Environment variable holding the consumer key, the `CK` of OVHcloud's guide.
 */
export const CONSUMER_KEY_ENV = 'OVH_CONSUMER_KEY';

/**
 Environment variable naming the Public Cloud project every call is scoped to.
 */
export const PROJECT_ID_ENV = 'MVM_OVH_PROJECT_ID';

//endregion Key names

//region Endpoints

/**
 API roots by the names OVHcloud's own clients accept, from their `ENDPOINTS`
 table (`github.com/ovh/python-ovh`, `ovh/client.py`, lines 66-75 of the clone
 read on 2026-11-28), minus the trailing `/1.0` this backend does not use.

 @example
 ```ts
 ENDPOINT_ROOTS['ovh-ca']; // 'https://ca.api.ovh.com'
 ```
 */
export const ENDPOINT_ROOTS = {
  'ovh-ca': 'https://ca.api.ovh.com',
  'ovh-eu': 'https://eu.api.ovh.com',
  'ovh-us': 'https://api.us.ovhcloud.com',
} as const;

/**
 Name of one API host, the names OVHcloud's clients accept.
 */
export type OvhEndpointName = keyof typeof ENDPOINT_ROOTS;

/**
 Endpoint used when neither the environment nor a caller names one: Canada,
 the region the owner's decision names.
 */
export const DEFAULT_ENDPOINT: OvhEndpointName = 'ovh-ca';

/**
 Prefix of a complete API root, which is accepted in place of a name.
 */
const URL_PREFIX = 'https://';

//endregion Endpoints

//region Tunables

/**
 Environment variable overriding the ordered region fallback list.
 */
export const REGIONS_ENV = 'MVM_OVH_REGIONS';

/**
 Environment variable overriding the flavor name or identifier.
 */
export const FLAVOR_ENV = 'MVM_OVH_FLAVOR';

/**
 Environment variable moving where the managed SSH keypair lives.
 */
export const DATA_DIR_ENV = 'MVM_OVH_DATA_DIR';

/**
 Environment variable holding the command that runs `ssh`, as a JSON array.
 */
export const SSH_COMMAND_ENV = 'MVM_OVH_SSH_COMMAND';

/**
 Environment variable holding the command that runs `scp`, as a JSON array.
 */
export const SCP_COMMAND_ENV = 'MVM_OVH_SCP_COMMAND';

/**
 Image shorthand used when `--image` is absent.
 */
export const DEFAULT_IMAGE = 'ubuntu';

/**
 ISO 3166-1 alpha-2 code, lowercase as the schema's `countryCode2LettersEnum`
 spells it, of the country whose regions the default fallback list holds.
 */
export const CANADA_COUNTRY_CODE = 'ca';

/**
 The one region status a create may be attempted in, from the schema's
 `cloud.RegionStatusEnum`.
 */
export const REGION_STATUS_UP = 'UP';

/**
 Name of the service a region must offer to create an instance, as the
 `services` of the schema's `cloud.Region` carry it. The schema types the name
 as a bare string, so this value is from a live project's answer of 2026-11-28:
 every region that listed it answered the flavor list, and every region that
 did not answered 404 to it.
 */
export const REGION_INSTANCE_SERVICE = 'instance';

/**
 The one service status a create may rely on, from the schema's
 `cloud.ServiceStatusEnum`.
 */
export const SERVICE_STATUS_UP = 'UP';

/**
 Advice shown when an OpenSSH executable the ovh backend runs does not exist.
 */
export const OPENSSH_REMEDY =
  'The ovh backend needs an OpenSSH client (`ssh`, `scp` and `ssh-keygen`), version 9.0 or newer, on PATH; install the host package that provides it.';

/**
 Directory holding the managed SSH keypair, unless moved.
 */
export const OVH_DATA_DIR: string = join(
  DATA_DIR,
  'ovh',
);

/**
 Where the managed SSH keypair lives now: {@link OVH_DATA_DIR}, or the path
 {@link DATA_DIR_ENV} names.
 
 @returns absolute directory path
 
 @example
 ```ts
 const directory = ovhDataDir();
 ```
 */
export function ovhDataDir(): string {
  /**
   Override from the environment, when one is set and non-empty.
   */
  const override = process.env[DATA_DIR_ENV];
  return ((override === undefined) || (override === '')) ? OVH_DATA_DIR : override;
}

//endregion Tunables

//region Keys

/**
 What one call to OVHcloud's cloud API needs besides its path.

 @example
 ```ts
 const credentials: OvhCredentials = requireCredentials();
 ```
 */
export type OvhCredentials = {
  /**
   The `AK`, sent as `X-Ovh-Application`.
   */
  readonly applicationKey: string;
  /**
   The `AS`, part of the signature and never sent.
   */
  readonly applicationSecret: string;
  /**
   The `CK`, sent as `X-Ovh-Consumer` and part of the signature.
   */
  readonly consumerKey: string;
  /**
   Root of the API host, without a version segment: `https://ca.api.ovh.com`.
   */
  readonly endpointRoot: string;
  /**
   Identifier of the one project every call is scoped to.
   */
  readonly projectId: string;
};

/**
 Reads one required name, reporting the name and never the value.
 
 @param variable - name of the environment variable
 
 @param remedy - what to do about it, shown after the name
 
 @returns the value
 
 @throws Error naming `variable` when it is unset or empty
 
 @example
 ```ts
 const projectId = requireName({ remedy: 'Set it.', variable: PROJECT_ID_ENV });
 ```
 */
function requireName({
  remedy,
  variable,
}: {
  readonly variable: string;
  readonly remedy: string;
}): string {
  /**
   Value of `variable`, validated for presence before it is returned.
   */
  const value = process.env[variable];
  if ((value === undefined) || (value === '')) {
    throw new Error(`${variable} is required for the ovh backend. ${remedy}`,);
  }
  return value;
}

/**
 Resolves the API root from a name, a complete `https://` root, or nothing.
 
 @param raw - value of {@link ENDPOINT_ENV}, or `undefined`
 
 @returns API root without a version segment
 
 @throws Error naming the accepted forms when `raw` is neither
 
 @example
 ```ts
 resolveEndpointRoot(undefined);  // 'https://ca.api.ovh.com'
 resolveEndpointRoot('ovh-eu');   // 'https://eu.api.ovh.com'
 ```
 */
export function resolveEndpointRoot(raw?: string,): string {
  if ((raw === undefined) || (raw === '')) {
    return ENDPOINT_ROOTS[DEFAULT_ENDPOINT];
  }
  if (isEndpointName(raw,)) {
    return ENDPOINT_ROOTS[raw];
  }
  if (raw.startsWith(URL_PREFIX,)) {
    return raw.endsWith('/',) ? raw.slice(
      0,
      -1,
    ) : raw;
  }
  throw new Error(
    `${ENDPOINT_ENV} must name an OVHcloud API host (${Object.keys(ENDPOINT_ROOTS,)
      .join(', ',)}) or be a complete https:// API root; it is set to "${raw}".`,
  );
}

/**
 Type guard: whether `value` names an API host this backend knows.
 
 @param value - candidate endpoint name
 
 @returns whether `value` is one of {@link ENDPOINT_ROOTS}'s keys
 
 @example
 ```ts
 isEndpointName('ovh-ca'); // true
 isEndpointName('https://ca.api.ovh.com'); // false
 ```
 */
export function isEndpointName(value: string,): value is OvhEndpointName {
  return Object.hasOwn(
    ENDPOINT_ROOTS,
    value,
  );
}

/**
 Reads everything the API layer needs, requiring the names in a fixed order so
 the first missing one is the one reported: the application key, then the
 application secret, then the consumer key, then the project.
 
 @returns the keys, the API root and the project
 
 @throws Error naming the first missing variable
 
 @example
 ```ts
 const credentials = requireCredentials();
 ```
 */
export function requireCredentials(): OvhCredentials {
  /**
   API root; {@link ENDPOINT_ENV} has a default, so this never reports a missing name.
   */
  const endpointRoot = resolveEndpointRoot(process.env[ENDPOINT_ENV],);
  /**
   The three keys OVHcloud issues together, then this tool's project name.
   */
  const applicationKey = requireName({
    remedy: 'Create a script key on the OVHcloud API page for your endpoint and store the three keys with `mise run --raw secrets:edit`.',
    variable: APPLICATION_KEY_ENV,
  },);
  /**
   The `AS`.
   */
  const applicationSecret = requireName({
    remedy: `It is issued beside ${APPLICATION_KEY_ENV}; store it with \`mise run --raw secrets:edit\`.`,
    variable: APPLICATION_SECRET_ENV,
  },);
  /**
   The `CK`.
   */
  const consumerKey = requireName({
    remedy: `It is issued beside ${APPLICATION_KEY_ENV}; store it with \`mise run --raw secrets:edit\`.`,
    variable: CONSUMER_KEY_ENV,
  },);
  /**
   The project every path is scoped to.
   */
  const projectId = requireName({
    remedy: 'It is the Public Cloud project identifier from the OVHcloud Control Panel; store it with `mise run --raw secrets:edit`.',
    variable: PROJECT_ID_ENV,
  },);
  return {
    applicationKey,
    applicationSecret,
    consumerKey,
    endpointRoot,
    projectId,
  };
}

//endregion Keys

//region Overrides

/**
 Resolves an explicit flavor override from the per-invocation value or the
 environment. Returns `''` when neither is set, signalling the caller to pick
 the smallest available flavor of the region at call time.
 
 @param override - per-invocation flavor name or identifier, or `undefined`
 
 @returns the explicit flavor name or identifier, or `''` for "smallest"
 
 @example
 ```ts
 flavorOverride('b3-8');     // 'b3-8'
 flavorOverride(undefined);  // '' unless MVM_OVH_FLAVOR is set
 ```
 */
export function flavorOverride(override?: string,): string {
  /**
   Explicit override wins over the environment; absence yields `''`.
   */
  const candidate = override ?? process.env[FLAVOR_ENV];
  return ((candidate === undefined) || (candidate === '')) ? '' : candidate;
}

/**
 Parses a comma-separated region series into a clean ordered list.
 
 @param raw - comma-separated regions, or `undefined`
 
 @returns ordered list of non-empty region names (empty when none parse)
 
 @example
 ```ts
 parseRegions('GRA7, BHS5'); // ['GRA7', 'BHS5']
 parseRegions('');           // []
 ```
 */
export function parseRegions(raw?: string,): readonly string[] {
  if ((raw === undefined) || (raw === '')) {
    return [];
  }
  return raw.split(',',)
    .map(function trimRegion(part,) {
      return part.trim();
    },)
    .filter(function keepNonEmpty(part,) {
      return part !== '';
    },);
}

/**
 Region list named by the environment, when one is set.
 
 @returns the named regions in order, or an empty list
 
 @example
 ```ts
 envRegions(); // ['BHS5'] when MVM_OVH_REGIONS=BHS5
 ```
 */
export function envRegions(): readonly string[] {
  return parseRegions(process.env[REGIONS_ENV],);
}

/**
 The command that runs one OpenSSH tool, from its variable or the bare name.
 
 @param envName - variable holding a JSON array, as `MVM_VIRSH_COMMAND` does
 
 @param fallback - executable name used when the variable is unset
 
 @returns executable followed by its leading arguments
 
 @throws {@link ToolCommandConfigError} when the variable is not a JSON array of non-empty strings
 
 @example
 ```ts
 readToolCommand({ envName: SSH_COMMAND_ENV, fallback: 'ssh' }); // ['ssh']
 ```
 */
function readToolCommand({
  envName,
  fallback,
}: {
  readonly envName: string;
  readonly fallback: string;
}): readonly [
  string,
  ...string[],
] {
  /**
   Value of the variable, `''` when unset.
   */
  const value = process.env[envName] ?? '';
  if (value === '') {
    return [fallback,];
  }
  return parseToolCommand({
    example: JSON.stringify([fallback,],),
    value,
    variable: envName,
  },);
}

/**
 The command that runs `ssh`: {@link SSH_COMMAND_ENV} or `ssh`.
 
 @returns executable followed by its leading arguments
 
 @example
 ```ts
 sshCommand(); // ['ssh']
 ```
 */
export function sshCommand(): readonly [
  string,
  ...string[],
] {
  return readToolCommand({
    envName: SSH_COMMAND_ENV,
    fallback: 'ssh',
  },);
}

/**
 The command that runs `scp`: {@link SCP_COMMAND_ENV} or `scp`.
 
 @returns executable followed by its leading arguments
 
 @example
 ```ts
 scpCommand(); // ['scp']
 ```
 */
export function scpCommand(): readonly [
  string,
  ...string[],
] {
  return readToolCommand({
    envName: SCP_COMMAND_ENV,
    fallback: 'scp',
  },);
}

//endregion Overrides

//region Name validation

/**
 Longest single DNS label (RFC 1123); the full `mvm-<name>` instance name must
 fit, because OVHcloud turns an instance name into its hostname.
 */
const MAX_LABEL_LENGTH = 63;

/**
 Checks whether `c` is allowed in a hostname label body.
 
 @param c - single-character string to inspect
 
 @returns whether `c` is `[A-Za-z0-9-]`
 
 @example
 ```ts
 isHostnameChar('a'); // true
 isHostnameChar('_'); // false
 ```
 */
function isHostnameChar(c: string,): boolean {
  return ((c >= 'a') && (c <= 'z'))
    || ((c >= 'A') && (c <= 'Z'))
    || ((c >= '0') && (c <= '9'))
    || (c === '-');
}

/**
 Validates a VM name so the resulting `mvm-<name>` is a valid RFC 1123 hostname.
 
 The schema constrains `name` only to a string; the rule is mvm's own, taken
 unchanged from the Hetzner backend's, because the name becomes the machine's
 hostname and mvm's `list` prefix match must be unambiguous.
 
 @param name - candidate VM name without the mvm- prefix
 
 @throws Error when the resulting hostname is invalid
 
 @example
 ```ts
 validateOvhName('dev-01'); // OK
 validateOvhName('dev_01'); // throws (underscore)
 ```
 */
export function validateOvhName(name: string,): void {
  /**
   Full instance name; the constraints below apply to this single label.
   */
  const fullName = `${VM_PREFIX}${name}`;
  if (name.length === 0) {
    throw new Error('invalid VM name: must not be empty',);
  }
  if (fullName.length > MAX_LABEL_LENGTH) {
    throw new Error(
      `invalid VM name "${name}": instance name ${fullName} exceeds ${String(MAX_LABEL_LENGTH,)} characters`,
    );
  }
  if (name.startsWith('-',) || name.endsWith('-',)) {
    throw new Error(
      `invalid VM name "${name}": must not start or end with a hyphen`,
    );
  }
  for (const c of name) {
    if (!isHostnameChar(c,)) {
      throw new Error(
        `invalid VM name "${name}": only letters, digits, and hyphens are allowed (no underscores or periods) for OVHcloud instance names`,
      );
    }
  }
}

//endregion Name validation
