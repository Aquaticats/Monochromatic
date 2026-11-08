/**
 Arguments of the two rootful `sudo podman` commands of the build pipeline, so a test can check them without running
 either.

 Repository rule HXR (`AGENTS.md`) starts containers that can exhaust the host with CPU, memory, process and
 disk-weight limits.
 Both commands are rootful, so they sit in a system scope outside the launcher's unit and the launcher's `IOWeight`
 never reaches them.
 */
import {
  containerLimitArguments,
  type ContainerLimits,
  type ImageBuildLimits,
  imageBuildLimitArguments,
} from '@monochromatic-dev/container-limit-core/ts';

/**
 Bound of the rootful image build: 12g and 8 CPUs, the same bound `mise run image:build` passes
 (`--memory 12g --cpu-quota 800000`, `mise.toml`).
 `podman build` has no process option, so there is no process limit here.
 */
export const IMAGE_BUILD_LIMITS: ImageBuildLimits = {
  memory: '12g',
  cpus: 8,
};

/**
 Processes the privileged bootc-image-builder run may start: 2048, what `podman run --help` prints as the default, so
 today's effective cap does not change and becomes explicit.
 */
export const BOOTC_CONVERTER_PIDS_LIMIT = 2_048;

/**
 Bound of the privileged bootc-image-builder run: the memory and CPUs of {@link IMAGE_BUILD_LIMITS}, which that build
 already holds on this host (adopted, not measured), and {@link BOOTC_CONVERTER_PIDS_LIMIT} processes.
 The process cap is untested on a privileged `osbuild` partitioning run: the run needs root and has not been exercised
 under it.
 */
export const BOOTC_CONVERTER_LIMITS: ContainerLimits = {
  memory: IMAGE_BUILD_LIMITS.memory,
  cpus: IMAGE_BUILD_LIMITS.cpus,
  pidsLimit: BOOTC_CONVERTER_PIDS_LIMIT,
};

/**
 Arguments after `sudo` of the rootful image build.

 @param imageTag - tag of the built image in the rootful store

 @param containerfile - absolute path of the Containerfile

 @param ignorefile - absolute path of the ignore file

 @param contextDirectory - build context, the monorepo root

 @returns `podman build` with its memory and CFS bound ahead of the tag

 @example
 ```ts
 const args = imageBuildArguments({ imageTag: 'localhost/x:latest', containerfile: '/p/Containerfile', ignorefile: '/p/.containerignore', contextDirectory: '/repo' });
 ```
 */
export function imageBuildArguments({
  imageTag,
  containerfile,
  ignorefile,
  contextDirectory,
}: {
  readonly imageTag: string;
  readonly containerfile: string;
  readonly ignorefile: string;
  readonly contextDirectory: string;
},): readonly string[] {
  return [
    'podman',
    'build',
    ...imageBuildLimitArguments(IMAGE_BUILD_LIMITS,),
    '--tag',
    imageTag,
    '--file',
    containerfile,
    '--ignorefile',
    ignorefile,
    contextDirectory,
  ];
}

/**
 Arguments after `sudo` of the privileged bootc-image-builder run.

 @param outputDirectory - host directory mounted at `/output`

 @param diskConfigDirectory - host directory mounted read-only at `/config`

 @param imageTag - tag of the image in the rootful store that the converter reads

 @returns `podman run` with its limits and the lowest disk weight ahead of `--privileged`

 @example
 ```ts
 const args = bootcConverterArguments({ outputDirectory: '/out', diskConfigDirectory: '/config-dir', imageTag: 'localhost/x:latest' });
 ```
 */
export function bootcConverterArguments({
  outputDirectory,
  diskConfigDirectory,
  imageTag,
}: {
  readonly outputDirectory: string;
  readonly diskConfigDirectory: string;
  readonly imageTag: string;
},): readonly string[] {
  return [
    'podman',
    'run',
    '--rm',
    ...containerLimitArguments(BOOTC_CONVERTER_LIMITS,),
    '--privileged',
    '--security-opt',
    'label=type:unconfined_t',
    '--volume',
    `${outputDirectory}:/output`,
    '--volume',
    '/var/lib/containers/storage:/var/lib/containers/storage',
    '--volume',
    `${diskConfigDirectory}:/config:ro`,
    'quay.io/centos-bootc/bootc-image-builder:latest',
    '--type',
    'qcow2',
    '--config',
    '/config/disk.toml',
    imageTag,
  ];
}
