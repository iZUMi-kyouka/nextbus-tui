# NextBus Edge App (Worker + WASM assets)

This Cloudflare Worker serves the web build for `nextbus-tui` and proxies API calls to `nnextbus.nusmods.com`.

## Setup

1. Install dependencies:
   ```bash
   npm install
   ```

2. Start the development server:
   ```bash
   npm run dev
   ```

3. Deploy Worker only:
   ```bash
   npm run deploy
   ```

4. Build + deploy web bundle and Worker together:
   ```bash
   npm run deploy:web
   ```

## Configuration

The Worker and static asset binding are configured in `wrangler.toml`.

## API

Routing:

- `GET /ShuttleService?busstopname=COM3` -> proxied to `https://nnextbus.nusmods.com/ShuttleService?busstopname=COM3`
- Any other path (for example `/`, `/index.html`) -> served from static assets (`../dist`)

It adds CORS headers to allow cross-origin requests from your application.
