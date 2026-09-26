"""The shapes `calculate` and the formats read and return.

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
    "EsiFitting",
    "EsiFittingItem",
    "Fit",
    "FitItem",
    "GroupLimit",
    "ItemResult",
    "Mutation",
    "Options",
    "ProjectedBuff",
    "ProjectedEffect",
    "Projection",
    "ReactiveArmor",
    "Resource",
    "Rule",
    "Security",
    "Ship",
    "Slot",
    "SlotKind",
    "Source",
    "SourceRef",
    "Spool",
    "State",
    "Target",
    "Violation",
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


class EsiFittingItem(TypedDict):
    """An item of an ESI fitting; `flag` is where it is, like `HiSlot0`,
    `DroneBay` or `Cargo`."""

    flag: str
    quantity: int
    type_id: int


class _EsiFittingRequired(TypedDict):
    name: str
    description: str
    ship_type_id: int
    items: list[EsiFittingItem]


class EsiFitting(_EsiFittingRequired, total=False):
    """A fitting as ESI saves it for a character. Only a fitting ESI returns
    has a `fitting_id`."""

    fitting_id: int


class Options(TypedDict, total=False):
    """What `calculate` reports on top of the values."""

    sources: bool
    validate: bool


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
    violations: list["Violation"]


Resource = Literal[
    "cpu",
    "powergrid",
    "calibration",
    "drone_bay",
    "drone_bandwidth",
    "launched_drones",
    "fighter_bay",
    "fighter_tubes",
    "light_fighter_tubes",
    "support_fighter_tubes",
    "heavy_fighter_tubes",
    "cargo_bay",
    "charge_capacity",
]

SlotKind = Literal[
    "high",
    "medium",
    "low",
    "rig",
    "subsystem",
    "service",
    "turret",
    "launcher",
]

GroupLimit = Literal["fitted", "online", "active"]


class _ShipTarget(TypedDict):
    type: Literal["ship"]


class _IndexedTarget(TypedDict):
    """An item of the fit, or the charge in it, by its place in `items`."""

    type: Literal["item", "charge"]
    index: int


Target = _ShipTarget | _IndexedTarget


class _PlainRule(TypedDict):
    """A rule that is broken or not, with nothing to say about by how much."""

    type: Literal[
        "slot_taken",
        "subsystem_taken",
        "ship_restricted",
        "capital_item",
        "charge_group",
    ]


class _ResourceRule(TypedDict):
    type: Literal["resource"]
    resource: Resource
    used: float
    available: float


class _SlotsRule(TypedDict):
    type: Literal["slots"]
    slot: SlotKind
    used: int
    available: int


class _WrongSlotRule(TypedDict):
    type: Literal["wrong_slot"]
    expected: SlotKind


class _WrongSlotIndexRule(TypedDict):
    type: Literal["wrong_slot_index"]
    expected: int


class _SkillRule(TypedDict):
    type: Literal["skill"]
    type_id: int
    required: int
    level: int


class _RigSizeRule(TypedDict):
    type: Literal["rig_size"]
    ship: int
    item: int


class _MaxGroupRule(TypedDict):
    type: Literal["max_group"]
    group_id: int
    limit: GroupLimit
    used: int
    allowed: int


class _MaxTypeRule(TypedDict):
    type: Literal["max_type"]
    type_id: int
    used: int
    allowed: int


class _ChargeSizeRule(TypedDict):
    type: Literal["charge_size"]
    module: int
    charge: int


Rule = (
    _PlainRule
    | _ResourceRule
    | _SlotsRule
    | _WrongSlotRule
    | _WrongSlotIndexRule
    | _SkillRule
    | _RigSizeRule
    | _MaxGroupRule
    | _MaxTypeRule
    | _ChargeSizeRule
)
"""A rule of EVE's, and the values that failed it.

What an item would accept instead is not repeated here; it is on the item
itself, as `chargeGroup1`, `canFitShipType1` and the like.
"""


class Violation(TypedDict):
    """One rule the fit breaks, and what breaks it."""

    target: Target
    rule: Rule
