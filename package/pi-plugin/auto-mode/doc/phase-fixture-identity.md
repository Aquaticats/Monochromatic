# Phase fixture identity

The phase fixture records its authored tool-update result objects when it emits them.
The native subscriber matches those opaque original objects by identity;
it does not inspect the SDK event's untyped `partialResult.details` or validate a replacement schema.
The fixture retains only its finite authored updates until the fixture process ends.
Repeated emission of the same result still counts as repeated updates.

The installed agent loop's `emitToolExecutionUpdate` forwards `partialResult` directly,
and its execution callback passes the original result to that emitter.
The complete native phase catalog verifies that the configured native dispatch preserves this identity
and still waits for the original execution-return guard.
Cloning an update without preserving identity must fail the update expectations;
the fixture must not manufacture a successful observation for a missing event.

`FixtureBashExecute` derives directly from the installed SDK's `ToolDefinition`
with the original Bash parameter schema and the fixture's own details type.
The SDK supplies the complete positional callback contract,
including its absent update callback value.
No nullish-union suppression or new runtime input protocol is required.
