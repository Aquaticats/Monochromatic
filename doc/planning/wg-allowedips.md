# Plan: generate WireGuard AllowedIPs

Status:
 implemented specification.
 Recovered from Pi session `019fa835-e9e8-7523-9746-e9b7a0305a84`,
 simplified and implemented on 2026-07-28.

## Goal

The `wg-allowedips` CLI computes a WireGuard `AllowedIPs` value from an allowed set and a disallowed set:

```text
result = union(allowed) − union(disallowed)
```

Each set may contain:

- IPv4 addresses;
- IPv6 addresses;
- IPv4 CIDR blocks;
- IPv6 CIDR blocks;
- autonomous system numbers;
- domains.

A domain contributes every address returned for it by the operating system resolver during that run.
The tool must preserve this direct set model.
It must not infer `0.0.0.0/0` or `::/0` from a disallowed entry.

## Package and command

Create the package with these identities:

- Path is `package/wg/cli-allowedips`.
- Package name is `@monochromatic-dev/wg-cli-allowedips`.
- Executable name is `wg-allowedips`.

The command contract is:

```text
wg-allowedips --allowed <path> --disallowed <path>
```

Both flags are required.
An empty disallowed file represents an empty set.
Do not add short aliases or positional input.
Do not add interactive prompts or alternate output modes.

Add `cidr-tools: '>=12.1.3'` to the pnpm catalog.
The package must depend on it through `catalog:`.
This validated version floor supports Node 22 or newer and has the required array input for both `excludeCidr`
parameters.

Call `excludeCidr(allowedNetworks, disallowedNetworks)` directly.
The operation unions both arrays before subtraction and returns a minimized sorted set of IPv4 and IPv6
networks.
Do not build another merge or interval layer around it.

## Input format

Each file is plain text with one entry per line.
For each line:

1. Trim surrounding whitespace.
2. Skip it when the result is empty or starts with `#`.
3. Treat a valid IP literal as one host address.
4. Treat an entry containing `/` as a CIDR block.
5. Treat a case-insensitive `AS<number>` entry as an autonomous system number.
6. Treat any other entry as a domain for name resolution.

Validate IP literals with `isIP` from `node:net`.
For a CIDR entry use `parseCidr` from `cidr-tools` and validate its address portion with `isIP`.
Reject a prefix above 32 for IPv4 or above 128 for IPv6.
This closes the bounds gap in the dependency's deliberately rudimentary validation without adding another
validation dependency.

A range means a CIDR block such as `10.0.0.0/8`.
Start-to-end syntax such as `10.0.0.1-10.0.0.20` is unsupported.
Comments occupy a whole trimmed line.
Inline comments are unsupported.

Resolve domains with `lookup` from `node:dns/promises` and `{ all: true }`.
This follows the operating system's name-resolution behavior.
That behavior includes hosts-file and split-horizon results.
Use every returned address as a host route.
When lookup reports `ENOTFOUND`,
write one warning to stderr for that domain and let the entry contribute no addresses.
Propagate every other lookup failure.
The generated result is a point-in-time snapshot.

Resolve ASNs through the shared IPinfo Lite integration under `package/config/tofu`.
Each ASN contributes every matching database network or single address.
Reuse its per-ASN cache,
month-scale refresh policy,
`IPINFO_TOKEN` configuration,
and stale-cache fallback.
When an ASN contributes no networks,
write one warning to stderr for that entry and let it contribute nothing.

Do not add:

- DNS caching;
- retries or backoff;
- TTL handling;
- resolver configuration.

## Processing and output

After parsing and resolving both files:

1. Compute the parts of IPv4 `127.0.0.0/8` and IPv6 `::1/128` not covered by the disallowed set.
2. Write one warning listing the uncovered remainder when either loopback range is not completely covered.
3. Fail when the allowed set is empty.
4. Pass both input sets directly to `excludeCidr` for output subtraction.
5. Join a nonempty result with `, ` and write one newline-terminated line to stdout.
6. Write nothing to stdout when the result is empty.

Loopback coverage is semantic.
Exact ranges and broader disallowed CIDRs satisfy the check.
The warning does not alter subtraction or stdout.

Every emitted token must include a CIDR prefix.
Individual IPv4 and IPv6 inputs therefore become `/32` and `/128` host routes.

Keep name resolution as an internal seam of the function that turns both file texts into output text.
Production passes the Node lookup adapter.
Tests pass a deterministic lookup adapter.
Do not expose this seam through the package interface.

Example:

```text
# allowed.txt
10.0.0.0/8
2001:db8::/126
```

```text
# disallowed.txt
10.0.0.0/9
2001:db8::/127
```

```console
$ wg-allowedips --allowed allowed.txt --disallowed disallowed.txt
10.128.0.0/9, 2001:db8::2/127
```

The output is only the value.
It does not include `AllowedIPs =`.

## Failure behavior

These conditions must fail the command before it writes a result:

- a required flag is missing;
- an input file cannot be read;
- a CIDR parser rejects an entry;
- a domain lookup fails with a code other than `ENOTFOUND`;
- an ASN database lookup fails without a cached fallback.

Let argument-parser and filesystem errors propagate.
Let CIDR-parser errors propagate.
Let resolver errors propagate except for domain `ENOTFOUND` and empty ASN results.
The explicit prefix-bound error must name the rejected entry.
Do not add retry loops or custom line-number diagnostics.
Do not add a separate exit-code taxonomy.

Set arithmetic needs no special diagnostics:

- Duplicate and overlapping allowed entries merge naturally.
- A disallowed entry outside the allowed set is a no-op.
- Partial overlap subtracts only the intersection.
- An empty disallowed set returns the minimized allowed set without a warning.
- Complete subtraction succeeds with empty stdout.

## Non-goals

Do not add:

- WireGuard or `wg-quick` configuration editing;
- non-portable `!` exclusion syntax;
- an implicit full-tunnel universe;
- arbitrary start-to-end address ranges;
- warnings for duplicates or overlap;
- warnings for cancellation or no-op exclusions;
- persistent state or network watching;
- automatic regeneration;
- JSON or TOML input;
- multiple output formats.

## Completion criteria

The package must include:

- normal repository scaffolding;
- `README.md`;
- license material;
- tests for every exported code path.

Tests must cover:

- IPv4 union and subtraction;
- IPv6 union and subtraction;
- individual IPs becoming host routes;
- deterministic domain results entering the correct set through the test lookup adapter;
- deterministic ASN results entering both sets through the test lookup adapter;
- ASN database single addresses becoming host routes;
- warnings for empty ASN results and failures for invalid ASN results;
- missing or partial loopback coverage warnings and exact or broader loopback coverage acceptance;
- blank and comment lines;
- duplicate and overlapping inputs;
- partial-overlap and out-of-set inputs;
- empty allowed behavior;
- empty disallowed behavior;
- empty result behavior;
- CIDR parser failure;
- lookup failure;
- missing flag and missing file failures;
- exact stdout formatting.

These package tasks must pass with zero lint findings:

- `lint`;
- `lint:types`;
- `test:unit`;
- `build`.

Finally invoke the built `wg-allowedips` executable against disposable input files.
Verify its stdout and exit status.

## API references

- [`cidr-tools` API][cidr-tools]
- [Node.js operating-system lookup API][node-lookup]
- [Node.js IP literal validator][node-is-ip]

[cidr-tools]: https://github.com/silverwind/cidr-tools#api
[node-is-ip]: https://nodejs.org/api/net.html#netisipinput
[node-lookup]: https://nodejs.org/api/dns.html#dnspromiseslookuphostname-options
