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
 Reads a field a metadata record must carry as non-empty text.

 The record is rebuilt by its caller from the values this returns, so a wrong
 shape cannot reach exec or file transfer as metadata whose fields are
 `undefined`.

 @param field - field name, named in the refusal

 @param metaPath - metadata file, named in the refusal

 @param parsed - parsed record

 @returns the field's text

 @throws {@link InvalidVmMetaError} when the field is missing, not text, or empty

 @example
 ```ts
 requiredText({ field: 'image', metaPath: '/vm/meta.json', parsed: { image: 'ubuntu' } }); // 'ubuntu'
 ```
 */
function requiredText({
  field,
  metaPath,
  parsed,
}: {
  readonly field: string;
  readonly metaPath: string;
  readonly parsed: object;
},): string {
  /**
   Value the record carries under this field, usable only when it is non-empty text.
   */
  const value: unknown = (field in parsed)
    ? Reflect.get(
      parsed,
      field,
    )
    : undefined;
  if (((typeof value) !== 'string') || (value === '')) {
    throw new InvalidVmMetaError({
      metaPath,
      problem: `the field \`${field}\` is missing, not text, or empty`,
    },);
  }
  return value;
}

/**
 Reads a metadata record into the fields its callers use.

 @param metaPath - metadata file, named in any refusal

 @param parsed - value `JSON.parse` produced from `meta.json`

 @returns the metadata the record holds

 @throws {@link InvalidVmMetaError} when the value is not a metadata record

 @example
 ```ts
 vmMetaRecord({ metaPath: '/vm/meta.json', parsed: JSON.parse('{}') });
 // throws: the field `createdAt` is missing, not text, or empty
 ```
 */
function vmMetaRecord({
  metaPath,
  parsed,
}: {
  readonly metaPath: string;
  readonly parsed: unknown;
},): VmMeta {
  if (((typeof parsed) !== 'object') || (parsed === null)) {
    throw new InvalidVmMetaError({
      metaPath,
      problem: `it holds ${typeof parsed}, not a JSON object`,
    },);
  }

  /**
   Creation timestamp the record carries; reported back to callers only.
   */
  const createdAt = requiredText({
    field: 'createdAt',
    metaPath,
    parsed,
  },);
  /**
   Login user the record carries; decides which account a guest command runs as.
   */
  const defaultUser = requiredText({
    field: 'defaultUser',
    metaPath,
    parsed,
  },);
  /**
   Image identifier the record carries; named when a VM is listed or cloned.
   */
  const image = requiredText({
    field: 'image',
    metaPath,
    parsed,
  },);
  /**
   Shell the record carries; decides the interpreter a guest command runs under.
   */
  const shell = requiredText({
    field: 'shell',
    metaPath,
    parsed,
  },);
  /**
   OS family the record carries, which dispatches exec and domain XML.
   */
  const osFamily = requiredText({
    field: 'osFamily',
    metaPath,
    parsed,
  },);
  if ((osFamily !== 'linux') && (osFamily !== 'windows')) {
    throw new InvalidVmMetaError({
      metaPath,
      problem: `the field \`osFamily\` is ${osFamily} rather than linux or windows`,
    },);
  }

  /**
   File-transfer route the record carries, absent in metadata written before
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
    throw new InvalidVmMetaError({
      metaPath,
      problem: `the field \`fileTransfer\` is ${JSON.stringify(fileTransfer,)} `
        + 'rather than guest-agent or virtiofs',
    },);
  }

  return {
    createdAt,
    defaultUser,
    ...(fileTransfer === undefined ? {} : { fileTransfer, }),
    image,
    osFamily,
    shell,
  };
}

//endregion Record validation

//region Read

/**
 Answer for a `meta.json` that is not there, so absence travels as a distinct
 symbol rather than a nullish union.
 */
const META_FILE_ABSENT: unique symbol = Symbol('no meta.json in the VM directory',);

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
 
 @returns The file's text, or {@link META_FILE_ABSENT} when no file stands there
 
 @throws {@link UnreadableVmMetaError} When the file is there and cannot be read
 */
async function readMetaFile(
  metaPath: string,
): Promise<string | typeof META_FILE_ABSENT> {
  try {
    return await readFile(
      metaPath,
      'utf8',
    );
  }
  catch (error) {
    if (isAbsentPathError(error,))
      return META_FILE_ABSENT;
    throw new UnreadableVmMetaError({
      cause: error,
      metaPath,
    },);
  }
}

/**
 Parses the text of a VM's `meta.json`, refusing text that is not JSON.
 
 @param content - text read from the metadata file
 
 @param metaPath - Absolute path of the metadata file, named in the refusal
 
 @returns The value the text parses to, which the caller still has to check
 
 @throws {@link InvalidVmMetaError} When the text is not JSON
 */
function parseJsonText({
  content,
  metaPath,
}: {
  readonly content: string;
  readonly metaPath: string;
},): unknown {
  try {
    return JSON.parse(content,);
  }
  catch (error) {
    throw new InvalidVmMetaError({
      cause: error,
      metaPath,
      problem: `its text is not JSON (${caughtValueText(error,)})`,
    },);
  }
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
   Text of `meta.json`, or {@link META_FILE_ABSENT} when the file is not there.
   */
  const metaText = await readMetaFile(metaPath,);
  if ((typeof metaText) === 'string') {
    /**
     Metadata the file holds, refused here when the text is not usable metadata.
     */
    const meta = vmMetaRecord({
      metaPath,
      parsed: parseJsonText({
        content: metaText,
        metaPath,
      },),
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
      const legacyContent = await readFile(
        legacyImagePath,
        'utf8',
      );
      return legacyContent.trim();
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
