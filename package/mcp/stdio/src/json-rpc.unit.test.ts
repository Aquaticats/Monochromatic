import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  isJsonRpcMessage,
  JSON_RPC_INTERNAL_ERROR,
  JSON_RPC_INVALID_PARAMS,
  JSON_RPC_INVALID_REQUEST,
  JSON_RPC_METHOD_NOT_FOUND,
  JSON_RPC_PARSE_ERROR,
  JSON_RPC_UNSUPPORTED_PROTOCOL_VERSION,
} from '@monochromatic-dev/mcp-stdio';

//region isJsonRpcMessage: validates minimum JSON-RPC 2.0 shape

await describe({
  name: '',
  children: [
    describe({
      name: isJsonRpcMessage.name,
      children: [
        it({
          name: 'accepts a valid request with id and method',
          fn: async () => {
            const message = { jsonrpc: '2.0', id: 1, method: 'tools/list', };
            expect(isJsonRpcMessage(message,),).toBe(true,);
          },
        },),
        it({
          name: 'accepts a valid notification without id',
          fn: async () => {
            const notification = { jsonrpc: '2.0', method: 'notifications/initialized', };
            expect(isJsonRpcMessage(notification,),).toBe(true,);
          },
        },),
        it({
          name: 'accepts a request with params',
          fn: async () => {
            const message = { jsonrpc: '2.0', id: 'abc', method: 'tools/call',
              params: { name: 'test', }, };
            expect(isJsonRpcMessage(message,),).toBe(true,);
          },
        },),
        it({
          name: 'rejects null',
          fn: async () => {
            expect(isJsonRpcMessage(null,),).toBe(false,);
          },
        },),
        it({
          name: 'rejects undefined',
          fn: async () => {
            expect(isJsonRpcMessage(undefined,),).toBe(false,);
          },
        },),
        it({
          name: 'rejects a string',
          fn: async () => {
            expect(isJsonRpcMessage('hello',),).toBe(false,);
          },
        },),
        it({
          name: 'rejects a number',
          fn: async () => {
            expect(isJsonRpcMessage(42,),).toBe(false,);
          },
        },),
        it({
          name: 'rejects an array',
          fn: async () => {
            expect(isJsonRpcMessage([1, 2, 3,],),).toBe(false,);
          },
        },),
        it({
          name: 'rejects object without jsonrpc field',
          fn: async () => {
            expect(isJsonRpcMessage({ method: 'tools/list', },),).toBe(false,);
          },
        },),
        it({
          name: 'rejects object with wrong jsonrpc version',
          fn: async () => {
            expect(isJsonRpcMessage({ jsonrpc: '1.0', method: 'tools/list', },),).toBe(
              false,
            );
          },
        },),
        it({
          name: 'rejects object without method field',
          fn: async () => {
            expect(isJsonRpcMessage({ jsonrpc: '2.0', id: 1, },),).toBe(false,);
          },
        },),
        it({
          name: 'rejects object with non-string method',
          fn: async () => {
            expect(isJsonRpcMessage({ jsonrpc: '2.0', method: 42, },),).toBe(false,);
          },
        },),
        it({
          name: 'rejects empty object',
          fn: async () => {
            expect(isJsonRpcMessage({},),).toBe(false,);
          },
        },),
        it({
          name: 'rejects a null id, which no response could echo back',
          fn: async () => {
            expect(
              isJsonRpcMessage({ jsonrpc: '2.0', id: null, method: 'tools/list', },),
            ).toBe(false,);
          },
        },),
        it({
          name: 'rejects a boolean id',
          fn: async () => {
            expect(
              isJsonRpcMessage({ jsonrpc: '2.0', id: true, method: 'tools/list', },),
            ).toBe(false,);
          },
        },),
        it({
          name: 'rejects an object id',
          fn: async () => {
            expect(
              isJsonRpcMessage({ jsonrpc: '2.0', id: {}, method: 'tools/list', },),
            ).toBe(false,);
          },
        },),
        it({
          name: 'accepts a numeric id',
          fn: async () => {
            expect(
              isJsonRpcMessage({ jsonrpc: '2.0', id: 7, method: 'tools/list', },),
            ).toBe(true,);
          },
        },),
        it({
          name: 'rejects array params',
          fn: async () => {
            expect(
              isJsonRpcMessage({ jsonrpc: '2.0', id: 1, method: 'tools/list', params: [], },),
            ).toBe(false,);
          },
        },),
        it({
          name: 'rejects null params',
          fn: async () => {
            expect(
              isJsonRpcMessage({ jsonrpc: '2.0', id: 1, method: 'tools/list', params: null, },),
            ).toBe(false,);
          },
        },),
      ],
    },),

    //endregion isJsonRpcMessage

    //region error code constants: verify expected values

    describe({
      name: 'error code constants',
      children: [
        it({
          name: 'JSON_RPC_METHOD_NOT_FOUND is -32601',
          fn: async () => {
            expect(JSON_RPC_METHOD_NOT_FOUND,).toBe(-32_601,);
          },
        },),
        it({
          name: 'JSON_RPC_INVALID_PARAMS is -32602',
          fn: async () => {
            expect(JSON_RPC_INVALID_PARAMS,).toBe(-32_602,);
          },
        },),
        it({
          name: 'JSON_RPC_INTERNAL_ERROR is -32603',
          fn: async () => {
            expect(JSON_RPC_INTERNAL_ERROR,).toBe(-32_603,);
          },
        },),
        it({
          name: 'JSON_RPC_PARSE_ERROR is -32700',
          fn: async () => {
            expect(JSON_RPC_PARSE_ERROR,).toBe(-32_700,);
          },
        },),
        it({
          name: 'JSON_RPC_INVALID_REQUEST is -32600',
          fn: async () => {
            expect(JSON_RPC_INVALID_REQUEST,).toBe(-32_600,);
          },
        },),
        it({
          name: 'JSON_RPC_UNSUPPORTED_PROTOCOL_VERSION is -32022',
          fn: async () => {
            expect(JSON_RPC_UNSUPPORTED_PROTOCOL_VERSION,).toBe(-32_022,);
          },
        },),
      ],
    },),
    //endregion error code constants
  ],
},);
