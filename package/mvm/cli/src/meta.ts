import {
  readFile,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';
import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import { tagged, } from '@monochromatic-dev/logger-core/ts';

import {
  InvalidVmMetaError,
  UnreadableLegacyImageError,
  UnreadableVmMetaError,
} from './meta-errors.ts';
import { isAbsentPathError, } from './path-exists.ts';
import {
  CUSTOM_GUEST_DEFAULTS,
  DEFAULT_IMAGE,
  type GuestConfig,
  type OsFamily,
  resolveImage,
} from './registry.ts';

/**
 Logger root for mvm after removing the package log shim.
 
 @example
 ```ts
 const rl = tagged({ tag: someFunction.name, l, },);
 ```
 */
const l = tagged({ tag: 'mvm', },);

//region VM metadata type

/**
 Persistent metadata stored alongside each VM in `meta.json`.
 Captures guest OS properties needed by exec and clone without
 re-resolving through the image registry.
 
 @example
 ```ts
 const meta: VmMeta = await readVmMeta('/home/user/.local/share/mvm/vms/dev-01');
 meta.osFamily; // => 'linux'
 ```
 */
export type VmMeta = {
  /**
   Registry image identifier (e.g. `ubuntu`, `windows`, or custom name).
   */
  image: string;
  /**
   OS family discriminant for exec shell and domain XML dispatch.
   */
  osFamily: OsFamily;
  /**
   Shell executable path or name for guest-exec commands.
   */
  shell: string;
  /**
   Default login user for this guest OS.
   */
  defaultUser: string;
  /**
   ISO 8601 timestamp of VM creation.
   */
  createdAt: string;
  /**
   How files move between host and guest: the virtiofs shared directory, or the
   guest agent for a VM defined without the share. Absent in metadata written
   before the guest agent route existed, which means the shared directory.
   */
  fileTransfer?: FileTransferRoute;
};

/**
 Channel a VM's files travel through.

 @example
 ```ts
 const route: FileTransferRoute = 'guest-agent';
 ```
 */
export type FileTransferRoute = 'guest-agent' | 'virtiofs';

//endregion VM metadata type

//region Write

/**
 Writes VM metadata to `meta.json` in the VM directory.
 Also writes the legacy `image` text file for backwards compatibility.
 
 @param fileTransfer - How files move between host and guest; defaults to the virtiofs shared directory
 
 @param guest - Guest config for OS family, shell, and default user
 
 @param image - Registry image identifier
 
 @param vmDir - VM directory to write metadata into
 
 @example
 ```ts
 await writeVmMeta({
 
   vmDir: '/home/user/.local/share/mvm/vms/dev-01',
 
   image: 'ubuntu',
 
   guest: IMAGES['ubuntu'],
 });
 ```
 */
export async function writeVmMeta({
  fileTransfer = 'virtiofs',
  guest,
  image,
  vmDir,
}: {
  readonly fileTransfer?: FileTransferRoute;
  readonly guest: GuestConfig;
  readonly image: string;
  readonly vmDir: string;
},): Promise<void> {
  /**
   Logger scoped to this writer so legacy-file fallbacks log with context.
   */
  const rl = tagged({
    tag: writeVmMeta.name,
    l,
  },);
  /**
   Metadata record persisted to `meta.json`; captures the guest config snapshot at creation time.
   */
  const meta: VmMeta = {
    createdAt: new Date().toISOString(),
    defaultUser: guest.defaultUser,
    fileTransfer,
    image,
    osFamily: guest.osFamily,
    shell: guest.shell,
  };

  /**
   Path of the metadata file inside the VM directory; chosen once and reused in the debug log.
   */
  const metaPath = join(
    vmDir,
    'meta.json',
  );
  await writeFile(
    metaPath,
    JSON.stringify(
      meta,
      null,
      2,
    ),
  );
  rl.debug(`wrote VM metadata to ${metaPath}`,);

  // Legacy compatibility: also write the image text file
  await writeFile(
    join(
      vmDir,
      'image',
    ),
    image,
  );
}

//endregion Write

//region Record validation

/**
 What a parsed `meta.json` value provided, or why it cannot be used.
 */
type MetaVerdict =
  | {
    readonly meta: VmMeta;
    readonly usable: true;
  }
  | {
    readonly problem: string;
    readonly usable: false;
  };

/**
 Reads a field a metadata record must carry as non-empty text.

 @param field - field name, named in the refusal that follows a miss

 @param parsed - parsed record

 @returns the field's text, or `undefined` when it is missing, not text, or empty

 @example
 ```ts
 requiredText({ field: 'image', parsed: { image: 'ubuntu' } }); // 'ubuntu'
 ```
 */
function requiredText({
  field,
  parsed,
}: {
  readonly field: string;
  readonly parsed: object;
},): string | undefined {
  if (!(field in parsed))
    return undefined;
  /**
   Value the record carries under this field, usable only when it is non-empty text.
   */
  const value: unknown = Reflect.get(
    parsed,
    field,
  );
  if (((typeof value) !== 'string') || (value === ''))
    return undefined;
  return value;
}

/**
 Reads a metadata record into the fields its callers use, or says why the
 value is not one.

 The record is rebuilt from the fields this function checked, so a caller never
 receives metadata whose fields were never tested: a corrupt record would
 otherwise decide the exec shell and the file-transfer route from `undefined`.

 @param parsed - value `JSON.parse` produced from `meta.json`

 @returns the metadata the record holds, or what it fails to provide

 @example
 ```ts
 metaVerdict({});
 // => { usable: false, problem: 'the field `createdAt` is missing, not text, or empty' }
 ```
 */
function metaVerdict(parsed: unknown,): MetaVerdict {
  if (((typeof parsed) !== 'object') || (parsed === null)) {
    return {
      problem: `it holds ${typeof parsed}, not a JSON object`,
      usable: false,
    };
  }

  /**
   Creation timestamp the record carries; only ever reported back to the caller.
   */
  const createdAt = requiredText({
    field: 'createdAt',
    parsed,
  },);
  if (createdAt === undefined) {
    return {
      problem: 'the field `createdAt` is missing, not text, or empty',
      usable: false,
    };
  }

  /**
   Login user the record carries; decides which account a guest command runs as.
   */
  const defaultUser = requiredText({
    field: 'defaultUser',
    parsed,
  },);
  if (defaultUser === undefined) {
    return {
      problem: 'the field `defaultUser` is missing, not text, or empty',
      usable: false,
    };
  }

  /**
   Image identifier the record carries; named when a VM is listed or cloned.
   */
  const image = requiredText({
    field: 'image',
    parsed,
  },);
  if (image === undefined) {
    return {
      problem: 'the field `image` is missing, not text, or empty',
      usable: false,
    };
  }

  /**
   Shell the record carries; decides the interpreter a guest command runs under.
   */
  const shell = requiredText({
    field: 'shell',
    parsed,
  },);
  if (shell === undefined) {
    return {
      problem: 'the field `shell` is missing, not text, or empty',
      usable: false,
    };
  }

  /**
   OS family the record carries, which dispatches exec and domain XML.
   */
  const osFamily = requiredText({
    field: 'osFamily',
    parsed,
  },);
  if ((osFamily !== 'linux') && (osFamily !== 'windows')) {
    return {
      problem: `the field \`osFamily\` is ${String(osFamily,)} rather than linux or windows`,
      usable: false,
    };
  }

  /**
   File-transfer route the record carries; absent in metadata written before
   the guest agent route existed, which means the shared directory.
   */
  const fileTransfer: unknown = ('fileTransfer' in parsed)
    ? Reflect.get(
      parsed,
      'fileTransfer',
    )
    : undefined;
  if (
    !((fileTransfer === undefined)
      || (fileTransfer === 'guest-agent')
      || (fileTransfer === 'virtiofs'))
  ) {
    return {
      problem: `the field \`fileTransfer\` is ${String(fileTransfer,)} rather than guest-agent or virtiofs`,
      usable: false,
    };
  }

  return {
    meta: {
      createdAt,
      defaultUser,
      ...(fileTransfer === undefined ? {} : { fileTransfer, }),
      image,
      osFamily,
      shell,
    },
    usable: true,
  };
}

//endregion Record validation

//region Read

/**
 Path of a VM's metadata file, named once so every message and probe agrees.
 
 @param vmDir - Absolute path to the VM directory
 
 @returns Absolute path of `meta.json`
 
 @example
 ```ts
 vmMetaPath('/home/user/.local/share/mvm/vms/dev-01'); // '.../dev-01/meta.json'
 ```
 */
function vmMetaPath(vmDir: string,): string {
  return join(
    vmDir,
    'meta.json',
  );
}

/**
 Reads the VM's `meta.json`, answering absence only for a file that is not there.
 
 @param metaPath - Absolute path of the metadata file
 
 @returns The file's text, or `undefined` when no file stands there
 
 @throws {@link UnreadableVmMetaError} When the file is there and cannot be read
 */
async function readMetaFile(metaPath: string,): Promise<string | undefined> {
  try {
    return await readFile(
      metaPath,
      'utf8',
    );
  }
  catch (error) {
    if (isAbsentPathError(error,))
      return undefined;
    throw new UnreadableVmMetaError({
      cause: error,
      metaPath,
    },);
  }
}

/**
 Parses the text of a VM's `meta.json` into metadata, refusing anything else.
 
 @param args - file path and the text read from it
 
 @returns The metadata the file holds
 
 @throws {@link InvalidVmMetaError} When the text is not JSON or not a metadata record
 */
function parseVmMeta({
  content,
  metaPath,
}: {
  readonly content: string;
  readonly metaPath: string;
},): VmMeta {
  /**
   Value the file's text parses to, which is only metadata if it has the shape below.
   */
  let parsed: unknown;
  try {
    parsed = JSON.parse(content,);
  }
  catch (error) {
    throw new InvalidVmMetaError({
      cause: error,
      metaPath,
      problem: `its text is not JSON (${caughtValueText(error,)})`,
    },);
  }

  /**
   What the parsed value provided, or why it is not metadata.
   */
  const verdict = metaVerdict(parsed,);
  if (!verdict.usable) {
    throw new InvalidVmMetaError({
      metaPath,
      problem: verdict.problem,
    },);
  }

  return verdict.meta;
}

/**
 Reads VM metadata from `meta.json` in the VM directory.
 Falls back to the legacy `image` text file and derives metadata
 from the registry for VMs created before meta.json was introduced.
 Only an absent `meta.json` falls back: a file that is there but cannot be read
 or parsed stops the caller with the file named, because the guest it records
 cannot be recovered from anywhere else.
 
 @param vmDir - Absolute path to the VM directory
 
 @returns Resolved VM metadata
 
 @throws {@link UnreadableVmMetaError} When `meta.json` exists and cannot be read
 
 @throws {@link InvalidVmMetaError} When `meta.json` exists and does not hold usable metadata
 
 @example
 ```ts
 const meta = await readVmMeta('/home/user/.local/share/mvm/vms/dev-01');
 if (meta.osFamily === 'windows') {
 
   // use PowerShell for exec
 }
 ```
 */
export async function readVmMeta(vmDir: string,): Promise<VmMeta> {
  /**
   Logger scoped to this reader so legacy-fallback messages are namespaced.
   */
  const rl = tagged({
    tag: readVmMeta.name,
    l,
  },);
  /**
   Absolute path of the metadata file, named in every failure this reader raises.
   */
  const metaPath = vmMetaPath(vmDir,);
  /**
   Text of `meta.json`, or `undefined` when the file is not there.
   */
  const content = await readMetaFile(metaPath,);
  if (content !== undefined) {
    /**
     Metadata the file holds, refused here when the text is not usable metadata.
     */
    const meta = parseVmMeta({
      content,
      metaPath,
    },);
    rl.debug(`read VM metadata from ${metaPath}`,);
    return meta;
  }
  rl.debug(`${metaPath} is not there, falling back to legacy image file`,);

  // Fall back to legacy image text file
  /**
   Absolute path of the legacy image file, named in the failure it can raise.
   */
  const legacyImagePath = join(
    vmDir,
    'image',
  );
  /**
   Image identifier resolved from the legacy `image` text file, or {@link DEFAULT_IMAGE} when missing.
   */
  const image = await (async function readLegacyImage(): Promise<string> {
    try {
      /**
       Raw legacy file contents; trimmed because old writers added a trailing newline.
       */
      const content = await readFile(
        legacyImagePath,
        'utf8',
      );
      return content.trim();
    }
    catch (error) {
      if (!isAbsentPathError(error,)) {
        throw new UnreadableLegacyImageError({
          cause: error,
          legacyImagePath,
        },);
      }

      rl.debug(`${legacyImagePath} is not there, assuming ${DEFAULT_IMAGE}`,);
      return DEFAULT_IMAGE;
    }
  })();

  /**
   Image record resolved from the legacy identifier; registry or custom.
   */
  const resolved = await resolveImage(image,);
  /**
   Guest config used to fill the synthetic {@link VmMeta}; defaults for custom images.
   */
  const guest = resolved.kind
    === 'registry'
    ? resolved.spec
    : CUSTOM_GUEST_DEFAULTS;

  return {
    createdAt: '',
    defaultUser: guest.defaultUser,
    image,
    osFamily: guest.osFamily,
    shell: guest.shell,
  };
}

//endregion Read
