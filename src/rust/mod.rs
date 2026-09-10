mod info;
mod output;
mod protobuf;

pub use info::{InfoMain, InfoNameMain};
pub use output::Output;
pub use protobuf::Data;

pub mod esf_data {
    include!(concat!(env!("OUT_DIR"), "/esf.rs"));
}
