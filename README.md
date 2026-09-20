# EVEShip.fit's Dogma Engine

[![crates.io](https://img.shields.io/crates/v/esf-dogma-engine.svg)](https://crates.io/crates/esf-dogma-engine)
[![npm](https://img.shields.io/npm/v/%40eveshipfit%2Fdogma-engine.svg)](https://www.npmjs.com/package/@eveshipfit/dogma-engine)
[![CI](https://github.com/EVEShipFit/dogma-engine/actions/workflows/testing.yml/badge.svg)](https://github.com/EVEShipFit/dogma-engine/actions/workflows/testing.yml)
[![docs.rs](https://img.shields.io/docsrs/esf-dogma-engine)](https://docs.rs/esf-dogma-engine)

[![Discord](https://img.shields.io/badge/Discord-Join-5865F2?logo=discord&logoColor=white)](https://discord.gg/S5V5BkvNf7)

This library calculates accurately statistics of an EVE Online ship fit.

The input are several data-files provided by EVE Online, together with a ship fit.
The output are all the Dogma attributes of the ship, its items and the character.

## Implementation

This Dogma engine implements a multi-pass approach.

- [pass 1](./crates/esf-dogma-engine/src/calculate/pass_1.rs): collect all the Dogma attributes of the hull and modules.
- [pass 2](./crates/esf-dogma-engine/src/calculate/pass_2.rs): collect all the Dogma effects of the hull and modules.
- [pass 3](./crates/esf-dogma-engine/src/calculate/pass_3.rs): apply all the Dogma effects to the hull/modules, calculating the actual Dogma attribute values.
- [pass 4](./crates/esf-dogma-engine/src/calculate/pass_4.rs): augment the Dogma attributes with EVEShip.fit specific attributes, that are too complex for the Dogma itself to handle.

## Input and output

`calculate` takes a fit and options, and returns a calculation.
All identifiers are those from the SDE.

### Fit

- `name` (optional): name of the fit.
- `ship`: the ship being fitted.
  - `type_id`: its type.
  - `mode` (optional): type ID of the active mode, for ships that have modes.
    _Whether the mode belongs to the ship is not checked._
- `items`: everything fitted or carried. Each item has:
  - `type_id`: its type.
  - `slot`: where the item is.
    - `type`: `high`, `medium`, `low`, `rig`, `subsystem`, `service`, `fighter_tube`, `fighter_bay`, `implant`, `booster`, `drone_bay` or `cargo`.
    - `index`: position within that slot type, starting at 0 (`implant` and `booster` starting at 1).
      Absent for `fighter_bay`, `drone_bay` and `cargo`.
  - `quantity` (optional, default 1): stack size for drones, fighters and cargo. For fighters in a tube, the squadron size.
  - `state`: requested state; `offline`, `online`, `active` or `overload`.
  - `charge` (optional): the loaded charge, as `type_id`.
  - `mutation` (optional): for mutated items (Abyssal modules, mutated drones, ...).
    - `base`: type ID of the item before it was mutated.
    - `attributes`: the rolled value per attribute ID.
  - `fighter_abilities` (optional): the abilities a fighter uses, as effect IDs. Absent means the fighter's default abilities.
  - `booster_side_effects` (optional): the side effects a booster rolled, as effect IDs. Absent means none.
  - `spool` (optional): for modules whose bonus grows every cycle, how far it has spooled.
    Only per-second stats use it; volley is always unspooled. Absent means fully spooled.
    - `multiplier_bonus`: the bonus reached so far; 0.0 is unspooled, 2.125 is +212.5%.
- `character` (optional):
  - `skills`: level (0 to 5) per skill type ID. A missing skill gives no bonuses.
  - `security_status` (optional, default 0.0): the pilot's security status, -10.0 to 5.0.
- `environment` (optional): where the fit is.
  - `damage_profile` (optional, default 0.25 each): incoming damage for effective hitpoints, as `em`, `explosive`, `kinetic` and `thermal` (relative to each other).
  - `security` (optional, default `high_sec`): `high_sec`, `low_sec`, `null_sec` or `wormhole`.
  - `reactive_armor` (optional, default `do_not_adapt`): what a Reactive Armor Hardener shifts its resistances towards.
    `do_not_adapt` leaves them where EVE shows them, `damage_profile` shifts towards `damage_profile`, and
    `{"profile": {..}}` shifts towards a profile of its own, written the same way as `damage_profile`.
- `incoming` (optional): what effects and buffs to apply that come from outside the ship.
  A calculation reports the same shape as `outgoing`: feed one fit's result into another's `incoming` links them up.
  - `buffs` (optional, default none): buffs to apply, like the ones a command burst hands out.
    - `id`: which buff, as `dbuffCollections` in the SDE numbers them.
    - `value`: how strong it is, in whatever the buff's operation reads.
  - `effects` (optional, default none): effects aimed at the fit, like a stasis webifier.
    - `type_id`: the type the effect belongs to; its category decides the stacking penalty.
    - `effect_id`: which effect, as `dogmaEffects` in the SDE numbers them.
    - `attributes`: the value per attribute ID the effect reads, worked out by the fit that aimed it.

### Options

- `sources` (optional, default false): report per attribute what its value was calculated from.
  Leave it off unless you show it; it makes the calculation several times bigger.

### Calculation

- `ship`: result for the ship.
- `mode`: result for the mode; absent when the fit has no mode.
- `items`: one result per item of the fit, in the same order.
- `character`: result for the character.
- `buffs`: the buffs that landed, ordered by id. Those of `incoming`, plus the ones the fit's own bursts hand out: a fleet boost reaches the ship running it. What is missing lost to another source of the same buff, or the SDE has no such buff.
  - `id`: which buff, as `dbuffCollections` in the SDE numbers them.
  - `value`: how strong it is, in whatever the buff's operation reads.
- `outgoing`: what the fit hands to other fits, in the shape `incoming` takes.

Each result has:

- `attributes`: per attribute ID, its `base` value before effects and its final `value`.
  With the `sources` option, also `sources`: every modifier on it, in the order they were applied. Each has:
  - `from`: where it comes from; `type` is `ship`, `mode`, `character`, `item` or `charge` (with the `index` into `items`), `projected` (with the `index` into `incoming.effects`), `skill` (with its `type_id`) or `buff` (with its `id`).
  - `effect_id`: the effect that holds the modifier; `null` for a buff, which has none.
  - `source_attribute_id`: the attribute on the source that holds `value`; `null` for a buff, which carries its own strength.
  - `operator`: `pre_assign`, `pre_mul`, `pre_div`, `mod_add`, `mod_sub`, `post_mul`, `post_div`, `post_percent` or `post_assign`.
  - `value`: the value of the modifying attribute, or the strength of the buff.
  - `quantity`: how many times it counts. A stacking penalised stack is listed once per item instead.
  - `penalty`: the stacking penalty factor it got, or `null` if not penalised.
  - `applied`: false when the source's state is too low for the effect.

  How much each source added is not reported: multiplications compound and stacking penalties depend on order, so there is no single answer.
- `state`: the state the item reached, which can be lower than requested.
- `max_state`: the highest state the item can reach.
- `charge`: result for its charge, if it has one.

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
flatc --rust --gen-onefile -o crates/esf-data/src/sde/ node_modules/@eveshipfit/sde/specs/eve.fbs node_modules/@eveshipfit/sde/specs/names.fbs
cargo run --release -p esf-cli
```

For example, some attributes of a fit with every skill at L0 except two:

```bash
printf '[Nergal, Spool]\nLight Entropic Disintegrator II, Occult S\n' \
  | cargo run --release -p esf-cli -- -l 0 --skill "Gunnery=4" --skill "Rapid Firing=2" -a damage -a speed
```

It prints a table on a terminal and JSON otherwise; see `--help` for the rest.

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
wasm-pack build crates/esf-wasm --release --out-dir ../../pkg
```

In the `pkg` folder is now a NPM module to use.

Javascript hands over `sde.dat` once, and every lookup after that happens inside WebAssembly.
The file is a Flatbuffer, so nothing is parsed: the bytes are used where they land.

```js
import init, { init as initPanicHook, load_sde, calculate, beacon } from "@eveshipfit/dogma-engine";

await init();
initPanicHook();

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
```
