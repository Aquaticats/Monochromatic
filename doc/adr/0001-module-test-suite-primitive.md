# module-test remains a suite primitive

`@monochromatic-dev/test-harness-core` exposes `describe`,
 `it`,
 and `expect` as suite and assertion primitives.
We decided not to add a file-level runner:
 file discovery,
 per-file process isolation,
 and aggregate execution
belong to the root `mise` `test:unit` template,
 which runs each `*.unit.test.ts` with `node`.
Keeping this seam avoids duplicating orchestration policy inside the harness while preserving `module-test`'s
small Interface for suite behavior.

## Considered options

- Add a file runner to `module-test`:
   rejected because it would overlap the existing `mise` execution seam.
- Keep `module-test` as a suite primitive only:
   accepted because the harness owns in-file suite behavior,
  and `mise` owns file-level orchestration.
