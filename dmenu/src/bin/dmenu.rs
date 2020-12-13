use dmenu_lib::config;
use dmenu_lib::drw;
use dmenu_lib::globals;
use dmenu_lib::result;
use dmenu_lib::VERSION;

use libc::{setlocale, LC_CTYPE};
#[cfg(target_os = "openbsd")]
use pledge;
use std::mem::MaybeUninit;
use std::ptr;
use x11::xlib::*;

use config::*;
use drw::Drw;
use globals::*;
use result::*;

use clap::{App, Arg};

fn main() {
    // just a wrapper to ensure a clean death in the event of error
    std::process::exit(match try_main() {
        Ok(_) => 0,
        Err(Die::Stdout(msg)) => {
            if msg.len() > 0 {
                println!("{}", msg)
            }
            0
        }
        Err(Die::Stderr(msg)) => {
            if msg.len() > 0 {
                eprintln!("{}", msg)
            }
            1
        }
    });
}

fn try_main() -> CompResult<()> {
    let pseudo_globals = PseudoGlobals::default();

    let app = create_app();
    let args = app.get_matches();

    match args.occurrences_of("version") {
        1 => {
            println!("dmenu-rs {}", VERSION);
            return Ok(());
        }
        2 => {
            // TODO plugins
            println!(
                "dmenu-rs {}\n\
                Compiled with rustc {}\n\
                Compiled without plugins",
                VERSION,
                rustc_version_runtime::version()
            );

            return Ok(());
        }
        _ => {}
    }

    let config = Config::from_args(args)?;

    // SAFETY: FFI call
    unsafe {
        if setlocale(LC_CTYPE, ptr::null()) == ptr::null_mut() || XSupportsLocale() == 0 {
            return Die::stderr("warning: no locale support".to_owned());
        }
    }
    // SAFETY: FFI call
    let dpy = unsafe { XOpenDisplay(ptr::null_mut()) };
    if dpy == ptr::null_mut() {
        return Die::stderr("cannot open display".to_owned());
    }
    // SAFETY: FFI call
    let screen = unsafe { XDefaultScreen(dpy) };
    // SAFETY: FFI call
    let root = unsafe { XRootWindow(dpy, screen) };
    let parentwin = root.max(config.embed);

    // SAFETY: `XGetWindowAttributes` should initialize everything
    let wa = unsafe {
        let mut wa = MaybeUninit::uninit();
        XGetWindowAttributes(dpy, parentwin, wa.as_mut_ptr());
        wa.assume_init()
    };

    let mut drw = Drw::new(dpy, screen, root, wa, pseudo_globals, config)?;
    if cfg!(target_os = "openbsd") {
        pledge::pledge("stdio rpath", None)
            .map_err(|_| Die::Stderr("Could not pledge".to_owned()))?;
    }

    drw.setup(parentwin, root)?;
    drw.run()
}

fn create_app() -> App<'static, 'static> {
    App::new("dmenu")
        .version(VERSION)
        .author("Shizcow <pohl.devin@gmail.com>")
        .about("dynamic menu")
        .arg(
            Arg::with_name("version")
                .help("Prints version and build information. Specify twice for additional info.")
                .long("version")
                .short("v")
                .multiple(true),
        )
        .arg(
            Arg::with_name("bottom")
                .help("Places menu at bottom of the screen")
                .long("bottom")
                .short("b")
        )
        .arg(
            Arg::with_name("fast")
                .help("Grabs keyboard before reading stdin")
                .long("fast")
                .short("f")
        )
        .arg(
            Arg::with_name("insensitive")
                .help("Case insensitive item matching")
                .long("insensitive")
                .short("i")
        )
        .arg(
            Arg::with_name("lines")
                .help("Number of vertical listing lines")
                .long("lines")
                .short("l")
                .takes_value(true)
                .value_name("LINES")
        )
        .arg(
            Arg::with_name("monitor")
                .help("X monitor to display on")
                .long("monitor")
                .short("m")
                .takes_value(true)
                .value_name("MONITOR")
        )
        .arg(
            Arg::with_name("prompt")
                .help("Display a prompt")
                .long("prompt")
                .short("p")
                .takes_value(true)
                .value_name("PROMPT")
        )
        .arg(
            Arg::with_name("font")
                .help("Add menu font")
                .long("font")
                // TODO: multiline string
                .long_help( "Add menu font. Can be specified multiple times to give fallback fonts. For example, --font Terminus --font 'Font Awesome' would draw everything with Terminus, and fall back to Font Awesome for symbols.\nIf a glyph is not found in any of the supplied fonts, it will be provided by the default font (:mono). \nIf a glyph is not found in any fonts, it will render as the no-character box.\n [aliases: --fn]")
                .multiple(true)
                .takes_value(true)
                .value_name("FONT")
        )
        .arg(
            Arg::with_name("color_normal_background")
                .help("Normal Background Color")
                .long("nb")
                .takes_value(true)
                .value_name("COLOR")
        )
        .arg(
            Arg::with_name("color_normal_foreground")
                .help("Normal Foreground Color")
                .long("nf")
                .takes_value(true)
                .value_name("COLOR")
        )
        .arg(
            Arg::with_name("color_selected_background")
                .help("Selected Background Color")
                .long("sb")
                .takes_value(true)
                .value_name("COLOR")
        )
        .arg(
            Arg::with_name("color_selected_foreground")
                .help("Selected Foreground Color")
                .long("sf")
                .takes_value(true)
                .value_name("COLOR")
        )
        .arg(
            Arg::with_name("window")
                .help("Embed into window ID")
                .short("w")
                .long("window")
                .takes_value(true)
                .value_name("ID")
        )
        .arg(
            Arg::with_name("render_minheight")
                .help("Minimum menu height")
                .long("render_minheight")
                .long_help("Minimum menu draw height. Normally, the menu height is decided by the font size, however this option overrides that. Useful for getting things aligned with elements always on screen, such as i3 statusbar.")
                .takes_value(true)
                .value_name("PIXELS")
        )
        .arg(
            Arg::with_name("render_overrun")
                .help("Draw behavior of input box. If specified will draw input over the top of items when input exceeds the width of input box")
                .long("render_overrun")
        )
        .arg(
            Arg::with_name("render_flex")
                .help("Draw behavior of input box. If specified will expand input box when input exceeds the width of input box, gracefully moving items out of the way")
                .long("render_flex")
        )
        .arg(
            Arg::with_name("render_default_width")
                // TODO: move to long help
                .help("Default size of input box. Options are:\n  min          - input box remains as small as possible\n  items        - same size as the largest menu item (default)\n                 yields the most static layout\n  max          - only one menu item at a time is displayed, right aligned\n  custom=WIDTH - fixed width, percentage of total menu width\n                 ranges from 0 (min) to 100 (max)\n")
                .long("render_default_width")
                .takes_value(true)
                .value_name("DEFAULT_WIDTH")
        )
        .arg(
            Arg::with_name("nostdin")
                // TODO: move to long help
                .help("Do not read from stdin. Probably not useful unless compiled with plugins")
                .long("nostdin")
        )

    // TODO: add plugin stuff
}
