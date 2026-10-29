/**
 Worker-source builders for the make-asynchronous fork.
 
 Upstream `make-asynchronous` builds each worker's module source by
 concatenating fixed template strings with the wrapped function's
 serialized source. The fork keeps that structure exactly: string constants
 hold the shared error reporter, the node preamble, and the per-kind worker
 bodies, and builders join them with the serialized function source at wrap
 time. Keeping every template in one module lets the differential oracle
 compare built worker sources verbatim.
 
 Derived from [`make-asynchronous`](https://github.com/sindresorhus/make-asynchronous)
 by Sindre Sorhus (MIT); see `LICENSES/MIT.txt` and this package's README
 for the full attribution. Behavior matches `make-asynchronous` 2.1.0.
 
 @module
 */

//region Shared error reporter

/**
 Worker-side error serializer shared by both worker kinds.
 
 Cloning an error keeps only its message, stack, and cause, so every other
 own property travels separately. A property that cannot be cloned would
 take the whole message down with it, hence the fallback, and a thrown value
 that cannot be cloned at all is replaced by the error that says so.
 
 Verbatim from upstream `make-asynchronous` 2.1.0, kept byte-identical so
 the differential oracle can compare worker sources directly.
 
 @example
 ```ts
 errorReporterSource; // => "const getErrorProperties = ..."
 ```
 */
export const errorReporterSource: string = String.raw`
	const getErrorProperties = error => {
		if (typeof error !== 'object' || error === null) {
			return undefined;
		}

		const properties = {...error};

		// Not an own enumerable property, so spreading the error would miss it.
		if (Array.isArray(error.errors)) {
			properties.errors = error.errors;
		}

		return properties;
	};

	const reportError = (error, id) => {
		let errorName;

		try {
			// A name lives outside the own enumerable properties, and cloning drops a custom one, so it travels on its own.
			errorName = typeof error?.name === 'string' ? error.name : undefined;

			globalThis.postMessage({id, error, errorName, errorProperties: getErrorProperties(error)});
		} catch {
			try {
				// The name is a string, so it can always come along.
				globalThis.postMessage({id, error, errorName});
			} catch (cloneError) {
				globalThis.postMessage({id, error: cloneError});
			}
		}
	};
`;

//endregion Shared error reporter

//region Node preamble

/**
 Node.js preamble shimmed in front of the worker body on Node.js hosts.
 
 On Node.js the worker source runs through `eval`, and this preamble shims
 the Web Worker globals (`postMessage`/`onmessage`) that the worker body
 uses onto `worker_threads` `parentPort`. The same worker body then runs
 unchanged in Node.js and browsers. It also installs the `baseUrl`
 module-hook wiring that resolves bare dynamic imports from the caller's
 module.
 
 Verbatim from upstream `make-asynchronous` 2.1.0, kept byte-identical so
 the differential oracle can compare worker sources directly.
 
 @example
 ```ts
 nodeWorkerPreambleSource; // => "import {parentPort, workerData} ..."
 ```
 */
export const nodeWorkerPreambleSource: string = String.raw`
	import {parentPort, workerData} from 'node:worker_threads';
	import {registerHooks} from 'node:module';
	const isBareSpecifier = specifier => !specifier.startsWith('.') && !specifier.startsWith('/') && !/^[a-z\d+.-]+:/i.test(specifier);
	if (workerData.baseUrl) {
		registerHooks({
			resolve(specifier, context, nextResolve) {
				if (context.parentURL === import.meta.url && isBareSpecifier(specifier)) {
					return nextResolve(specifier, {...context, parentURL: workerData.baseUrl});
				}

				return nextResolve(specifier, context);
			},
		});
	}
	globalThis.self = globalThis;
	globalThis.postMessage = data => parentPort.postMessage(data);
	parentPort.on('message', data => globalThis.onmessage({data}));
`;

//endregion Node preamble

//region Worker bodies

/**
 Builds the worker body running one wrapped call per message.
 
 @param serializedFunction - Wrapped function source from `Function` text
 conversion.
 
 @returns Worker body invoking the serialized function per message.
 
 @example
 ```ts
 makeCallWorkerBody('function (x) { return x; }',);
 ```
 */
export function makeCallWorkerBody(serializedFunction: string,): string {
  return `${errorReporterSource}\n\tglobalThis.onmessage = async ({data: {id, arguments_}}) => {\n\t\ttry {\n\t\t\tconst output = await (${serializedFunction})(...arguments_);\n\t\t\tglobalThis.postMessage({id, output});\n\t\t} catch (error) {\n\t\t\treportError(error, id);\n\t\t}\n\t};\n\t`;
}

/**
 Builds the worker body draining one wrapped iterable across messages.
 
 The first message runs the wrapped function and normalizes its return into
 an iterator; later messages carry no arguments and pull the next item. The
 iterator protocol requires an object per `next()`, otherwise iteration
 would never end.
 
 @param serializedFunction - Wrapped function source from `Function` text
 conversion.
 
 @returns Worker body pulling iterator items per message.
 
 @example
 ```ts
 makeIterableWorkerBody('function * (x) { yield x; }',);
 ```
 */
export function makeIterableWorkerBody(serializedFunction: string,): string {
  return `${errorReporterSource}\n\tconst nothing = Symbol('nothing');\n\tlet iterator = nothing;\n\n\tglobalThis.onmessage = async ({data: {id, arguments_}}) => {\n\t\ttry {\n\t\t\tif (iterator === nothing) {\n\t\t\t\tconst iterable = await (${serializedFunction})(...arguments_);\n\t\t\t\t// A function can return any iterable, not just an iterator.\n\t\t\t\titerator = iterable[Symbol.asyncIterator]?.() ?? iterable[Symbol.iterator]?.() ?? iterable;\n\t\t\t}\n\n\t\t\tconst output = await iterator.next();\n\n\t\t\t// The iterator protocol requires an object, otherwise the iteration would never end.\n\t\t\tif (typeof output !== 'object' || output === null) {\n\t\t\t\tthrow new TypeError('Iterator result is not an object');\n\t\t\t}\n\n\t\t\tglobalThis.postMessage({id, output});\n\t\t} catch (error) {\n\t\t\treportError(error, id);\n\t\t}\n\t};\n\t`;
}

//endregion Worker bodies
