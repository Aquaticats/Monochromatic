/**
 What the install proof's command line says, read once before any machine exists.

 The proof starts a container, so an argument it cannot use has to stop the run before the container exists rather than after
 it, and this module is where that reading happens. It asks nothing of a container and reads no file, so a unit test holds
 every answer it gives.
 */
import { NativeVerificationError, } from './native-verification-process.ts';

/**
 The arguments one option takes: its own name and the value after it.
 */
const argumentsPerOption = 2;

/**
 The line this run prints when it was given no image or no execution of the release before this one.
 */
const usage = 'Usage: node bin/service-manager-install-proof.ts --image <tag> --older-wrapper <path> [--candidate-wrapper <path>]';

/**
 What this run was asked to do.
 */
export type Options = {
  /**
   The gate's image, which carries systemd, Git and dbus.
   */
  readonly image: string;
  /**
   The build of the release the account had before this one.
   */
  readonly olderWrapper: string;
  /**
   The release the install puts in place, or nothing when the run installs this worktree's build.
   */
  readonly candidateWrapper?: string;
};

/**
 Reads the options this run was given.

 @param argv - the arguments after the program's name

 @returns the image, the older release's executable, and the release being installed when the caller names one

 @throws NativeVerificationError when either of the two options every run takes is absent, naming the line this run takes

 @example
 ```ts
 options({ argv: ['--image', 'localhost/x', '--older-wrapper', '/opt/git'] });
 ```
 */
export function options({ argv, }: { readonly argv: readonly string[]; }): Options {
  /**
   The options as given, the last one winning, which is how a person's later value replaces a default.
   */
  const given = new Map<string, string>();
  for (let index = 0; index < argv.length; index += argumentsPerOption) {
    /**
     The option's name and its value.
     */
    const [name = '', value = ''] = argv.slice(
      index,
      index + argumentsPerOption,
    );
    given.set(
      name,
      value,
    );
  }
  /**
   The gate's image, which the caller always names.
   */
  const image = given.get('--image',);
  /**
   The executable of the release the account had before this one, which the caller always names.
   */
  const olderWrapper = given.get('--older-wrapper',);
  /**
   The release the install puts in place, which a caller names when it is not this worktree's build.
   */
  const candidateWrapper = given.get('--candidate-wrapper',);
  if ((image === undefined) || (olderWrapper === undefined)) throw new NativeVerificationError(usage);
  return {
    image,
    olderWrapper,
    // What: The release being installed is named here only when the caller named one.
    // Why:  An absent property is how this package says a caller gave no release, and `exactOptionalPropertyTypes` refuses
    //       a property that is present and undefined.
    ...(candidateWrapper === undefined ? {} : { candidateWrapper }),
  };
}
