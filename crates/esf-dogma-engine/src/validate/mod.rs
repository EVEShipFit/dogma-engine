//! Checks a calculated fit against EVE's fitting rules.

use serde::Serialize;

use esf_data::{Info, eve};

use crate::calculate::{Calculation, ItemResult};
use crate::fit::{Fit, FitItem, Slot};

mod charge;
mod item;
mod resource;
mod skill;
mod slot;

/// One rule the fit breaks, and what breaks it.
#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct Violation {
    /// What the rule is about.
    pub target: Target,
    /// The rule, and the values that failed it.
    pub rule: Rule,
}

/// What a [`Violation`] is about. `Item` and `Charge` index into `Fit::items`.
#[derive(Serialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Target {
    /// The ship, for what it carries as a whole.
    Ship,
    /// An item of the fit.
    Item {
        /// The position in `Fit::items`.
        index: usize,
    },
    /// The charge in an item of the fit.
    Charge {
        /// The position in `Fit::items` of the item holding the charge.
        index: usize,
    },
}

/// A rule of EVE's, and the values that failed it.
///
/// What an item would accept instead is not repeated here; it is on the item
/// itself, as `chargeGroup1`, `canFitShipType1` and the like.
#[non_exhaustive]
#[derive(Serialize, Debug, Clone, Copy, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Rule {
    /// More of a fitting resource used than the ship has.
    Resource {
        /// Which resource ran out.
        resource: Resource,
        /// How much is used.
        used: f64,
        /// How much there is.
        available: f64,
    },
    /// More items in a rack than the ship has slots.
    Slots {
        /// Which rack is full.
        slot: SlotKind,
        /// How many items are in it.
        used: u32,
        /// How many fit.
        available: u32,
    },
    /// The item belongs in another kind of slot.
    WrongSlot {
        /// The rack the item asks for.
        expected: SlotKind,
    },
    /// Another item of the fit is in this slot too.
    SlotTaken,
    /// An implant or booster in a slot other than the one it occupies. The
    /// number is `implantness` or `boosterness`.
    WrongSlotIndex {
        /// The slot the item belongs in.
        expected: u16,
    },
    /// Another subsystem covers the same part of the ship.
    SubsystemTaken,
    /// A skill the character is missing, or has not trained far enough.
    Skill {
        /// The type id of the skill.
        type_id: i32,
        /// The level the item asks for.
        required: u8,
        /// The level the character has; 0 when untrained.
        level: u8,
    },
    /// A rig of another size than the ship takes.
    RigSize {
        /// The size the ship takes.
        ship: u8,
        /// The size of the rig.
        item: u8,
    },
    /// The item cannot go on this ship at all.
    ShipRestricted,
    /// A capital item on a ship that is not a capital.
    CapitalItem,
    /// More of a group than the ship may hold in that state.
    MaxGroup {
        /// The group the limit is on.
        group_id: i32,
        /// The state the limit counts.
        limit: GroupLimit,
        /// How many are in that state.
        used: u32,
        /// How many may be.
        allowed: u32,
    },
    /// More of one type fitted than allowed.
    MaxType {
        /// The type the limit is on.
        type_id: i32,
        /// How many are fitted.
        used: u32,
        /// How many may be.
        allowed: u32,
    },
    /// A charge of a group the module does not take.
    ChargeGroup,
    /// A charge of another size than the module takes.
    ChargeSize {
        /// The size the module takes.
        module: u8,
        /// The size of the charge.
        charge: u8,
    },
}

/// A resource the ship only has so much of.
#[derive(Serialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Resource {
    /// CPU.
    Cpu,
    /// Powergrid.
    Powergrid,
    /// Calibration, which the rigs take up.
    Calibration,
    /// Room in the drone bay.
    DroneBay,
    /// Bandwidth for the drones in space.
    DroneBandwidth,
    /// Drones in space at once.
    LaunchedDrones,
    /// Room in the fighter bay.
    FighterBay,
    /// Fighter tubes.
    FighterTubes,
    /// Tubes that take a light squadron.
    LightFighterTubes,
    /// Tubes that take a support squadron.
    SupportFighterTubes,
    /// Tubes that take a heavy squadron.
    HeavyFighterTubes,
    /// Room in the cargo hold.
    CargoBay,
    /// Room in the module for the charge it holds.
    ChargeCapacity,
}

/// A rack of slots, or a hardpoint a weapon needs.
#[derive(Serialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SlotKind {
    /// A high slot.
    High,
    /// A medium slot.
    Medium,
    /// A low slot.
    Low,
    /// A rig slot.
    Rig,
    /// A subsystem slot.
    Subsystem,
    /// A service slot, on a structure.
    Service,
    /// A turret hardpoint.
    Turret,
    /// A launcher hardpoint.
    Launcher,
}

/// Which state a group limit counts.
#[derive(Serialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GroupLimit {
    /// Fitted at all.
    Fitted,
    /// Online or higher.
    Online,
    /// Active or higher.
    Active,
}

/// Report every rule the fit breaks.
///
/// The calculation has to be the one [`calculate()`](crate::calculate) made
/// for this fit: a rule reads the values after skills and modules changed
/// them, so a gun's powergrid is the one its owner really pays.
///
/// Violations come out grouped by the kind of rule, and within a kind in the
/// order of `Fit::items`. An empty result means the fit breaks nothing.
pub fn validate(info: &impl Info, fit: &Fit, calculation: &Calculation) -> Vec<Violation> {
    let context = Context::new(info, fit, calculation);

    let mut found = Vec::new();
    resource::validate(&context, &mut found);
    slot::validate(&context, &mut found);
    item::validate(&context, &mut found);
    charge::validate(&context, &mut found);
    skill::validate(&context, &mut found);
    found
}

/// The fit and its calculation side by side, with what the SDE says about
/// each item gathered once rather than per rule.
struct Context<'a, I> {
    info: &'a I,
    fit: &'a Fit,
    calculation: &'a Calculation,
    items: Vec<Item<'a>>,
}

/// One item of the fit: what was asked for, what it calculated to, and what
/// only the SDE knows about it.
struct Item<'a> {
    index: usize,
    fit: &'a FitItem,
    result: &'a ItemResult,
    group_id: i32,
    rack: Option<SlotKind>,
    hardpoint: Option<SlotKind>,
}

impl<'a, I: Info> Context<'a, I> {
    fn new(info: &'a I, fit: &'a Fit, calculation: &'a Calculation) -> Context<'a, I> {
        let items = fit
            .items
            .iter()
            .zip(&calculation.items)
            .enumerate()
            .map(|(index, (fit_item, result))| {
                let (rack, hardpoint) = slots_of(info, fit_item.type_id);
                Item {
                    index,
                    fit: fit_item,
                    result,
                    group_id: info
                        .get_type(fit_item.type_id)
                        .map_or(0, |r#type| r#type.group_id()),
                    rack,
                    hardpoint,
                }
            })
            .collect();

        Context {
            info,
            fit,
            calculation,
            items,
        }
    }

    fn attribute_id(&self, name: &str) -> Option<i32> {
        self.info.attribute_name_to_id(name)
    }

    /// What the attribute reached, or `None` when the item does not carry it.
    fn value(&self, result: &ItemResult, attribute_id: i32) -> Option<f64> {
        result
            .attributes
            .get(&attribute_id)
            .map(|attribute| attribute.value)
    }

    /// What the attribute reached, reading one the item does not carry as
    /// zero. That is what a resource nothing uses adds up to.
    fn amount(&self, result: &ItemResult, attribute_id: i32) -> f64 {
        self.value(result, attribute_id).unwrap_or(0.0)
    }

    fn ship(&self) -> &ItemResult {
        &self.calculation.ship
    }

    fn ship_group_id(&self) -> i32 {
        self.ship_type().map_or(0, |r#type| r#type.group_id())
    }

    fn ship_category_id(&self) -> i32 {
        self.ship_type().map_or(0, |r#type| r#type.category_id())
    }

    fn ship_type(&self) -> Option<eve::Type<'_>> {
        self.info.get_type(self.fit.ship.type_id)
    }
}

/// The slot effects EVE marks an item with. An item carries at most one of
/// each kind.
fn slots_of(info: &impl Info, type_id: i32) -> (Option<SlotKind>, Option<SlotKind>) {
    let mut rack = None;
    let mut hardpoint = None;

    for type_effect in info.get_dogma_effects(type_id).into_iter().flatten() {
        let Some(effect) = info.get_dogma_effect(type_effect.effect_id()) else {
            continue;
        };
        match effect.name() {
            "hiPower" => rack = Some(SlotKind::High),
            "medPower" => rack = Some(SlotKind::Medium),
            "loPower" => rack = Some(SlotKind::Low),
            "rigSlot" => rack = Some(SlotKind::Rig),
            "subSystem" => rack = Some(SlotKind::Subsystem),
            "serviceSlot" => rack = Some(SlotKind::Service),
            "turretFitted" => hardpoint = Some(SlotKind::Turret),
            "launcherFitted" => hardpoint = Some(SlotKind::Launcher),
            _ => {}
        }
    }

    (rack, hardpoint)
}

impl Item<'_> {
    /// Which rack the fit put it in.
    fn rack(&self) -> Option<SlotKind> {
        match self.fit.slot {
            Slot::High(_) => Some(SlotKind::High),
            Slot::Medium(_) => Some(SlotKind::Medium),
            Slot::Low(_) => Some(SlotKind::Low),
            Slot::Rig(_) => Some(SlotKind::Rig),
            Slot::Subsystem(_) => Some(SlotKind::Subsystem),
            Slot::Service(_) => Some(SlotKind::Service),
            _ => None,
        }
    }

    /// Whether it is fitted to the ship, rather than carried by it or by the
    /// character.
    fn is_fitted(&self) -> bool {
        self.rack().is_some()
    }

    /// Whether the character has to be able to use it. Cargo is only hauled.
    fn is_used(&self) -> bool {
        self.fit.slot != Slot::Cargo
    }

    fn violation(&self, rule: Rule) -> Violation {
        Violation {
            target: Target::Item { index: self.index },
            rule,
        }
    }
}
