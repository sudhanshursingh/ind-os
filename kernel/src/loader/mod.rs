//! Executable Binary Loaders for IndK
//! Supports native INDBIN (ADR-001) format and clean-room executable parsing.

pub mod indbin;

pub use indbin::{
    build_indbin_image, get_alpha_user_code, get_beta_user_code, load_indbin, parse_header,
    IndbinHeader, LoaderError, DEFAULT_USER_STACK_SIZE, HELLO_USER_CODE, INDBIN_MAGIC,
    INDBIN_VERSION, USER_STACK_TOP,
};
