# The commit service cannot find the programs the committing shell has

## Symptom

A commit lands and its automatic push is refused with
`monochromatic-git-policy-cli: this push was stopped before it sent anything.`
and,
on the old text,
`git-lfs was not found in any directory of PATH.`
The same push typed by hand in the same shell succeeds.

Measured on 2026-11-12 in two linked worktrees:
the commit service's `PATH`,
taken from the systemd user manager that started it,
held no folder `git-lfs` is installed in,
while the shell that made the commits had one.
Both commits landed and both pushes were refused for a program their own shell could start.

## Root cause

The commit service runs the push as its own child,
so the push searched the environment the manager started the service with.
Nothing of the committing shell reached it:
a wrapper command may start no program beyond the Git it forwards,
and the service is a long-lived process of another cgroup and another environment.

Two `GIT_*` variables make the same gap worse than a missing `git-lfs`:

- `GIT_SSH_COMMAND` and `GIT_PROXY_COMMAND` are whole command lines,
  and a command line can hold a password,
  so the working directory of a repository was never a place to copy them into.
- `GIT_SSH` and `GIT_EXEC_PATH` name a program and a folder outright,
  and the manager's values name programs the person never chose.

An earlier fix wrote the five values into
`<git-dir>/monochromatic-git-policy-cli/push-programs.json` at the start of every commit.
That record was never removed,
a typed push never refreshed it,
and a killed write left it whole in a `.push-programs.json.<random>.partial` beside it,
so a credential outlived the command that owned it and every process of the account could read it.

## What the current release does

The owner's decision of 2026-11-12T11:58:45-05:00 is "Memory only".

- The five values (`PATH`,
  `GIT_EXEC_PATH`,
  `GIT_SSH`,
  `GIT_SSH_COMMAND`,
  `GIT_PROXY_COMMAND`)
  travel with the commit request and with the wake that announces a push record,
  are held in the service's memory against that record's own id,
  and are dropped when the record can no longer run:
  when its push lands,
  fails for good or is skipped,
  when a retry plan that carried it is dropped or replaced by another push's plan,
  when a batch that carries it is given back,
  when the recovery gives the record up after its tries run out,
  when a repository the filesystem proves gone ends its plans,
  and when the committing wrapper settles or removes the record itself,
  which tells the service which records can no longer run.
  None of the three transport values,
  and no value at all except the folder list a missing-program refusal names,
  reaches a file,
  a record,
  a journal line or a log.
- The five names the service's own environment held are removed from a push that carries a list,
  so a value the manager fixed never stands in for one the shell set,
  and a value the shell left unset never comes back from the manager.
- A push record keeps one flag and no value:
  whether the command that asked for the push chose `GIT_SSH`,
  `GIT_SSH_COMMAND` or `GIT_PROXY_COMMAND`.
  The journal's intent keeps that flag per commit,
  so a service that restarts re-holds the push with it.
- A push whose transport the command chose,
  and whose settings a restart took away,
  stops before it runs,
  says nothing was sent through another transport,
  and prints the one command that pushes (the command the service itself would have run).
  The words name both ways the settings can be gone:
  a service started again since,
  or a command that could not reach it.
- The missing-program refusal names the program,
  every folder of the list it searched,
  and where that list came from.
  A list that came with the request is remedied with a commit made again,
  which hands that shell's list over again;
  a push that searched the service's own list is remedied with a push by hand once from a shell that holds the program,
  which searches that shell's own `PATH`.
- The first command that meets a `push-programs.json`,
  or a staging file of one of its writes,
  removes it and says so in the wrapper's own log.
  A link of that name is removed itself,
  never what it points at,
  and nothing else in the folder is touched.

## Verification

- `a_commit_and_a_typed_push_write_neither_command_line_anywhere` sweeps the fixture's Git directory,
  runtime directory,
  state directory and home for a planted command line,
  and finds it before removing it,
  so the search is proved able to see a value on a disk.
- `the_service_push_searches_the_list_the_command_that_asked_gave` builds a push record with a list held for it and
  without one,
  and reads the marker each push's environment carries:
  the command's list is searched and named for the record whose values the service holds,
  and the service's own list and its own name for a record it holds nothing for.
- `a_push_a_restarted_service_has_to_try_again_stops_and_names_the_command` kills a committing wrapper once its gate
  approved the push and before it promoted the record,
  stops the service that holds the settings while the record waits where no round looks,
  promotes the record as the killed command would have,
  starts the next instance,
  reads the words on the next command,
  and runs the command they print against a real bare remote.
  `doc/troubleshooting/git-policy-cli-a-control-stops-a-service-no-command-reaches.md` says why it does each step.
- `the_missing_program_message_names_the_list_the_command_that_asked_gave` reads the refusal of a push whose values came
  with the request and proves it names the shell's folders and the shell's own `PATH` as the one to install the program
  on,
  never the commit service's list.
- `the_record_an_unreleased_branch_wrote_is_removed_by_the_first_command` writes both shapes of the old record,
  runs one command,
  and proves no value of either is left on any disk.
- `a_later_push_that_lands_gives_up_the_values_of_the_plan_it_cancels` and
  `a_plan_replaced_by_a_later_pushs_plan_gives_up_the_values_of_the_record_it_carried` give a retryable push a plan
  and then end that plan the two ways a later commit ends it,
  by landing and by failing retryably in its place,
  and prove the older record keeps no value while asserting it held one before.
- `a_push_the_recovery_gives_up_on_gives_up_the_programs_it_held` drives the real recovery over a push on its last
  hold,
  so the record moves to `failed` by the step no handler performs,
  and proves it keeps no value while asserting it held one first.
- `a_record_settled_where_no_service_runs_starts_none_to_tell_it` proves the message that gives a record's programs
  up asks the manager for the unit's state first,
  starts no service,
  and waits for no start,
  while `a_running_service_is_told_which_records_can_no_longer_run` proves a service that is running is told exactly
  the records the command named.

## Limits

- A service that restarted holds nothing,
  so a push whose command chose a transport stops there.
  That is the decision:
  speaking another transport would use credentials nobody chose,
  and writing the values is what this change removes.
- A push an earlier release queued carries no list,
  so it searches the service's own and its refusal says so.
- A merge's,
  a cherry-pick's or a pull's own push is written as a record before the wake that carries its values,
  so a round that claims that record in between runs the push with the service's own list,
  and a push whose command chose a transport stops with the words above rather than speaking another.
  That window is issue 6332,
  which carries the ranked shapes of a fix;
  a commit's own push is not exposed,
  because the service writes its record and holds its values in the same step.
- The values are a few hundred bytes per push in flight;
  a service that keeps running holds them until each record settles,
  or until a plan that carries the push gives it up.
- `GIT_ASKPASS` and `SSH_ASKPASS` are not carried:
  the service's pushes set the never-prompt pair,
  so a login prompt cannot be carried either.
