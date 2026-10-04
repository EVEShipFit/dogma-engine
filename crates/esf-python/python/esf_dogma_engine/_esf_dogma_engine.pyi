from .types import Calculation, EsiFitting, EsiKillmail, Fit, Options, Projection

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

def post_load(fit: Fit) -> Fit:
    """Set a fit loaded from EFT, an ESI fitting or DNA to the states EVE
    gives it on import. Set the skills of the character first.

    A ship with modes starts in its first; a cloak is online rather than
    active; of modules with a limit on how many can be online or active, like
    a microwarpdrive and an afterburner, only the first keep their state; only
    the drones that fit in space are launched, the most damaging first; and
    fighters fill the empty tubes.

    Raises:
        RuntimeError: the SDE is not loaded.
        ValueError: the fit does not describe what it should.
    """

def load_killmail(killmail: EsiKillmail) -> Fit:
    """Load the fit of the ship that died from a killmail, as ESI returns it.

    The fit has no skills, and is named after the killmail. A charge in the
    slot of a module is loaded in it; what was destroyed and what dropped are
    added up. Items under a flag a fit has no place for, and what was inside a
    container, are left out.

    Raises:
        RuntimeError: the SDE is not loaded.
        ValueError: the killmail does not describe what it should.
    """

def load_dna(dna: str) -> Fit:
    """Load a fit from DNA, the fits EVE links to in chat, with or without its
    `fitting:` prefix.

    The fit has no skills and no name. A module fills the next free slot of
    its rack; drones and fighters go in their bay, anything else in the cargo.
    A DNA cut short, without its closing `::`, loses its last item.

    Raises:
        RuntimeError: the SDE is not loaded.
        ValueError: the DNA has no ship, or a type or quantity is not a number.
    """

def load_link(version: str, payload: str) -> Fit:
    """Load a fit from an EVEShip.fit link, of any version before `v4`.

    A link is `<version>:<payload>`; hand over the payload once it is
    base64-decoded and gunzipped. The fit has no skills.

    Raises:
        RuntimeError: the SDE is not loaded.
        ValueError: the version is not one ever written, or the payload is not
            a fit of that version.
    """

def load_esf(text: str) -> Fit:
    """Load a fit from an esf/1 document, as text.

    Of a document with several fits, this is the first one no other fit
    carries. The fit has no skills.

    Raises:
        RuntimeError: the SDE is not loaded.
        ValueError: the document is not valid esf/1, or its fit has no ship.
    """

def load_esf_link(link: str) -> Fit:
    """Load a fit from an esf/1 link: the binary form, in base64url.

    Raises:
        RuntimeError: the SDE is not loaded.
        ValueError: the link is not valid esf/1, or its fit has no ship.
    """

def save_esf(fit: Fit) -> str:
    """Write a fit as an esf/1 document, as text, in canonical form.

    Raises:
        RuntimeError: the SDE is not loaded.
        ValueError: the fit names a type the SDE does not know.
    """

def save_esf_link(fit: Fit) -> str:
    """Write a fit as an esf/1 link: the binary form, in base64url.

    Raises:
        RuntimeError: the SDE is not loaded.
        ValueError: the fit names a type the SDE does not know.
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
