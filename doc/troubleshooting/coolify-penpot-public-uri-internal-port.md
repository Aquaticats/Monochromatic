# Coolify v4 shared environment gives Penpot 2.17.2 frontend an unreachable public port

A Penpot frontend behind HTTPS can load its initial page but time out on API requests
when Coolify v4 injects a service-wide `PENPOT_PUBLIC_URI` containing frontend container port `8080`.
The Source Compose can omit that frontend variable entirely.

Investigated on 2026-09-03.

## Symptom

The browser console reports:

```text
:8080/api/main/methods/get-enabled-flags:1 Failed to load resource: net::ERR_CONNECTION_TIMED_OUT
logging.cljc:294 WRN [app.main.repo] hint="retrying request", attempt=1, delay=1000, error="Failed to fetch"
```

Penpot's startup log identifies the unexpected origin:

```text
public-uri="https://penpot.c.aquati.cat:8080/"
```

The public and container-local frontend responses both contained:

```javascript
var penpotPublicURI = "https://penpot.c.aquati.cat:8080";
```

The Source Compose did not set frontend `PENPOT_PUBLIC_URI`.
It set only:

```yaml
frontend:
  environment:
    - SERVICE_URL_FRONTEND_8080
    - 'PENPOT_FLAGS=${PENPOT_FRONTEND_FLAGS:-enable-login-with-password}'
```

Backend and exporter already had the correct explicit external value:

```yaml
- 'PENPOT_PUBLIC_URI=https://penpot.c.aquati.cat'
```

## Root cause

### The runtime frontend has a variable absent from Source Compose

A terminal inside the live frontend proved the runtime value and generated file agree:

```text
PENPOT_PUBLIC_URI=<https://penpot.c.aquati.cat:8080>
SERVICE_URL_FRONTEND_8080=<https://penpot.c.aquati.cat:8080>
4:var penpotPublicURI = "https://penpot.c.aquati.cat:8080";
```

Fetching `http://127.0.0.1:8080/js/config.js` inside that container returned the same JavaScript.
This rules out Caddy response rewriting or caching as the source of the bad value.

The published `penpotapp/frontend:latest` image resolved to Penpot `2.17.2`,
manifest `sha256:94fa2864d8fc0cd62245af95c03cca89306a7fd23c206a98a3e9dc9a376ea27e`.
A disposable container received the exact two frontend variables shown in Source Compose.
Its generated file contained flags but no public URI:

```javascript
// Frontend configuration
var penpotFlags = "enable-login-with-password";
//var penpotOIDCName = "";
```

Its runtime environment contained only:

```text
SERVICE_URL_FRONTEND_8080=https://penpot.c.aquati.cat:8080
```

This rules out the official image translating `SERVICE_URL_FRONTEND_8080` into `PENPOT_PUBLIC_URI`.

### Coolify v4 injects one service-wide environment file into every container

Current Coolify's service parser adds `.env` to every Compose component at
[`bootstrap/helpers/parsers.php:2758-2766`][coolify-env-file-injection]:

```php
// Auto-inject .env file so Coolify environment variables are available inside containers
// This makes Services behave consistently with Applications
$existingEnvFiles = data_get($service, 'env_file');
$envFiles = collect(is_null($existingEnvFiles) ? [] :
    (is_array($existingEnvFiles) ? $existingEnvFiles : [$existingEnvFiles]))
    ->push('.env')
    ->unique()
    ->values();

$payload['env_file'] = $envFiles;
```

Coolify writes every service-level environment record into that one file at
[`app/Models/Service.php:1613-1649`][coolify-shared-env-write]:

```php
$commands[] = 'rm -f .env || true';
$envs_from_coolify = $this->environment_variables()->get();

foreach ($sorted as $env) {
    $envs->push("{$env->key}={$env->real_value}");
}

$envs_base64 = base64_encode($envs->implode("\n"));
$commands[] = "echo '$envs_base64' | base64 -d | tee .env > /dev/null";
```

Commit [`712d60c75b5d`][coolify-env-injection-commit] introduced the service injection on 2025-11-07.
Its commit message is `feat: ensure .env file exists for docker compose and auto-inject in payloads`.

Coolify issue [#7655][coolify-issue-7655] independently documents that all variables reach every Compose container.
A Coolify maintainer confirms this behavior will not be properly changed in v4
and says v5 will use separate files per container.

The live frontend therefore receives the stale port-suffixed `PENPOT_PUBLIC_URI` through Coolify's shared `.env`,
even though its own Source Compose environment omits the key.
The Coolify UI shows backend and exporter values as Compose-managed and correct,
but it does not expose the inherited stale frontend value as an editable frontend entry.

The exact database history that created the stale record was not read.
Its value equals the generated `SERVICE_URL_FRONTEND_8080`,
and Coolify's Penpot template assigns that generated value to `PENPOT_PUBLIC_URI` at
[`templates/compose/penpot.yaml:37-41`][coolify-template-backend] and
[`templates/compose/penpot.yaml:64-68`][coolify-template-exporter].
This is the likely origin,
not proof of the record's historical write path.

### Explicit Compose environment wins over the shared file

Docker's documented precedence places an `environment` attribute above an `env_file` attribute in
[Environment variables precedence][docker-env-precedence].
Its example concludes:

```text
The environment variable defined with the environment attribute takes precedence.
```

That explains both sides of the incident:

- Backend and exporter use their explicit correct values instead of the shared stale value.
- Frontend lacks an explicit value,
  so it inherits the stale value from Coolify's injected file.

Adding the same correct explicit value only to frontend overrides the injected value without touching Caddy or data.

### Penpot turns the injected value into browser request URLs

Penpot `2.17.2` frontend startup appends a non-empty `PENPOT_PUBLIC_URI` directly to `config.js` at
[`docker/images/files/nginx-entrypoint.sh:27-48`][penpot-entrypoint]:

```bash
if [ -n "$PENPOT_PUBLIC_URI" ]; then
    echo "var penpotPublicURI = \"$PENPOT_PUBLIC_URI\";" >> "$1";
fi

update_flags /var/www/app/js/config.js
```

The frontend prefers that global over the browser's actual origin at
[`frontend/src/app/config.cljs:175-183`][penpot-public-uri]:

```clojure
(def public-uri
  (normalize-uri (or (obj/get global "penpotPublicURI")
                     (obj/get location "origin"))))
```

The API repository joins requests onto that value at
[`frontend/src/app/main/repo.cljs:173-195`][penpot-api-uri]:

```clojure
(let [id     (or rename-to id)
      nid    (name id)
      request
      {:method method
       :uri (u/join cf/public-uri "api/main/methods/" nid)
       :credentials "include"}]
```

This call chain produces the observed request to external port `8080`.

### Earlier diagnosis was wrong

The first diagnosis inferred that the user had added a frontend
`PENPOT_PUBLIC_URI=$SERVICE_URL_FRONTEND_8080` assignment and recommended changing all three components.
The pasted Source Compose disproved that inference.
The exact-image fixture then proved that the shown frontend environment cannot generate `penpotPublicURI` by itself.
The running-container probe and Coolify source trace identified the shared `env_file` boundary instead.

The healthcheck edit did not alter the public URI.
The subsequent forced recreation most likely exposed the existing shared value by starting frontend again and
regenerating `config.js`.
The old container's environment and file were not captured,
so this timing explanation remains an evidence-backed hypothesis rather than a measured before-state.

## Verification

### Versions and source under test

- Live endpoint `https://penpot.c.aquati.cat`,
   probed on 2026-09-03.
- Published `penpotapp/frontend:latest` and `penpotapp/frontend:2.17.2` image.
- Penpot tag `2.17.2`,
   commit `1d2c37e52c733f74017d90b0fd1ae2d074a5c33d`.
- Coolify commit `b8866b87e8e855e041c21330352ca615521afed3`.
- Coolify injection commit `712d60c75b5db2cad57906c9a71fb3c6538fa29c`.

### Failing live probe

```bash
printf 'PENPOT_PUBLIC_URI=<%s>\n' "${PENPOT_PUBLIC_URI-<unset>}"
printf 'SERVICE_URL_FRONTEND_8080=<%s>\n' "${SERVICE_URL_FRONTEND_8080-<unset>}"
grep -n 'penpotPublicURI' /var/www/app/js/config.js || true
curl --fail --silent --show-error http://127.0.0.1:8080/js/config.js
```

Observed inside frontend before correction:

```text
PENPOT_PUBLIC_URI=<https://penpot.c.aquati.cat:8080>
SERVICE_URL_FRONTEND_8080=<https://penpot.c.aquati.cat:8080>
var penpotPublicURI = "https://penpot.c.aquati.cat:8080";
```

External port `8080` reproduced the browser failure:

```text
status=000 error=Connection timed out after 5002 milliseconds
```

### Passing live probe

The frontend Source Compose received one explicit entry:

```yaml
- 'PENPOT_PUBLIC_URI=https://penpot.c.aquati.cat'
```

After **Force Restart**,
the cache-busting public response contained:

```javascript
var penpotPublicURI = "https://penpot.c.aquati.cat";
```

An unauthenticated `agent-browser` session loaded the application and observed:

```text
version="2.17.2", public-uri="https://penpot.c.aquati.cat/"
GET https://penpot.c.aquati.cat/api/main/methods/get-enabled-flags 401
```

The page reported no browser errors.
The request used standard HTTPS rather than external port `8080`.
Status `401` is the expected unauthenticated application boundary response.

### Configurations that work

- Explicit frontend `PENPOT_PUBLIC_URI=https://penpot.c.aquati.cat` overrides Coolify's injected file.
- Existing explicit backend and exporter values remain correct.
- `SERVICE_URL_FRONTEND_8080` remains available for Coolify's internal-port routing.
- Custom Caddy continues serving the normal HTTPS origin without a configuration change.

### Configurations that fail

- Omitting frontend `PENPOT_PUBLIC_URI` lets the shared stale file supply the `:8080` value.
- Setting only `SERVICE_URL_FRONTEND_8080` does not make the official Penpot image derive a safe public URI.
- A public URI ending in `:8080` directs browser API requests to the unreachable external port.

## Verified workarounds

### Explicitly override only the frontend value

Keep the port-specific Coolify declaration and add one frontend entry:

```yaml
services:
  frontend:
    environment:
      - SERVICE_URL_FRONTEND_8080
      - 'PENPOT_PUBLIC_URI=https://penpot.c.aquati.cat'
      - 'PENPOT_FLAGS=${PENPOT_FRONTEND_FLAGS:-enable-login-with-password}'
```

Keep existing backend and exporter values unchanged.
Recreate frontend so its entrypoint regenerates `config.js`.

Tradeoff:
this is a component-local and verified correction,
but Coolify v4 still injects every service variable into every container.
The broader cross-container environment exposure remains until Coolify v5 changes the architecture.

### Remove or correct the stale service-level record only with direct evidence

Correcting the underlying shared record would remove the need for a frontend override.
The current Coolify UI displayed only correct Compose-managed backend and exporter entries,
and those dialogs instructed the operator to update Compose.
It did not expose the stale inherited frontend record for safe editing.

Tradeoff:
direct database or generated `.env` edits bypass Coolify ownership and can be regenerated.
Do not use them for this incident when the explicit frontend override is sufficient.

## What does not work

- **Blaming or changing Caddy for the injected value.**
  Container-local HTTP returned the same bad file before Caddy handled it.
- **Changing backend or exporter public URI again.**
  Their Source Compose values were already correct and explicit.
- **Changing only the healthcheck.**
  The healthcheck repair starts the service but does not control frontend runtime configuration.
- **Exposing external port `8080`.**
  This creates a noncanonical second origin instead of correcting Penpot's browser-facing URI.
- **Removing `SERVICE_URL_FRONTEND_8080`.**
  It identifies Coolify's intended frontend container port and is separate from Penpot's public URI.
- **Editing Coolify's generated `.env` on the host.**
  Coolify rewrites the file from its environment records during deployment.
- **Refreshing browser storage without correcting frontend environment.**
  The container-local generated file itself contained the wrong value.
- **Changing PostgreSQL,
   Redis,
   or asset volumes.**
  The issue is environment propagation and does not involve persistent Penpot data.

## Upstream filing artifact

### Upstream filing decision

No `.out-of-scope/` entry covers Coolify,
Penpot,
Compose environment sharing,
or reverse-proxy port handling.

Coolify issue [#7655][coolify-issue-7655] is an exact upstream match for the shared-environment mechanism.
Its issue body and all comments were reviewed.
The Penpot incident is a concrete application-level consequence and adds a verified consumer workaround,
but the maintainer has already accepted the mechanism and stated the version policy.
No duplicate issue should be filed.

The filing constraints resolve as follows:

1.  **Is it really upstream's fault?**
    Yes.
    Coolify v4 deliberately adds one service-wide environment file to every Compose component.
2.  **Can upstream fix it?**
    Yes in a new architecture.
    The maintainer plans per-container files in Coolify v5.
3.  **Are they supporting this use case?**
    Yes.
    Coolify ships a Penpot service template and supports multi-component Compose services.
4.  **Would the repo welcome our contribution?**
    Not for a v4 implementation from this investigation.
    The maintainer explicitly rejected a proper v4 fix as incompatible with the maintenance and migration plan,
    and characterized issue-thread AI patch attempts as unwanted.
5.  **Will they likely fix it?**
    Not in v4.
    The maintainer explicitly deferred the architectural correction to v5.
6.  **Have we prototyped a minimal fix compatible with their architecture?**
    No upstream patch was created because constraints 4 and 5 fail for v4,
    while the v5 architecture referenced by the maintainer is not the current source under investigation.
    The consumer-side Penpot override was implemented and verified instead.

The existing issue already establishes the mechanism and intended upstream direction.
The additional Penpot example does not justify posting after the maintainer's explicit guidance,
so there is no comment draft.

[coolify-env-file-injection]: https://github.com/coollabsio/coolify/blob/b8866b87e8e855e041c21330352ca615521afed3/bootstrap/helpers/parsers.php#L2758-L2766
[coolify-env-injection-commit]: https://github.com/coollabsio/coolify/commit/712d60c75b5db2cad57906c9a71fb3c6538fa29c
[coolify-issue-7655]: https://github.com/coollabsio/coolify/issues/7655
[coolify-shared-env-write]: https://github.com/coollabsio/coolify/blob/b8866b87e8e855e041c21330352ca615521afed3/app/Models/Service.php#L1613-L1649
[coolify-template-backend]: https://github.com/coollabsio/coolify/blob/b8866b87e8e855e041c21330352ca615521afed3/templates/compose/penpot.yaml#L37-L41
[coolify-template-exporter]: https://github.com/coollabsio/coolify/blob/b8866b87e8e855e041c21330352ca615521afed3/templates/compose/penpot.yaml#L64-L68
[docker-env-precedence]: https://docs.docker.com/compose/how-tos/environment-variables/envvars-precedence/
[penpot-api-uri]: https://github.com/penpot/penpot/blob/1d2c37e52c733f74017d90b0fd1ae2d074a5c33d/frontend/src/app/main/repo.cljs#L173-L195
[penpot-entrypoint]: https://github.com/penpot/penpot/blob/1d2c37e52c733f74017d90b0fd1ae2d074a5c33d/docker/images/files/nginx-entrypoint.sh#L27-L48
[penpot-public-uri]: https://github.com/penpot/penpot/blob/1d2c37e52c733f74017d90b0fd1ae2d074a5c33d/frontend/src/app/config.cljs#L175-L183
