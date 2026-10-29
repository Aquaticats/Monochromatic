/**
 GitHub CLI version-floor validation.
 
 @module
 */

/**
 Required semantic version component count.
 */
const SEMVER_COMPONENT_COUNT = 3;

/**
 Audited minimum GitHub CLI major version.
 */
const MINIMUM_MAJOR = 2;

/**
 Audited minimum GitHub CLI minor version within major two.
 */
const MINIMUM_MINOR = 97;

/**
 Audited minimum GitHub CLI patch version within 2.97.
 */
const MINIMUM_PATCH = 0;

/**
 Parsed semantic GitHub CLI version.
 */
export type GitHubCliVersion = {
  readonly major: number;
  readonly minor: number;
  readonly patch: number;
  readonly text: string;
};

/**
 Reports missing, malformed, or unsupported GitHub CLI version.
 */
export class GitHubCliVersionError extends Error {
  /**
   Creates GitHub CLI version validation failure.
   
   @param message - Safe version diagnostic.
   
   @example
   ```ts
   const error = new GitHubCliVersionError('gh is too old');
   ```
   */
  public constructor(message: string,) {
    super(message,);
    this.name = 'GitHubCliVersionError';
  }
}

/**
 Checks one non-empty ASCII decimal version component.
 
 @param component - Major, minor, or patch text.
 
 @returns Whether every code unit is decimal digit.
 
 @example
 ```ts
 isDecimalComponent('97'); // true
 ```
 */
function isDecimalComponent(component: string,): boolean {
  if (component === '') {
    return false;
  }
  for (let index = 0; index < component.length; index += 1) {
    /**
     Current UTF-16 code unit; version grammar is ASCII-only.
     */
    const character = component.charAt(index,);
    if ((character < '0') || (character > '9')) {
      return false;
    }
  }
  return true;
}

/**
 Determines whether parsed version meets audited floor.
 
 @param version - Parsed semantic GitHub CLI version.
 
 @returns Whether version is 2.97.0 or newer.
 
 @example
 ```ts
 isSupported({ major: 2, minor: 97, patch: 0, text: '2.97.0' }); // true
 ```
 */
function isSupported(version: GitHubCliVersion,): boolean {
  if (version.major > MINIMUM_MAJOR) {
    return true;
  }
  if (version.major < MINIMUM_MAJOR) {
    return false;
  }
  if (version.minor > MINIMUM_MINOR) {
    return true;
  }
  if (version.minor < MINIMUM_MINOR) {
    return false;
  }
  return version.patch >= MINIMUM_PATCH;
}

/**
 Parses and validates `gh --version` output against audited minimum.
 
 @param stdout - Captured GitHub CLI version output.
 
 @returns Parsed supported version.
 
 @throws {@link GitHubCliVersionError} when output is malformed or too old.
 
 @example
 ```ts
 parseGitHubCliVersion({ stdout: 'gh version 2.97.0' });
 ```
 */
export function parseGitHubCliVersion({
  stdout,
}: {
  readonly stdout: string;
},): GitHubCliVersion {
  /**
   First output line containing version declaration.
   */
  const [firstLine,] = stdout.split('\n',);
  if (firstLine === undefined) {
    throw new GitHubCliVersionError('GitHub CLI version output is empty',);
  }
  /**
   Non-empty declaration tokens.
   */
  const tokens = firstLine.split(' ',)
    .filter(function nonEmpty(token,): boolean {
      return token !== '';
    },);
  /**
   Semantic version token after `gh version`.
   */
  const [commandName, versionWord, text,] = tokens;
  if ((commandName !== 'gh') || (versionWord !== 'version')
    || (text === undefined)) {
    throw new GitHubCliVersionError(`cannot parse GitHub CLI version output: ${firstLine}`,);
  }
  /**
   Decimal semantic version components.
   */
  const components = text.split('.',);
  if ((components.length !== SEMVER_COMPONENT_COUNT) || (!components.every(isDecimalComponent,))) {
    throw new GitHubCliVersionError(`cannot parse GitHub CLI version ${text}`,);
  }
  /**
   Parsed major version.
   */
  const major = Number(components[0],);
  /**
   Parsed minor version.
   */
  const minor = Number(components[1],);
  /**
   Parsed patch version.
   */
  const patch = Number(components[2],);
  /**
   Complete parsed version contract.
   */
  const version: GitHubCliVersion = {
    major,
    minor,
    patch,
    text,
  };
  if (!isSupported(version,)) {
    throw new GitHubCliVersionError(`GitHub CLI ${text} is unsupported; version 2.97.0 or newer is required`,);
  }
  return version;
}
