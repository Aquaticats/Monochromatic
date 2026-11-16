/**
 Host files a guest downloads while it is provisioned.

 The guest reaches the host through QEMU's user-mode network, where the host
 answers at `10.0.2.2` on the loopback interface. Files are served there
 instead of pushed through the guest agent: a push moves about 0.4 MiB per
 second, and the whole checkout plus two toolchains is a hundred times that.

 The server listens on the loopback interface only, on a port the kernel
 chooses, so nothing outside this host can reach it and no firewall is touched.

 @module
 */

import {
  createServer,
  type Server,
} from 'node:http';
import { readFile, } from 'node:fs/promises';
import { basename, } from 'node:path';

/**
 The host's address as the guest sees it in QEMU's user-mode network.
 */
export const GUEST_VISIBLE_HOST = '10.0.2.2';

/**
 A directory of files a guest may download while it is provisioned.
 */
export type ServedFiles = {
  /**
   Port the server listens on.
   */
  readonly port: number;

  /**
   Guest URL of one file of the directory.
   *
   @param path - host path of a file inside the served directory
   */
  readonly urlOf: (path: string,) => string;

  /**
   Stops the server.
   */
  readonly close: () => Promise<void>;
};

/**
 Serves one directory on the loopback interface for as long as provisioning
 runs.

 @param directory - host directory whose files the guest fetches

 @returns the running server and the URLs it answers

 @throws when the port cannot be bound
 */
export async function serveFiles({ directory, }: { readonly directory: string; },): Promise<ServedFiles> {
  /**
   The directory's own name, which is the only prefix the guest's paths carry.
   */
  const root = directory.replace(
    /\/+$/u,
    '',
  );
  const server: Server = createServer(function answer(
    request,
    response,
  ): void {
    /**
     Requested file's path, without the query the guest never sends.
     */
    const requested = (request.url ?? '/').split('?',)[0] ?? '/';
    /**
     Name of the requested file, which may not leave the served directory.
     */
    const name = basename(decodeURIComponent(requested,),);
    if (name === '') {
      response.writeHead(
        404,
        { 'content-type': 'text/plain', },
      );
      response.end('not found\n',);
      return;
    }
    void (async function send(): Promise<void> {
      /**
       The file's bytes, or null when it is not there.
       */
      const body = await readFile(
        `${root}/${name}`,
      )
        .catch(function missing(): null {
          return null;
        },);
      if (body === null) {
        response.writeHead(
          404,
          { 'content-type': 'text/plain', },
        );
        response.end('not found\n',);
        return;
      }
      response.writeHead(
        200,
        {
          'content-length': String(body.byteLength,),
          'content-type': 'application/octet-stream',
        },
      );
      response.end(body,);
    })();
  },);
  /**
   Port the kernel chose for the loopback listener.
   */
  const port = await new Promise<number>(function listen(
    resolve,
    reject,
  ): void {
    server.once(
      'error',
      reject,
    );
    server.listen(
      0,
      '127.0.0.1',
      function listening(): void {
        /**
         Bound address, which carries the chosen port.
         */
        const address = server.address();
        if ((address === null) || ((typeof address) === 'string')) {
          reject(new Error('the file server reported no port.',),);
          return;
        }
        resolve(address.port,);
      },
    );
  },);
  return {
    port,
    urlOf: function urlOf(path: string,): string {
      return `http://${GUEST_VISIBLE_HOST}:${String(port,)}/${encodeURIComponent(basename(path,),)}`;
    },
    close: async function close(): Promise<void> {
      await new Promise<void>(function stop(
        resolve,
        reject,
      ): void {
        server.close(function closed(error,): void {
          if (error === undefined) {
            resolve();
            return;
          }
          reject(error,);
        },);
        server.closeAllConnections();
      },);
    },
  };
}
