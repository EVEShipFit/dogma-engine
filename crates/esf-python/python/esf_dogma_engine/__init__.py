"""Calculates the dogma attributes of an EVE Online ship fit.

Load `sde.dat` once, then calculate as many fits as you like::

    import esf_dogma_engine as dogma

    dogma.load_sde_from_file("sde.dat")
    calculation = dogma.calculate(
        {
            "ship": {"type_id": 587},
            "items": [
                {"type_id": 2873, "slot": {"type": "high", "index": 0}, "state": "active"}
            ],
        }
    )

`pip install eveshipfit-dogma-engine[sde]` brings the data along::

    from eveshipfit_sde import sde_path

    dogma.load_sde_from_file(sde_path())

An EFT-fit imports into a fit `calculate` reads::

    fit = dogma.load_eft("[Rifter, My Rifter]\n200mm AutoCannon I")

`save_eft` writes one back out, in English::

    eft = dogma.save_eft(fit)

An ESI fitting, as a character saves it in game, goes both ways too::

    fit = dogma.load_esi_fitting(fitting)
    fitting = dogma.save_esi_fitting(fit)

An import matches English names. To also match the other seven languages EVE
supports, load `names.dat` as well::

    from eveshipfit_sde import names_path

    dogma.load_names_from_file(names_path())
"""

import os

from ._esf_dogma_engine import (
    beacon,
    calculate,
    load_eft,
    load_esi_fitting,
    load_names,
    load_sde,
    save_eft,
    save_esi_fitting,
)
from .types import Calculation, EsiFitting, Fit, Options, Projection, Violation

__all__ = [
    "Calculation",
    "EsiFitting",
    "Fit",
    "Options",
    "Projection",
    "Violation",
    "beacon",
    "calculate",
    "load_eft",
    "load_esi_fitting",
    "load_names",
    "load_names_from_file",
    "load_sde",
    "load_sde_from_file",
    "save_eft",
    "save_esi_fitting",
]


def load_sde_from_file(path: str | os.PathLike[str]) -> int:
    """Load `sde.dat` from disk. Returns the SDE build number.

    Like `load_sde`, this may only be called once per process.
    """
    with open(path, "rb") as handle:
        return load_sde(handle.read())


def load_names_from_file(path: str | os.PathLike[str]) -> int:
    """Load `names.dat` from disk. Returns its build number.

    Like `load_names`, this may only be called once per process.
    """
    with open(path, "rb") as handle:
        return load_names(handle.read())
