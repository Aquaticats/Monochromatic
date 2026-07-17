/**
 * Cross-platform process birth identities for transaction ownership.
 *
 * @module
 */
import { readFile, } from 'node:fs/promises';
import nanoSpawn, { SubprocessError, } from 'nano-spawn';

/**
 * Recorded PID does not currently identify a process.
 */
export const PROCESS_IDENTITY_ABSENT: unique symbol = Symbol('transaction owner process identity absent',);

/**
 * Resolves Linux kernel process start tick.
 *
 * @param pid - process identifier
 *
 * @returns birth identity or absence after process exit
 */
async function resolveLinuxIdentity(pid: number,): Promise<string | typeof PROCESS_IDENTITY_ABSENT> {
  try {
    /**
     * Proc stat containing parenthesized command followed by numbered fields.
     */
    const stat = await readFile(
      `/proc/${String(pid,)}/stat`,
      'utf8',
    );
    /**
     * Final command delimiter before field three.
     */
    const commandEnd = stat.lastIndexOf(')',);
    if (commandEnd === (-1))
      throw new TypeError(`Malformed Linux process stat for PID ${String(pid,)}`,);
    /**
     * Fields three onward.
     */
    const fields = stat.slice(commandEnd + 2,)
      .trim()
      .split(' ',);
    /**
     * Field twenty-two relative to field-three origin.
     */
    const startTick = fields[(2 ** (2 + 2)) + 2
      + 1];
    if ((startTick === undefined) || (startTick.length === 0))
      throw new TypeError(`Linux process start identity is unavailable for PID ${String(pid,)}`,);
    return `linux:${startTick}`;
  }
  catch (error: unknown) {
    if (Error.isError(error,) && ('code' in error)
      && (error.code === 'ENOENT'))
      return PROCESS_IDENTITY_ABSENT;
    throw error;
  }
}

/**
 * Resolves process identity through bounded platform command.
 *
 * @param command - platform process inspector
 *
 * @param args - exact inspector arguments
 *
 * @param prefix - platform identity prefix
 *
 * @returns birth identity or absence after process exit
 */
async function resolveCommandIdentity({
  command,
  args,
  prefix,
}: Readonly<{
  command: string;
  args: readonly string[];
  prefix: string;
}>,): Promise<string | typeof PROCESS_IDENTITY_ABSENT> {
  try {
    /**
     * Locale-stable process inspector result.
     */
    const result = await nanoSpawn(
      command,
      [...args,],
      { env: { LC_ALL: 'C', }, },
    );
    /**
     * Trimmed platform birth identity.
     */
    const value = result.stdout
      .trim();
    return value.length === 0 ? PROCESS_IDENTITY_ABSENT : `${prefix}:${value}`;
  }
  catch (error: unknown) {
    if (error instanceof SubprocessError)
      return PROCESS_IDENTITY_ABSENT;
    throw error;
  }
}

/**
 * Resolves stable PID plus process-birth identity on supported hosts.
 *
 * @param pid - process identifier
 *
 * @returns platform birth identity or absence after process exit
 *
 * @example
 * ```ts
 * await resolveProcessBirthIdentity(process.pid);
 * ```
 */
export function resolveProcessBirthIdentity(pid: number,): Promise<string | typeof PROCESS_IDENTITY_ABSENT> {
  if (process.platform === 'linux')
    return resolveLinuxIdentity(pid,);
  if (process.platform === 'darwin')
    return resolveCommandIdentity({
      command: 'ps',
      args: [
        '-o',
        'lstart=',
        '-p',
        String(pid,),
      ],
      prefix: 'darwin',
    },);
  if (process.platform === 'win32')
    return resolveCommandIdentity({
      command: 'powershell.exe',
      args: [
        '-NoLogo',
        '-NoProfile',
        '-NonInteractive',
        '-Command',
        `(Get-Process -Id ${String(pid,)}).StartTime.ToUniversalTime().Ticks`,
      ],
      prefix: 'win32',
    },);
  return Promise.reject(new TypeError(`Unsupported transaction process identity platform: ${process.platform}`,),);
}
