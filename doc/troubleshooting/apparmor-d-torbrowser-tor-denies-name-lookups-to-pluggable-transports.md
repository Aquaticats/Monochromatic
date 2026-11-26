# apparmor.d `torbrowser-tor` blocks Snowflake: transports inherit Tor's lookup denies and TCP-only network

Diagnosis recorded on `2026-11-26T02:23:59-05:00`.
The local workaround is applied on this host; nothing was posted upstream.
This record is unapproved.

## Symptom

Tor Browser 16.0.3 with Snowflake bridges (`Bridge snowflake ...` in `TorBrowser/Data/Tor/torrc`)
never connected while apparmor.d's `torbrowser-tor` profile enforced.
`tor` restarted its pluggable transport, `lyrebird` 0.10.0, every few minutes.
The audit log showed only the socket half of the problem:

```text
apparmor="DENIED" operation="create" class="net" info="failed type match" profile="torbrowser-tor" comm="lyrebird" family="inet" sock_type="dgram"
apparmor="DENIED" operation="create" class="net" info="failed af match" profile="torbrowser-tor" comm="lyrebird" family="inet6" sock_type="dgram"
apparmor="DENIED" operation="create" class="net" info="failed af match" profile="torbrowser-tor" comm="lyrebird" family="inet6" sock_type="stream"
```

After those were allowed,
Tor still did not connect and nothing more was logged.

## Root cause

The profile runs every pluggable transport inside Tor's own profile,
allows only IPv4 TCP,
and denies the name-service files. [Profile source][prof]

```text
# apparmor.d/groups/browsers/torbrowser-tor:18-44 (apparmor.d main 4c4fec70fc)
  network inet stream,
  ...
  @{lib_dirs}/TorBrowser/Tor/PluggableTransports/**  mix,
  ...
  deny /etc/nsswitch.conf r,
  ...
  deny /etc/resolv.conf r,
```

`tor` needs neither file as a client, so the denies suit it.
Snowflake does need them:
it resolves its domain front (`fronts=app.datapacket.com,...`) and its STUN servers (`ice=stun:stun.epygi.com:3478,...`)
and sends UDP to them.
`deny` rules are not logged, so the lookup failure is silent,
and they hold even in complain mode
(apparmor.d `docs/issues.md:26`: "`deny` rules are enforced even in *complain* mode"). [Known issues][issues]
Without `/etc/resolv.conf` a resolver falls back to `127.0.0.1:53`;
on this host only systemd-resolved's `127.0.0.53` and `127.0.0.54` listen on port 53.

## Verification

Versions:
`apparmor` 4.1.8-2.1,
`apparmor.d-git` v0.4916.0.r38.g4c4fec7-1
(the same commit as upstream `main` `4c4fec70fcd19307a871b3ecc7f117a9ad989c9f`),
Tor Browser 16.0.3 (`tbb_version.json`),
`lyrebird` 0.10.0.

```sh
# Any working directory; the profile name is the stock one
aa-exec --profile torbrowser-tor -- getent hosts app.datapacket.com; echo "exit=$?"
getent hosts app.datapacket.com; echo "exit=$?"
```

### Fails

- Under `torbrowser-tor`: no output, `exit=2`, and no AVC.
- Snowflake under the stock profile: no connection; with only the socket rules added, still no connection.

### Works

- Outside the profile: `185.59.222.177  1232748490.rsc.cdn77.org`, `exit=0`.
- With the local workaround (next section), after a full Tor Browser restart:
  `lyrebird` ran as `torbrowser-tor//lyrebird`,
  the owner reported Tor Browser connected,
  and `curl --socks5-hostname 127.0.0.1:9150 https://check.torproject.org/api/ip` returned
  `{"IsTor":true,"IP":"204.8.96.168"}`, with the child profile enforcing.
- The upstream patch beside this file
  ([the patch file][patch]),
  applied to the installed abi 4.0 build and loaded under a test name with no attachment:
  `getent` under its `pluggable-transports` child resolved the front (`exit=0`),
  the same lookup under the patched parent still failed (`exit=2`, plus a logged `inet dgram` denial),
  and `lyrebird` started under the child and printed `CMETHOD snowflake socks5 127.0.0.1:44999` with no denial.
  The test profile was removed afterwards.
  The unmodified upstream file declares `abi <abi/5.0>`, which this host's parser does not ship,
  so the patch was compiled against the installed build, not the upstream file itself.

## Verified workarounds

### A local child profile chosen with `priority=1`

`/etc/apparmor.d/local/torbrowser-tor` on this host adds
`priority=1 @{lib_dirs}/TorBrowser/Tor/PluggableTransports/lyrebird rcx -> lyrebird,`
and a `profile lyrebird { ... }` with `base-strict`, `nameservice-strict`, `ssl_certs`,
IPv4 and IPv6 stream and datagram sockets, `netlink raw`, its `pt_state` folder and the Go runtime reads;
the full text is under "Tor Browser's pluggable transport and shader cache"
in `doc/troubleshooting/cachyos-apparmor-enforcement-workload-tuning.md`.
Tradeoffs:
it needs AppArmor 4.1's `priority=` (`apparmor.d(5)`, "priority"),
it names only `lyrebird`, so another transport binary would still run under the parent,
and it must be kept in step if upstream renames the folder.

## What does not work

- Adding `network inet dgram` and friends to the local file alone: sockets open, lookups still fail.
- Allowing `/etc/resolv.conf` in the local file: a `deny` cannot be lifted by an allow at the same priority.
- An exact-path `cx` rule for `lyrebird` without `priority`:
  `apparmor_parser` stops with "profile has merged rule with conflicting x modifiers".
- Putting `torbrowser-tor` in complain mode: the `deny` rules still apply.

## Upstream filing decision

Target: `roddhjav/apparmor.d`.
No `.out-of-scope/` entry covers apparmor.d or Tor.
Searches on 2026-11-26:
`gh search issues` and `gh search prs` for `lyrebird`, `snowflake`, `torbrowser-tor resolv` and `pluggable transport`
returned nothing;
a broad `torbrowser` search returned closed issues (#1117, #539, #491, #490, #465, #251), so the search works.
Work in progress:
the file's commits are `7b4db8fd41` (2024-09-12, profile added), `5ed4df6916`, `38b973c596` and `8bba967187`
(abi and whitespace changes),
and none of the 21 open pull requests touches Tor.

1.  **Is it really upstream's fault?**
     Yes: the shipped rules stop a supported Tor Browser feature on every system that uses them.
2.  **Can upstream fix it?**
     Yes; the patch is one changed rule, one signal rule and one child profile.
3.  **Are they supporting this use case?**
     Yes: they ship `torbrowser` and `torbrowser-tor`,
     and Snowflake is one of Tor Browser's built-in bridge choices.
4.  **Would the repo welcome our contribution?**
     `docs/report.md:9` asks for "an issue on Github" with a paste of `aa-log -R`. [Report docs][report]
    There are no issue templates (`.github` holds `FUNDING.yml` and `workflows`),
     and `docs/` has no rule on AI-assisted reports; no ban was found.
5.  **Will they likely fix it?**
     Likely: in #490 the maintainer answered the same day, "I will fix this shortly". [Issue 490][i490]
    That issue argues for strict Tor Browser profiles;
     the patch keeps every `deny` on `tor` itself and confines the transports separately,
     in line with the guidelines' "The use of `deny` should be limited to the minimum". [Guidelines][guide]
6.  **Have we prototyped a minimal fix compatible with their architecture?**
     Yes; see "Verification".

Readiness on 2026-11-26:
reproduced on the only line apparmor.d ships (`main`, the installed commit);
prior art read (#490 in full);
intake read (no templates; `aa-log -R` paste requested);
no trust or vouch system (`.github/workflows/main.yml` has none, and no `VOUCHED` file exists);
the work-in-progress search is recorded.
Reproduction repository: [apparmor-d-torbrowser-tor-pluggable-transport-dns-reproduction](https://github.com/Aquaticat/apparmor-d-torbrowser-tor-pluggable-transport-dns-reproduction),
created private on 2026-11-26 in the owner's account (author `Aquaticat` with the account's noreply address).
Its workflow builds apparmor.d at `4c4fec70fc` with `just build` and `prebuild` on `ubuntu-26.04`
(the only Ubuntu release that `prebuild` accepts),
installs `torbrowser-tor` alone in enforce mode,
and checks the mode with `aa-status --json`.
Runs [45776389452](https://github.com/Aquaticat/apparmor-d-torbrowser-tor-pluggable-transport-dns-reproduction/actions/runs/45776389452) (push) and
[45776560760](https://github.com/Aquaticat/apparmor-d-torbrowser-tor-pluggable-transport-dns-reproduction/actions/runs/45776560760) (manual) gave the same result:
the stock job's control lookup passed and its asserting step,
`aa-exec --profile torbrowser-tor -- getent hosts app.datapacket.com`, exited `2`;
the patched job passed the lookup under `torbrowser-tor//pluggable-transports`
and the check that `torbrowser-tor` itself still fails it.
The two earlier runs failed in setup (an unknown `--enforce` option, a patch the repository's whitespace hook
had changed, and Ubuntu 24.04) and do not count.
That workflow applies the exact patch to the `abi/5.0` source with `git apply`, so that item is closed too.
The copy beside this file has its blank context lines stripped by this repository's commit hook;
GNU `patch --strip=1` applies it, while `git apply` needs the copy in the reproduction repository.
On the owner's choice it was made public at `2026-11-26T02:38:35-05:00`;
the repository page and run 45776389452 then returned HTTP 200 to an anonymous `curl`.
Every file and commit was read before the switch:
three files (`README.md`, the workflow, the patch) and three commits,
all authored and committed as `Aquaticat` with the noreply address,
with no local path, host name, e-mail address or token.

With every readiness item done, filing is recommended:
the six constraints hold, the reproduction was re-run on the day on this host and on two clean runners,
the patch was checked against the stock profile in the same workflow,
and the duplicate, work-in-progress and trust-system searches are recorded above with their dates.
The risk stays the poster's:
the issue and the repository carry the owner's GitHub account,
and a maintainer may still prefer the current strictness.
Filing is the owner's decision.
Check the tracker once more on the day of posting.
The patch could go as a pull request with the issue;
the repository's contribution docs ask for nothing first.
Posting puts the owner's GitHub account in front of the maintainer,
and filing is the owner's decision.
Before posting, check the live repository again for a duplicate or a fix.

~~~md
**Title:** torbrowser-tor: pluggable transports cannot resolve names or open UDP sockets, so Snowflake never connects

With `torbrowser-tor` enforcing, Tor Browser cannot connect through Snowflake bridges.

**Cause.** The profile runs every pluggable transport inside Tor's own profile
([`PluggableTransports/** mix`][pt]), which allows only [`network inet stream`][net]
and denies [`/etc/nsswitch.conf` and `/etc/resolv.conf`][deny].
Snowflake's client needs name resolution and UDP:
its configuration names a [domain front][front] and [ICE server URLs][ice].
The socket half is logged.
The lookup half is not, because `deny` rules are silent,
and they [are enforced even in complain mode][complain].
Without a readable `resolv.conf`, glibc [uses `INADDR_LOOPBACK`][glibc]
and Go's resolver [uses `127.0.0.1:53` and `[::1]:53`][go];
on systems using systemd-resolved nothing listens there.

**Logs** (trimmed, from `/var/log/audit/audit.log`):

```text
apparmor="DENIED" operation="create" class="net" profile="torbrowser-tor" comm="lyrebird" family="inet" sock_type="dgram"
apparmor="DENIED" operation="create" class="net" profile="torbrowser-tor" comm="lyrebird" family="inet6" sock_type="dgram"
apparmor="DENIED" operation="create" class="net" profile="torbrowser-tor" comm="lyrebird" family="inet6" sock_type="stream"
apparmor="DENIED" operation="signal" class="signal" profile="torbrowser-tor" requested_mask="receive" signal=term peer="torbrowser"
```

**Reproduction.** [This workflow][repo] builds apparmor.d at `4c4fec7` on a clean Ubuntu 26.04 runner
with `just build` and `prebuild`, installs only `torbrowser-tor` in enforce mode,
and runs `getent hosts app.datapacket.com`:
the name resolves outside the profile and fails under it ([run 1][run1], [run 2][run2]).
Locally:

```console
$ aa-exec --profile torbrowser-tor -- getent hosts app.datapacket.com; echo $?
2
$ getent hosts app.datapacket.com; echo $?
185.59.222.177  1232748490.rsc.cdn77.org
0
```

**Proposed fix** ([patch][patch]), in `apparmor.d/groups/browsers/torbrowser-tor`:

- run the transports in a child profile, `PluggableTransports/** rcx -> pluggable-transports`,
  with `golang`, `nameservice-strict` and `ssl_certs`,
  IPv4 and IPv6 stream and datagram sockets, `netlink raw`, and their `pt_state` folder;
- keep every `deny` on `tor` itself, so the change adds no `deny` and loosens nothing for Tor
  (the guidelines ask to [limit `deny` to the minimum][guide]);
- add `signal receive set=(term kill) peer=torbrowser`,
  since the browser's `term` to Tor was denied when it closed.

The same workflow applies the patch:
the lookup then works under `torbrowser-tor//pluggable-transports` and still fails under `torbrowser-tor`.
With an equivalent local child profile, Tor Browser 16.0.3 connected through Snowflake
on Arch-based CachyOS (AppArmor 4.1.8, apparmor.d built from the same commit).

This report was prepared with an AI assistant;
the commands and results above were run on the reporter's machine and in the linked workflow.

[pt]: https://github.com/roddhjav/apparmor.d/blob/4c4fec70fcd19307a871b3ecc7f117a9ad989c9f/apparmor.d/groups/browsers/torbrowser-tor#L25
[net]: https://github.com/roddhjav/apparmor.d/blob/4c4fec70fcd19307a871b3ecc7f117a9ad989c9f/apparmor.d/groups/browsers/torbrowser-tor#L18
[deny]: https://github.com/roddhjav/apparmor.d/blob/4c4fec70fcd19307a871b3ecc7f117a9ad989c9f/apparmor.d/groups/browsers/torbrowser-tor#L41-L44
[complain]: https://github.com/roddhjav/apparmor.d/blob/4c4fec70fcd19307a871b3ecc7f117a9ad989c9f/docs/issues.md#L26
[guide]: https://github.com/roddhjav/apparmor.d/blob/4c4fec70fcd19307a871b3ecc7f117a9ad989c9f/docs/development/guidelines.md#L119-L125
[glibc]: https://github.com/bminor/glibc/blob/04e750e75b73957cf1c791535a3f4319534a52fc/resolv/res_init.c#L613-L617
[go]: https://github.com/golang/go/blob/983064c1b5bd8e0ad581607aa140167c698cd4eb/src/net/dnsconfig.go#L27
[front]: https://pkg.go.dev/gitlab.torproject.org/tpo/anti-censorship/pluggable-transports/snowflake/v2@v2.15.4/client/lib#:~:text=FrontDomain%20is%20the%20full%20URL%20of%20an%20optional%20front%20domain
[ice]: https://pkg.go.dev/gitlab.torproject.org/tpo/anti-censorship/pluggable-transports/snowflake/v2@v2.15.4/client/lib#:~:text=ICEAddresses%20are%20a%20slice%20of%20ICE%20server%20URLs
[repo]: https://github.com/Aquaticat/apparmor-d-torbrowser-tor-pluggable-transport-dns-reproduction
[run1]: https://github.com/Aquaticat/apparmor-d-torbrowser-tor-pluggable-transport-dns-reproduction/actions/runs/45776389452
[run2]: https://github.com/Aquaticat/apparmor-d-torbrowser-tor-pluggable-transport-dns-reproduction/actions/runs/45776560760
[patch]: https://github.com/Aquaticat/apparmor-d-torbrowser-tor-pluggable-transport-dns-reproduction/blob/55449290df95bc6eb915a955879b3a9d8f4a54dc/torbrowser-tor-pluggable-transports.patch
~~~

[patch]: apparmor-d-torbrowser-tor-denies-name-lookups-to-pluggable-transports.patch
[prof]: https://github.com/roddhjav/apparmor.d/blob/4c4fec70fcd19307a871b3ecc7f117a9ad989c9f/apparmor.d/groups/browsers/torbrowser-tor#L18-L44
[issues]: https://github.com/roddhjav/apparmor.d/blob/4c4fec70fcd19307a871b3ecc7f117a9ad989c9f/docs/issues.md#L22-L28
[guide]: https://github.com/roddhjav/apparmor.d/blob/4c4fec70fcd19307a871b3ecc7f117a9ad989c9f/docs/development/guidelines.md#L119-L125
[report]: https://github.com/roddhjav/apparmor.d/blob/4c4fec70fcd19307a871b3ecc7f117a9ad989c9f/docs/report.md#L9-L12
[i490]: https://github.com/roddhjav/apparmor.d/issues/490

All five links were opened in `agent-browser` on 2026-11-26 and land on the quoted text.
Every link inside the draft was opened in `agent-browser` on 2026-11-26 as well and lands on its quoted text:
the profile lines, `docs/issues.md:26`, the guidelines,
glibc `resolv/res_init.c:613-617` at `04e750e75b73`, Go `src/net/dnsconfig.go:27` at `983064c1b5bd`,
the Snowflake client's `FrontDomain` and `ICEAddresses` fields (pkg.go.dev, `v2.15.4`, text fragments),
the pinned patch and both runs.
Snowflake's own GitLab sends anonymous browsers to a sign-in page, so it is not linked.
