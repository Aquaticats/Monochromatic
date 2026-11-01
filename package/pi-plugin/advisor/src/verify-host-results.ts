/**
 Assertions over real Pi host events and persisted disposable session data. @module
 */
import assert from 'node:assert/strict';
import * as v from 'valibot';

/**
 JSON object boundary used before inspecting host output.
 */
const RecordSchema = v.record(
  v.string(),
  v.unknown(),
);
/**
 JSON array of records used for ledger and result collections.
 */
const RecordsSchema = v.array(RecordSchema,);
/**
 Preparation, actual dispatch, and terminal observation are the per-attempt update boundaries.
 */
const MAX_PROGRESS_EVENTS_PER_ATTEMPT = 3;

/**
 Parse strict JSONL without dropping non-JSON diagnostics or truncating the evidence.
 
 @param text - complete bounded host output or session file
 
 @returns parsed records
 
 @example
 ```ts
 const entries = parseAdvisorHostJsonl(text);
 ```
 */
export function parseAdvisorHostJsonl(text: string,): readonly Record<string, unknown>[] {
  return text.split('\n',)
    .filter(function nonempty(line: string,): boolean {
      return line.trim() !== '';
    },)
    .map(function parse(line: string,): Record<string, unknown> {
      return v.parse(
        RecordSchema,
        JSON.parse(line,),
      );
    },);
}

/**
 Validate available accounting against the actual per-attempt records.
 
 @param value - persisted operation
 
 @returns validated operation record
 */
function verifyUsage(value: unknown,): Record<string, unknown> {
  /**
   Operation object persisted by the extension.
   */
  const operation = v.parse(
    RecordSchema,
    value,
  );
  /**
   Each logical provider attempt contributes at most its latest observed usage.
   */
  const attempts = v.parse(
    RecordsSchema,
    operation.attempts,
  );
  /**
   Aggregate counters delivered by the extension.
   */
  const usage = v.parse(
    RecordSchema,
    operation.usage,
  );
  /**
   Sum only attempts with available provider counters.
   */
  const total = attempts.reduce(
    function sum(
      acc: number,
      attempt: Readonly<Record<string, unknown>>,
    ): number {
      if (attempt.usage === undefined)
        return acc;
      return acc + v.parse(
        v.number(),
        v.parse(
          RecordSchema,
          attempt.usage,
        )
          .totalTokens,
      );
    },
    0,
  );
  assert.equal(
    usage.totalTokens,
    total,
  );
  assert.ok(
    v.parse(
      v.number(),
      usage.input,
    ) > 0,
    'usage positive control must observe input tokens',
  );
  return operation;
}

/**
 Verify registered-tool execution, progress privacy, cancellation, and slash-command persistence.
 
 @param mode - finite fixture scenario
 
 @param events - complete JSON-mode event stream
 
 @param entries - persisted session entries
 
 @param trace - provider-owned dispatch and cancellation trace
 
 @example
 ```ts
 verifyAdvisorHostEvidence({ mode, events, entries, trace });
 ```
 */
export function verifyAdvisorHostEvidence({
  mode,
  events,
  entries,
  trace,
}: {
  readonly mode: string;
  readonly events: readonly Readonly<Record<string, unknown>>[];
  readonly entries: readonly Readonly<Record<string, unknown>>[];
  readonly trace: readonly Readonly<Record<string, unknown>>[];
},): void {
  if ((mode === 'slash') || (mode === 'slash-error')) {
    /**
     Slash reviews persist either a custom review or a failed-operation entry.
     */
    const entry = entries.find(function commandEntry(value): boolean {
      return value.customType === (mode === 'slash' ? 'pi-advisor.review' : 'pi-advisor.operation');
    },);
    assert.ok(
      entry,
      'missing persisted slash-command operation',
    );
    /**
     Success stores operation alongside review details; failure stores the operation directly.
     */
    const operation = verifyUsage(mode === 'slash' ? v.parse(
      RecordSchema,
      entry.details,
    )
      .operation : entry.data,);
    assert.equal(
      v.parse(
        RecordsSchema,
        operation.reviews,
      )
        .length,
      mode === 'slash' ? 1 : 0,
    );
    return;
  }
  /**
   Locate actual persisted tool result rather than trusting provider fixture return values.
   */
  const toolEntries = entries.filter(function toolEntry(entry): boolean {
    return (entry.type === 'message') && (v.parse(
      RecordSchema,
      entry.message,
    )
      .role
      === 'toolResult');
  },);
  assert.equal(
    toolEntries.length,
    1,
  );
  /**
   Verified single tool result.
   */
  const [entry,] = toolEntries;
  assert.ok(entry,);
  /**
   Pi's final tool-result record.
   */
  const result = v.parse(
    RecordSchema,
    entry.message,
  );
  assert.equal(
    result.toolName,
    'advisor',
  );
  assert.equal(
    result.isError,
    mode === 'explicit-credit',
  );
  /**
   Operation ledger crosses the real host's success or thrown-error boundary.
   */
  const operation = verifyUsage(v.parse(
    RecordSchema,
    result.details,
  )
    .operation,);
  assert.deepEqual(
    result.usage,
    operation.usage,
  );
  /**
   Actual provider calls, independent of the extension's ledger claims.
   */
  const dispatches = trace.filter(function dispatched(record): boolean {
    return (record.event === 'initial-dispatch') || (record.event === 'alternate-dispatch');
  },);
  assert.equal(
    dispatches.length,
    mode === 'explicit-credit' ? 1 : 2,
  );
  assert.equal(
    v.parse(
      RecordsSchema,
      operation.reviews,
    )
      .length,
    mode === 'explicit-credit' ? 0 : mode === 'collect' ? 2 : 1,
  );
  if ((mode === 'serial-credit') || (mode === 'explicit-credit'))
    assert.deepEqual(
      operation.blockedProviders,
      ['fixture-credit',],
    );
  if (mode === 'straggler') {
    assert.equal(
      operation.end,
      'collection',
    );
    assert.equal(
      operation.usageIncomplete,
      true,
    );
    assert.ok(trace.some(function cancelled(record): boolean {
      return record.event === 'initial-cancelled';
    },),);
  }
  /**
   Partial updates must contain metadata only, including their structured details.
   */
  const updates = events.filter(function partial(event): boolean {
    return event.type === 'tool_execution_update';
  },);
  assert.ok(
    updates.length > 0,
    'missing real host tool updates',
  );
  assert.ok(
    updates.length <= ((dispatches.length * MAX_PROGRESS_EVENTS_PER_ATTEMPT)
      + 1),
    'progress must be transition-bounded, not token-driven',
  );
  for (const update of updates) {
    assert.ok(!JSON.stringify(update.partialResult,)
      .includes('PRIVATE_PROVIDER_PAYLOAD',),);
    assert.deepEqual(
      v.parse(
        RecordSchema,
        update.partialResult,
      )
        .details,
      { kind: 'advisor-progress', },
    );
  }
}
