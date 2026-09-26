# EVEShip.fit's Dogma Engine for Python

[![PyPI](https://img.shields.io/pypi/v/eveshipfit-dogma-engine.svg)](https://pypi.org/project/eveshipfit-dogma-engine/)
[![Discord](https://img.shields.io/badge/Discord-Join-5865F2?logo=discord&logoColor=white)](https://discord.gg/S5V5BkvNf7)

This library calculates accurately statistics of an EVE Online ship fit.

The input are several data-files provided by EVE Online, together with a ship fit.
The output are all the Dogma attributes of the ship, its items and the character.

The engine is written in Rust; this package wraps it, so every lookup happens inside Rust.

## Install

The `sde` extra brings in [`eveshipfit-sde`](https://pypi.org/project/eveshipfit-sde/), which ships `sde.dat`.

```bash
pip install eveshipfit-dogma-engine[sde]
```

## Usage

```python
import esf_dogma_engine as dogma
from eveshipfit_sde import sde_path

build_number = dogma.load_sde_from_file(sde_path())

fit = {
    "ship": {"type_id": 587},
    "items": [
        {
            "type_id": 2873,
            "slot": {"type": "high", "index": 0},
            "state": "active",
            "charge": {"type_id": 185},
        }
    ],
    "character": {"skills": {3300: 5}},
}

calculation = dogma.calculate(fit)
# Or if you want to know the source of the effects:
with_sources = dogma.calculate(fit, {"sources": True})
# Or if you have a beacon in space (like wormhole effects):
with_beacon = dogma.calculate({**fit, "incoming": dogma.beacon(beacon_type_id)})
# Or if you have an EFT, the text format EVE copies a fit to the clipboard in:
imported = dogma.calculate(dogma.load_eft("[Rifter, My Rifter]\n200mm AutoCannon I"))
# And to write a fit back out as EFT:
eft = dogma.save_eft(fit)
# Or if you have a fitting a character saved in game, as ESI returns it (and the other way around):
from_esi = dogma.load_esi_fitting(fitting)
to_esi = dogma.save_esi_fitting(fit)
# Or if you have a killmail, as ESI returns it, for the fit of the ship that died:
from_killmail = dogma.load_killmail(killmail)
# Or if you have an EVEShip.fit link, `<version>:<payload>`, with the payload gunzipped and unbase64'd:
from_link = dogma.load_link(version, payload)
# And to write the payload of a `v4` link; gzip and base64url it, and put `v4:` in front:
to_link = dogma.save_link(fit)

# What EVE would not let you fly, in `violations` of the calculation:
validated = dogma.calculate(fit, {"validate": True})
```

`load_sde_from_file` may only be called once per process.

An EFT import matches the English names in `sde.dat`.
To also match the other languages EVE supports, load `names.dat` as well:

```python
from eveshipfit_sde import names_path

dogma.load_names_from_file(names_path())
```

Fits and calculations are plain dicts, typed with `TypedDict` in `esf_dogma_engine.types`.
What every field means is explained under [Input and output](https://github.com/EVEShipFit/dogma-engine#input-and-output).

Every function but `load_sde` and `load_names` releases the GIL while it works, so a thread pool calculates fits in parallel.

## More

How the engine works, the attributes EVEShip.fit adds and the fitting rules it validates are all in the [main repository](https://github.com/EVEShipFit/dogma-engine).
