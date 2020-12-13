pub const VERSION: &'static str = include_str!("../VERSION");
#[allow(unused)] // TODO: remove
pub(crate) const PLUGINS: &'static str = "";


pub mod result;
pub mod drw;
pub mod util;
pub mod fnt;
pub mod config;
pub mod globals;
pub mod init;
pub mod item;
pub mod run;
pub mod setup;
