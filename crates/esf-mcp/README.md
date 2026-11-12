# esf-mcp

An [MCP](https://modelcontextprotocol.io) server that lets AI assistants calculate EVE Online ship fits with the EVEShip.fit dogma engine.

It runs locally over stdio, and remembers nothing between calls.

## Tools

- `calculate_fit`: DPS, volley, EHP, resists, capacitor, speed, targeting, fitting use, and every rule the fit breaks.
- `get_attributes`: the raw attributes of the ship, an item or a charge, optionally with what modified them.
- `find_type`: search items by name.
- `find_variations`: the meta variants of an item; given a fit, each is swapped in and recalculated.
- `compare_fits`: every stat that differs between two fits.
- `convert_fit`: to esf/1, EFT or ESI fitting JSON.

Fits are esf/1, EFT, DNA, ESI fitting JSON or ESI killmail JSON.
The [esf/1 specification](./esf-spec.md) is offered as the resource `esf://spec`.
Skills default to all level 5.

## Install

```bash
cargo install esf-mcp
```

On first start it downloads the latest [`@eveshipfit/sde`](https://www.npmjs.com/package/@eveshipfit/sde) to the user's cache directory, and checks for a newer one once a day.
That first download is about 25 MB; on a slow connection, start it once by hand so the AI client does not time out waiting.
`--sde` and `--names` use local files instead; `--cache-dir` moves the cache.

## Use

Claude Code:

```bash
claude mcp add esf -- esf-mcp
```

Claude Desktop, Cursor and others take the same command in their JSON config:

```json
{
  "mcpServers": {
    "esf": { "command": "esf-mcp" }
  }
}
```

## As a library

`esf_mcp::Server` is the same server, for any rmcp transport:

```rust
let (_, game) = esf_mcp::data::load(&cache_dir)?;
let server = esf_mcp::Server::new(game);
```
