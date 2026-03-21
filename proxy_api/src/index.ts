/**
 * Welcome to Cloudflare Workers!
 *
 * This worker acts as a caching proxy for the nextbus.nusmods.com API
 * and the LTA DataMall API (via a server-side key injection).
 *
 * - It validates requests to ensure they are for the correct endpoint.
 * - It caches responses for 15 seconds to reduce load on the upstream server
 *   and improve response times.
 * - It handles CORS preflight requests and adds CORS headers to all responses.
 */

export interface Env {
	// Static assets binding configured in wrangler.toml
	ASSETS: Fetcher;
	// LTA DataMall API key (set via: wrangler secret put LTA_ACCOUNT_KEY)
	LTA_ACCOUNT_KEY: string;
}

export default {
	async fetch(request: Request, env: Env, ctx: ExecutionContext): Promise<Response> {
		const url = new URL(request.url);

		// Handle CORS preflight requests
		if (request.method === 'OPTIONS') {
			return handleOptions();
		}

		if (url.pathname === '/ShuttleService') {
			return handleShuttleService(request, env, ctx, url);
		}

		if (url.pathname === '/BusArrival') {
			return handleLtaProxy(request, env, ctx, '/ltaodataservice/v3/BusArrival');
		}

		if (url.pathname === '/BusStops') {
			return handleLtaProxy(request, env, ctx, '/ltaodataservice/BusStops');
		}

		// Serve the wasm bundle and static files at the root domain.
		return env.ASSETS.fetch(request);
	},
};

async function handleShuttleService(
	request: Request,
	env: Env,
	ctx: ExecutionContext,
	url: URL,
): Promise<Response> {
	if (request.method !== 'GET' && request.method !== 'HEAD') {
		return createErrorResponse('Method Not Allowed', 405);
	}

	// --- 1. Validate the Request ---
	const busStopName = url.searchParams.get('busstopname');
	if (!busStopName || busStopName.trim() === '') {
		return createErrorResponse('Missing or empty busstopname parameter', 400);
	}

	// --- 2. Check the Cache ---
	// Use the Cloudflare default cache.
	// We create a custom cache key based on the full URL to distinguish between query params.
	const cache = caches.default;
	// We MUST use the full request URL as the key.
	const cacheKey = new Request(url.toString(), request);

	let response = await cache.match(cacheKey);

	if (response) {
		// Return cached response
		// We need to re-create the response to modify headers (like adding CORS if missing from cache)
		const newResponse = new Response(response.body, response);
		newResponse.headers.set('X-Cache-Status', 'HIT');
		addCorsHeaders(newResponse.headers);
		return newResponse;
	}

	// --- 3. Fetch from Origin (if not in cache) ---
	try {
		// Construct target URL
		const targetUrl = new URL(url.pathname + url.search, 'https://nnextbus.nusmods.com');

		// Prepare headers for the target request
		const proxyHeaders = new Headers(request.headers);
		proxyHeaders.delete('Host');
		proxyHeaders.set('Referer', 'https://nnextbus.nusmods.com/');
		proxyHeaders.set('Origin', 'https://nnextbus.nusmods.com');

		const originResponse = await fetch(targetUrl.toString(), {
			method: request.method,
			headers: proxyHeaders,
			redirect: 'follow',
		});

		// Prepare the response to be sent back and cached.
		const responseHeaders = new Headers(originResponse.headers);
		addCorsHeaders(responseHeaders);
		responseHeaders.set('Cache-Control', 'public, max-age=15');
		responseHeaders.set('X-Cache-Status', 'MISS');

		const responseToCache = new Response(originResponse.body, {
			status: originResponse.status,
			statusText: originResponse.statusText,
			headers: responseHeaders,
		});

        // Store in cache
        // clone() is needed because the body can only be consumed once
		ctx.waitUntil(cache.put(cacheKey, responseToCache.clone()));

		return responseToCache;
	} catch (e) {
		return createErrorResponse(String(e), 500);
	}
}

async function handleLtaProxy(
	request: Request,
	env: Env,
	ctx: ExecutionContext,
	ltaPath: string,
): Promise<Response> {
	const url = new URL(request.url);
	const targetUrl = new URL('https://datamall2.mytransport.sg');
	targetUrl.pathname = ltaPath;
	targetUrl.search = url.search;
	const cache = caches.default;
	const cacheKey = new Request(targetUrl.toString());
	let response = await cache.match(cacheKey);
	if (response) {
		const newResponse = new Response(response.body, response);
		newResponse.headers.set('X-Cache-Status', 'HIT');
		addCorsHeaders(newResponse.headers);
		return newResponse;
	}
	const originResponse = await fetch(targetUrl.toString(), {
		method: 'GET',
		headers: { 'AccountKey': env.LTA_ACCOUNT_KEY, 'Accept': 'application/json' },
	});
	const responseHeaders = new Headers(originResponse.headers);
	addCorsHeaders(responseHeaders);
	responseHeaders.set('Cache-Control', 'public, max-age=15');
	responseHeaders.set('X-Cache-Status', 'MISS');
	const responseToCache = new Response(originResponse.body, {
		status: originResponse.status, statusText: originResponse.statusText, headers: responseHeaders,
	});
	ctx.waitUntil(cache.put(cacheKey, responseToCache.clone()));
	return responseToCache;
}

/**
 * Handles CORS preflight (OPTIONS) requests.
 */
function handleOptions(): Response {
	const headers = new Headers();
	addCorsHeaders(headers);
	return new Response(null, { headers });
}

/**
 * Creates a JSON error response.
 */
function createErrorResponse(message: string, status: number): Response {
	const headers = new Headers({
		'Content-Type': 'application/json',
	});
	addCorsHeaders(headers);
	return new Response(JSON.stringify({ error: message }), { status, headers });
}

/**
 * Adds standard CORS headers to a Headers object.
 */
function addCorsHeaders(headers: Headers) {
	headers.set('Access-Control-Allow-Origin', '*');
	headers.set('Access-Control-Allow-Methods', 'GET, HEAD, POST, OPTIONS');
	headers.set('Access-Control-Allow-Headers', '*');
}
