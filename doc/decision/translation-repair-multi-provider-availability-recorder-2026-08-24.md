# Translation repair multi-provider: the availability recorder built on 2026-08-24

Why availability is recorded rather than argued:
the meter line promoted to info,
the unreadable-meter reading that used to count as available,
and outages kept as ranges.

Part of [Translation repair: two providers, and the quota reader that was never wired](translation-repair-multi-provider.md),
which lists every file of this record.

## Availability is now recorded rather than argued, 2026-08-24

The availability recorder was built because three writer seats sit on Charm Hyper only,
and the case for them rested on the forty-round quality pass
plus an availability adjustment that was reasoned about,
never measured.
Two things were missing:
a record,
and a way to read it.

### The record already existed and was invisible

`provider-budget.ts` reads both meters on a sixty-second cache
and already decides dry from wet for routing.
It was saying so at `debug`,
which a corpus run does not record.
Promoting that one line to `info` turns every future run
into an availability record at no extra cost,
bounded by the freshness window rather than by call volume,
so a busy run and a quiet one both cost about a line a minute.

Refusals cannot substitute for it.
`NoProviderForModelError` appears only where something happened
to ask for a model that provider serves,
so an hour when nothing asked reads exactly like an hour when everything worked.
That is why the earlier estimate off 759 existing logs
could find only 26 that could have shown a refusal at all.

### An unreadable meter was reading back as an available provider

Found while building the reader,
before it produced any number.
`drynessOf` returned `false` both when a meter reported budget
and when the meter could not be read at all.
For routing that is deliberate and stays:
a monitoring failure must not become an outage.
For the record it is backwards,
because a duty cycle counting an unreachable endpoint as a working provider
reports an outage as uptime.

The meter now answers with `wet`,
`dry` or `unreadable`,
and `routesAsDry` maps that back to the bit the router wants.
Routing behaviour is unchanged;
the existing test that an unreadable meter stays spendable passes untouched.

### Every outage is a range, and the denominator excludes non-answers

Readings happen when a run asks to spend,
so two can be a minute apart or a day apart.
An outage seen at one reading and gone by the next
began and ended somewhere in between.
`meter-report` therefore reports a confirmed lower bound,
an upper bound running to the surrounding wet readings,
and names an end open where no wet reading closes it.
Only `wet` proves recovery and only `dry` proves outage,
so an `unreadable` reading ends a confirmed stretch without closing its bound.

The duty cycle divides by readings that answered.

### What this still does not decide

The record starts empty.
It measures availability WHEN WE WERE ASKING,
which is the quantity that prices a seat and is not the same as availability,
and `budget-sample` exists to fill the quiet stretches
so a recovery gets observed rather than inferred.

Nothing here re-opens the seating.
It replaces an argument with an instrument,
and the instrument has one reading in it:
2026-08-24T18:17:35Z,
`synthetic=wet hyper=dry`,
which is the same outage the roster probe found earlier that day.
