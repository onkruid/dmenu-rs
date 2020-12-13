use libc::{c_int, c_uint};
use std::mem::MaybeUninit;
use x11::xlib::Window;

pub enum Schemes {
    SchemeNorm,
    SchemeSel,
    SchemeOut,
    SchemeLast,
}
pub enum Clrs {
    ColFg,
    ColBg,
}
pub use Clrs::*;
pub use Schemes::*;

#[derive(Debug, PartialEq)]
pub enum DefaultWidth {
    Min,
    Items,
    Max,
    Custom(u8),
}

#[derive(Debug)]
pub struct Config {
    pub lines: c_uint,
    pub topbar: bool,
    pub prompt: String,
    pub promptw: c_int,
    pub fontstrings: Vec<String>,
    pub fast: bool,
    pub embed: Window,
    pub case_sensitive: bool,
    pub mon: c_int,
    pub colors: [[[u8; 8]; 2]; SchemeLast as usize],
    pub render_minheight: u32,
    pub render_overrun: bool,
    pub render_flex: bool,
    pub render_rightalign: bool,
    pub render_default_width: DefaultWidth,
    pub nostdin: bool,
}

impl Default for Config {
    fn default() -> Self {
        // TODO: remove unsafe? (this is not safe)
        // TODO: does is have to be this weird value?
        let promptw = unsafe { MaybeUninit::uninit().assume_init() };

        // [fg, bg]
        let mut colors = [[[0; 8]; 2]; SchemeLast as usize];
        colors[SchemeNorm as usize] = [*b"#bbbbbb\0", *b"#222222\0"];
        colors[SchemeSel as usize] = [*b"#eeeeee\0", *b"#005577\0"];
        colors[SchemeOut as usize] = [*b"#000000\0", *b"#00ffff\0"];

        Self {
            lines: 0,
            topbar: true,
            prompt: Default::default(),
            promptw,
            fontstrings: vec!["mono:size=10".to_owned()],
            fast: false,
            embed: 0,
            case_sensitive: true,
            mon: -1,
            colors,
            render_minheight: 4,
            render_overrun: false,
            render_flex: false,
            render_rightalign: false,
            render_default_width: DefaultWidth::Items,
            nostdin: false,
        }
    }
}
