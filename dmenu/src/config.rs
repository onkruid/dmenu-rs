use libc::{c_int, c_uint};
use regex::RegexBuilder;
use x11::xlib::Window;

use crate::result::*;

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
    // TODO: move these to clap args?
    fn default() -> Self {
        // [fg, bg]
        let mut colors = [[[0; 8]; 2]; SchemeLast as usize];
        colors[SchemeNorm as usize] = [*b"#bbbbbb\0", *b"#222222\0"];
        colors[SchemeSel as usize] = [*b"#eeeeee\0", *b"#005577\0"];
        colors[SchemeOut as usize] = [*b"#000000\0", *b"#00ffff\0"];

        Self {
            lines: 0,
            topbar: true,
            prompt: Default::default(),
            promptw: 0,
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

impl Config {
    pub fn from_args(args: clap::ArgMatches) -> CompResult<Self> {
        let mut config = Self::default();

        // let color_regex = RegexBuilder::new("^#([A-Fa-f0-9]{6}|[A-Fa-f0-9]{3})\0$")
        //     .case_insensitive(true)
        //     .build()
        //     .expect("Could not build color regex");

        let color_regex = RegexBuilder::new("^#([0-9a-f]{6}|[0-9a-f]{3})$")
            .case_insensitive(true)
            .build()
            .expect("Could not build color regex");

        // bottom
        if args.is_present("topbar") {
            config.topbar = false;
        }

        // fast
        if args.is_present("fast") {
            config.fast = true;
        }

        // insensitive
        if args.is_present("insensitive") {
            config.case_sensitive = false;
        }

        // lines
        if let Some(lines) = args.value_of("lines") {
            // TODO: fix error handling
            config.lines = lines
                .parse()
                .map_err(|_| Die::Stderr("-l: Lines must be a non-negative integer".to_owned()))?;
        }

        // monitor
        if let Some(monitor) = args.value_of("monitor") {
            config.mon = monitor.parse().map_err(|_| {
                Die::Stderr("-m: Monitor must be a non-negaitve integer".to_owned())
            })?;
        }

        // prompt
        if let Some(prompt) = args.value_of("prompt") {
            // TODO: remove alloc
            config.prompt = prompt.to_string();
        }

        // font
        if let Some(fonts) = args.values_of("font") {
            let default = config.fontstrings.pop().unwrap();
            // TODO: remove alloc
            config.fontstrings = fonts.map(|f| f.to_string()).collect();
            config.fontstrings.push(default);
        }

        // color_normal_background
        if let Some(color) = args.value_of("color_normal_background") {
            if color_regex.is_match(&color) {
                let mut color = color.to_string();
                color.push('\0');
                config.colors[SchemeNorm as usize][ColBg as usize].copy_from_slice(color.as_bytes());
            } else {
                Die::stderr(
                    "--nb: Color must be in hex format (#123456 or #123)".to_owned(),
                )?
            }
        }

        // color_normal_foreground
        if let Some(color) = args.value_of("color_normal_foreground") {
            if color_regex.is_match(&color) {
                let mut color = color.to_string();
                color.push('\0');
                config.colors[SchemeNorm as usize][ColFg as usize].copy_from_slice(color.as_bytes());
            } else {
                Die::stderr(
                    "--nf: Color must be in hex format (#123456 or #123)".to_owned(),
                )?
            }
        }

        // color_selected_background
        if let Some(color) = args.value_of("color_selected_background") {
            if color_regex.is_match(&color) {
                let mut color = color.to_string();
                color.push('\0');
            config.colors[SchemeSel as usize][ColBg as usize].copy_from_slice(color.as_bytes());
            } else {
                Die::stderr(
                "--sb: Color must be in hex format (#123456 or #123)".to_owned(),
                )?
            }
        }

        // color_selected_foreground
        if let Some(color) = args.value_of("color_selected_foreground") {
            if color_regex.is_match(&color) {
                let mut color = color.to_string();
                color.push('\0');
            config.colors[SchemeSel as usize][ColFg as usize].copy_from_slice(color.as_bytes());
            } else {
                Die::stderr(
                "--sf: Color must be in hex format (#123456 or #123)".to_owned(),
                )?
            }
        }

        // window
        if let Some(window) = args.value_of("window") {
            config.embed = window.parse().map_err(|_| {
                Die::Stderr("-w: Window ID must be a valid X window ID string".to_owned())
            })?;
        }

        // nostdin
        if args.is_present("nostdin") {
            config.nostdin = true;
        }

        // render_minheight
        if let Some(minheight) = args.value_of("render_minheight") {
            config.render_minheight = minheight.parse().map_err(|_| {
                Die::Stderr(
                    "--render_minheight: Height must be an integet number of \
				  pixels"
                        .to_owned(),
                )
            })?;
        }

        // render_overrun
        if args.is_present("render_overrun") {
            config.render_overrun = true;
            config.render_flex = true;
        }

        // render_flex
        if args.is_present("render_flex") {
            config.render_flex = true;
        }

        // render_rightalign
        if args.is_present("render_rightalign") {
            config.render_rightalign = true;
        }

        // render_default_width
        if let Some(arg) = args.value_of("render_default_width") {
            if !arg.contains("=") {
                config.render_default_width = match arg {
                    "min" => DefaultWidth::Min,
                    "items" => DefaultWidth::Items,
                    "max" => {
                        config.render_rightalign = true;
                        DefaultWidth::Max
                    }
                    _ => return Die::stderr("--render_default_width: invalid argument".to_owned()),
                }
            } else {
                let vec: Vec<&str> = arg.split("=").collect();
                if vec.len() != 2 || (vec.len() > 0 && vec[0] != "custom") {
                    return Die::stderr(
                        "Incorrect format for --render_default_width, \
				    see help for details"
                            .to_owned(),
                    );
                }
                let width = vec[1].parse::<u8>();
                if width.is_err() || *width.as_ref().unwrap() > 100 {
                    return Die::stderr(
                        "--render_default_width: custom width \
				      must be a positive integer"
                            .to_owned(),
                    );
                }
                config.render_default_width = DefaultWidth::Custom(width.unwrap());
            }
        }

        Ok(config)
    }
}
