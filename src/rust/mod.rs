mod info;
mod protobuf;

pub use info::{InfoMain, InfoNameMain};
pub use protobuf::Data;

pub mod esf_data {
    include!(concat!(env!("OUT_DIR"), "/esf.rs"));
}
