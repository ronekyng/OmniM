pub mod gen {
    include!(concat!(env!("OUT_DIR"), "/omnim.rs"));
}

pub use gen::*;
