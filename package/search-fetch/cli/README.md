# @monochromatic-dev/search-fetch-cli

`search-fetch`:
the terminal front door for the shared web search and fetch
tools,
writing the shared tools' rendered text to stdout.

## Why

The pi extension `@monochromatic-dev/search-fetch-pi-plugin` serves pi hosts and
`@monochromatic-dev/search-fetch-mcp` serves MCP hosts.
This package serves humans in a shell and agents invoking it through bash,
with the identical behavior:
DeepSeek-first search (maximum results with page content) with Exa and Linkup
fallback,
`gh` CLI routing for mapped GitHub URLs,
DeepSeek fetch before Linkup and Exa,
the global host blocklist,
base64 data-URL image stripping,
one whole-call deadline,
and the same response truncation.
All three surfaces delegate to `@monochromatic-dev/search-fetch-core`,
so behavior has exactly one implementation.

One surface per host:
a terminal session should run this command instead of loading an adapter.

## Invocation

Two forms run the same command:

```bash
# Root-linked bin, resolved through node_modules/.bin like mvm
search-fetch --help

# Built artifact by absolute path
node package/search-fetch/cli/dist/final/node/index.mjs --help
```

Build the artifact first with `mise run //package/search-fetch/cli:build`.
The root-linked form works in this checkout because the root `package.json`
depends on `@monochromatic-dev/search-fetch-cli`,
which links the `search-fetch` bin into the root `node_modules/.bin`.

Two verbs run one shared tool each:

- `search-fetch search <query>` runs `web_search`.
- `search-fetch fetch <url>` runs `web_fetch`.

## Grammar

The argument grammar is literal and uniform for both verbs:

- Exactly one positional argument names the query or URL.
- Everything after the first bare `--` is data:
  those words join with single spaces into the one argument,
  so `search-fetch search -- cachyos kernel 2026` builds one query string.
- Tokens after `--` are always data,
  never options:
  `search-fetch search -- --help` searches for the literal text `--help`.
- Supplying both forms together,
  a positional and `--` words at once,
  is unsupported and outside the contract.
  The grammar there is undefined by design,
  the README presents no behavior for it,
  and today's build refuses it as a usage error.
  Pass the argument in exactly one form.

Flags are long-form only,
and the flag grammar is deliberately strict:

- `--include-domains a.com,b.com` takes a comma list in one flag.
  Entries are trimmed and empty entries dropped,
  and a list whose entries are all empty after trimming (`--include-domains ",,"`)
  is the same as the flag being absent.
- `--from-date` and `--to-date` take ISO date strings,
  passed through exactly as the shared tool schema accepts them,
  with no extra validation.
- Repeating any single-valued flag is a usage error,
  not last-wins.
- An unknown flag,
  a missing flag value,
  several positionals before `--`,
  zero arguments,
  and an unknown verb are usage errors.
- A bare `search-fetch` with no verb prints usage on stderr and exits 2.
- `--help` prints global usage from `search-fetch --help`
  and verb usage from `search-fetch search --help`.
  `--version` prints this package's version.
  Both print to stdout and exit 0,
  and `--help` wins when both appear.

Known flags feed the tolerant shared tool schema:
`search-fetch fetch <url> --from-date ...` still fetches,
and the shared tool layer warns about and strips the key its fetch schema does
not accept,
exactly as it does for a host surface.

## Output

A data run writes only the shared tools' rendered text to stdout:
JSONL result lines for provider responses shaped like `{"results":[...]}`,
JSON or raw page markdown otherwise,
always ending with the truncation notice naming the full-response temp path when
truncated.
There is no pretty mode and no `--json` flag.

No `search-fetch://response/<id>` URIs and no `resource_link` blocks appear in
CLI output;
those are MCP transport features.
The truncation notice and its temp path serve the same purpose here,
and the temp file stays readable after the run.

Diagnostics and warnings go to stderr through tagged `@monochromatic-dev/logger-core`
loggers,
never interleaved with results.
A failing run leaves stdout empty.

## Exit codes

Exit codes and stream ownership are recorded once in
[doc/decision/cli-output-and-exit-codes.md](../../doc/decision/cli-output-and-exit-codes.md):

- `0`:
  success,
  and explicit `--help` or `--version` output.
- `1`:
  the operation failed:
  provider failure or deadline expiry.
- `2`:
  the caller or configuration is wrong:
  usage error,
  configuration error,
  or blocklist refusal.

`process.exitCode` is set explicitly only for the non-standard code `2`;
`0` and `1` come from normal return and thrown errors.

Argument parsing follows
[doc/decision/cli-arg-validation.md](../../doc/decision/cli-arg-validation.md):
`node:util` `parseArgs` in strict mode plus the shared Valibot tool specs.

## Configuration

Loads the same global config file as the pi extension
(`~/.pi/agent/extensions/pi-search-fetch.json`),
which now holds only the blocklist.
Provider keys come from `EXA_API_KEY`,
`LINKUP_API_KEY`,
and `DEEPSEEK_API_KEY` in
the process environment,
or from the repo's sops/age-encrypted `.env.local.json` through
`@monochromatic-dev/env-local-json-core`.
The `pi-` prefixes in the config filename and in the `pi-search-fetch-response-`
temp-dir prefix are historical and intentionally kept.

The reader locates the encrypted store in this order:

1. an explicit path,
   from `ENV_LOCAL_JSON_PATH` or a caller-supplied path,
2. `.env.local.json` at the nearest repository root,
3. a bounded walk from the working directory,
   stopping at the home directory,
   the containing repository,
   or the filesystem root.

Decryption needs `sops` on `PATH` and the age identity from the shared mise
chain,
which defaults to `~/.config/mise/age.txt`.
The reader returns only keys the environment did not supply,
and it never mutates `process.env`.

When the store is unavailable,
the command still runs the blocklist and
GitHub-routed `web_fetch` through `gh`.
Calls that need a missing provider key fail with a value-free store diagnostic
and exit 1.

The old `exaApiKey` and `linkupApiKey` config fields are rejected.
A config still carrying them raises `SearchFetchMigrationRequiredError` naming
`mise run secrets:migrate`,
reported as a configuration error on exit 2.

## Tests

`src/run.unit.test.ts` drives the built artifact through `runSearchFetchCli`:
both verbs,
every grammar form,
the furniture cases,
stream separation,
exit-status mapping,
and truncation with its readable temp path.
`src/params.unit.test.ts` covers the shared-schema validation gate.
`src/bin.unit.test.ts` runs the built bin as a real process for the
user-boundary cases:
bare invocation,
`--help`,
`--version`,
usage errors,
a blocklist refusal through a disposable config,
and a provider failure in degraded mode.
