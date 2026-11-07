/**
 The image the native wrapper's gate builds, and how the parallel launcher finds it.

 `native:test:container` builds `localhost/git-policy-native-test:<tag>`, where the tag is the value of
 `GIT_POLICY_NATIVE_IMAGE_TAG` or, when it is not set, `development`. Everything that reads "the last gate image" (the
 gate, the campaign task, the controls-only task, and the parallel launcher) takes the tag from `gateImageTagOf`, so
 they name one image.
 */

/**
 Image repository of the tested images: the gate builds them and the campaign units run them.
 */
export const nativeTestImageRepository = 'localhost/git-policy-native-test';

/**
 Environment variable that names the tag of the tested image; concurrent worktrees set it so one snapshot never runs
 another's image.
 */
export const gateImageTagVariable = 'GIT_POLICY_NATIVE_IMAGE_TAG';

/**
 Tag of the tested image when `gateImageTagVariable` is not set.
 */
export const defaultGateImageTag = 'development';

/**
 The part of a process environment that names the gate image; `process.env` fits it.
 */
export type GateEnvironment = {
  readonly GIT_POLICY_NATIVE_IMAGE_TAG?: string;
};

/**
 Tag of the tested image the last gate of this environment built.

 @param environment - environment of the process that runs the gate, the campaign task or the launcher

 @returns the value of `GIT_POLICY_NATIVE_IMAGE_TAG`, or `development` when it is not set

 @example
 ```ts
 gateImageTagOf(process.env);
 ```
 */
export function gateImageTagOf(environment: GateEnvironment): string {
  return environment[gateImageTagVariable] ?? defaultGateImageTag;
}

/**
 Launcher arguments that name the gate image as the one every unit's tag is made from.

 They go before the person's own arguments, so a `--from-tag` the person passes (the later one wins) names another gate
 image.

 @param environment - environment of the launcher

 @returns `--image-repository` and `--from-tag` with their values

 @example
 ```ts
 launcherImageArguments(process.env);
 ```
 */
export function launcherImageArguments(environment: GateEnvironment): readonly string[] {
  return [
    '--image-repository',
    nativeTestImageRepository,
    '--from-tag',
    gateImageTagOf(environment),
  ];
}
