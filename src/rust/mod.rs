mod info;
mod protobuf;

pub use info::InfoMain;

pub mod esf_data {
    include!(concat!(env!("OUT_DIR"), "/esf.rs"));
}
