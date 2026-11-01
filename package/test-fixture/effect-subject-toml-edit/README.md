# test-fixture-effect-subject-toml-edit

Source text that really imports `@monochromatic-dev/toml-edit-core`,
read by the `prefer-readonly-parameter-type` oxlint plugin's workspace-source test
(`package/oxlint-plugin/prefer-readonly-parameter-type/src/workspace-source-effect.unit.test.ts`).

`src/cargo/apply-plan.ts` and its helpers are a verbatim copy of the retired TypeScript file-enforcer's Cargo manifest
planner (`package/file-enforcer/core/src/cargo/`, removed at the Rust file-enforcer cutover).
The test pins how the effect analysis resolves calls through live workspace source into `toml-edit-core`
without catalog entries,
so the subject must keep calling that package;
nothing here runs and nothing builds.
