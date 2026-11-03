/**
 Lowest weight podman accepts for a container's disk I/O (`podman run --blkio-weight` takes 10 to 1000; the default is 100).

 Repository rule HXR (`AGENTS.md`) starts containers that can exhaust the host at this weight. The containers here are
 rootful, so they sit in a system scope outside the launcher's unit, and the launcher's `IOWeight` does not reach them.
 `mise.toml` carries the same value as the `lowest_disk_weight` variable for the `image:check` task, whose commands are
 shell strings; `disk-weight.unit.test.ts` reads that file and fails when the two differ.
 */
export const LOWEST_DISK_WEIGHT: number = 10;

/**
 Disk-weight options of a started container, in the two forms the two block-layer weight files need.

 `--blkio-weight` sets the cgroup file `io.bfq.weight`, which the BFQ I/O scheduler reads, and leaves `io.weight`
 (which other schedulers read) at the default; `--cgroup-conf=io.weight=` sets `io.weight` and leaves `io.bfq.weight`
 at the default. Passing both puts whichever file the host's scheduler reads at {@link LOWEST_DISK_WEIGHT}.
 `podman build` has neither option, so image builds are not covered.

 @returns `['--blkio-weight=10', '--cgroup-conf=io.weight=10']`

 @example
 ```ts
 const options = diskWeightArguments();
 ```
 */
export function diskWeightArguments(): readonly string[] {
  return [
    `--blkio-weight=${String(LOWEST_DISK_WEIGHT,)}`,
    `--cgroup-conf=io.weight=${String(LOWEST_DISK_WEIGHT,)}`,
  ];
}
