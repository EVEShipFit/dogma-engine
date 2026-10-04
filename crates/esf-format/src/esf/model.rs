use std::fmt;

/// A fit in canonical esf/1 form, with SDE ids in place of names.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct EsfFit {
    /// The ship, structure or container; `None` for a fit without a ship.
    pub hull: Option<i32>,
    /// The fit name.
    pub name: Option<String>,
    /// The tactical mode.
    pub mode: Option<i32>,
    /// Every line after the hull line, in canonical order.
    pub entries: Vec<Entry>,
}

/// A line of a fit.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Entry {
    /// The type; `None` for an empty slot.
    pub type_id: Option<i32>,
    /// How many, when not one.
    pub count: Option<u32>,
    /// The fit this line refers to.
    pub fit_name: Option<String>,
    /// The charge loaded.
    pub charge: Option<i32>,
    /// How many charges are loaded, when not a full load.
    pub charge_count: Option<u32>,
    /// The mutaplasmid applied.
    pub mutaplasmid: Option<i32>,
    /// Attribute values replacing the type's own, by attribute id.
    pub overrides: Vec<(i32, f64)>,
    /// The state, when not the default.
    pub state: Option<State>,
    /// The location, when not the default placement.
    pub location: Option<Location>,
}

/// The state of an item.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum State {
    /// Offline.
    Off,
    /// Online, but not running.
    On,
    /// Overloaded.
    Heat,
}

impl State {
    pub(super) const ALL: [State; 3] = [State::Off, State::On, State::Heat];

    pub(super) fn name(self) -> &'static str {
        match self {
            State::Off => "off",
            State::On => "on",
            State::Heat => "heat",
        }
    }

    pub(super) fn from_name(name: &str) -> Option<State> {
        State::ALL.into_iter().find(|state| state.name() == name)
    }

    pub(super) fn number(self) -> u64 {
        match self {
            State::Off => 1,
            State::On => 2,
            State::Heat => 3,
        }
    }
}

macro_rules! locations {
    ($($variant:ident = $number:literal, $name:literal, $doc:literal;)*) => {
        /// A rack, the cargo, the drone or fighter bay, or another hold.
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
        pub enum Location {
            $(#[doc = $doc] $variant,)*
        }

        impl Location {
            pub(super) const ALL: &[Location] = &[$(Location::$variant,)*];

            pub(super) fn name(self) -> &'static str {
                match self {
                    $(Location::$variant => $name,)*
                }
            }

            pub(super) fn number(self) -> u64 {
                match self {
                    $(Location::$variant => $number,)*
                }
            }
        }
    };
}

locations! {
    High = 1, "high", "The high slots.";
    Mid = 2, "mid", "The medium slots.";
    Low = 3, "low", "The low slots.";
    Rig = 4, "rig", "The rig slots.";
    Sub = 5, "sub", "The subsystem slots.";
    Svc = 6, "svc", "The service slots.";
    Cargo = 7, "cargo", "The cargo hold.";
    Bay = 8, "bay", "The drone or fighter bay.";
    Ammo = 9, "ammo", "The ammo hold.";
    Booster = 10, "booster", "The booster hold.";
    Command = 11, "command", "The command center hold.";
    Corpse = 12, "corpse", "The corpse hold.";
    Depot = 13, "depot", "The mobile depot hold.";
    Expedition = 14, "expedition", "The expedition hold.";
    Fleet = 15, "fleet", "The fleet hangar.";
    Frigate = 16, "frigate", "The frigate escape bay.";
    Fuel = 17, "fuel", "The fuel bay.";
    Gas = 18, "gas", "The gas hold.";
    Ice = 19, "ice", "The ice hold.";
    Infrastructure = 20, "infrastructure", "The infrastructure hold.";
    Maintenance = 21, "maintenance", "The ship maintenance bay.";
    Mineral = 22, "mineral", "The mineral hold.";
    Mining = 23, "mining", "The mining hold.";
    Moon = 24, "moon", "The moon material output bay.";
    Planetary = 25, "planetary", "The planetary commodities hold.";
    Quafe = 26, "quafe", "The quafe hold.";
    Subsystem = 27, "subsystem", "The subsystem hold.";
}

impl Location {
    pub(super) fn from_name(name: &str) -> Option<Location> {
        Location::ALL
            .iter()
            .copied()
            .find(|location| location.name() == name)
    }

    pub(super) fn from_number(number: u64) -> Option<Location> {
        Location::ALL
            .iter()
            .copied()
            .find(|location| location.number() == number)
    }

    /// Whether this is a rack of slots.
    pub fn is_rack(self) -> bool {
        self.number() <= Location::Svc.number()
    }
}

/// Why an esf/1 document could not be read or written.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Error {
    /// The line of the text the error is on, counting from 1.
    pub line: Option<usize>,
    /// What is wrong.
    pub message: String,
}

impl Error {
    pub(super) fn new(message: impl Into<String>) -> Error {
        Error {
            line: None,
            message: message.into(),
        }
    }

    pub(super) fn at(line: usize, message: impl Into<String>) -> Error {
        Error {
            line: Some(line),
            message: message.into(),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.line {
            Some(line) => write!(f, "line {line}: {}", self.message),
            None => write!(f, "{}", self.message),
        }
    }
}

impl std::error::Error for Error {}
