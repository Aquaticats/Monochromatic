/**
 The name and label every container of one gate run carries, and the lines the gate prints so a person can prove which
 containers are their own.

 Several sessions run this gate on this host at once. Until issue 6334 a gate container carried neither a name nor a
 label, so a session that wanted to stop its own gate could only match containers by image or by creation time, and one
 round of 2026-11-21 stopped three containers another round's gate was running (`podman`'s own random names,
 `trusting_poincare`, `gallant_moore` and `affectionate_diffie`). One run id, made by the runner at start from its
 evidence directory's random name and never from a clock, names every container of that run: the printed lines and the
 run's `manifest.json` carry it, so `podman ps --filter label=dev.monochromatic.git-policy.gate-run=<run id>` lists one
 gate's containers and no other session's.
 */
import { basename, } from 'node:path';

/**
 The label every container of one gate run carries, whose value is that run's id.

 Podman's documentation takes `--label key=value` and filters on it with `--filter label=key=value`; the key is a
 reverse-domain name so it names this gate rather than an image's own labels.
 */
export const gateRunLabelName = 'dev.monochromatic.git-policy.gate-run';

/**
 Prefix of every gate container's name, the same prefix the gate's systemd unit carries, so a `podman ps` listing and a
 `systemctl --user list-units` listing are read with one word in hand.
 */
const gateContainerNamePrefix = 'gpc-gate';

/**
 The two checks one gate runs, each in a container of its own.

 The role is part of the container's name, so a name says which check it runs as well as which run it belongs to.
 */
export type GateContainerRole = 'suite' | 'clippy';

/**
 What one container of a gate run is called, and what proves it belongs to that run.

 Podman refuses a name another container holds and keeps a name while its container exists, so two runs must never make
 the same name: the run id is random enough that they do not.
 */
export type GateContainerIdentity = {
  /**
   The check this container runs.
   */
  readonly role: GateContainerRole;
  /**
   The `--name` podman assigns it, which `podman stop <name>` takes.
   */
  readonly name: string;
  /**
   The `--label` it carries, in the `key=value` form podman writes and filters on.
   */
  readonly label: string;
};

/**
 The run id every container and line of one gate carries: the name of the evidence directory the runner makes at start.

 That name is `native-` and six random characters from `mkdtemp`, so two runs never share it and nothing about it comes
 from a clock; a person reads it in the gate's log, in every container's name and label, and in the evidence directory
 beside `manifest.json`.

 @param evidenceDirectory - the evidence directory `test-native-container.ts` made for this run

 @returns its last path segment, which is the run id

 @example
 ```ts
 gateRunIdOf({ evidenceDirectory: '/repo/package/git-policy/cli/target/verification/native-HZPg8O' });
 ```
 */
export function gateRunIdOf({ evidenceDirectory }: { readonly evidenceDirectory: string; },): string {
  return basename(evidenceDirectory,);
}

/**
 The name and label of one container of a gate run.

 @param runId - the run's id, from {@link gateRunIdOf}

 @param role - the check this container runs

 @returns the `--name` and `--label` values, the name carrying the role and the label carrying the run id

 @example
 ```ts
 gateContainerIdentityOf({ runId: 'native-HZPg8O', role: 'suite' });
 ```
 */
export function gateContainerIdentityOf({
  runId,
  role,
}: {
  readonly runId: string;
  readonly role: GateContainerRole;
},): GateContainerIdentity {
  return {
    role,
    name: `${gateContainerNamePrefix}-${runId}-${role}`,
    label: `${gateRunLabelName}=${runId}`,
  };
}

/**
 The `podman run` options that give a container its identity.

 Podman takes options before the image, so these join the run's other options and never its command.

 @param identity - the container's name and label

 @returns `--name` and `--label` with their values, in the order podman reads them

 @example
 ```ts
 gateContainerArgumentsOf({ identity: gateContainerIdentityOf({ runId: 'native-HZPg8O', role: 'clippy' }) });
 ```
 */
export function gateContainerArgumentsOf({ identity }: { readonly identity: GateContainerIdentity; },): readonly string[] {
  return [
    '--name',
    identity.name,
    '--label',
    identity.label,
  ];
}

/**
 The line a gate prints once, before any container starts, naming the run and the filter that lists its containers.

 @param runId - the run's id

 @returns the line, holding the label and the `podman ps` filter a person copies

 @example
 ```ts
 gateRunLineOf({ runId: 'native-HZPg8O' });
 ```
 */
export function gateRunLineOf({ runId }: { readonly runId: string; },): string {
  return `Gate run id ${runId}: every container this gate starts carries the label ${gateRunLabelName}=${runId}, so \`podman ps --filter label=${gateRunLabelName}=${runId}\` lists exactly this run's containers and no other session's.`;
}

/**
 The line a gate prints before it starts one container, naming what its own `podman run` is about to carry.

 @param identity - the container's name and label

 @param cidFile - the file the client writes the container's ID to

 @returns the line, with the name `podman stop` takes, the label `podman ps --filter` matches and the ID file's path

 @example
 ```ts
 gateContainerLineOf({ identity, cidFile: 'target/verification/native-HZPg8O/suite.cid' });
 ```
 */
export function gateContainerLineOf({
  identity,
  cidFile,
}: {
  readonly identity: GateContainerIdentity;
  readonly cidFile: string;
},): string {
  return `Gate container ${identity.role}: name ${identity.name}, label ${identity.label}, ID file ${cidFile}`;
}

/**
 The line a gate prints as soon as its client writes the container's ID, which is before the container's own output.

 @param identity - the container's name and label

 @param id - the container's ID, as podman wrote it to the ID file

 @returns the line, so a person stopping this gate takes the name beside the ID that `podman ps` and `podman events` show

 @example
 ```ts
 gateContainerIdLineOf({ identity, id: '3f2a9c1e...' });
 ```
 */
export function gateContainerIdLineOf({
  identity,
  id,
}: {
  readonly identity: GateContainerIdentity;
  readonly id: string;
},): string {
  return `Gate container ${identity.role}: ID ${id} (name ${identity.name}, label ${identity.label})`;
}

/**
 The line a gate prints when a container's client ended before writing an ID, which means it created no container and
 the gate's refusal carries podman's own reason.

 @param identity - the container's name and label

 @param cidFile - the file the ID was expected in

 @returns the line, saying that no container was created and that podman's failure is what refused the gate

 @example
 ```ts
 gateContainerNotCreatedLineOf({ identity, cidFile: 'target/verification/native-HZPg8O/clippy.cid' });
 ```
 */
export function gateContainerNotCreatedLineOf({
  identity,
  cidFile,
}: {
  readonly identity: GateContainerIdentity;
  readonly cidFile: string;
},): string {
  return `Gate container ${identity.role}: its client ended before writing an ID to ${cidFile}, so it created no container, and podman's own failure is what the gate refuses with.`;
}

/**
 The line a gate prints when no ID reached its container's ID file in the time it waited, so the log says a person has
 only the name and the label to stop it by.

 @param identity - the container's name and label

 @param cidFile - the file the ID was expected in

 @param waitedMilliseconds - how long the gate waited for it

 @returns the line, naming what was missing and what is left to identify the container by

 @example
 ```ts
 gateContainerNoIdLineOf({ identity, cidFile: 'target/verification/native-HZPg8O/suite.cid', waitedMilliseconds: 60_000 });
 ```
 */
export function gateContainerNoIdLineOf({
  identity,
  cidFile,
  waitedMilliseconds,
}: {
  readonly identity: GateContainerIdentity;
  readonly cidFile: string;
  readonly waitedMilliseconds: number;
},): string {
  return `Gate container ${identity.role}: no ID reached ${cidFile} in ${String(waitedMilliseconds)} ms, so this container is known by its name ${identity.name} and its label ${identity.label} alone.`;
}
