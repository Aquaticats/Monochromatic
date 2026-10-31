# systemctl show stubs a bare transient scope name with .service defaults (systemd 259)

A transient scope started with
`systemd-run --user --scope --unit=NAME -p CPUQuota=37% -p MemoryMax=128M`
carries its limits in the kernel cgroup,
but `systemctl --user show NAME`,
with the bare name,
silently answers for `NAME.service`:
a not-found placeholder with infinity defaults,
exit 0.
`show NAME.scope -p CPUQuota` prints nothing for a different reason,
and `status`/`is-active` on the bare name fail loudly instead.
This file records the source trace,
the reproduction,
the workarounds,
and the decision not to file upstream.

## Symptom

Setup (this repo's harness convention wraps every command in a sized scope):

```sh
# start a transient user scope with CPU and memory limits
systemd-run --user --scope --unit=rr-sysd-doc-probe -p CPUQuota=37% -p MemoryMax=128M -- sleep 300
```

With the scope running,
`systemctl --user show rr-sysd-doc-probe`,
the bare name,
prints a stub and exits 0:

```
Id=rr-sysd-doc-probe.service
LoadState=not-found
CPUQuotaPerSecUSec=infinity
MemoryMax=infinity
```

`systemctl --user show rr-sysd-doc-probe.scope -p CPUQuota` prints nothing,
exit 0.

`systemctl --user status rr-sysd-doc-probe` prints
`Unit rr-sysd-doc-probe.service could not be found.` on stderr and exits 4.

`systemctl --user is-active rr-sysd-doc-probe` prints `inactive` and exits 4.

The queries that work:

```sh
# suffixed name, normalized property spellings
systemctl --user show rr-sysd-doc-probe.scope -p Id -p LoadState -p ControlGroup -p CPUQuotaPerSecUSec -p MemoryMax
```

```
Id=rr-sysd-doc-probe.scope
LoadState=loaded
ControlGroup=/user.slice/user-1000.slice/user@1000.service/app.slice/rr-sysd-doc-probe.scope
CPUQuotaPerSecUSec=370ms
MemoryMax=134217728
```

The kernel files agree (`cpu.max` then `memory.max` under the scope's cgroup
directory):
`37000 100000` and `134217728`.

Three failure variants exist:

- silent stub:
  bare name in `show`,
  wrong unit answered for,
  defaults that look like real values.
- silent empty:
  `-p CPUQuota` on the correctly named scope,
  empty output,
  exit 0.
- loud:
  bare name in `status` or `is-active`,
  exit 4 with an error or `inactive`.

## Root cause

Upstream source cited at tag `v259`,
commit `9ca433482f2281d71718718705ca8cd3bf562ad6`.
Every excerpt below was fetched from
raw.githubusercontent.com at that commit and compared byte-for-byte;
see "Verification" for the method and
"Earlier citation readings that were wrong" for the corrections.

### The client appends .service to the bare name

`verb_show` passes the patterns to `expand_unit_names` with a `NULL` suffix
argument
([src/systemctl/systemctl-show.c:2571-2576](https://github.com/systemd/systemd/blob/9ca433482f2281d71718718705ca8cd3bf562ad6/src/systemctl/systemctl-show.c#L2571-L2576)):

```c
                if (!strv_isempty(patterns)) {
                        _cleanup_strv_free_ char **names = NULL;

                        r = expand_unit_names(bus, patterns, NULL, &names, NULL);
                        if (r < 0)
                                return log_error_errno(r, "Failed to expand names: %m");
```

and turns each expanded name into a unit D-Bus path for `show_one`
([src/systemctl/systemctl-show.c:2582-2589](https://github.com/systemd/systemd/blob/9ca433482f2281d71718718705ca8cd3bf562ad6/src/systemctl/systemctl-show.c#L2582-L2589)):

```c
                        STRV_FOREACH(name, names) {
                                _cleanup_free_ char *path = NULL;

                                path = unit_dbus_path_from_name(*name);
                                if (!path)
                                        return log_oom();

                                r = show_one(bus, path, *name, show_mode, &new_line, &ellipsized);
```

`expand_unit_names` falls back to the literal `.service` when the suffix is
unset
([src/systemctl/systemctl-util.c:287-293](https://github.com/systemd/systemd/blob/9ca433482f2281d71718718705ca8cd3bf562ad6/src/systemctl/systemctl-util.c#L287-L293)):

```c
        STRV_FOREACH(name, names) {
                UnitNameMangle options = UNIT_NAME_MANGLE_GLOB | (arg_quiet ? 0 : UNIT_NAME_MANGLE_WARN);
                char *t;

                r = unit_name_mangle_with_suffix(*name, NULL, options, suffix ?: ".service", &t);
                if (r < 0)
                        return log_error_errno(r, "Failed to mangle name: %m");
```

The mangle helper appends the suffix to any name without a unit type
([src/basic/unit-name.c:709-714](https://github.com/systemd/systemd/blob/9ca433482f2281d71718718705ca8cd3bf562ad6/src/basic/unit-name.c#L709-L714)
for the definition and
[src/basic/unit-name.c:777-780](https://github.com/systemd/systemd/blob/9ca433482f2281d71718718705ca8cd3bf562ad6/src/basic/unit-name.c#L777-L780)
for the append branch):

```c
        /* Append a suffix if it doesn't have any, but only if this is not a glob, so that we can allow
         * "foo.*" as a valid glob. */
        if ((!(flags & UNIT_NAME_MANGLE_GLOB) || !string_is_glob(s)) && unit_name_to_type(s) < 0)
                strcat(s, suffix);
```

`NAME` has no suffix,
so `NAME.service` is the name queried.

### The manager answers with a not-found placeholder instead of an error

`show_one` does not call `GetUnit`;
it sends `org.freedesktop.DBus.Properties.GetAll` straight to the unit path
([src/shared/bus-map-properties.c:230-238](https://github.com/systemd/systemd/blob/9ca433482f2281d71718718705ca8cd3bf562ad6/src/shared/bus-map-properties.c#L230-L238)):

```c
        r = sd_bus_call_method(
                        bus,
                        destination,
                        path,
                        "org.freedesktop.DBus.Properties",
                        "GetAll",
                        reterr_error,
                        &reply,
                        "s", "");
```

The manager's fallback find handler resolves unit paths by loading the unit
([src/core/dbus.c:248-253](https://github.com/systemd/systemd/blob/9ca433482f2281d71718718705ca8cd3bf562ad6/src/core/dbus.c#L248-L253)):

```c
        } else {
                r = manager_load_unit_from_dbus_path(m, path, reterr_error, &u);
                if (r < 0)
                        return 0;
                assert(u);
        }
```

The load chain creates the unit object for the unseen name and returns it
rather than an error
([src/core/manager.c:3370-3375](https://github.com/systemd/systemd/blob/9ca433482f2281d71718718705ca8cd3bf562ad6/src/core/manager.c#L3370-L3375),
then
[src/core/manager.c:2475-2484](https://github.com/systemd/systemd/blob/9ca433482f2281d71718718705ca8cd3bf562ad6/src/core/manager.c#L2475-L2484),
and the creation site
[src/core/manager.c:2438-2442](https://github.com/systemd/systemd/blob/9ca433482f2281d71718718705ca8cd3bf562ad6/src/core/manager.c#L2438-L2442)):

```c
        r = manager_load_unit(m, n, NULL, e, &u);
        if (r < 0)
                return r;

        *_u = u;
        return 0;
```

A missing fragment ends the load as `UNIT_NOT_FOUND`,
and the unit object is kept
([src/core/unit.c:1735-1742](https://github.com/systemd/systemd/blob/9ca433482f2281d71718718705ca8cd3bf562ad6/src/core/unit.c#L1735-L1742)):

```c
fail:
        /* We convert ENOEXEC errors to the UNIT_BAD_SETTING load state here. Configuration parsing code
         * should hence return ENOEXEC to ensure units are placed in this state after loading. */

        u->load_state = u->load_state == UNIT_STUB ? UNIT_NOT_FOUND :
                                     r == -ENOEXEC ? UNIT_BAD_SETTING :
                                                     UNIT_ERROR;
        u->load_error = r;
```

The stub still exports its properties over D-Bus,
including the load state
([src/core/dbus-unit.c:936](https://github.com/systemd/systemd/blob/9ca433482f2281d71718718705ca8cd3bf562ad6/src/core/dbus-unit.c#L936)):

```c
        SD_BUS_PROPERTY("LoadState", "s", property_get_load_state, offsetof(Unit, load_state), SD_BUS_VTABLE_PROPERTY_CONST),
```

### The placeholder's cgroup defaults print as infinity

The placeholder carries a default-initialized cgroup context
([src/core/cgroup.c:157-166](https://github.com/systemd/systemd/blob/9ca433482f2281d71718718705ca8cd3bf562ad6/src/core/cgroup.c#L157-L166)):

```c
        *c = (CGroupContext) {
                .cpu_weight = CGROUP_WEIGHT_INVALID,
                .startup_cpu_weight = CGROUP_WEIGHT_INVALID,
                .cpu_quota_per_sec_usec = USEC_INFINITY,
                .cpu_quota_period_usec = USEC_INFINITY,

                .memory_high = CGROUP_LIMIT_MAX,
                .startup_memory_high = CGROUP_LIMIT_MAX,
                .memory_max = CGROUP_LIMIT_MAX,
                .startup_memory_max = CGROUP_LIMIT_MAX,
```

so `GetAll` on the mangled name returns
`CPUQuotaPerSecUSec=infinity` and `MemoryMax=infinity`.

### The client exits 0 in properties mode

The not-found detection logs at debug level for property output and only
fails for status and help
([src/systemctl/systemctl-show.c:2343-2351](https://github.com/systemd/systemd/blob/9ca433482f2281d71718718705ca8cd3bf562ad6/src/systemctl/systemctl-show.c#L2343-L2351)):

```c
        if (unit && streq_ptr(info.load_state, "not-found") && streq_ptr(info.active_state, "inactive")) {
                log_full(show_mode == SYSTEMCTL_SHOW_PROPERTIES ? LOG_DEBUG : LOG_ERR,
                         "Unit %s could not be found.", unit);

                if (show_mode == SYSTEMCTL_SHOW_STATUS)
                        return EXIT_PROGRAM_OR_SERVICES_STATUS_UNKNOWN;
                if (show_mode == SYSTEMCTL_SHOW_HELP)
                        return -ENOENT;
        }
```

`show_one` then returns 0 for the properties mode,
which is why `show NAME` exits 0 on the stub.

### The real scope does carry the limits

`systemd-run` mangles `--unit` with the `.scope` suffix on its own side
([src/run/run.c:2691-2696](https://github.com/systemd/systemd/blob/9ca433482f2281d71718718705ca8cd3bf562ad6/src/run/run.c#L2691-L2696)):

```c
        if (arg_unit) {
                r = unit_name_mangle_with_suffix(arg_unit, "as unit",
                                                 arg_quiet ? 0 : UNIT_NAME_MANGLE_WARN,
                                                 ".scope", &scope);
                if (r < 0)
                        return log_error_errno(r, "Failed to mangle scope name: %m");
```

and sends `StartTransientUnit` with the `-p` assignments
([src/run/run.c:2710-2724](https://github.com/systemd/systemd/blob/9ca433482f2281d71718718705ca8cd3bf562ad6/src/run/run.c#L2710-L2724),
with the property serialization entry at
[src/run/run.c:1671-1676](https://github.com/systemd/systemd/blob/9ca433482f2281d71718718705ca8cd3bf562ad6/src/run/run.c#L1671-L1676)
and its tail
[src/run/run.c:1353](https://github.com/systemd/systemd/blob/9ca433482f2281d71718718705ca8cd3bf562ad6/src/run/run.c#L1353)):

```c
                r = bus_message_new_method_call(bus, &m, bus_systemd_mgr, "StartTransientUnit");
                if (r < 0)
                        return bus_log_create_error(r);

                /* Name and Mode */
                r = sd_bus_message_append(m, "ss", scope, job_mode_to_string(arg_job_mode));
                if (r < 0)
                        return bus_log_create_error(r);

                /* Properties */
                r = sd_bus_message_open_container(m, 'a', "(sv)");
                if (r < 0)
                        return bus_log_create_error(r);

                r = transient_scope_set_properties(m, allow_pidfd);
```

The manager fills the transient unit and stores the properties
([src/core/dbus-manager.c:1040-1055](https://github.com/systemd/systemd/blob/9ca433482f2281d71718718705ca8cd3bf562ad6/src/core/dbus-manager.c#L1040-L1055),
job dispatch at
[src/core/dbus-manager.c:1148-1157](https://github.com/systemd/systemd/blob/9ca433482f2281d71718718705ca8cd3bf562ad6/src/core/dbus-manager.c#L1148-L1157)):
the scope property dispatch reaches the cgroup context at
[src/core/dbus-scope.c:225-227](https://github.com/systemd/systemd/blob/9ca433482f2281d71718718705ca8cd3bf562ad6/src/core/dbus-scope.c#L225-L227),
and the quota lands at
[src/core/dbus-cgroup.c:1189-1204](https://github.com/systemd/systemd/blob/9ca433482f2281d71718718705ca8cd3bf562ad6/src/core/dbus-cgroup.c#L1189-L1204):

```c
        if (streq(name, "CPUQuotaPerSecUSec")) {
                uint64_t u64;

                r = sd_bus_message_read(message, "t", &u64);
                if (r < 0)
                        return r;

                if (u64 <= 0)
                        return sd_bus_error_set(reterr_error, SD_BUS_ERROR_INVALID_ARGS, "CPUQuotaPerSecUSec= value out of range");

                if (!UNIT_WRITE_FLAGS_NOOP(flags)) {
                        c->cpu_quota_per_sec_usec = u64;
                        CGroupRuntime *crt = unit_get_cgroup_runtime(u);
                        if (crt)
                                crt->warned_clamping_cpu_quota_period = false;
                        unit_invalidate_cgroup(u, CGROUP_MASK_CPU);
```

At start the stored quota is written to the kernel
([src/core/cgroup.c:1495](https://github.com/systemd/systemd/blob/9ca433482f2281d71718718705ca8cd3bf562ad6/src/core/cgroup.c#L1495)
calls the writer at
[src/core/cgroup.c:1146-1158](https://github.com/systemd/systemd/blob/9ca433482f2281d71718718705ca8cd3bf562ad6/src/core/cgroup.c#L1146-L1158)):

```c
static void cgroup_apply_cpu_quota(Unit *u, usec_t quota, usec_t period) {
        char buf[(DECIMAL_STR_MAX(usec_t) + 1) * 2 + 1];

        assert(u);

        period = cgroup_cpu_adjust_period_and_log(u, period, quota);
        if (quota != USEC_INFINITY)
                xsprintf(buf, USEC_FMT " " USEC_FMT "\n",
                         MAX(quota * period / USEC_PER_SEC, USEC_PER_MSEC), period);
        else
                xsprintf(buf, "max " USEC_FMT "\n", period);
        (void) set_attribute_and_warn(u, "cpu.max", buf);
}
```

With `CPUQuota=37%` stored as 370000 usec/s and the default period
`100 * USEC_PER_MSEC`
([src/core/cgroup.c:58](https://github.com/systemd/systemd/blob/9ca433482f2281d71718718705ca8cd3bf562ad6/src/core/cgroup.c#L58)),
the write is `37000 100000`,
which is what the kernel file shows.
`memory.max` goes through the same apply step
([src/core/cgroup.c:1538-1539](https://github.com/systemd/systemd/blob/9ca433482f2281d71718718705ca8cd3bf562ad6/src/core/cgroup.c#L1538-L1539)
selecting the limit,
[src/core/cgroup.c:1547](https://github.com/systemd/systemd/blob/9ca433482f2281d71718718705ca8cd3bf562ad6/src/core/cgroup.c#L1547)
calling the writer at
[src/core/cgroup.c:1300-1307](https://github.com/systemd/systemd/blob/9ca433482f2281d71718718705ca8cd3bf562ad6/src/core/cgroup.c#L1300-L1307)):

```c
                        high = startup && c->startup_memory_high_set ? c->startup_memory_high : c->memory_high;
                        max = startup && c->startup_memory_max_set ? c->startup_memory_max : c->memory_max;
```

```c
                cgroup_apply_memory_limit(u, "memory.max", max);
```

### CPUQuota is an input alias, not a bus property

The `-p CPUQuota=37%` spelling is translated by the client's assignment
table
([src/shared/bus-unit-util.c:2355-2360](https://github.com/systemd/systemd/blob/9ca433482f2281d71718718705ca8cd3bf562ad6/src/shared/bus-unit-util.c#L2355-L2360)):

```c
        { "MemoryMax",                             bus_append_parse_resource_limit               },
        { "MemorySwapMax",                         bus_append_parse_resource_limit               },
        { "MemoryZSwapMax",                        bus_append_parse_resource_limit               },
        { "TasksMax",                              bus_append_parse_resource_limit               },
        { "CPUQuota",                              bus_append_parse_cpu_quota                    },
        { "CPUQuotaPeriodSec",                     bus_append_parse_sec_rename_infinity          },
```

The parser converts the percentage to microseconds per second and appends the
property under the real name
([src/shared/bus-unit-util.c:379-394](https://github.com/systemd/systemd/blob/9ca433482f2281d71718718705ca8cd3bf562ad6/src/shared/bus-unit-util.c#L379-L394)):

```c
static int bus_append_parse_cpu_quota(sd_bus_message *m, const char *field, const char *eq) {
        uint64_t x;
        int r;

        if (isempty(eq))
                x = USEC_INFINITY;
        else {
                r = parse_permyriad_unbounded(eq);
                if (r < 0)
                        return parse_log_error(r, field, eq);
                if (r == 0)
                        return parse_log_error(SYNTHETIC_ERRNO(ERANGE), field, eq);
                x = r * USEC_PER_SEC / 10000U;
        }

        r = sd_bus_message_append(m, "(sv)", "CPUQuotaPerSecUSec", "t", x);
```

The manager only exports the normalized spelling
([src/core/dbus-cgroup.c:382](https://github.com/systemd/systemd/blob/9ca433482f2281d71718718705ca8cd3bf562ad6/src/core/dbus-cgroup.c#L382)):

```c
        SD_BUS_PROPERTY("CPUQuotaPerSecUSec", "t", bus_property_get_usec, offsetof(CGroupContext, cpu_quota_per_sec_usec), 0),
```

and `CPUQuota` survives only as the unit-file serialization label in the
runtime drop-in writer
([src/core/dbus-cgroup.c:1206-1212](https://github.com/systemd/systemd/blob/9ca433482f2281d71718718705ca8cd3bf562ad6/src/core/dbus-cgroup.c#L1206-L1212)).
There is no D-Bus property named `CPUQuota`,
so `GetAll` never returns it.

The client prints only names present in the `GetAll` reply
([src/shared/bus-print-properties.c:376-378](https://github.com/systemd/systemd/blob/9ca433482f2281d71718718705ca8cd3bf562ad6/src/shared/bus-print-properties.c#L376-L378)):

```c
                if (!filter ||
                    strv_contains(filter, name) ||
                    (expected_value = strv_find_startswith(filter, name_with_equal))) {
```

and a requested but absent name is logged only at debug level
([src/systemctl/systemctl-show.c:2379-2381](https://github.com/systemd/systemd/blob/9ca433482f2281d71718718705ca8cd3bf562ad6/src/systemctl/systemctl-show.c#L2379-L2381)):

```c
        STRV_FOREACH(pp, arg_properties)
                if (!set_contains(found_properties, *pp))
                        log_debug("Property %s does not exist.", *pp);
```

which is why `show NAME.scope -p CPUQuota` prints nothing and exits 0.

### The behavior is documented

`systemctl(1)` documents the suffix default under "Parameter Syntax"
([man/systemctl.xml:1916-1928](https://github.com/systemd/systemd/blob/9ca433482f2281d71718718705ca8cd3bf562ad6/man/systemctl.xml#L1916-L1928)):

```xml
      <para>Unit commands listed above take either a single unit name (designated as <replaceable>UNIT</replaceable>),
      or multiple unit specifications (designated as <replaceable>PATTERN</replaceable>…). In the first case, the
      unit name with or without a suffix must be given. If the suffix is not specified (unit name is "abbreviated"),
      systemctl will append a suitable suffix, <literal>.service</literal> by default, and a type-specific suffix in
      case of commands which operate only on specific unit types. For example,
      <programlisting># systemctl start sshd</programlisting> and
      <programlisting># systemctl start sshd.service</programlisting>
      are equivalent, as are
      <programlisting># systemctl isolate default</programlisting>
      and
      <programlisting># systemctl isolate default.target</programlisting>
      Note that (absolute) paths to device nodes are automatically converted to device unit names, and other (absolute)
      paths to mount unit names.
```

The `show` description documents the normalized property naming,
which is why the quota reads back as `CPUQuotaPerSecUSec`
([man/systemctl.xml:333-340](https://github.com/systemd/systemd/blob/9ca433482f2281d71718718705ca8cd3bf562ad6/man/systemctl.xml#L333-L340)):
"time settings are always exposed as properties ending in the
`…USec` suffix even if a matching configuration options end in `…Sec`".
The unit name grammar with the type suffix list is defined at
[man/systemd.unit.xml:121-125](https://github.com/systemd/systemd/blob/9ca433482f2281d71718718705ca8cd3bf562ad6/man/systemd.unit.xml#L121-L125).

### Earlier citation readings that were wrong

The first source-trace pass
(`$HOME/temp/agent/repo-review-2026-10-31/findings/sysd-source-trace.md`)
had four citation defects that a second,
byte-for-byte pass
(`$HOME/temp/agent/repo-review-2026-10-31/sysd-permalink-report.md`)
caught and that this document corrects:

- the `man/systemctl.xml:1916-1928` excerpt merged source lines 1924 and 1925
  into one line ending in `and`,
  dropping the standalone `and`;
  the excerpt above is the verbatim source.
- `src/systemctl/systemctl-show.c:2571-2589` was presented as one contiguous
  excerpt,
  but lines 2577-2581 were never quoted;
  this document cites the two quoted sub-ranges.
- `src/core/cgroup.c:1538-1547` was likewise over-wide;
  this document cites 1538-1539 and 1547.
- `src/core/unit.c:1735-1747` extended past the quoted lines;
  this document cites 1735-1742.

## Verification

- Version under test:
  systemd 259 (259.10-1.fc44),
  kernel 7.2.11-ogc4.1.fc44.x86_64,
  user manager,
  cgroup v2.
- Upstream source:
  tag `v259`,
  commit `9ca433482f2281d71718718705ca8cd3bf562ad6`
  (2025-12-17),
  verified an ancestor of main
  (main 7928 commits ahead of the tag, 0 behind),
  annotated tag object
  `b2a31f27d42a98086ef1945e7e136fc7cb908ad0`
  dereferencing exactly to the commit,
  and the release page resolving.
- Harness:
  `$HOME/temp/agent/repo-review-2026-10-31/sysd-repro.ts`.
  It starts the transient scope with `CPUQuota=37%` and `MemoryMax=128M`,
  waits for it,
  runs every catalog below,
  prints one JSON object,
  and stops the scope.
  Run it inside a sized scope:
  `systemd-run --user --scope --unit=rr-sysd-repro -p CPUQuota=100% -p MemoryMax=1G -p TasksMax=128 -- node sysd-repro.ts`.
- Captured output (2026-10-31,
  third run,
  identical to the two earlier runs):

```json
{
  "wait-for-probe": { "code": 0, "stdout": "" },
  "A1-show-suffixed": {
    "code": 0,
    "stdout": "Id=rr-sysd-doc-probe.scope\nLoadState=loaded\nControlGroup=/user.slice/user-1000.slice/user@1000.service/app.slice/rr-sysd-doc-probe.scope\nCPUQuotaPerSecUSec=370ms\nMemoryMax=134217728\n"
  },
  "A2-kernel-cpu-max": { "code": 0, "stdout": "37000 100000\n" },
  "A3-kernel-memory-max": { "code": 0, "stdout": "134217728\n" },
  "B1-show-bare-name": {
    "code": 0,
    "stdout": "Id=rr-sysd-doc-probe.service\nLoadState=not-found\nCPUQuotaPerSecUSec=infinity\nMemoryMax=infinity\n"
  },
  "B2-show-suffixed-p-CPUQuota": { "code": 0, "stdout": "" },
  "B3-status-bare-name": {
    "code": 4,
    "stdout": "",
    "stderr": "Unit rr-sysd-doc-probe.service could not be found.\n"
  },
  "B4-is-active-bare-name": { "code": 4, "stdout": "inactive\n", "stderr": "" },
  "B5-is-active-suffixed": { "code": 0, "stdout": "active\n" },
  "cleanup-stop-scope": { "code": 0, "stdout": "" }
}
```

Catalogs that work cleanly:

- `show NAME.scope` with the normalized spellings:
  real values,
  `LoadState=loaded` (A1).
- kernel `cpu.max` and `memory.max`:
  `37000 100000` and `134217728` (A2, A3).
- `is-active NAME.scope`:
  `active`,
  exit 0 (B5).

Catalogs that fail,
by variant:

- silent stub:
  `show NAME` on the bare name (B1),
  exit 0 with the mangled `Id`,
  `LoadState=not-found`,
  infinity defaults.
- silent empty:
  `show NAME.scope -p CPUQuota` (B2),
  exit 0 with no output.
- loud:
  `status NAME` (B3) with exit 4 and the not-found error,
  `is-active NAME` (B4) with exit 4 and `inactive`.

Link verification method:
this harness has no browser tool,
so the permalinks were content-verified rather than loaded as pages.
All 17 cited files were fetched from raw.githubusercontent.com at the pinned
commit and every fenced excerpt was compared byte-for-byte
(30 exact,
3 over-wide ranges narrowed,
1 merged line corrected;
see "Earlier citation readings that were wrong").
`man/systemctl.xml` was cross-checked a second way through the GitHub
contents API at the same commit
(160143 bytes,
byte-identical),
and the commit,
tag,
and release URLs were resolved read-only through `gh api`.
A reader loading the links should land on the quoted lines
because each anchor points into content verified identical at that commit,
but the page loads themselves were not exercised here.

## Verified workarounds

- Query with the full unit name including the `.scope` suffix.
  Tradeoff:
  the caller must know the unit type;
  a script that only receives the bare name passed to `--unit` must derive
  or remember the suffix,
  and the same trap exists for every other unit type.
- Read the quota as `CPUQuotaPerSecUSec`,
  not `CPUQuota`.
  Tradeoff:
  two spellings for one concept,
  and the percentage input `37%` reads back as `370ms`,
  so a checker must convert units.
- Treat the kernel files as ground truth:
  read `cpu.max` and `memory.max` under the scope's cgroup directory
  (the `ControlGroup` property gives the path).
  Tradeoff:
  requires path traversal the `show` call did not need,
  and the file names differ from the systemd spellings.
- Guard existence with `status` or `is-active`,
  not with `show`'s exit code.
  Tradeoff:
  `is-active` exit 4 covers both not-found and inactive;
  distinguish them by the printed text.

## What does not work

- Bare name in `show`:
  prints the mangled `Id`,
  `LoadState=not-found`,
  and infinity defaults,
  exit 0.
  The output looks like a real answer about a real unit.
- `-p CPUQuota` on the correctly named scope:
  empty output,
  exit 0,
  because the requested name never appears in the `GetAll` reply and the
  miss is logged only at debug level.
- Trusting `show`'s exit code:
  0 for the real unit and 0 for the stub alike.
- Grepping `show` output for `CPUQuota`:
  the exact token exists only in the client assignment table and the unit-file
  serialization label,
  never in the bus reply.

## Upstream filing decision

Default policy is do not file.
The six-constraint walk:

1. Is it really upstream's fault?
   No defect exists.
   The `.service` default for abbreviated names is documented in
   `systemctl(1)`
   (the excerpt in "The behavior is documented"),
   the normalized property naming is documented in the same page,
   and the silent exit-0 for properties mode is a deliberate client choice
   (the `LOG_DEBUG` branch quoted in "The client exits 0 in properties mode").
   The behavior is a sharp edge,
   documented and intended,
   not a bug.
   This constraint fails,
   so there is nothing to file.
2. Can upstream fix it?
   A warning or non-zero exit for not-found in properties mode is
   mechanically possible
   (the branch already exists),
   but the question is moot given constraint 1.
3. Are they supporting this use case?
   Yes.
   Transient scopes from `systemd-run` are first-class
   (the `StartTransientUnit` path in "The real scope does carry the limits"),
   and abbreviated unit names are a documented input form.
4. Would the repo welcome the contribution?
   `docs/CONTRIBUTING.md`
   (fetched 2026-10-31 from main,
   [docs/CONTRIBUTING.md](https://github.com/systemd/systemd/blob/main/docs/CONTRIBUTING.md))
   uses GitHub Issues exclusively for bugs and feature requests,
   asks for the systemd version,
   the distribution,
   and a reproduction,
   and tracks only the two most recently released non-rc versions;
   the host build 259.10-1.fc44 would need checking against the current
   release pair on any filing day.
   Its LLM policy allows AI assistance with human review and authorship
   ("You are the author").
   No ban was found;
   nothing about this report would be unwelcome as a filing,
   but there is no defect to file.
5. Will they likely fix it?
   No signal either way:
   the duplicate search below found no issue asking for it.
   Related evidence:
   commit `14b1a92e`
   (2026-06-17)
   added a strict-mangling flag to `unit_name_mangle_with_suffix` after a
   systemd-run type-transition confusion,
   showing the maintainers treat mangling sharp edges as fixable,
   but scoped to systemd-run,
   not `systemctl show`;
   the recent `src/systemctl/systemctl-show.c` commits
   (`5071cd10`, `ef23a1e3`, `a83a0d2f`, `407358d9`)
   do not touch the not-found handling.
6. Has a minimal fix been prototyped?
   Not reached:
   constraint 1 fails,
   so there is no candidate fix to prototype.

Duplicate search (2026-10-31,
`gh search issues --repo systemd/systemd`,
open and closed state):

- `systemctl show transient scope`:
  no results.
- `systemctl show not-found mangled`:
  no results.
- `systemctl show abbreviated unit name`:
  no results.
- `systemctl show exit 0 unit not found`:
  no results.
- `systemd-run scope systemctl show properties`:
  no results.

Work in progress check (2026-10-31):

- `gh search prs --repo systemd/systemd "systemctl show not-found properties"`:
  no results.
- `gh api "repos/systemd/systemd/commits?path=src/systemctl/systemctl-show.c&per_page=5"`
  and the same for `src/basic/unit-name.c`:
  the commits named in constraint 5 and no change touching the not-found
  stub path or the show mangling.

`.out-of-scope/` check:
the directory holds 11 files
(read 2026-10-31);
none names systemd or this bug class,
so no exemption applies.

Risk to the poster:
this document makes no recommendation to file,
and filing anywhere would be the poster's decision.
A search of the systemd repository's workflow files and docs found no trust
or blocklist system,
but that search was not re-run on any future filing day;
anyone filing must re-check the live repository on the day of posting.

Decision:
do not file.
The behavior is documented upstream,
and this record already contains the reproduction,
source trace,
and workarounds that a filing or comment would add.
There is no draft issue to keep.
