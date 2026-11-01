# tofu - Hetzner Cloud firewall manager

OpenTofu configuration for managing Hetzner Cloud firewalls
with dynamic IP range aggregation from multiple CDNs and services.

## Overview

Manages firewall rules on Hetzner Cloud,
 with support for:

- Dynamic IP range fetching from multiple sources (Cloudflare,
   CloudFront,
   Fastly,
   GitHub,
   YouTube,
   etc.)
- ASN-based IP lookups via ipinfo.
  io
- CIDR block summarization to reduce rule counts
- Home ISP ranges for ssh/ping access (supplied locally,
   never committed)

## Components

### `@monochromatic-dev/wg-core-allowedips`

Shared IPinfo Lite adapter under `package/wg/core-allowedips` resolves case-insensitive `AS<number>` input to every
database network or single address.
It owns cache freshness,
streaming database filtering,
network validation,
atomic replacement,
and stale-cache fallback.
OpenTofu passes `src/` as an explicit cache directory,
so existing local snapshots remain usable without runtime package-path discovery.

### `fetch_ips.ts`

OpenTofu external-data wrapper around the shared static module import.
Features:

- Streaming JSON parsing with minimal memory usage
- 30-day cache for fetched data
- Graceful fallback to expired cache on failure

### `fetch_tor_relays.ts`

TypeScript script that fetches the top Tor guard relays from Onionoo and emits
their `ORPort 443` IPs as `/32` and `/128` CIDRs.
Features:

- Filter to `ORPort 443` only,
   so the firewall rules use a single port and the per-IP-per-port effective-rule count stays predictable
- 1-hour cache (the Tor guard set rotates on a different cadence than ASN data)
- Graceful fallback to expired cache on failure
- Feeds the balanced firewall rule list without consuming a separate descriptive firewall slot

### `hetzner.tf`

Main Terraform configuration that:

- Fetches IP ranges from multiple CDN APIs (Cloudflare,
   CloudFront,
   Fastly,
   GitHub,
   Coolify,
   YouTube)
- Looks up ASNs (Ubuntu,
   home ISP) via external data source
- Aggregates and summarizes IP blocks to minimize firewall rules
- Splits large source and destination CIDR lists into per-rule chunks
- Creates five generic Hetzner Cloud firewalls with balanced effective rule counts

## Setup

1. Store `HCLOUD_TOKEN` and `IPINFO_TOKEN` in the monorepo-root,
   SOPS-encrypted `.env.local.json`:

```bash
mise run --raw secrets:edit
```

2. Copy `hetzner.auto.tfvars.json.example` to `hetzner.auto.tfvars.json` and fill in the non-secret,
   machine-specific inputs:

```bash
cp package/config/tofu/hetzner.auto.tfvars.json.example \
  package/config/tofu/hetzner.auto.tfvars.json
```

3. Install dependencies:

```bash
pnpm install
```

4. Initialize,
   plan,
   and apply through mise so the root encrypted environment is loaded:

```bash
mise run //package/config/tofu:init
mise run //package/config/tofu:plan
mise run //package/config/tofu:apply
```

## Firewall rules

The configuration emits five generic firewalls named `tofu-0` through `tofu-4`.
Rules are intentionally mixed across those firewalls so each one stays under
Hetzner's active-firewall and effective-rule limits.
All five firewalls must be applied to the target server for the allowlist to be complete.
Set `firewall_server_ids` or `firewall_label_selectors` to let OpenTofu attach
all generated firewalls without using the Hetzner web UI.

### Balancing decision

Rules are balanced by effective-rule cost,
 not by traffic category.
Hetzner counts one rule with many source or destination CIDRs as many effective rules,
so descriptive firewalls such as `web_out` and `ubuntu_http` can exceed the 500-effective-rule cap
while other firewalls still have spare capacity.

The balancer sorts generated rules by effective-rule cost,
 heaviest first,
then assigns them in a snake pattern across the five firewalls:
`tofu-0`,
 `tofu-1`,
 `tofu-2`,
 `tofu-3`,
 `tofu-4`,
 `tofu-4`,
 `tofu-3`,
 `tofu-2`,
 `tofu-1`,
 `tofu-0`.
This keeps large rules from accumulating on one firewall while staying deterministic and readable in HCL.

A true greedy bin-packer could produce a marginally tighter distribution,
but it needs per-bucket running totals.
That stateful algorithm is simple in TypeScript or Python and awkward in declarative HCL.
The snake pattern keeps the implementation local to `hetzner.tf` and provides enough headroom
without adding another helper script to the OpenTofu plan path.

### Inbound

- HTTP (80):
   from all
- HTTPS TCP/UDP (443):
   from all
- SSH (22):
   from home ISP and Coolify IPs
- ICMP;
   from home ISP IPs
- Syncthing (21027/22000/22067/22070):
   from all

### Outbound

- DHCP (67-68)
- DNS to Hetzner (53)
- HTTPS TCP/UDP to CDN IPs (chunked to respect rule limits)
- HTTP TCP to package repository ranges for Ubuntu APT,
   archive.
  ubuntu.
  com,
   and nginx.
  org
- HTTPS TCP (443) to top Tor guards by consensus weight,
   filtered to ORPort 443 (for the v3 onion service)
- SMB/CIFS TCP (445) to configured Hetzner Storage Box hostnames or CIDRs

## IP sources

The configuration aggregates IPs from:

- Cloudflare (via API)
- CloudFront (via AWS API)
- Fastly (via API)
- GitHub (via meta API)
- Ubuntu (ASN AS41231)
- archive.
  ubuntu.
  com DNS edge IPs for HTTP APT bootstrap
- nginx.
  org DNS edge IPs for HTTP and HTTPS package access
- YouTube (via GitHub repo)
- Coolify (via API)
- Tor guards advertising ORPort 443 (via Onionoo,
   refreshed hourly)
- Hetzner Storage Box destinations (`*.your-storagebox.de`) via configured concrete hostnames or explicit CIDRs
- Various static IPs (LetsEncrypt,
   pCloud,
   Linkup,
   Resend,
   OpenRouter,
   etc.)

## Caddy

The server runs a custom Caddy build with plugins via xcaddy.
Rebuild when upgrading Caddy or changing the plugin set:

```bash
xcaddy build v2.11.2 \
  --with github.com/mholt/caddy-l4 \
  --with github.com/caddyserver/cache-handler \
  --with github.com/darkweak/storages/otter/caddy \
  --with github.com/greenpau/caddy-security \
  --with github.com/mholt/caddy-ratelimit \
  --with github.com/mholt/caddy-webdav
```

Provides these Caddy modules:

- `github.com/mholt/caddy-l4`:
   all `layer4.*`,
   `caddy.listeners.layer4`,
   `tls.handshake_match.alpn`
- `github.com/caddyserver/cache-handler`:
   `http.handlers.cache`,
   `admin.api.souin`
- `github.com/darkweak/storages/otter/caddy`:
   `storages.cache.otter`
- `github.com/greenpau/caddy-security`:
   `security`,
   `http.authentication.providers.authorizer`,
   `http.handlers.authenticator`
- `github.com/mholt/caddy-ratelimit`:
   `http.handlers.rate_limit`
- `github.com/mholt/caddy-webdav`:
   `http.handlers.webdav`

Verify after building:
 `./caddy list-modules` and `./caddy version`.

## SSH authentication

The server uses key-only root authentication with no root password set.
Coolify connects exclusively via SSH key (stored in Coolify's dashboard under **Security > Private Keys**),
and `sshd_config` enforces this with:

```sshconfig
PermitRootLogin prohibit-password
PubkeyAuthentication yes
PasswordAuthentication no
```

This eliminates brute-force password attacks entirely.
SSH access is further restricted at the firewall level to home ISP and Coolify IP ranges only (see `hetzner.tf`,
 port 22 rule).

For emergency access without the SSH key,
 use Hetzner Cloud Console's **Rescue** tab to boot a rescue system
and reset credentials or mount the filesystem.

## Local-only files

The following are gitignored:

- `/.env.local.json`:
   monorepo-root SOPS-encrypted store for `HCLOUD_TOKEN`,
   `IPINFO_TOKEN`,
   and every other local developer secret
- `*.auto.tfvars.json`:
   non-secret machine-specific values such as home ISP ASNs and Storage Box hostnames
- `src/cache_AS*.txt`:
   ASN lookup caches
- `terraform.tfstate*`:
   Terraform state
