from .types import Calculation, Fit, Options, Projection

def load_sde(bytes: bytes) -> int:
    """Load `sde.dat`. Returns the SDE build number.

    Has to be called before `calculate`; calling it twice raises, as the first
    buffer is borrowed for the rest of the session.

    Raises:
        RuntimeError: the SDE was already loaded.
        ValueError: the bytes are not an SDE.
    """

def calculate(fit: Fit, options: Options | None = None) -> Calculation:
    """Calculate every attribute of the ship, its items and the character.

    Raises:
        RuntimeError: the SDE is not loaded.
        ValueError: the fit or the options do not describe what they should.
    """

def beacon(type_id: int) -> Projection:
    """What a beacon in space hands to every fit in there with it. Put the
    result in `incoming` of a fit to have it applied.

    Raises:
        RuntimeError: the SDE is not loaded.
    """
