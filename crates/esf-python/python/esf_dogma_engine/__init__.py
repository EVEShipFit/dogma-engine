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
"""

import os

from ._esf_dogma_engine import beacon, calculate, load_sde
from .types import Calculation, Fit, Options, Projection

__all__ = [
    "Calculation",
    "Fit",
    "Options",
    "Projection",
    "beacon",
    "calculate",
    "load_sde",
    "load_sde_from_file",
]


def load_sde_from_file(path: str | os.PathLike[str]) -> int:
    """Load `sde.dat` from disk. Returns the SDE build number.

    Like `load_sde`, this may only be called once per process.
    """
    with open(path, "rb") as handle:
        return load_sde(handle.read())
