# EVEShip.fit's Dogma Engine

This library calculates accurately statistics of an EVE Online ship fit.

The input are several data-files provided by EVE Online, together with a ship fit.
The output are all the Dogma attributes, containing all the details of the ship.

## Implementation

This Dogma engine implements a multi-pass approach.

- [pass 1](./src/calculate/pass_1.rs): collect all the Dogma attributes of the hull and modules.
- [pass 2](./src/calculate/pass_2.rs): collect all the Dogma effects of the hull and modules.
- [pass 3](./src/calculate/pass_3.rs): apply all the Dogma effects to the hull/modules, calculating the actual Dogma attribute values.
- [pass 4](./src/calculate/pass_4.rs): augment the Dogma attributes with EVEShip.fit specific attributes.

## EVEShip.fit's specific attributes

`Pass 4` create Dogma attributes that do not exist in-game, but are calculated based on other Dogma attributes.
To make rendering a fit easier, these are calculated by this library, and presented as new Dogma attributes.

Their identifier is always a negative value, to visually separate them.
What additional attributes exist are defined in [EVEShipFit/data](https://github.com/EVEShipFit/data) repository.

## Development

Make sure you have [Rust installed](https://www.rust-lang.org/tools/install).

```bash
cargo install wasm-pack
wasm-pack build --release
```

In the `pkg` folder is now a NPM module to use.
See below on how to integrate this in your own website.

## Integration

### Javascript (WebAssembly)

The primary goal of this library is to build a WebAssembly variant that can easily be used in the browser.
This means that there is no need for a server-component, and everything can be calculated in the browser.

This is done with [wasm-pack](https://rustwasm.github.io/wasm-pack/).

To make sure that EVEShip.fit is as fast as possible, all data-files are read by Javascript, and made available to this library by callbacks.
Transferring all data-files from Javascript to Rust is simply too expensive.

In result, Javascript needs to have the following functions defined:

- `get_dogma_attributes(type_id)` - To get a list of all Dogma attributes for a given item.
- `get_dogma_attribute(attribute_id)` - To get all the details of a single Dogma attribute.
- `get_dogma_effects(type_id)` - To get a list of all Dogma effects for a given item.
- `get_dogma_effect(effect_id)` - To get all the details of a single Dogma effect.
- `get_type_id(type_id)` - To get all the details of a single item.

The returning value should be a Javascript object.
The fields are defined in in [data_types.rs](./src/data_types.rs).
