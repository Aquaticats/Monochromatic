# Write completion and file ownership

Logging has separate delivery and completion boundaries.
Every logger call still invokes every available sink.
Several invocations may return the same pending completion because they belong to one backend operation.
Treating those invocations as separate pending operations retained redundant async continuations during producer bursts.

## Pending write observations

`src/pending-writes.ts` owns completion observations for one logger.
Its set supplies the iterable snapshot consumed by `flush()`.
Its weak map associates each original sink completion with its current observation.
The set supports drainage and deadline abandonment;
the weak map supports identity reuse without keeping settled backend operations alive.
Neither structure stores log records.

A shared completion never skips a sink invocation.
The logger calls `sink.write(record)` first,
then admits the returned completion.
Rejections are reported by the existing internal diagnostic path and do not retire the sink.

A flush deadline clears the logger's observation set without cancelling backend work.
A later write returning the same unfinished operation receives a new observation.
An old observer cannot erase the newer identity mapping when it eventually settles.
The existing flush boundary remains a snapshot rather than an indefinitely expanding wait for future writes.

Console and synchronous storage sinks reuse a fulfilled completion owned by their factory.
The no-op sink creates its shared completion lazily,
preserving imports that perform no runtime discovery or asynchronous work.
The public sink contract remains `Promise<void>`.
Adding a second synchronous return type would enlarge the caller's contract
without solving file-write observation costs.

## File append ownership

`src/sink/file-append.ts` owns serial appends for one verified file sink.
Queued records share batches capped at 65,536 UTF-8 bytes.
An individual larger record remains whole in its own batch.
The cap bounds aggregation size,
not total queued data or the size of an individual caller-supplied record.
The logger does not add a record-dropping policy or producer backpressure.

Each batch owns captured strings and one completion.
A linked queue avoids repeatedly shifting a growing array and avoids a promise chain per record.
Async iteration starts a batch's append after the preceding append finishes.
Detaching the batch before issuing its append prevents later producers from changing an in-progress write.
The iterator releases its running state before reporting completion,
so a producer arriving during the final asynchronous iterator step can start another drain.

Completion means the complete `appendFile` attempt has settled.
It does not imply `fsync` or survival of a power failure.
Append failures retain the existing internal reporting behavior,
release waiting callers,
and allow subsequent batches to run.

The sink serializes a record before admitting it to the append queue.
`JSON.stringify` may invoke `toJSON`,
getters,
or proxy traps that synchronously write another record through the same sink.
Admission after serialization preserves that inner record's place without allowing overlapping append operations.
Serialization failures and serializers that produce no JSON value report an error and leave later writes usable.

Timestamp-only filenames do not establish ownership:
independent sinks or separate bundled copies can initialize within the same millisecond.
Filenames include a UUID generated during verification,
and the verification append creates the file exclusively.
The append queue becomes available only after the probe has been read successfully.
Each sink therefore serializes writes to its own file rather than relying on another instance's queue.

## Evidence and limits

`src/burst-capacity.test.ts` imports the built neutral entry point,
delivers 100,000 records in one synchronous producer frame,
forces collection before microtasks drain,
and requires retained completion state below 8,388,608 bytes.
The committed regression first failed with 93,863,816 retained heap bytes.
The repaired artifact passed with 48,304 bytes in the subsequent verification.
This measures that workload's retained heap,
not a universal process-memory bound.

`src/pending-completion.unit.test.ts` exercises shared rejection,
continued delivery,
and re-observation of the same operation after a flush deadline.
Existing orchestration tests cover late settlement,
backend verification,
sink flush hooks,
and deadlines.

`src/sink/file-batch.unit.test.ts` reads real JSONL files under disposable homes.
Its regressions cover captured records,
multiple bounded batches,
drain restart,
reentrant multi-write records,
same-millisecond independent sinks,
and rejected serializations.
Before their respective repairs,
the reentrant and independent-sink cases produced malformed JSONL,
and a serializer returning `undefined` inserted a non-JSON line.

The auto-mode host loads logger source through configured extensions and also includes a bundled logger.
Those copies have separate module-local default instances.
Changing a bundled append function alone does not establish what the source instance did.
Consumer verification therefore rebuilds the auto-mode bundle and native host after the logger changes,
then exercises the configured Pi host with actual provider requests and streaming input.
The isolated logger measurements do not establish that the host's streaming-memory failure is resolved.
