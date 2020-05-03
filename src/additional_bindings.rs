// A few additional bindings are needed from fondconfig.h
// Because servo-fontconfig provides very clean bindings for everything,
// only the bindings not included there are mapped here
pub mod fontconfig {
    #![allow(non_upper_case_globals)]
    #![allow(non_camel_case_types)]
    #![allow(non_snake_case)]
    mod raw {
	include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
    }
    pub const FcTrue:  raw::FcBool   = raw::FcTrue  as raw::FcBool;
    pub const FcFalse: raw::FcBool   = raw::FcFalse as raw::FcBool;
    pub const FC_SCALABLE: *const i8 = raw::FC_SCALABLE.as_ptr() as *const i8;
    pub const FC_CHARSET:  *const i8 = raw::FC_CHARSET.as_ptr()  as *const i8;
    pub const FC_COLOR:    *const i8 = raw::FC_COLOR.as_ptr()    as *const i8;
}
