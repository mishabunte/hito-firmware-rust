use core::ffi::{c_char, c_int};

extern "C" {
  pub fn crypt0_bech32_encode(data: *const u8, datalen: c_int, buf: *mut c_char, buflen: c_int) -> c_int;
}
