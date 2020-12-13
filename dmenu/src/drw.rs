use dmenu_extra_bindings::fontconfig::{FcFalse, FcTrue, FC_CHARSET, FC_COLOR, FC_SCALABLE};
use fontconfig::fontconfig::{
    FcCharSetAddChar, FcCharSetCreate, FcCharSetDestroy, FcConfigSubstitute, FcMatchPattern,
    FcPatternAddBool, FcPatternAddCharSet, FcPatternDestroy, FcPatternDuplicate,
};
use itertools::Itertools;
use libc::{c_int, c_uchar, c_uint, c_void, free};
use std::{mem::MaybeUninit, ptr};
use unicode_segmentation::UnicodeSegmentation;
use x11::xft::{
    FcPattern, XftCharExists, XftColor, XftDraw, XftDrawCreate, XftDrawDestroy, XftDrawStringUtf8,
    XftFontMatch, XftTextExtentsUtf8,
};
use x11::xlib::{
    AnyKey, AnyModifier, Display, Drawable, False, Window, XCloseDisplay, XCopyArea,
    XDefaultColormap, XDefaultVisual, XDrawRectangle, XFillRectangle, XFreeGC, XFreePixmap,
    XSetForeground, XSync, XUngrabKey, XWindowAttributes, GC,
};
use x11::xrender::XGlyphInfo;

#[cfg(feature = "fuzzy")]
use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};

use crate::item::{Item, MatchCode};
use crate::result::*;

use regex::RegexBuilder;

use crate::config::Schemes::*;

use crate::config::*;
use crate::fnt::*;
use crate::globals::*;
use crate::item::{Direction::*, Items};

#[derive(PartialEq, Debug)]
pub enum TextOption<'a> {
    Prompt,
    Input,
    Other(&'a String),
}
use TextOption::*;

#[derive(Debug)]
pub struct Drw {
    pub wa: XWindowAttributes,
    pub dpy: *mut Display,
    pub screen: c_int,
    pub root: Window,
    pub drawable: Drawable,
    pub gc: GC,
    pub scheme: [*mut XftColor; 2],
    pub fonts: Vec<Fnt>,
    pub pseudo_globals: PseudoGlobals,
    pub w: c_int,
    pub h: c_int,
    pub config: Config,
    pub input: String,
    pub items: Option<Items>,
}

impl Drw {
    pub fn fontset_getwidth(&mut self, text: TextOption) -> CompResult<c_int> {
        if self.fonts.len() == 0 {
            Ok(0)
        } else {
            self.text(0, 0, 0, 0, 0, text, false).map(|o| o.0)
        }
    }

    pub fn text(
        &mut self,
        mut x: c_int,
        y: c_int,
        mut w: c_uint,
        h: c_uint,
        lpad: c_uint,
        text_opt: TextOption,
        invert: bool,
    ) -> CompResult<(c_int, Option<i32>)> {
        let mut text: String = {
            match text_opt {
                Prompt => self.config.prompt.clone(),
                Input => self.format_input()?,
                Other(string) => string.to_string(),
            }
        };
        unsafe {
            let render = x > 0 || y > 0 || w > 0 || h > 0;

            if text.len() == 0 || self.fonts.len() == 0 {
                return Ok((0, None));
            }

            let mut d: *mut XftDraw = ptr::null_mut();

            if !render {
                w = !0; // maximize w so that underflow never occurs
            } else {
                XSetForeground(
                    self.dpy,
                    self.gc,
                    (*self.scheme[if invert { ColFg } else { ColBg } as usize]).pixel,
                );
                XFillRectangle(self.dpy, self.drawable, self.gc, x, y, w as u32, h);
                d = XftDrawCreate(
                    self.dpy,
                    self.drawable,
                    XDefaultVisual(self.dpy, self.screen),
                    XDefaultColormap(self.dpy, self.screen),
                );
                x += lpad as c_int;
                w -= lpad;
            }

            let mut slice_start = 0;
            let mut slice_end = 0;
            let mut cur_font: Option<usize> = None;
            let mut spool = Spool::new();

            // TODO: this prevents us from using Cow
            text.push_str("."); // this will be removed later; turned into elipses

            for cur_char in text.chars() {
                // String is already utf8 so we don't need to do extra conversions
                // As such, this logic is changed from the source dmenu quite a bit

                let mut found_font = self
                    .fonts
                    .iter()
                    .position(|font| XftCharExists(self.dpy, font.xfont, cur_char as u32) == 1);
                if cur_font.is_some() && cur_font == found_font {
                    // append to list to be printed
                    slice_end += cur_char.len_utf8();
                }
                if cur_font.is_none() || cur_font != found_font {
                    if found_font.is_none() {
                        // char is not found in any fonts
                        // In this case, pretend it's in the first font, as it must be drawn

                        let fccharset = FcCharSetCreate();
                        FcCharSetAddChar(fccharset, cur_char as u32);
                        if self.fonts[0].pattern_pointer == ptr::null_mut() {
                            /* Refer to the comment in xfont_create for more information. */
                            return Die::stderr(
                                "fonts must be loaded from font strings".to_owned(),
                            );
                        }

                        let fcpattern =
                            FcPatternDuplicate(self.fonts[0].pattern_pointer as *const c_void);
                        FcPatternAddCharSet(fcpattern as *mut c_void, FC_CHARSET, fccharset);
                        FcPatternAddBool(fcpattern as *mut c_void, FC_SCALABLE, FcTrue);
                        FcPatternAddBool(fcpattern as *mut c_void, FC_COLOR, FcFalse);

                        FcConfigSubstitute(
                            ptr::null_mut(),
                            fcpattern as *mut c_void,
                            FcMatchPattern,
                        );
                        let mut result: x11::xft::FcResult = x11::xft::FcResult::NoId; // XftFontMatch isn't null safe so we need some memory (result is actually discarded)
                        let font_match = XftFontMatch(
                            self.dpy,
                            self.screen,
                            fcpattern as *const FcPattern,
                            &mut result,
                        );

                        FcCharSetDestroy(fccharset);
                        FcPatternDestroy(fcpattern);

                        if font_match != ptr::null_mut() {
                            let mut usedfont = Fnt::new(self, None, font_match)?;

                            if XftCharExists(self.dpy, usedfont.xfont, cur_char as u32) != 0 {
                                found_font = Some(self.fonts.len());
                                self.fonts.push(usedfont);
                            } else {
                                usedfont.free(self.dpy);
                                found_font = Some(0);
                            }
                        }
                    }
                    // Need to switch fonts
                    // First, take care of the stuff pending print
                    if cur_font.is_some() {
                        spool.push((
                            String::from_utf8_unchecked(
                                text.as_bytes()[slice_start..slice_end].to_vec(),
                            ),
                            cur_font,
                        ));
                    }
                    // Then, set up next thing to print
                    cur_font = found_font;
                    slice_start = slice_end;
                    slice_end += cur_char.len_utf8();
                }
            }
            // take care of the remaining slice, if it exists
            spool.push((
                String::from_utf8_unchecked(text.as_bytes()[slice_start..slice_end].to_vec()),
                cur_font,
            ));

            let padded_width = w - self.pseudo_globals.lrpad as u32 / 2;
            spool.elipsate(&self, padded_width);
            while render && spool.width(&self) > padded_width {
                spool.elipse_pop();
            }

            let elip_width = spool.elip_width(&self);
            for (slice, font) in spool.into_iter() {
                // Do early truncation (...)
                self.render(&mut x, &y, &mut w, &h, slice, &font, d, render, invert);
            }

            if d != ptr::null_mut() {
                XftDrawDestroy(d);
            }

            Ok((x + if render { w } else { 0 } as i32, elip_width))
        }
    }

    fn render(
        &self,
        x: &mut i32,
        y: &i32,
        w: &mut u32,
        h: &u32,
        text: String,
        cur_font: &Option<usize>,
        d: *mut XftDraw,
        render: bool,
        invert: bool,
    ) {
        if text.len() == 0 {
            return;
        }
        unsafe {
            let usedfont = cur_font.map(|i| &self.fonts[i]).unwrap();
            let font_ref = usedfont;
            let (substr_width, _) =
                self.font_getexts(font_ref, text.as_ptr() as *mut c_uchar, text.len() as c_int);
            if render {
                let ty = *y + (*h as i32 - usedfont.height as i32) / 2 + (*usedfont.xfont).ascent;
                XftDrawStringUtf8(
                    d,
                    self.scheme[if invert { ColBg } else { ColFg } as usize],
                    self.fonts[cur_font.unwrap()].xfont,
                    *x,
                    ty,
                    text.as_ptr() as *mut c_uchar,
                    text.len() as c_int,
                );
            }
            *x += substr_width as i32;
            *w -= substr_width;
        }
    }

    pub fn font_getexts(
        &self,
        font: &Fnt,
        subtext: *const c_uchar,
        len: c_int,
    ) -> (c_uint, c_uint) {
        unsafe {
            //                                                                (width,  height)
            let mut ext: XGlyphInfo = MaybeUninit::uninit().assume_init();
            XftTextExtentsUtf8(self.dpy, font.xfont, subtext, len, &mut ext);
            (ext.xOff as c_uint, font.height) // (width, height)
        }
    }

    pub fn draw(&mut self) -> CompResult<()> {
        self.pseudo_globals.promptw = if self.config.prompt.len() != 0 {
            self.textw(Prompt)?
        } else {
            0
        };

        self.setscheme(SchemeNorm);
        self.rect(0, 0, self.w as u32, self.h as u32, true, true); // clear menu

        let mut x = 0;

        if self.config.prompt.len() > 0 {
            // draw prompt
            self.setscheme(SchemeSel);
            x = self
                .text(
                    x,
                    0,
                    self.pseudo_globals.promptw as c_uint,
                    self.pseudo_globals.bh as u32,
                    self.pseudo_globals.lrpad as u32 / 2,
                    Prompt,
                    false,
                )?
                .0;
        }

        // draw menu
        let matches = Items::draw(
            self,
            if self.config.lines > 0 {
                Vertical
            } else {
                Horizontal
            },
        )?;

        // draw input field
        let w =
            if self.config.lines > 0 || self.items.as_mut().unwrap().match_len() == 0 || !matches {
                self.w - x
            } else {
                if self.config.render_overrun {
                    self.textw(Input)?.min(self.w - x)
                } else {
                    self.pseudo_globals.inputw
                }
            };
        self.setscheme(SchemeNorm);
        let truncated = self
            .text(
                x,
                0,
                w as c_uint,
                self.pseudo_globals.bh as c_uint,
                self.pseudo_globals.lrpad as c_uint / 2,
                Input,
                false,
            )?
            .1
            .map(|u| u + self.pseudo_globals.lrpad / 2);
        let inputw = self.textw(Input)?;
        let otherw = self.textw(Other(
            &self
                .input
                .graphemes(true)
                .skip(self.pseudo_globals.cursor)
                .join(""),
        ))?;

        let curpos: c_int = inputw - otherw + self.pseudo_globals.lrpad / 2 - 1;

        if curpos < truncated.unwrap_or(w - self.pseudo_globals.lrpad / 2) {
            self.setscheme(SchemeNorm);
            let tallest_font = self.fonts.iter().map(|f| f.height).max().unwrap();
            self.rect(
                x + curpos,
                (self.pseudo_globals.bh - tallest_font) as i32 / 2 + 2,
                2,
                tallest_font - 4,
                true,
                false,
            );
        }

        self.map(self.pseudo_globals.win, 0, 0, self.w, self.h);
        Ok(())
    }

    pub fn map(&self, win: Window, x: c_int, y: c_int, w: c_int, h: c_int) {
        unsafe {
            XCopyArea(
                self.dpy,
                self.drawable,
                win,
                self.gc,
                x,
                y,
                w as u32,
                h as u32,
                x,
                y,
            );
            XSync(self.dpy, False);
        }
    }

    pub fn textw(&mut self, text: TextOption) -> CompResult<c_int> {
        self.fontset_getwidth(text)
            .map(|computed_width| computed_width + self.pseudo_globals.lrpad)
    }

    pub fn setscheme(&mut self, scm: Schemes) {
        self.scheme = self.pseudo_globals.schemeset[scm as usize];
    }

    fn rect(&self, x: c_int, y: c_int, w: c_uint, h: c_uint, filled: bool, invert: bool) {
        unsafe {
            XSetForeground(
                self.dpy,
                self.gc,
                (*self.scheme[if invert { ColBg } else { ColFg } as usize]).pixel,
            );
            if filled {
                XFillRectangle(self.dpy, self.drawable, self.gc, x, y, w, h);
            } else {
                XDrawRectangle(self.dpy, self.drawable, self.gc, x, y, w - 1, h - 1);
            }
        }
    }

    /**
     * When taking input from stdin, apply post-processing
     */
    pub fn format_stdin(&mut self, lines: Vec<String>) -> CompResult<Vec<String>> {
        Ok(lines)
    }

    /**
     * Every time the input is drawn, how should it be presented?
     * Does it need additional processing?
     */
    pub fn format_input(&mut self) -> CompResult<String> {
        // TODO: CowString
        Ok(self.input.clone())
    }

    /**
     * What to do when printing to stdout / program termination?
     *
     * Args:
     * - output: what's being processed
     * - recommendation: is exiting recommended? C-Enter will not normally exit
     *
     * Returns - true if program should exit
     */
    pub fn dispose(&mut self, output: String, recommendation: bool) -> CompResult<bool> {
        println!("{}", output);
        Ok(recommendation)
    }

    /**
     * The following is called immediatly after gen_matches, taking its unwrapped output
     *
     * This is particularly useful for doing something based on a match method defined
     * elsewhere. For example, if any matched items contain a key, highlight them,
     * but still allow a custom matching algorithm (such as from the fuzzy plugin)
     */
    pub fn postprocess_matches(&mut self, items: Vec<Item>) -> CompResult<Vec<Item>> {
        Ok(items)
    }

    /**
     * Every time the input changes, what items should be shown
     * And, how should they be shown?
     *
     * Returns - Vector of items to be drawn
     */
    #[cfg(not(feature = "fuzzy"))]
    pub fn gen_matches(&mut self) -> CompResult<Vec<Item>> {
        let re = RegexBuilder::new(&regex::escape(&self.input))
            .case_insensitive(!self.config.case_sensitive)
            .build()
            .map_err(|_| Die::Stderr("Could not build regex".to_owned()))?;
        let mut exact: Vec<Item> = Vec::new();
        let mut prefix: Vec<Item> = Vec::new();
        let mut substring: Vec<Item> = Vec::new();
        for item in self.get_items() {
            match item.matches(&re) {
                MatchCode::Exact => exact.push(item.clone()),
                MatchCode::Prefix => prefix.push(item.clone()),
                MatchCode::Substring => substring.push(item.clone()),
                MatchCode::None => {}
            }
        }
        exact.reserve(prefix.len() + substring.len());
        for item in prefix {
            // extend is broken for pointers
            exact.push(item);
        }
        for item in substring {
            exact.push(item);
        }
        Ok(exact)
    }

    /// Fuzzy implementation
    #[cfg(feature = "fuzzy")]
    pub fn gen_matches(&mut self) -> CompResult<Vec<Item>> {
        println!("Fuzzy");
        let searchterm = self.input.clone();
        let matcher: Box<dyn FuzzyMatcher> = Box::new(SkimMatcherV2::default());
        let mut items: Vec<(Item, i64)> = self
            .get_items()
            .iter()
            .map(|item| {
                (
                    item.clone(),
                    if let Some(score) = matcher.fuzzy_match(&item.text, &searchterm) {
                        -score
                    } else {
                        1
                    },
                )
            })
            .collect();
        items.retain(|(_, score)| *score <= 0);
        items.sort_by_key(|(item, _)| item.text.len()); // this prioritizes exact matches
        items.sort_by_key(|(_, score)| *score);

        Ok(items.into_iter().map(|(item, _)| item).collect())
    }
}

impl Drop for Drw {
    fn drop(&mut self) {
        unsafe {
            for font in &mut self.fonts {
                font.free(self.dpy);
            }
            XUngrabKey(self.dpy, AnyKey, AnyModifier, self.root);
            for i in 0..SchemeLast as usize {
                free(self.pseudo_globals.schemeset[i][0] as *mut c_void);
                free(self.pseudo_globals.schemeset[i][1] as *mut c_void);
            }
            XFreePixmap(self.dpy, self.drawable);
            XFreeGC(self.dpy, self.gc);
            XSync(self.dpy, False);
            XCloseDisplay(self.dpy);
        }
    }
}

// Utility struct; contains chars and fonts
struct Spool {
    data: Vec<(String, Option<usize>)>,
    elipsed: bool,
}

impl Spool {
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            elipsed: false,
        }
    }
    pub fn width(&self, drw: &Drw) -> u32 {
        self.data
            .iter()
            .map(|(slice, font)| {
                drw.font_getexts(
                    &drw.fonts[font.unwrap()],
                    slice.as_ptr() as *mut c_uchar,
                    slice.len() as c_int,
                )
                .0
            })
            .fold(0, |sum, i| sum + i)
    }
    pub fn elipsate(&mut self, drw: &Drw, w: u32) {
        let elipse = self.pop();
        if self.width(drw) > w {
            self.elipsed = true;
            self.push(elipse.clone());
            self.push(elipse.clone());
            self.push(elipse);
        }
    }
    fn pop(&mut self) -> (String, Option<usize>) {
        let len = self.data.len();
        if self.data[len - 1].0.len() == 1 {
            self.data.pop().unwrap()
        } else {
            (
                self.data[len - 1].0.pop().unwrap().to_string(),
                self.data[len - 1].1,
            )
        }
    }
    pub fn elipse_pop(&mut self) {
        let len = self.data.len();
        if len == 0 {
            return;
        } else if len <= 3 {
            self.data.pop();
        } else {
            if self.data[len - 4].0.len() <= 1 {
                self.data.remove(len - 4);
            } else {
                self.data[len - 4].0.pop();
            }
        }
    }
    pub fn push(&mut self, arg: (String, Option<usize>)) {
        self.data.push(arg);
    }
    pub fn into_iter(self) -> std::vec::IntoIter<(String, Option<usize>)> {
        self.data.into_iter()
    }
    pub fn elip_width(&self, drw: &Drw) -> Option<i32> {
        if !self.elipsed {
            None
        } else {
            Some(if self.data.len() <= 3 {
                self.width(drw)
            } else {
                self.data
                    .iter()
                    .rev()
                    .skip(3)
                    .map(|(slice, font)| {
                        drw.font_getexts(
                            &drw.fonts[font.unwrap()],
                            slice.as_ptr() as *mut c_uchar,
                            slice.len() as c_int,
                        )
                        .0
                    })
                    .fold(0, |sum, i| sum + i)
            } as i32)
        }
    }
}
