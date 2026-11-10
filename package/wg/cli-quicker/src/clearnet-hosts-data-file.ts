/**
 The name of the data file that carries the clearnet host lists of this build.

 The build copies that file beside the route-watcher bundle,
 and the running watcher reads the copy again at every refresh,
 so a newer list installed beside a running watcher takes effect with nothing for a person to do,
 and without a restart.
 The name lives in a module of its own because the build names the copy from the same constant the runtime looks it up by.
 */
export const CLEARNET_HOSTS_DATA_FILENAME = 'clearnet-hosts.data.json';
