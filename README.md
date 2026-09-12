# EVEShip.fit's Dogma Engine

This library calculates accurately statistics of an EVE Online ship fit.

The input are several data-files provided by EVE Online, together with a ship fit.
The output are all the Dogma attributes, containing all the details of the ship.

## Implementation

This Dogma engine implements a multi-pass approach.

- [pass 1](./src/calculate/pass_1.rs): collect all the Dogma attributes of the hull and modules.
- [pass 2](./src/calculate/pass_2.rs): collect all the Dogma effects of the hull and modules.
- [pass 3](./src/calculate/pass_3.rs): apply all the Dogma effects to the hull/modules, calculating the actual Dogma attribute values.
- [pass 4](./src/calculate/pass_4.rs): augment the Dogma attributes with EVEShip.fit specific attributes, that are too complex for the Dogma itself to handle.

## EVEShip.fit's specific attributes

`Pass 4` create Dogma attributes that do not exist in-game, but are rather complicated to calculate.
To make rendering a fit easier, these are calculated by this library, and presented as new Dogma attributes.

Their identifier is always a negative value, to visually separate them.
What additional attributes exist are defined in [EVEShipFit/sde-patched](https://github.com/EVEShipFit/sde-patched) repository.

## Development

Make sure you have [Rust installed](https://www.rust-lang.org/tools/install).

Next, we need the data-files.
They are Flatbuffers, built by [sde-patched](https://github.com/EVEShipFit/sde-patched) and published on npm as [`@eveshipfit/sde`](https://www.npmjs.com/package/@eveshipfit/sde):

```bash
npm ci
```

- `sde.dat` holds everything needed to calculate a fit.
- `names.dat` holds the type names in the other seven languages EVE supports.
  It is optional.

English names live in `sde.dat`, so an EFT-fit written in English imports without it; `names.dat` is only consulted when a name does not match.

After that, we can run the application.

```bash
flatc --rust --gen-onefile -o src/sde/ node_modules/@eveshipfit/sde/specs/eve.fbs node_modules/@eveshipfit/sde/specs/names.fbs
cargo run --release --no-default-features --features rust
```

The regression suite reads the same paths; set `ESF_SDE` and `ESF_NAMES` to point it elsewhere.

## Regression

The engine is locked down by snapshot tests.
A case calculates one fit with one set of skills, and compares the result against a stored snapshot in [tests/snapshots](./tests/snapshots).

```bash
cargo test
```

If failures are expected differences, use `insta` to resolve them:

```bash
cargo install cargo-insta
cargo insta review
```

## Integration

### Javascript (WebAssembly)

The primary goal of this library is to build a WebAssembly variant that can easily be used in the browser.
This means that there is no need for a server-component, and everything can be calculated in the browser.

This is done with [wasm-pack](https://rustwasm.github.io/wasm-pack/):

```bash
cargo install wasm-pack
wasm-pack build --release -- --no-default-features --features wasm
```

In the `pkg` folder is now a NPM module to use.

Javascript hands over `sde.dat` once, and every lookup after that happens inside WebAssembly.
The file is a Flatbuffer, so nothing is parsed: the bytes are used where they land.

```js
import init, { init as initPanicHook, load_sde, calculate } from "@eveshipfit/dogma-engine";

await init();
initPanicHook();

const sde = await fetch("/sde.dat").then((response) => response.arrayBuffer());
const buildNumber = load_sde(new Uint8Array(sde));

const statistics = calculate(fit, skills);
```
