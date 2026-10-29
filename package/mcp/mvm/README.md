# @monochromatic-dev/mcp-mvm

MCP server that exposes [mvm](../../mvm/cli/) VM management operations to AI agents over stdio.

## Tools

- **list_vms**:
   lists all managed VMs with their current state
- **create_vm**:
   creates and starts a new VM (image plus,
   for hetzner,
   `server_type`/`location`)
- **destroy_vm**:
   force-stops and deletes a VM by name,
   or all managed VMs when `all` is true.
   Exactly one of the two:
   a call carrying both is refused before any backend is resolved
- **exec_in_vm**:
   runs a shell command inside a running VM (guest agent for libvirt,
   SSH for hetzner)
- **run_in_vm**,
   **push_to_vm**,
   **pull_from_vm**,
   **update_templates**:
   ephemeral run,
   file transfer,
   and image refresh

Every tool accepts an optional `backend` argument (`libvirt`,
 the default,
 or
`hetzner`).
 It mirrors the CLI's `--backend`/`MVM_BACKEND` selection.
 Since
there is no record of which backend a VM lives on,
 pass the same `backend` to
follow-up tool calls that you used in `create_vm`.

## Backends

- **libvirt** (default):
   local QEMU/KVM.
   Linux only.
- **hetzner**:
   Hetzner Cloud over the HTTP API;
   requires `HCLOUD_TOKEN` and an
  OpenSSH 9.0+ client.
   Provisions real,
   billed servers.
   Tunables:
  `MVM_HCLOUD_SERVER_TYPE`,
   `MVM_HCLOUD_LOCATIONS` (ordered fallback list).

## Usage

Build the server,
then register the built bin by absolute path:

```sh
mise run //package/mcp/mvm:build
claude mcp add --scope user mvm -- node /path/to/package/mcp/mvm/dist/final/node/index.mjs
```

Registering it by hand instead produces the same entry:

```json
{
  "mcpServers": {
    "mvm": {
      "type": "stdio",
      "command": "node",
      "args": ["/path/to/package/mcp/mvm/dist/final/node/index.mjs"]
    }
  }
}
```

Point the path at the built bin,
not at `src/index.ts`:
the source entry imports workspace packages by name and does not run standalone.
A registration whose path no longer resolves fails as `CONNECTION_CLOSED`,
because the process exits before writing a JSON-RPC frame;
`claude mcp get mvm` reports the path it actually tried.

## Protocol

Serves MCP revision **2026-07-28** and only that revision,
through [`@monochromatic-dev/mcp-stdio`](../stdio/).
Clients that open with the removed `initialize` handshake are not served;
see that package's README for what the revision changed.

## Requirements

- **libvirt backend** (default):
   libvirt/QEMU with a user session connection (`qemu:///session`);
   Linux only.
- **hetzner backend**:
   `HCLOUD_TOKEN` plus an OpenSSH 9.0+ client;
   runs on any platform.
