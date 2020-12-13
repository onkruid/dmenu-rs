#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]

mod raw {
    #![allow(unused)]
    pub mod xlib;
    pub mod fontconfig;
}

pub mod fontconfig {
    #![allow(non_upper_case_globals)]
    #![allow(non_camel_case_types)]
    #![allow(non_snake_case)]

    use super::raw::fontconfig as fc;

    pub const FcTrue:  fc::FcBool  = fc::FcTrue  as fc::FcBool;
    pub const FcFalse: fc::FcBool  = fc::FcFalse as fc::FcBool;
    pub const FC_SCALABLE: *const i8 = fc::FC_SCALABLE.as_ptr() as *const i8;
    pub const FC_CHARSET:  *const i8 = fc::FC_CHARSET.as_ptr()  as *const i8;
    pub const FC_COLOR:    *const i8 = fc::FC_COLOR.as_ptr()    as *const i8;
    pub const FC_FAMILY:   *mut   i8 = fc::FC_FAMILY.as_ptr()   as *mut   i8;
}

pub mod xlib {
    #![allow(non_upper_case_globals)]
    #![allow(non_camel_case_types)]
    #![allow(non_snake_case)]
    use super::raw::xlib;
    pub use xlib::{XNInputStyle, XNClientWindow, XNFocusWindow};
}
