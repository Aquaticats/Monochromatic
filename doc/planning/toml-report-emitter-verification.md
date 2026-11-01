# Verify report emitters before choosing artifact paths

The TOML continuation initially retained private shard JSON without copying the native CLI's default final report.
The frozen `package/mutation-test/cli/src/cli.ts` writes `mutation-report.json` beside its current directory
after orchestration completes.
The [TOML audit](../audit/depot-toml-reconciliation-2026-10-17.md)
records the separate default-report capture and source-preserving recovery bridge.

## Proposed instruction clarification

Clarify the existing `FCH` rule in `AGENTS.md` rather than adding another always-loaded rule.
The current rule requires fetching unseen material or reading on before concluding.
Its proposed replacement makes the output-discovery branch explicit:

```text
# Proposed AGENTS.md replacement
FCH:
Fetch unseen material or read on before concluding.
For artifacts, read each report writer and default destination.
Never hedge about text one read away.
```

The intended completion criterion is an inventory of every emitted report and its default destination
before choosing capture paths.
This is a proposal;
`AGENTS.md` is unchanged.
