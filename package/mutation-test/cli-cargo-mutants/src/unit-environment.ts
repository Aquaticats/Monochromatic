import type { LauncherOptions } from './options.ts';

/**
 Build the `NAME=value` entries every unit of a launch runs with, in the order they are applied.

 The launcher's own assignments come last for TMPDIR, PATH and HOME, so a `--setenv` of those names cannot break the
 unit's tools; the image tag comes first, so a caller's `--setenv` of the tag variable still wins. A product that adds
 coordination entries for some units appends them to `extraEnvironment` of the options it plans from, which puts them
 after the person's entries and before the launcher's.

 @param options - validated launcher options
 
 @param tag - image tag of this unit
 
 @returns the entries for `systemd-run --setenv`

 @example
 ```ts
 buildUnitEnvironment({ options, tag: 'w50' });
 ```
 */
export function buildUnitEnvironment({
  options,
  tag,
}: {
  readonly options: LauncherOptions;
  readonly tag: string;
}): readonly string[] {
  return [
    `${options.tagEnvironmentVariable}=${tag}`,
    ...options.extraEnvironment,
    `TMPDIR=${options.temporaryDirectory}`,
    // The units start with a bare environment, so the tools the campaign needs come from the launcher's.
    ...(options.path === undefined ? [] : [`PATH=${options.path}`]),
    `HOME=${options.home}`,
  ];
}
