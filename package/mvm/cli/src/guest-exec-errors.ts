/**
 Errors raised when a command run in a guest cannot report a result that
 provably belongs to it.

 @module
 */

//region Discarded results

/**
 A finished result the guest agent reported under the awaited process ID
 that does not carry the awaited command's marker line.

 @example
 ```ts
 const discarded: DiscardedGuestResult = { exitCode: 0, stderr: '', stdout: 'older output\n' };
 ```
 */
export type DiscardedGuestResult = {
  /**
   Exit status the agent reported.
   */
  readonly exitCode: number;
  /**
   Captured standard error, decoded.
   */
  readonly stderr: string;
  /**
   Captured standard output, decoded.
   */
  readonly stdout: string;
};

/**
 Longest part of a discarded output stream quoted in an error message.
 */
const EXCERPT_CHARACTERS = 2_000;

/**
 Shortens captured output for an error message.

 @param text - Captured output

 @returns `text`, or its first {@link EXCERPT_CHARACTERS} characters with a note that more followed

 @example
 ```ts
 excerpt('short'); // => 'short'
 ```
 */
function excerpt(text: string,): string {
  if (text.length <= EXCERPT_CHARACTERS) {
    return text;
  }
  return `${
    text.slice(
      0,
      EXCERPT_CHARACTERS,
    )
  }... (${String(text.length - EXCERPT_CHARACTERS,)} more characters)`;
}

/**
 Describes discarded results for an error message, one block per result.

 @param discarded - Results reported without the awaited marker

 @returns Text listing each result's exit status and output

 @example
 ```ts
 describeDiscarded([{ exitCode: 1, stderr: 'oops', stdout: '' }]);
 ```
 */
function describeDiscarded(discarded: readonly DiscardedGuestResult[],): string {
  return discarded
    .map(function describeOne(
      result,
      index,
    ) {
      return [
        `Discarded result ${String(index + 1,)}: exit status ${String(result.exitCode,)}`,
        `  standard output: ${JSON.stringify(excerpt(result.stdout,),)}`,
        `  standard error: ${JSON.stringify(excerpt(result.stderr,),)}`,
      ].join('\n',);
    },)
    .join('\n',);
}

//endregion Discarded results

//region Errors

/**
 The guest agent did not start the command, or did not confirm that it did.

 @example
 ```ts
 try {
   await runGuestCommand({ command: 'true', domain: 'mvm-dev', osFamily: 'linux', shell: '/bin/bash' });
 }
 catch (error) {
   if ((error instanceof GuestExecLaunchError) && (!error.mayBeRunning)) console.error('nothing ran');
 }
 ```
 */
export class GuestExecLaunchError extends Error {
  /**
   Whether the command may be running in the guest although no process ID came back.
   */
  readonly mayBeRunning: boolean;

  /**
   @param cause - Failure of the launch request, kept for its details

   @param domain - Prefixed libvirt domain name, named in the message

   @param mayBeRunning - Whether no answer arrived, so the command may have started anyway
   */
  constructor({
    cause,
    domain,
    mayBeRunning,
  }: {
    readonly cause: Error;
    readonly domain: string;
    readonly mayBeRunning: boolean;
  },) {
    super(
      mayBeRunning
        ? [
          `The guest agent in ${domain} did not confirm that the command started: ${cause.message}`,
          'The command may or may not be running in the guest. Without a process ID its output and exit status cannot be read.',
          'Check that the guest is up and that its agent answers, then run the command again if running it twice is safe.',
        ].join('\n',)
        : `The guest agent in ${domain} refused to start the command, so nothing ran: ${cause.message}`,
      { cause, },
    );
    this.name = 'GuestExecLaunchError';
    this.mayBeRunning = mayBeRunning;
  }
}

/**
 The command started, but its status can no longer be read:
 the guest agent stopped answering for too long, or the domain is gone.

 @example
 ```ts
 try {
   await runGuestCommand({ command: 'make', domain: 'mvm-dev', osFamily: 'linux', shell: '/bin/bash' });
 }
 catch (error) {
   if (error instanceof GuestExecStatusUnavailableError) console.error(error.pid);
 }
 ```
 */
export class GuestExecStatusUnavailableError extends Error {
  /**
   Guest process ID the command was started as.
   */
  readonly pid: number;

  /**
   @param cause - Last failed status request, kept for its details

   @param domain - Prefixed libvirt domain name, named in the message

   @param pid - Guest process ID the command was started as, so the caller can look for it in the guest

   @param reason - What was observed, in a sentence ending without punctuation
   */
  constructor({
    cause,
    domain,
    pid,
    reason,
  }: {
    readonly cause: Error;
    readonly domain: string;
    readonly pid: number;
    readonly reason: string;
  },) {
    super(
      [
        `The command started in ${domain} as guest process ${String(pid,)}, but ${reason}.`,
        'Its output and exit status were not read; the command may still be running or may have finished.',
        `Last failed status request: ${cause.message}`,
      ].join('\n',),
      { cause, },
    );
    this.name = 'GuestExecStatusUnavailableError';
    this.pid = pid;
  }
}

/**
 The command finished, but its result is larger than libvirt hands over, so
 the output and exit status are lost: the guest agent removes a finished
 result once it has answered with it.

 @example
 ```ts
 try {
   await runGuestCommand({ command: 'cat huge.log', domain: 'mvm-dev', osFamily: 'linux', shell: '/bin/bash' });
 }
 catch (error) {
   if (error instanceof GuestExecOutputTooLargeError) console.error(error.pid);
 }
 ```
 */
export class GuestExecOutputTooLargeError extends Error {
  /**
   Guest process ID the command was started as.
   */
  readonly pid: number;

  /**
   @param cause - Status request whose answer was too large, kept for its text

   @param domain - Prefixed libvirt domain name, named in the message

   @param pid - Guest process ID the command was started as
   */
  constructor({
    cause,
    domain,
    pid,
  }: {
    readonly cause: Error;
    readonly domain: string;
    readonly pid: number;
  },) {
    super(
      [
        `The command started in ${domain} as guest process ${
          String(pid,)
        } finished, but its output is too large for libvirt to hand over, so the output and the exit status are lost.`,
        'One result may hold about 3 MiB of standard output and standard error together.',
        'The command did run. To get large output, redirect it to a file in the guest and fetch the file with mvm pull.',
        `Status request: ${cause.message}`,
      ].join('\n',),
      { cause, },
    );
    this.name = 'GuestExecOutputTooLargeError';
    this.pid = pid;
  }
}

/**
 The guest agent has no result that provably belongs to the started command.
 It no longer knows the process ID, and every finished result it reported
 under that ID lacks the command's marker line. A result without the marker
 belongs to an earlier command whose status was never read, or to this
 command when its shell stopped before running anything, for example a
 PowerShell text that does not parse; the two cannot be told apart, so
 neither is returned as the command's result.

 @example
 ```ts
 try {
   await runGuestCommand({ command: 'Get-Thing |', domain: 'mvm-win', osFamily: 'windows', shell: 'powershell.exe' });
 }
 catch (error) {
   if (error instanceof GuestExecAttributionError) console.error(error.discarded);
 }
 ```
 */
export class GuestExecAttributionError extends Error {
  /**
   Results reported under the process ID without the command's marker, oldest first.
   */
  readonly discarded: readonly DiscardedGuestResult[];

  /**
   Guest process ID the command was started as.
   */
  readonly pid: number;

  /**
   @param cause - Agent answer saying the process ID is unknown, kept for its text

   @param discarded - Results reported without the marker, shown so a shell error in them is visible

   @param domain - Prefixed libvirt domain name, named in the message

   @param pid - Guest process ID the command was started as
   */
  constructor({
    cause,
    discarded,
    domain,
    pid,
  }: {
    readonly cause: Error;
    readonly discarded: readonly DiscardedGuestResult[];
    readonly domain: string;
    readonly pid: number;
  },) {
    super(
      [
        `No result in ${domain} can be tied to the command started as guest process ${String(pid,)}.`,
        discarded.length === 0
          ? 'The guest agent no longer knows that process ID and reported no finished result for it; the result was read by something else or the agent restarted.'
          : 'The guest agent reported the finished results listed here under that process ID, none with this command\'s marker line, and then no longer knew the process ID.',
        'A result without the marker belongs to an earlier command whose status was never read, or to this command when its shell stopped before running anything (for example a PowerShell text that does not parse).',
        'Look for a shell error in the discarded output; otherwise run the command again if running it twice is safe.',
        ...(discarded.length === 0 ? [] : [describeDiscarded(discarded,),]),
        `Agent answer: ${cause.message}`,
      ].join('\n',),
      { cause, },
    );
    this.name = 'GuestExecAttributionError';
    this.discarded = discarded;
    this.pid = pid;
  }
}

//endregion Errors
