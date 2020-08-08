//use crate::item::{Items, Direction::*};
//use crate::globals::*;
//use crate::config::*;
//use crate::fnt::*;
use crate::result::*;
use crate::init::*;

pub struct Drw {
    xkb_state: xkbcommon::xkb::State,
    conn: xcb::Connection,
    cr: cairo::Context,
    layout: pango::Layout,

    w: u16,
    h: u16,
}

// TODO: automate
const FONT:  &str = "mono 30";
const HEIGHT: u16 = 180;

impl Drw {
    pub fn new(/*pseudo_globals: PseudoGlobals, config: Config*/) -> CompResult<Self> {
	// get a size hint for menu height
	let font = pango::FontDescription::from_string(FONT);
	let text_height = font.get_size() / pango::SCALE;
	if text_height <= 0 {
	    return Die::stderr(format!("Pango failed to parse font '{}': zero size. Try specifying a font size.", FONT));
	}
	// set up connection to X server
	let (conn, screen_num) = xcb::Connection::connect(None).unwrap();
	// init xinerama -- used later
	init_xinerama(&conn);
	// create window -- height is calculated later
	let (screen, window, (w, h)) = create_xcb_window(&conn, screen_num, ((text_height as f32) * 1.5) as u16);
	// grab keyboard
	let xkb_state = setup_xkb(&conn, window);
	// set up cairo
	let cr = create_cairo_context(&conn, &screen, &window, w.into(), h.into());
	// set up pango
	let layout = create_pango_layout(&cr, FONT);
	
	/*ret.items = if ret.config.nostdin {
	grabkeyboard(ret.dpy, ret.config.embed)?;
	Some(Items::new(Vec::new()))
    } else {Some(Items::new(
	if ret.config.fast && isatty(0) == 0 {
	grabkeyboard(ret.dpy, ret.config.embed)?;
	readstdin(&mut ret)?
    } else {
	let tmp = readstdin(&mut ret)?;
	grabkeyboard(ret.dpy, ret.config.embed)?;
	tmp
    }))
    };

	ret.config.lines = ret.config.lines.min(ret.get_items().len() as u32);
	 */

	let event = conn.wait_for_event();
        if let Some(event) = event {
            let r = event.response_type() & !0x80;
            match r {
                xcb::EXPOSE => {
		    return Ok(Self{xkb_state, conn, layout, cr, w, h});
		}
		_ => {}
	    }
	}
	Die::stderr("xcb could not spawn".to_owned())
    }
    pub fn draw(&self) -> CompResult<()> {

	let norm = [parse_color("#bbb"), parse_color("#222")];
	let sel  = [parse_color("#eee"), parse_color("#057")];
	
	self.cr.set_source_rgb(norm[1][0], norm[1][1], norm[1][2]);
        self.cr.paint();

	// red triangle
        self.cr.set_source_rgb(sel[1][0], sel[1][1], sel[1][2]);
        self.cr.rectangle(100.0, 0.0, 50.0, self.h.into());
        self.cr.fill();

	/*
	// get ready to draw text
	self.layout.set_text("hello world");
	// get a size hint for allignment
	let (mut text_width, mut text_height) = self.layout.get_size();
	// If the text is too wide, ellipsize to fit
	if text_width > self.w as i32*pango::SCALE {
	    text_width = self.w as i32*pango::SCALE;
	    self.layout.set_ellipsize(pango::EllipsizeMode::End);
	    self.layout.set_width(text_width);
	}
	// scale back -- pango doesn't uses large integers instead of floats
	text_width /= pango::SCALE;
	text_height /= pango::SCALE;
	// base text color (does not apply to color bitmap chars)
	self.cr.set_source_rgb(1.0, 1.0, 1.0);
	// place and draw text
	self.cr.move_to((self.w as i32 -text_width) as f64/2.0,
			(self.h as i32-text_height) as f64/2.0);
	pangocairo::show_layout(&self.cr, &self.layout);
	 */

	// wait for everything to finish drawing before moving on
	self.conn.flush();
	Ok(())
    }
}


fn parse_color(s: &str) -> [f64; 3] {
    let c: css_color_parser::Color = s.parse().unwrap();
    [(c.r as f64)/255.0, (c.g as f64)/255.0, (c.b as f64)/255.0]
}
