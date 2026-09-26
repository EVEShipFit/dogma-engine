# EVEShip.fit's Dogma Engine for Javascript

[![npm](https://img.shields.io/npm/v/%40eveshipfit%2Fdogma-engine.svg)](https://www.npmjs.com/package/@eveshipfit/dogma-engine)
[![Discord](https://img.shields.io/badge/Discord-Join-5865F2?logo=discord&logoColor=white)](https://discord.gg/S5V5BkvNf7)

This library calculates accurately statistics of an EVE Online ship fit.

The input are several data-files provided by EVE Online, together with a ship fit.
The output are all the Dogma attributes of the ship, its items and the character.

The engine is written in Rust and runs as WebAssembly, so everything is calculated in the browser; no server needed.

## Install

[`@eveshipfit/sde`](https://www.npmjs.com/package/@eveshipfit/sde) ships `sde.dat` in its `dist` folder; serve or bundle that file.

```bash
npm install @eveshipfit/dogma-engine @eveshipfit/sde
```

The package is an ES module with TypeScript types included, and it runs anywhere: under a bundler
(Vite, webpack, ...), from a CDN, or in a plain `<script type="module">`.

## Usage

The default export loads the WebAssembly and has to be awaited once before any other function is called.

```js
import wasmInit, {
  load_sde,
  load_eft,
  save_eft,
  load_esi_fitting,
  save_esi_fitting,
  load_killmail,
  load_link,
  save_link,
  calculate,
  beacon,
} from "@eveshipfit/dogma-engine";

await wasmInit();

const sde = await fetch("/sde.dat").then((response) => response.arrayBuffer());
const buildNumber = load_sde(new Uint8Array(sde));
const fit = {
  ship: { type_id: 587 },
  items: [{ type_id: 2873, slot: { type: "high", index: 0 }, state: "active", charge: { type_id: 185 } }],
  character: { skills: { 3300: 5 } },
};

const calculation = calculate(fit);
/* Or if you want to know the source of the effects: */
const withSources = calculate(fit, { sources: true });
/* Or if you have a beacon in space (like wormhole effects): */
const withBeacon = calculate({ ...fit, incoming: beacon(beaconTypeId) });
/* Or if you have an EFT, the text format EVE copies a fit to the clipboard in: */
const imported = calculate(load_eft("[Rifter, My Rifter]\n200mm AutoCannon I"));
/* And to write a fit back out as EFT: */
const eft = save_eft(fit);
/* Or if you have a fitting a character saved in game, as ESI returns it (and the other way around): */
const fromEsi = load_esi_fitting(fitting);
const toEsi = save_esi_fitting(fit);
/* Or if you have a killmail, as ESI returns it, for the fit of the ship that died: */
const fromKillmail = load_killmail(killmail);
/* Or if you have an EVEShip.fit link, `<version>:<payload>`, with the payload gunzipped and unbase64'd: */
const fromLink = load_link(version, payload);
/* And to write the payload of a `v4` link; gzip and base64url it, and put `v4:` in front: */
const toLink = save_link(fit);

/* What EVE would not let you fly, in `violations` of the calculation: */
const validated = calculate(fit, { validate: true });
```

`load_sde` may only be called once.

An EFT import matches the English names in `sde.dat`.
To also match the other languages EVE supports, hand `names.dat` (also in `@eveshipfit/sde`) to `load_names` the same way, after `load_sde`.

Fits and calculations are plain objects, described by the included TypeScript types.
What every field means is explained under [Input and output](https://github.com/EVEShipFit/dogma-engine#input-and-output).

## More

How the engine works, the attributes EVEShip.fit adds and the fitting rules it validates are all in the [main repository](https://github.com/EVEShipFit/dogma-engine).
