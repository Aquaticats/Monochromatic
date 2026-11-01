# Fastly's REST API answers 200 with an empty list, 500, or 404 for TLS and domain resources that exist and are serving traffic

**Date**:
 2026-10-09.
**Subject**:
 `GET https://api.fastly.com/service/{id}/version/{n}/domain` returns `[]` for
a service whose domain is live and serving TLS traffic,
`GET /service?page[size]=20` returns HTTP 500 with a generic retry message,
and four `/tls/...` read paths return 404 for resources whose identifiers the
API itself handed back moments earlier.
Discovered while adding a second hostname to a Fastly service during
[`doc/handover/acm-caa-renewal-blocked.md`](../handover/acm-caa-renewal-blocked.md)
work for
[issue 674](https://github.com/Aquaticat/Monochromatic/issues/674).

## Symptom

A service with one live domain,
`fastly.aquati.cat`,
serving HTTP 200 through a
valid Certainly certificate,
reports no domains at all:

```text
GET /service/<service-id>/version/4/domain
200
[]
```

The active version also carries no configuration in its own representation.
`GET /service/<service-id>/version/4` returns exactly twelve keys:

```text
active, comment, created_at, deleted_at, deployed, environments,
locked, number, service_id, staging, testing, updated_at
```

There is no `domains`,
`backends`,
`headers`,
`request_settings`,
or `settings`
member,
so a reader following the shape of a version object finds nothing to
inspect.
Other sub-resources of the same version do return their contents:

```text
GET /service/<service-id>/version/4/backend
200
[{"name":"Hetzner","address":"aquati.cat","port":443, ...}]
```

So the version is reachable and partly populated,
which makes the empty domain
list look like a configuration problem rather than a wrong endpoint.

Adding pagination to the service list turns a working call into a server error:

```text
GET /service
200
[{"type":"vcl","version":4, ...}]

GET /service?page[size]=20
500
{"msg":"An error occurred while connecting to the fastly API, please try your
 request again.","detail":"An internal error has occurred. Please contact
 Fastly at support@fastly.com if the error persists."}
```

The message names a connection failure and asks the caller to retry,
so the
first reading is a transient fault.
Retrying reproduces it every time.
The only
variable is the query parameter.

Four TLS read paths 404.
Two of them 404 for identifiers the API returned in a
preceding 200 response:

```text
GET /tls/domains/fastly.aquati.cat
404
<h1>Not Found</h1>

GET /tls/certificates/<tls-certificate-id>
404
{"errors":[{"title":"Not found","detail":"The certificate you requested was not found"}]}

GET /tls/authorizations
404
<h1>Not Found</h1>

GET /tls/activation-results/<tls-activation-id>
404
<h1>Not Found</h1>
```

`GET /tls/domains` lists `fastly.aquati.cat` as its only entry,
and the
certificate identifier came out of
`GET /tls/subscriptions/<id>?include=tls_certificates` in the same session.
Asking for either one individually says it does not exist.

A sibling command inconsistency appears on the CLI side.
`fastly domain list`
accepts `--json`;
`fastly domain create` does not,
and rejects it by printing
its usage block with no diagnostic naming the flag:

```text
$ fastly domain create --fqdn mirror.fastly.aquati.cat --service-id <service-id> --json
USAGE
  fastly domain create --fqdn=FQDN [<flags>]
```

Nothing is created,
and the exit path is the same one a missing required flag
takes.

## Root cause

Fastly's API is a closed service with no public source,
so none of the
server-side behaviour here can be traced to a code path.
What follows is
black-box measurement.
The CLI is open source,
and the `--json` asymmetry is
traced to source in this section.

### Two 404 shapes, and what each means

The four 404s are not one failure.
They come in two distinguishable bodies,
and the difference is diagnostic:

- `<h1>Not Found</h1>`,
   18 bytes of HTML,
   for `/tls/domains/{id}`,
  `/tls/authorizations`,
  `/tls/authorizations/{id}`,
  and
  `/tls/activation-results/{id}`.
   An HTML body from a JSON API means the
  router matched nothing.
   These routes do not exist.
- `{"errors":[{"title":"Not found", ...}]}`,
   JSON:API shaped,
   for
  `/tls/certificates/{id}`.
   The route exists and ran,
   and the lookup
  returned no row.

That split explains the certificate case.
`/tls/certificates` is the
custom-certificate collection,
the one an account populates by uploading a
`cert_blob`.
The certificate in question is a **managed** certificate that
Fastly procured through ACME.
It has an identifier and it appears in a
subscription's `included` array,
but it is not a row in the custom-certificate
store,
so the individual read finds nothing while the relationship read returns
it in full.
The collection read `GET /tls/certificates` returned
`{"data":[]}` in the same account,
which is consistent:
no custom certificates
were ever uploaded.

`/tls/authorizations` has no collection route at all.
Authorizations are
reachable only as an `include` on a subscription:

```text
GET /tls/subscriptions/<id>?include=tls_authorizations
200
```

The relationship is real;
the standalone path is not.

### The service list is not a JSON:API collection

`GET /service` returns a bare JSON array,
not the `{"data": [...], "links":
{...}, "meta": {...}}` envelope that every paginated Fastly endpoint returns.
It takes no `page[number]` or `page[size]`.
Passing one does not produce a
validation error naming the parameter;
it produces a 500 whose text blames a
connection failure.
The cause cannot be established from outside,
but the
behaviour is deterministic and the parameter is the only variable.

### Domains moved out of the version object

`GET /service/{id}/version/{n}` returns version metadata only,
and
`/version/{n}/domain` returned the account's auto-generated default hostname
for versions 1 to 3 while returning `[]` for the active version 4.
The live
domain is attached at the **service** level instead,
and the CLI reads it from
there:

```text
$ fastly domain list --service-id <service-id>
FQDN               DOMAIN ID              SERVICE ID             CREATED AT
fastly.aquati.cat  <domain-id>            <service-id>           2026-05-09 02:38:06 +0000 UTC
```

The service in this account reports
`"environments":[{"name":"production","active_version":4}]`,
and the version
object carries the same `environments` member.
The per-version domain list is
the legacy view,
and it is empty rather than absent for a service using the
environment model.
`GET /service/{id}/environments` is not a route;
it answers
`{"errors":[{"detail":"Route not found","title":"Not found"}]}`.

The practical consequence is the dangerous part:
a caller who reads
`/version/{active}/domain`,
sees `[]`,
and concludes the service has no domains
will reach a false conclusion about a service that is serving traffic.
Nothing
in the empty response hints that a different resource holds the answer.

### The `--json` asymmetry, traced to source

Cloned per this repository's convention:

```bash
gh repo clone fastly/cli "${HOME}/temp/agent/fastly-cli-2026-10-09" -- --depth 1
```

At commit `bb93017`,
`pkg/commands/domain/list.go:41` registers a JSON output
flag:

```go
c.RegisterFlagBool(c.JSONFlag()) // --json
```

`pkg/commands/domain/create.go:36` to `pkg/commands/domain/create.go:45`
registers the complete flag set for `create`,
and no JSON flag is among them:

```go
c.CmdClause = parent.Command("create", "Create a domain").Alias("add")

// Optional.
c.CmdClause.Flag("description", "The description for the domain").Action(c.description.Set).StringVar(&c.description.Value)
c.CmdClause.Flag("fqdn", "The fully qualified domain name").Required().StringVar(&c.fqdn)
c.RegisterFlag(argparser.StringFlagOpts{
	Name:        argparser.FlagServiceIDName,
	Description: "The service_id associated with your domain",
	Dst:         &c.serviceID,
	Short:       's',
})
```

So the flag genuinely does not exist on `create`.
The usage-only output is the
flag parser's unrecognised-argument path,
which does not name the offending
flag.

## Verification

Tested on 2026-10-09 against `https://api.fastly.com` with Fastly CLI 16.1.0
installed through `mise`,
using Node 26's `fetch` and a legacy API token from
`~/.config/fastly/config.toml`.
No client library was involved,
so the results
describe the service rather than a wrapper.
CLI source read at commit
`bb93017` of `fastly/cli`.

Runnable harness.
It prints one line per path and never prints the token:

```bash
node --input-type=module -e '
import {readFileSync} from "node:fs";
import {homedir} from "node:os";
const t = readFileSync(homedir() + "/.config/fastly/config.toml", "utf8");
const tok = t.match(/^\s*token\s*=\s*"([^"]+)"/m)[1];
const paths = ["/service", "/service?page[size]=20", "/tls/domains", "/tls/authorizations"];
for (const p of paths) {
  const r = await fetch("https://api.fastly.com" + p, {headers: {"Fastly-Key": tok, Accept: "application/vnd.api+json"}});
  const body = await r.text();
  console.log(String(r.status).padEnd(4), p.padEnd(30), body.slice(0, 80).replace(/\n/g, " "));
}'
```

### Catalog: reads that succeed

All with `Fastly-Key` and `Accept: application/vnd.api+json`:

- `GET /service`,
   bare array,
   no pagination parameters.
- `GET /service/{id}/details`
- `GET /service/{id}/version/{n}` and its `/backend`,
   `/header`,
  `/request_settings`,
  `/gzip`,
  `/snippet`,
  `/vcl`,
  `/healthcheck`,
  `/dictionary`,
  `/cache_settings`,
  `/response_object` sub-resources.
- `GET /tls/domains`,
   optionally with
  `?filter[in_use]=true&include=tls_activations,tls_certificates`.
- `GET /tls/subscriptions`,
   with `filter[tls_domains.id]`,
  `filter[state]`,
  and `include=tls_authorizations,tls_certificates`.
- `GET /tls/subscriptions/{id}?include=tls_authorizations,tls_certificates`,
  which is the only route that returns authorization state and challenge
  records.
- `GET /tls/activations?filter[tls_domain.id]={domain}&include=tls_certificate,tls_configuration`
- `GET /tls/configurations/{id}?include=dns_records`,
   which returns the
  anycast `A` records and the `CNAME` target Fastly expects for a hostname on
  that configuration.
- `GET /current_customer` and `GET /current_user`.

### Catalog: reads that fail, split by body shape

HTML `<h1>Not Found</h1>`,
meaning no such route:

- `GET /tls/domains/{id}`
- `GET /tls/authorizations`
- `GET /tls/authorizations/{id}`
- `GET /tls/activation-results/{id}`

JSON:API `{"errors":[{"title":"Not found", ...}]}`,
meaning the route ran and
found nothing:

- `GET /tls/certificates/{id}` for a **managed** certificate.
   The same
  identifier is returned in full by a subscription `include`.
- `GET /service/{id}/environments`,
   with `"detail":"Route not found"`.

HTTP 500 with a retry-and-contact-support message:

- `GET /service?page[size]=20`,
   and the same with `page[size]=100`.
  `GET /service` with no query string succeeds.

HTTP 200 with content that contradicts live state:

- `GET /service/{id}/version/{n}/domain` returning `[]` for the active version
  of a service whose domain is serving traffic.

### Positive control

The empty domain list is not an empty account.
In the same session the domain
answered HTTP 200 on two routes with body digests identical to the origin,
and
`fastly domain list --service-id <service-id>` printed it with an identifier.
A probe of the service's auto-generated default hostname returned
`HTTP/1.1 500 Domain Not Found` from the edge,
which shows the edge does
distinguish known from unknown hostnames and that the customer hostname was
genuinely routed.

## Verified workarounds

### 1. Read domains from the service-level CLI command

```bash
fastly domain list --service-id <service-id>
```

Tradeoff:
it shells out to the CLI rather than staying in one HTTP client,
and
it returns the environment-model view only.
It is the one read observed here
that reports a live domain correctly.

### 2. Treat the subscription include as the only TLS state source

Read authorization state,
challenge records,
and managed-certificate
attributes from
`GET /tls/subscriptions/{id}?include=tls_authorizations,tls_certificates`
rather than from any `/tls/authorizations` or `/tls/certificates/{id}` path.

Tradeoff:
one subscription identifier must be obtained first,
from
`GET /tls/subscriptions?filter[tls_domains.id]={domain}`.
The authorization's
`updated_at` can be months older than the certificate it authorized,
so the
`state` field is a cached value and is not evidence about a DNS change made
after it.

### 3. Distinguish the two 404 bodies before concluding a resource is missing

An HTML body means the path is wrong;
a JSON:API body means the resource is.

Tradeoff:
it requires reading the raw body instead of only the status,
which
most HTTP helpers discard.

### 4. Never pass pagination parameters to `GET /service`

Tradeoff:
none,
but the 500 gives no hint that the parameter is the cause,
so
this has to be known in advance or discovered by bisecting the URL.

### 5. Check a subcommand's own `--help` before assuming a sibling's flags carry over

`fastly domain create --help` lists every flag it accepts.

Tradeoff:
one extra invocation per command.
It is cheaper than a usage dump
that does not name the rejected flag.

## What does not work

- **Retrying the 500.**
   The message asks for a retry and names a connection
  failure.
   It is deterministic and parameter-driven,
   not transient.
- **Inferring that a service has no domains from
  `/version/{active}/domain`.**
   This is the failure that costs the most
  time,
   because the response is a well-formed empty list rather than an
  error.
- **Assuming `--json` is global because a sibling subcommand has it.**
  `pkg/commands/domain/list.go:41` registers it;
  `pkg/commands/domain/create.go`
  does not.
- **Reading managed-certificate attributes from `/tls/certificates/{id}`.**
  Use the subscription include.
- **Treating `tls_authorizations.state` as live.**
   Measured `state: passing`
  with `updated_at` four months before a DNS change that could invalidate it.

## Upstream filing decision

`.out-of-scope/` was checked:
eleven files,
none matching Fastly,
a CDN or
DNS provider,
or this bug class.

The upstream tracker was searched for a duplicate before drafting,
with
`gh search issues` against `fastly/cli` for "domain create json",
"missing json
output",
and "json flag create".
Only one thread came back,
[fastly/cli#1353](https://github.com/fastly/cli/issues/1353),
which is about a different command and is closed.
That single hit doubles as the
positive control proving the search runs rather than failing silently.
No
existing issue covers `domain create`.

The six-constraint check splits by finding.
The API findings end at do not
file.
The CLI `--json` finding ends at fileable,
and its draft is kept below.
An earlier version of this section recorded "do not file" for both and rated
constraint 6 not applicable;
that audit was written before the
auto-prototype requirement in
`.agents/skills/troubleshooting-doc/SKILL.md`
had been read,
and before #1353 had been found.
Both are corrected here.

1.  **Is it really upstream's fault?**
    Split.
    The HTML-versus-JSON:API 404
    split,
    the absence of a `/tls/authorizations` collection,
    and the
    managed-certificate lookup miss are API surface facts that are simply
    undocumented at the paths a reader would guess.
    The 500 for an
    unsupported query parameter is the clearest defect:
    an unsupported
    parameter should be a 400 naming the parameter,
    not a 500 blaming a
    connection failure.
    The empty per-version domain list is a migration
    artifact rather than a bug,
    but it is silent,
    and silence is what makes
    it costly.
    The `--json` asymmetry was first
    recorded here as a design choice rather than a defect.
    That reading does
    not survive the tracker:
    [fastly/cli#1353](https://github.com/fastly/cli/issues/1353),
    "Missing JSON output for service-version clone command",
    is labelled
    `bug` **and** `good first issue`,
    was closed as fixed on 2025-11-06,
    and
    its reporter's use case is scripting a create-then-read sequence,
    the same
    shape as `domain create`.
    Upstream therefore already classifies a
    subcommand lacking the `--json` its siblings carry as a bug.
2.  **Can upstream fix it?**
    The API items,
    yes,
    but only Fastly can,
    and
    no public source repository exists for `api.fastly.com`.
    The CLI item is
    fixable in `fastly/cli` by registering the JSON flag on `create`,
    or by
    making the parser name the unrecognised flag.
3.  **Are they supporting this use case?**
    Yes for the API:
    these are
    documented resource families,
    and
    `GET /tls/configurations/{id}?include=dns_records` is explicitly
    recommended for DNS setup.
    No for the guessed paths:
    nothing documents
    `/tls/authorizations` as a collection,
    and the absence is discoverable
    only by trying it.
4.  **Would the repo welcome our contribution?**
    For `fastly/cli`,
    probably.
    Checked `CONTRIBUTING.md`,
    which invites opening an issue to
    discuss before a pull request,
    `ISSUES.md`,
    which lists "A command
    produces incorrect output or wrong results" and "Commands or flags don't
    work as documented" as bug types and routes account-specific behaviour to
    Fastly support instead,
    and `SECURITY.md`,
    which directs security
    reports to a private process.
    No ban on external or AI-assisted reports
    was found in any of the three.
    The `--json` finding fits
    `ISSUES.md`'s Bug type on the strength of #1353's labels,
    not its Feature
    Request type as first recorded here.
5.  **Will they likely fix it?**
    Split.
    For the CLI `--json` finding,
    yes:
    #1353 is the same bug class on a sibling command and it was fixed
    and closed.
    For the API findings,
    no signal.
    The API behaviours are
    long-standing enough that four separate paths share them,
    which suggests
    they are settled surface rather than regressions.
    There is no public
    tracker for `api.fastly.com`,
    so no search is possible and none was run;
    the searches recorded at the head of this section covered the CLI side.
    Per
    the skill,
    absence of signal is not a fail.
6.  **Have we prototyped a minimal fix?**
    Split.
    For the API findings,
    genuinely not applicable:
    there is no source to patch,
    and the
    workarounds in this document are reading discipline rather than code.
    For
    the CLI `--json` finding,
    yes,
    and the earlier "not applicable" was
    wrong because it rested on the constraint 1 and 5 readings corrected above.

    Prototyped in the same disposable clone used for
    [`fastly-cli-profile-list-prints-api-token.md`](fastly-cli-profile-list-prints-api-token.md),
    origin verified as `https://github.com/fastly/cli.git` at `bb93017`.
    The
    fix follows `pkg/commands/service/version/clone.go`,
    which is how #1353
    was fixed:
    embed `argparser.JSONOutput`,
    register `c.JSONFlag()`,
    add
    the `--verbose`/`--json` guard that 388 files in the tree already carry,
    and call `c.WriteJSON(out, d)` before the `text.Success` line.
    Two files
    change,
    `pkg/commands/domain/create.go` and its existing test.

    The diff is
    [`fastly-api-reads-report-live-resources-absent.json-flag.patch`](fastly-api-reads-report-live-resources-absent.json-flag.patch),
    86 lines.
    It was confirmed to forward-apply to pristine `bb93017` bytes
    extracted with `git show`,
    and the applied result was confirmed
    byte-identical to the tree the tests ran against.

    ```bash
    go test ./pkg/commands/domain/ -run TestDomainCreate -count=1
    ```

    Pre-patch,
     with the original `create.go` and the updated test:

    ```text
    --- FAIL: TestDomainCreate/#03
        want no error, have "error parsing arguments: unknown long flag '--json'"
    --- FAIL: TestDomainCreate/#04
        want "invalid flag combination, --verbose and --json",
        have "error parsing arguments: unknown long flag '--json'"
    ```

    Post-patch:

    ```text
    ok  	github.com/fastly/cli/pkg/commands/domain	0.139s
    ```

    `go vet ./pkg/commands/domain/` is clean and `go build ./...` succeeds.

Decision,
 API findings:
do not file.
There is no public source repository and
no public tracker for `api.fastly.com`.
The one finding worth escalating if it
recurs is the 500 for an unsupported query parameter on `GET /service`,
and the
channel for it is Fastly support,
because the fault is server-side.

Decision,
 CLI `--json` finding:
all six constraints hold,
 so the draft below is
fileable as-is.
It has **not** been filed.
Opening an issue on a third-party
tracker is an external action,
 and this repository's agent guidance requires
authorization for that rather than treating it as implied by writing the draft.

~~~md
Title: `fastly domain create` has no `--json`, unlike `fastly domain list` and the other create commands

Labels: bug, good first issue

**Version**

Fastly CLI version 16.1.0
Source read at commit bb93017.

**What happened**

`fastly domain list` registers a JSON output flag at `pkg/commands/domain/list.go:41`:

    c.RegisterFlagBool(c.JSONFlag()) // --json

`fastly domain create` registers only `description`, `fqdn`, and `service-id`, at `pkg/commands/domain/create.go:36` to `:45`. Passing `--json` prints the usage block and creates nothing, without naming the rejected flag:

    $ fastly domain create --fqdn mirror.example.com --service-id <id> --json
    USAGE
      fastly domain create --fqdn=FQDN [<flags>]

This is the same gap as #1353, which was fixed for `service-version clone` by following the `WriteJSON` pattern. The scripting case is the one #1353's reporter described: create a resource, then need its identifier without parsing prose or making a second call. `domain create` already prints the domain-id in its success line, so the value is available internally and only the structured form is missing.

**Reproduction**

    fastly domain create --fqdn does-not-matter.example.com --json

Prints usage and exits without creating anything. Compare `fastly domain list --json`, which emits structured output.

**Suggested fix**

Mirror `pkg/commands/service/version/clone.go`:

1. Embed `argparser.JSONOutput` in `CreateCommand`.
2. Add `c.RegisterFlagBool(c.JSONFlag())` in `NewCreateCommand`.
3. Add the `--verbose`/`--json` guard at the top of `Exec`, as 388 files in the tree already do.
4. Call `c.WriteJSON(out, d)` before the existing `text.Success` line.

A patch implementing exactly this, with pre-patch and post-patch `go test ./pkg/commands/domain/ -run TestDomainCreate -count=1` output and two added scenarios, is available on request. Prepared with AI assistance; the reproduction and the test runs above were executed against a real install and a real clone at bb93017.
~~~

## References

- Fastly TLS API reference index:
  <https://www.fastly.com/documentation/reference/api/tls>
- The prototyped `--json` fix recorded under constraint 6:
  [`fastly-api-reads-report-live-resources-absent.json-flag.patch`](fastly-api-reads-report-live-resources-absent.json-flag.patch)
- Precedent for that fix,
  the same bug class on a sibling command,
  closed
  as fixed:
  <https://github.com/fastly/cli/issues/1353>
- Fastly TLS subscriptions API reference,
  which documents the `include`
  values that make authorizations reachable:
  <https://www.fastly.com/documentation/reference/api/tls/subs/>
- Fastly guide,
  using Fastly with apex domains,
  which is where the anycast
  `A` records and the guidance against ALIAS and ANAME flattening are
  documented:
  <https://www.fastly.com/documentation/guides/full-site-delivery/domains-and-origins/using-fastly-with-apex-domains>
- Fastly guide,
  TLS prerequisites and limitations,
  including the
  certification authorities available for managed certificates:
  <https://www.fastly.com/documentation/guides/getting-started/domains/securing-domains/tls-prerequisites-and-limitations>
- Fastly product page for Certainly,
  which states the 30-day validity and
  the day-20 re-verification contract:
  <https://docs.fastly.com/products/certainly>
- `fastly/cli` source read for the `--json` asymmetry,
  at commit `bb93017`:
  <https://github.com/fastly/cli>
- Related:
  [`njalla-dns-api-jsonrpc-shape.md`](njalla-dns-api-jsonrpc-shape.md),
  the same class of undocumented-wire-shape finding at the DNS provider,
  and
  [`doc/handover/acm-caa-renewal-blocked.md`](../handover/acm-caa-renewal-blocked.md),
  the task this surfaced during.
