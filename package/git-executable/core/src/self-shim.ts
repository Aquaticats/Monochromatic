import { constants, } from 'node:fs';
import {
  open,
  realpath,
} from 'node:fs/promises';
import { basename, } from 'node:path';

//region Native executable recognition

/**
 Header byte count covering supported native executable signatures.
 */
const NATIVE_EXECUTABLE_HEADER_BYTES = 4;

/**
 Bytes per kibibyte used to express script inspection bound.
 */
const KIBIBYTE_BYTES = 1_024;

/**
 Kibibytes inspected from each script candidate.
 */
const SELF_SHIM_INSPECTION_KIBIBYTES = 64;

/**
 Maximum bytes inspected from script candidates.
 Generated command shims are launchers;
 a fixed bound prevents arbitrary PATH files from controlling resolver memory use.
 */
const MAX_SELF_SHIM_INSPECTION_BYTES = SELF_SHIM_INSPECTION_KIBIBYTES * KIBIBYTE_BYTES;

/**
 Hex prefixes for ELF,
 PE,
 Mach-O,
 and universal Mach-O executables.
 */
const NATIVE_EXECUTABLE_HEX_PREFIXES: ReadonlySet<string> = new Set([
  '7f454c46',
  '4d5a',
  'feedface',
  'feedfacf',
  'cefaedfe',
  'cffaedfe',
  'cafebabe',
  'bebafeca',
  'cafebabf',
  'bfbafeca',
],);

/**
 Reports whether captured bytes identify a supported native executable format.
 
 @param header - Candidate file prefix.
 
 @returns Whether prefix is ELF,
 PE,
 Mach-O,
 or universal Mach-O.
 
 @example
 ```ts
 isNativeExecutableHeader(Buffer.from('7f454c46', 'hex'));
 // => true
 ```
 */
function isNativeExecutableHeader(header: Uint8Array,): boolean {
  /**
   Hexadecimal prefix compared without platform-endian conversion.
   */
  const hex = Buffer.from(header,)
    .toString('hex');
  return [...NATIVE_EXECUTABLE_HEX_PREFIXES,].some(function matchesNativePrefix(prefix,) {
    return hex.startsWith(prefix,);
  },);
}

//endregion Native executable recognition

//region Git policy wrapper markers

/**
 Reports PATH candidate whose opened filesystem object is not a regular file.
 
 @example
 ```ts
 throw new GitCandidateFileTypeError('/tmp/git');
 ```
 */
export class GitCandidateFileTypeError extends Error {
  /**
   Stable error classification independent of minification.
   */
  override readonly name = 'GitCandidateFileTypeError';

  /**
   Creates non-regular candidate evidence.
   
   @param candidatePath - Opened PATH candidate with unsupported filesystem type.
   
   @example
   ```ts
   const error = new GitCandidateFileTypeError('/tmp/git');
   ```
   */
  constructor(candidatePath: string,) {
    super(`Git candidate is not a regular file: ${candidatePath}`,);
  }
}

/**
 Reports non-native candidate whose script inspection would exceed fixed bound.
 
 @example
 ```ts
 throw new GitCandidateInspectionLimitError('/tmp/git');
 ```
 */
export class GitCandidateInspectionLimitError extends Error {
  /**
   Stable error classification independent of minification.
   */
  override readonly name = 'GitCandidateInspectionLimitError';

  /**
   Creates bounded-inspection rejection evidence.
   
   @param candidatePath - Non-native candidate larger than inspection bound.
   
   @example
   ```ts
   const error = new GitCandidateInspectionLimitError('/tmp/git');
   ```
   */
  constructor(candidatePath: string,) {
    super(`Non-native Git candidate exceeds script inspection bound: ${candidatePath}`,);
  }
}

/**
 Package name used by scripts that delegate to Git policy wrapper.
 */
const GIT_POLICY_CLI_PACKAGE_NAME = '@monochromatic-dev/git-policy-cli';

/**
 Bundle entry path used by pnpm command shims for Git policy wrapper.
 */
const GIT_POLICY_CLI_BUNDLED_ENTRY = 'package/git-policy/cli/dist/final/node/index.mjs';

/**
 Windows package path emitted when command shims spell package scope with backslash.
 */
const WINDOWS_GIT_POLICY_CLI_PACKAGE_PATH = String.raw`@monochromatic-dev\git-policy-cli`;

/**
 Windows bundle path emitted by command shims without package metadata.
 */
const WINDOWS_GIT_POLICY_CLI_BUNDLED_ENTRY = String.raw`package\git-policy\cli\dist\final\node\index.mjs`;

/**
 Text markers that identify scripts delegating to Git policy wrapper.
 */
const SELF_SHIM_MARKERS: ReadonlySet<string> = new Set([
  GIT_POLICY_CLI_PACKAGE_NAME,
  GIT_POLICY_CLI_BUNDLED_ENTRY,
  WINDOWS_GIT_POLICY_CLI_PACKAGE_PATH,
  WINDOWS_GIT_POLICY_CLI_BUNDLED_ENTRY,
],);

/**
 File names of the native Git policy wrapper executable (the crate binary), which the installed `git` link resolves to.
 */
const NATIVE_WRAPPER_FILE_NAMES: ReadonlySet<string> = new Set([
  'cli-git-native',
  'cli-git-native.exe',
],);

/**
 Checks whether candidate script delegates to Git policy wrapper.
 
 A candidate whose resolved file name is the native wrapper executable is the wrapper,
 because the installed `git` is a link to that file.
 Other native executables return after one header read.
 Only scripts and unknown formats undergo complete text inspection.
 
 @param candidatePath - Absolute candidate executable path.
 
 @returns Whether candidate delegates to Git policy wrapper.
 
 @throws When candidate cannot be opened or read,
 opened object is not a regular file,
 or non-native script exceeds inspection bound.
 
 @example
 ```ts
 await isGitPolicySelfShim('/workspace/node_modules/.bin/git');
 ```
 */
export async function isGitPolicySelfShim(candidatePath: string,): Promise<boolean> {
  /**
   Open candidate shared by native header and text fallback inspection.
   */
  await using candidate = await open(
    candidatePath,
    constants.O_RDONLY | constants.O_NONBLOCK,
  );
  /**
   Opened candidate metadata resistant to path replacement after open.
   */
  const candidateStats = await candidate.stat();
  if (!candidateStats.isFile())
    throw new GitCandidateFileTypeError(candidatePath,);
  /**
   Final file name after links resolve, so the installed `git` link names the wrapper executable.
   */
  const resolvedName = basename(await realpath(candidatePath,),);
  if (NATIVE_WRAPPER_FILE_NAMES.has(resolvedName,))
    return true;
  /**
   Fixed native-signature prefix.
   */
  const header = Buffer.alloc(NATIVE_EXECUTABLE_HEADER_BYTES,);
  /**
   Captured prefix length for files shorter than native headers.
   */
  const { bytesRead, } = await candidate.read(
    header,
    0,
    header.length,
    0,
  );
  if (isNativeExecutableHeader(
    header.subarray(
      0,
      bytesRead,
    ),
  ))
    return false;
  if (candidateStats.size > MAX_SELF_SHIM_INSPECTION_BYTES)
    throw new GitCandidateInspectionLimitError(candidatePath,);
  /**
   Bounded non-native candidate bytes inspected for self-shim markers.
   */
  const contentBytes = Buffer.alloc(MAX_SELF_SHIM_INSPECTION_BYTES,);
  /**
   Captured script bytes up to inspection bound.
   */
  const { bytesRead: contentBytesRead, } = await candidate.read(
    contentBytes,
    0,
    contentBytes.length,
    0,
  );
  /**
   UTF-8 script prefix containing command shim targets.
   */
  const content = contentBytes
    .subarray(
    0,
    contentBytesRead,
    )
    .toString('utf8');
  return [...SELF_SHIM_MARKERS,].some(function hasSelfShimMarker(marker,) {
    return content.includes(marker,);
  },);
}

//endregion Git policy wrapper markers
