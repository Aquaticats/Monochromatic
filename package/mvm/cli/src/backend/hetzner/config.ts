/**
 Hetzner backend configuration: token, defaults, labels, and name validation.
 
 Reads the API token and tunables from the environment, and validates VM names
 against Hetzner's RFC 1123 hostname rule (stricter than the libvirt name
 check, which permits underscores and trailing hyphens).
 
 @module
 */

import { join, } from 'node:path';

import {
  DATA_DIR,
  VM_PREFIX,
} from '../../config.ts';

//region Token

/**
 Environment variable holding the Hetzner Cloud API token.
 */
export const HCLOUD_TOKEN_ENV = 'HCLOUD_TOKEN';

/**
 Reads the Hetzner API token from the environment.
 
 @returns API token string
 
 @throws Error when {@link HCLOUD_TOKEN_ENV} is unset or empty
 
 @example
 ```ts
 const token = requireToken();
 ```
 */
export function requireToken(): string {
  /**
   Raw token from the environment; validated for presence before return.
   */
  const token = process.env[HCLOUD_TOKEN_ENV];
  if ((token === undefined) || (token === '')) {
    throw new Error(
      `${HCLOUD_TOKEN_ENV} is required for the hetzner backend. Create a token in the Hetzner Cloud Console and export it.`,
    );
  }
  return token;
}

//endregion Token

//region Host tools

/**
 Advice shown when an OpenSSH executable the hetzner backend runs does not exist.
 */
export const OPENSSH_REMEDY =
  'The hetzner backend needs an OpenSSH client (`ssh`, `scp` and `ssh-keygen`), version 9.0 or newer, on PATH; install the host package that provides it.';

//endregion Host tools

//region Defaults and labels

/**
 Environment variable overriding the default server type.
 */
export const SERVER_TYPE_ENV = 'MVM_HCLOUD_SERVER_TYPE';


/**
 Environment variable overriding the default location fallback list.
 */
export const LOCATIONS_ENV = 'MVM_HCLOUD_LOCATIONS';

/**
 Default ordered location fallback list: the three EU locations, which share
 the default server type's availability. Tried in order on out-of-stock.
 */
export const DEFAULT_LOCATIONS: readonly string[] = [
  'fsn1',
  'nbg1',
  'hel1',
];

/**
 Default image shorthand when none is supplied.
 */
export const DEFAULT_IMAGE = 'ubuntu';

/**
 Per-VM directory holding the managed SSH keypair for the Hetzner backend.
 */
export const HETZNER_DATA_DIR: string = join(
  DATA_DIR,
  'hetzner',
);

/**
 Label key applied to every mvm-managed Hetzner resource.
 */
export const MVM_LABEL_KEY = 'mvm';

/**
 Label value applied to every mvm-managed Hetzner resource.
 */
export const MVM_LABEL_VALUE = 'true';

/**
 Label selector matching every mvm-managed resource; the basis for safe
 list and bulk-delete operations.
 */
export const MVM_LABEL_SELECTOR: string = `${MVM_LABEL_KEY}=${MVM_LABEL_VALUE}`;

/**
 Login user for Hetzner cloud images.
 */
export const SSH_USER = 'root';

/**
 Resolves an explicit server-type override from the per-invocation value or
 the environment. Returns `''` when neither is set, signalling the caller to
 pick the cheapest non-deprecated type dynamically (there is no hardcoded
 default, since type slugs are deprecated over time).
 
 @param override - per-invocation server type, or `undefined`
 
 @returns the explicit server type slug, or `''` for "auto (cheapest)"
 
 @example
 ```ts
 serverTypeOverride('cpx21'); // 'cpx21'
 serverTypeOverride(undefined); // '' (auto) unless MVM_HCLOUD_SERVER_TYPE is set
 ```
 */
export function serverTypeOverride(override?: string,): string {
  /**
   Explicit override wins over env; absence yields `''` (auto).
   */
  const candidate = override ?? process.env[SERVER_TYPE_ENV];
  return ((candidate === undefined) || (candidate === '')) ? '' : candidate;
}

/**
 Parses a comma-separated location series into a clean ordered list.
 
 @param raw - comma-separated locations, or `undefined`
 
 @returns ordered list of non-empty location codes (empty when none parse)
 
 @example
 ```ts
 parseLocations('fsn1, nbg1'); // ['fsn1', 'nbg1']
 parseLocations('');           // []
 ```
 */
function parseLocations(raw?: string,): readonly string[] {
  if ((raw === undefined) || (raw === '')) {
    return [];
  }
  return raw.split(',',)
    .map(function trimLocation(part,) {
      return part.trim();
    },)
    .filter(function keepNonEmpty(part,) {
      return part !== '';
    },);
}

/**
 Resolves the ordered location fallback list from an explicit override, the
 environment, or the built-in default.
 
 @param override - per-invocation comma-separated locations, or `undefined`
 
 @returns ordered location fallback list (never empty)
 
 @example
 ```ts
 resolveLocations('ash,hil'); // ['ash', 'hil']
 resolveLocations(undefined); // ['fsn1', 'nbg1', 'hel1']
 ```
 */
export function resolveLocations(override?: string,): readonly string[] {
  /**
   Locations from the explicit per-invocation override, when any.
   */
  const fromOverride = parseLocations(override,);
  if (fromOverride.length > 0) {
    return fromOverride;
  }
  /**
   Locations from the environment, when the override was empty.
   */
  const fromEnv = parseLocations(process.env[LOCATIONS_ENV],);
  return (fromEnv.length > 0) ? fromEnv : DEFAULT_LOCATIONS;
}

//endregion Defaults and labels

//region Name validation

/**
 Maximum length of a single DNS label (RFC 1123); the full `mvm-<name>` server
 name must fit within it since it is one label.
 */
const MAX_LABEL_LENGTH = 63;

/**
 Checks whether `c` is allowed in a hostname label body: ASCII alphanumeric or
 hyphen. Periods (multi-label) and underscores are rejected.
 
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
 Validates a VM name so the resulting `mvm-<name>` is a valid RFC 1123
 hostname for Hetzner: non-empty, only `[A-Za-z0-9-]`, no leading or trailing
 hyphen, and within the single-label length limit. Stricter than the libvirt
 name check, which allows underscores and trailing hyphens.
 
 @param name - candidate VM name without the mvm- prefix
 
 @throws Error when the resulting hostname is invalid
 
 @example
 ```ts
 validateHetznerName('dev-01'); // OK
 validateHetznerName('dev_01'); // throws (underscore)
 validateHetznerName('dev-');   // throws (trailing hyphen)
 ```
 */
export function validateHetznerName(name: string,): void {
  /**
   Full Hetzner server name; the constraints below apply to this single label.
   */
  const fullName = `${VM_PREFIX}${name}`;
  if (name.length === 0) {
    throw new Error('invalid VM name: must not be empty',);
  }
  if (fullName.length > MAX_LABEL_LENGTH) {
    throw new Error(
      `invalid VM name "${name}": server name ${fullName} exceeds ${String(MAX_LABEL_LENGTH,)} characters`,
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
        `invalid VM name "${name}": only letters, digits, and hyphens are allowed (no underscores or periods) for Hetzner hostnames`,
      );
    }
  }
}

//endregion Name validation
