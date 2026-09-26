from .types import Calculation, EsiFitting, Fit, Options, Projection

def load_sde(bytes: bytes) -> int:
    """Load `sde.dat`. Returns the SDE build number.

    Has to be called before `calculate`; calling it twice raises, as the first
    buffer is borrowed for the rest of the session.

    Raises:
        RuntimeError: the SDE was already loaded.
        ValueError: the bytes are not an SDE.
    """

def load_names(bytes: bytes) -> int:
    """Load `names.dat`. Returns its build number.

    Optional: without it `load_eft` only matches English names. The SDE has to
    be loaded first, and calling it twice raises.

    Raises:
        RuntimeError: the SDE is not loaded, or the names already were.
        ValueError: the bytes are not a names file, or are of another build
            than the SDE.
    """

def load_eft(eft: str) -> Fit:
    """Load a fit from EFT, the text format EVE copies a fit to the clipboard in.

    The fit has no skills; fill in `character` yourself.

    Raises:
        RuntimeError: the SDE is not loaded.
        ValueError: the text is not a fit this can read.
    """

def save_eft(fit: Fit) -> str:
    """Write a fit as EFT, the text format EVE copies a fit to the clipboard in.

    Raises:
        RuntimeError: the SDE is not loaded.
        ValueError: the fit does not describe what it should, or names a type
            the SDE does not know.
    """

def load_esi_fitting(fitting: EsiFitting) -> Fit:
    """Load a fit from an ESI fitting, the fits a character saves in game.

    The fit has no skills; fill in `character` yourself. A charge in the slot
    of a module is loaded in it; items under a flag a fit has no place for are
    left out.

    Raises:
        RuntimeError: the SDE is not loaded.
        ValueError: the fitting does not describe what it should.
    """

def save_esi_fitting(fit: Fit) -> EsiFitting:
    """Write a fit as an ESI fitting, the fits a character saves in game.

    ESI has no fighter tubes, implants or boosters: a squadron goes in the
    fighter bay, the others in the cargo. States are not kept.

    Raises:
        RuntimeError: the SDE is not loaded.
        ValueError: the fit does not describe what it should.
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
