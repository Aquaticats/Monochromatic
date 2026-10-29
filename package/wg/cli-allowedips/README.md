# wg-allowedips

Generate the comma-separated value for a WireGuard `AllowedIPs` setting from allowed and disallowed address sets.
The command computes:

```text
union(allowed) − union(disallowed)
```

It does not infer a full-tunnel universe from disallowed entries.

## Usage

```console
$ wg-allowedips --allowed allowed.txt --disallowed disallowed.txt
10.128.0.0/9, 2001:db8::2/127
```

Both long options are required.
The command accepts no positional inputs or short aliases.
Its stdout contains only the value,
not `AllowedIPs =`.

## Input files

Each file contains one entry per line.
An entry can be:

- an IPv4 address;
- an IPv6 address;
- an IPv4 CIDR block;
- an IPv6 CIDR block;
- an autonomous system number in `AS<number>` form;
- a domain.

Surrounding whitespace is trimmed.
Blank lines and trimmed lines beginning with `#` are skipped.
Comments must occupy the whole line;
inline comments are not supported.

A range means a CIDR block such as `10.0.0.0/8`.
Start-to-end syntax is unsupported.

Example allowed file:

```text
# Private IPv4
10.0.0.0/8

# Documentation IPv6 range
2001:db8::/126
```

Example disallowed file:

```text
10.0.0.0/9
2001:db8::/127
```

## Domains

Each domain is resolved once for that input entry through Node's operating-system lookup with `{ all: true }`.
Every returned IPv4 and IPv6 address contributes one host route.
Results therefore reflect hosts-file,
split-horizon,
and other operating-system resolution behavior at that moment.

A hostname whose answers rotate between resolutions contributes only the addresses current at generation time.
Measured on 2026-10-06,
`huggingface.co` returned three distinct address sets within ten minutes behind CloudFront,
so a generated value can omit every address a later client actually connects to.
Applications that must stay outside the tunnel regardless of destination need the `wg-quicker`
`ExemptMark` cgroup exemption instead of a domain entry.

The command does not add DNS caching,
retries,
TTL handling,
or resolver configuration.
A lookup returning `ENOTFOUND` writes one warning to stderr for that domain and contributes no addresses.
Other lookup failures fail the command before it writes a result.

## Autonomous system numbers

ASN entries use case-insensitive `AS<number>` syntax,
for example `AS41231`.
Each entry contributes every network and single address assigned to that ASN by the [IPinfo Lite database][ipinfo-lite].
The same expansion works in allowed and disallowed inputs.
An ASN contributing no networks writes one warning to stderr for that entry and contributes nothing.
IPinfo Lite coverage is incomplete for some vendors:
[missing OpenAI and Anthropic ASNs](../../../doc/troubleshooting/ipinfo-lite-missing-openai-anthropic-asns.md)
records the measured 2026-10-06 gaps and the workarounds that replaced them.

ASN resolution comes from `@monochromatic-dev/wg-core-allowedips` through a static source import.
Fresh `cache_AS<number>.txt` snapshots avoid network access.
Snapshot entries are validated before use,
and refreshes replace snapshots atomically.
The cache directory is `$WG_ALLOWEDIPS_CACHE_DIRECTORY`,
then `$XDG_CACHE_HOME/wg-allowedips/asn`,
then `~/.cache/wg-allowedips/asn`.
An absent or expired snapshot refreshes from IPinfo using `IPINFO_TOKEN`.
A failed refresh uses a valid stale snapshot when available and fails when no valid snapshot exists.
No runtime workspace-package lookup is required.

IP address data is powered by [IPinfo][ipinfo] and used under CC-BY-SA-4.0.

## Loopback coverage

The disallowed networks should cover the complete IANA-designated loopback space:
IPv4 [`127.0.0.0/8`][iana-ipv4-special] and IPv6 [`::1/128`][iana-ipv6-special].
Coverage is semantic,
so broader disallowed CIDRs satisfy the check.
When any loopback addresses remain uncovered,
the command writes one warning to stderr listing the uncovered remainder.
The warning does not change set subtraction or stdout.

## Output

The result is a minimized,
sorted set of explicit CIDRs joined by `, ` and terminated by one newline.
Individual IPv4 and IPv6 addresses become `/32` and `/128` routes.

Complete subtraction succeeds and writes nothing to stdout.
An empty disallowed file returns the minimized allowed set.
An allowed file that contributes no addresses is an error.

## Validation

CIDR syntax is parsed by `cidr-tools`.
The command separately validates the original address text with Node's `isIP` and rejects prefixes above 32 for
IPv4 or 128 for IPv6.

Missing options,
unreadable files,
invalid CIDRs,
invalid family bounds,
and resolver failures other than `ENOTFOUND` propagate as command failures.
The command does not add an exit-code taxonomy.

## Shared library

Address-set parsing,
lookup seams,
ASN cache access,
and generation live under `package/wg/core-allowedips`.
This package owns only command-line parsing,
file reads,
and stdout output.
Both `wg-allowedips` and `wg-quicker` import the shared TypeScript source statically.

## Development

Run package checks through mise:

```console
mise run //package/wg/cli-allowedips:lint
mise run //package/wg/cli-allowedips:lint:types
mise run //package/wg/cli-allowedips:test:unit
mise run //package/wg/cli-allowedips:build
```

`buildAndTest` rebuilds the consumed artifacts before running unit and built-CLI tests:

```console
mise run //package/wg/cli-allowedips:buildAndTest
```

[iana-ipv4-special]: https://www.iana.org/assignments/iana-ipv4-special-registry/iana-ipv4-special-registry.xhtml
[iana-ipv6-special]: https://www.iana.org/assignments/iana-ipv6-special-registry/iana-ipv6-special-registry.xhtml
[ipinfo]: https://ipinfo.io
[ipinfo-lite]: https://ipinfo.io/developers/ipinfo-lite-database
