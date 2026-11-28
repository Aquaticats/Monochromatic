/**
 Type definitions for the subset of the Hetzner Cloud API mvm uses.
 
 Only the fields this backend reads are modelled; responses carry more.
 
 @module
 */

//region Server

/**
 IPv4 assignment on a server's public network.
 
 @example
 ```ts
 const ipv4: HetznerIpv4 = { ip: '203.0.113.7' };
 ```
 */
export type HetznerIpv4 = {
  /**
   Public IPv4 address, or empty when none is attached.
   */
  readonly ip: string;
};

/**
 Public network block of a server.
 
 @example
 ```ts
 const net: HetznerPublicNet = { ipv4: { ip: '203.0.113.7' } };
 ```
 */
export type HetznerPublicNet = {
  /**
   IPv4 assignment; absent (and `null` on the wire) when the server has no
   public IPv4, so access it with optional chaining.
   */
  readonly ipv4?: HetznerIpv4;
};

/**
 Server resource as returned by the API.
 
 @example
 ```ts
 const server: HetznerServer = await getMvmServerByName({ name: 'dev-01' });
 server.public_net.ipv4?.ip;
 ```
 */
export type HetznerServer = {
  /**
   Numeric server id used for delete and image actions.
   */
  readonly id: number;
  /**
   Full server name including the `mvm-` prefix.
   */
  readonly name: string;
  /**
   Lifecycle status (e.g. `running`, `initializing`).
   */
  readonly status: string;
  /**
   Public network block carrying the IPv4 used for SSH.
   */
  readonly public_net: HetznerPublicNet;
  /**
   Server type the instance runs on; its architecture must match a snapshot
   image when cloning. Hetzner embeds the whole server type object here, so the
   size a running server has can be read without a second call; the fields are
   optional because a response mvm did not capture may omit them.
   */
  readonly server_type: {
    readonly architecture: string;
    readonly cores?: number;
    readonly disk?: number;
    readonly memory?: number;
  };
  /**
   User-defined labels, including the mvm ownership label.
   */
  readonly labels: Readonly<Record<string, string>>;
};

//endregion Server

//region Server type

/**
 Hourly price of a server type in one location.
 
 @example
 ```ts
 const price: HetznerServerPrice = { location: 'fsn1', price_hourly: { gross: '0.0080' } };
 ```
 */
export type HetznerServerPrice = {
  /**
   Location code the price applies to (e.g. `fsn1`).
   */
  readonly location: string;
  /**
   Hourly gross price as a decimal string.
   */
  readonly price_hourly: { readonly gross: string; };
};

/**
 Server type (plan) as returned by `GET /server_types`.
 
 @example
 ```ts
 const type: HetznerServerType = types[0];
 if (type.deprecation === null) { useIt(type.name); }
 ```
 */
export type HetznerServerType = {
  /**
   Type slug (e.g. `cx23`, `cax11`) passed to server creation.
   */
  readonly name: string;
  /**
   CPU architecture, e.g. `x86` or `arm` (`x64` on older responses).
   */
  readonly architecture: string;
  /**
   Number of CPU cores, from the field every server type object carries.
   */
  readonly cores: number;
  /**
   Kind of CPU, `shared` or `dedicated`, which is what tells a general-purpose
   plan from one sold for dedicated cores.
   */
  readonly cpu_type: string;
  /**
   Deprecation marker: `null` when current, an object when deprecated. Typed
   `unknown` so it can be compared to `null` without a banned nullish union.
   */
  readonly deprecation?: unknown;
  /**
   Disk size in GB, from the field every server type object carries.
   */
  readonly disk: number;
  /**
   Memory in GB, from the field every server type object carries.
   */
  readonly memory: number;
  /**
   Per-location prices; a type is offered in a location when it has an entry.
   */
  readonly prices: readonly HetznerServerPrice[];
};

//endregion Server type

//region Action

/**
 Asynchronous action returned by mutating endpoints.
 Poll the generic `GET /actions/{id}` until `status` leaves `running`.
 
 @example
 ```ts
 const action: HetznerAction = create.action;
 if (action.status === 'error') { throw new Error(action.error?.message); }
 ```
 */
export type HetznerAction = {
  /**
   Numeric action id used for polling.
   */
  readonly id: number;
  /**
   Progress status; terminal values are `success` and `error`.
   */
  readonly status: 'error' | 'running' | 'success';
  /**
   Error detail present when `status` is `error` (absent, and `null` on the
   wire, otherwise).
   */
  readonly error?: {
    readonly code: string;
    readonly message: string;
  };
};

//endregion Action

//region Image

/**
 Image resource (system image or snapshot).
 
 @example
 ```ts
 const image: HetznerImage = images[0];
 if (image.deprecated === null) { useIt(image.name); }
 ```
 */
export type HetznerImage = {
  /**
   Numeric image id.
   */
  readonly id: number;
  /**
   Image slug for system images (e.g. `ubuntu-24.04`); absent (and `null` on
   the wire) for snapshots, so narrow with `typeof name === 'string'`.
   */
  readonly name?: string;
  /**
   OS family used to match a shorthand (e.g. `ubuntu`, `fedora`).
   */
  readonly os_flavor: string;
  /**
   Image type, e.g. `system` or `snapshot`.
   */
  readonly type: string;
  /**
   ISO 8601 deprecation timestamp; absent (and `null` on the wire) when not
   deprecated, so test with `typeof deprecated === 'string'`.
   */
  readonly deprecated?: string;
  /**
   ISO 8601 creation timestamp, used to pick the newest matching image.
   */
  readonly created: string;
  /**
   Free-text description.
   */
  readonly description: string;
};

//endregion Image

//region SSH key

/**
 SSH key resource in the project.
 
 @example
 ```ts
 const key: HetznerSshKey = keys[0];
 key.public_key; // 'ssh-ed25519 AAAA... comment'
 ```
 */
export type HetznerSshKey = {
  /**
   Numeric key id passed to server creation.
   */
  readonly id: number;
  /**
   Key name within the project.
   */
  readonly name: string;
  /**
   MD5 fingerprint as stored by Hetzner.
   */
  readonly fingerprint: string;
  /**
   Full public key line including type, material, and comment.
   */
  readonly public_key: string;
};

//endregion SSH key
