"""The shapes `calculate` reads and returns.

Every identifier is the one the SDE uses. Maps are keyed by id; a result fed
back in as `incoming` keeps its integer keys.
"""

from typing import Literal, TypedDict

__all__ = [
    "AttributeValue",
    "Calculation",
    "Character",
    "Charge",
    "DamageProfile",
    "EffectOperator",
    "Environment",
    "Fit",
    "FitItem",
    "ItemResult",
    "Mutation",
    "Options",
    "ProjectedBuff",
    "ProjectedEffect",
    "ReactiveArmor",
    "Projection",
    "Security",
    "Ship",
    "Slot",
    "Source",
    "SourceRef",
    "Spool",
    "State",
]

State = Literal["offline", "online", "active", "overload"]
Security = Literal["high_sec", "low_sec", "null_sec", "wormhole"]

EffectOperator = Literal[
    "pre_assign",
    "pre_mul",
    "pre_div",
    "mod_add",
    "mod_sub",
    "post_mul",
    "post_div",
    "post_percent",
    "post_assign",
]


class RackSlot(TypedDict):
    """A slot with a position in it, starting at 0. Implants and boosters are
    numbered as EVE numbers them, starting at 1."""

    type: Literal[
        "high",
        "medium",
        "low",
        "rig",
        "subsystem",
        "service",
        "fighter_tube",
        "implant",
        "booster",
    ]
    index: int


class BaySlot(TypedDict):
    """A bay, which holds any number of items."""

    type: Literal["fighter_bay", "drone_bay", "cargo"]


Slot = RackSlot | BaySlot


class Charge(TypedDict):
    """A charge loaded in a module."""

    type_id: int


class Mutation(TypedDict):
    """How a module or drone was mutated."""

    base: int
    attributes: dict[int, float]


class Spool(TypedDict):
    """How far a module has spooled; 0.0 is unspooled, 2.125 is +212.5%."""

    multiplier_bonus: float


class _FitItemRequired(TypedDict):
    type_id: int
    slot: Slot
    state: State


class FitItem(_FitItemRequired, total=False):
    """A module, drone, fighter squadron, implant, booster or item in cargo."""

    quantity: int
    charge: Charge | None
    mutation: Mutation
    fighter_abilities: list[int]
    booster_side_effects: list[int]
    spool: Spool


class _ShipRequired(TypedDict):
    type_id: int


class Ship(_ShipRequired, total=False):
    """The ship of a fit."""

    mode: int


class Character(TypedDict, total=False):
    """The character flying the ship."""

    skills: dict[int, int]
    security_status: float


class DamageProfile(TypedDict, total=False):
    """Incoming damage, relative to each other."""

    em: float
    explosive: float
    kinetic: float
    thermal: float


class ReactiveArmorProfile(TypedDict):
    """Shift towards damage of its own, which the rest of the fit ignores."""

    profile: DamageProfile


ReactiveArmor = Literal["do_not_adapt", "damage_profile"] | ReactiveArmorProfile


class Environment(TypedDict, total=False):
    """Where the ship is."""

    damage_profile: DamageProfile
    security: Security
    reactive_armor: ReactiveArmor


class ProjectedBuff(TypedDict):
    """A buff, as `dbuffCollections` in the SDE numbers them."""

    id: int
    value: float


class _ProjectedEffectRequired(TypedDict):
    type_id: int
    effect_id: int


class ProjectedEffect(_ProjectedEffectRequired, total=False):
    """An effect aimed at a fit, like a stasis webifier."""

    attributes: dict[int, float]


class Projection(TypedDict, total=False):
    """What one fit hands to another."""

    buffs: list[ProjectedBuff]
    effects: list[ProjectedEffect]


class _FitRequired(TypedDict):
    ship: Ship
    items: list[FitItem]


class Fit(_FitRequired, total=False):
    """A ship, what is fitted to it, and the character flying it."""

    name: str | None
    character: Character
    environment: Environment
    incoming: Projection


class Options(TypedDict, total=False):
    """What `calculate` reports on top of the values."""

    sources: bool


class _NamedRef(TypedDict):
    type: Literal["ship", "mode", "character"]


class _IndexedRef(TypedDict):
    type: Literal["item", "charge", "projected"]
    index: int


class _SkillRef(TypedDict):
    type: Literal["skill"]
    type_id: int


class _BuffRef(TypedDict):
    type: Literal["buff"]
    id: int


SourceRef = _NamedRef | _IndexedRef | _SkillRef | _BuffRef


# `from` is a keyword, which the class syntax cannot spell.
Source = TypedDict(
    "Source",
    {
        "from": SourceRef,
        "effect_id": int | None,
        "source_attribute_id": int | None,
        "operator": EffectOperator,
        "value": float,
        "quantity": int,
        "penalty": float | None,
        "applied": bool,
    },
)
"""One modifier on an attribute, and where it came from."""


class _AttributeValueRequired(TypedDict):
    base: float
    value: float


class AttributeValue(_AttributeValueRequired, total=False):
    """One attribute, before and after the effects on it. `sources` is only
    reported when the `sources` option is set."""

    sources: list[Source]


class ItemResult(TypedDict):
    """The calculated attributes of the ship, its mode, the character, or one
    item."""

    attributes: dict[int, AttributeValue]
    state: State
    max_state: State
    charge: "ItemResult | None"


class _CalculationRequired(TypedDict):
    ship: ItemResult
    items: list[ItemResult]
    character: ItemResult


class Calculation(_CalculationRequired, total=False):
    """The result of `calculate`."""

    mode: ItemResult
    buffs: list[ProjectedBuff]
    outgoing: Projection
