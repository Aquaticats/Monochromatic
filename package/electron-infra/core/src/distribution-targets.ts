/**
 Supported Electron Packager distribution target matrix.
 
 @example
 ```ts
 console.log(DISTRIBUTION_TARGETS.map((target) => targetKey({ target })));
 ```
 */

/**
 Electron Packager platforms this repo distributes.
 
 @example
 ```ts
 const platform: DistributionPlatform = 'linux';
 ```
 */
export type DistributionPlatform = 'darwin' | 'linux' | 'win32';

/**
 Electron Packager architectures this repo distributes.
 
 @example
 ```ts
 const arch: DistributionArch = 'x64';
 ```
 */
export type DistributionArch = 'arm64' | 'x64';

/**
 One platform and architecture bundle target.
 
 @example
 ```ts
 const target: DistributionTarget = { platform: 'linux', arch: 'x64' };
 ```
 */
export type DistributionTarget = {
  readonly arch: DistributionArch;
  readonly platform: DistributionPlatform;
};

/**
 Platforms covered by default distribution.
 
 @example
 ```ts
 console.log(distributionPlatforms.length);
 ```
 */
const distributionPlatforms: readonly DistributionPlatform[] = [
  'linux',
  'win32',
  'darwin',
];

/**
 Architectures covered by default distribution.
 
 @example
 ```ts
 console.log(distributionArchitectures.length);
 ```
 */
const distributionArchitectures: readonly DistributionArch[] = [
  'x64',
  'arm64',
];

/**
 Default Linux, Windows, and macOS x64/arm64 distribution matrix.
 
 @example
 ```ts
 console.log(DISTRIBUTION_TARGETS.length);
 ```
 */
export const DISTRIBUTION_TARGETS: readonly DistributionTarget[] = distributionPlatforms
  .flatMap(function targetsForPlatform(platform,): readonly DistributionTarget[] {
    return distributionArchitectures.map(function targetForArchitecture(arch,): DistributionTarget {
      return {
        arch,
        platform,
      };
    },);
  },);

/**
 Stable target key used by CLI filters and manifests.
 
 @param target - Distribution target to format.
 
 @returns Platform-architecture key.
 
 @example
 ```ts
 targetKey({ target: { platform: 'linux', arch: 'x64' } });
 ```
 */
export function targetKey({ target, }: { readonly target: DistributionTarget; },): string {
  return `${target.platform}-${target.arch}`;
}
