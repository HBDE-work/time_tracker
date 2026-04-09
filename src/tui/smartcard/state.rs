use super::pcsclib::PcscDword;
use super::pcsclib::SCARD_STATE_UNAWARE;

/// `SCARD_READERSTATE`: the struct passed to SCardGetStatusChange
/// Layout is the same on both platforms (C ABI)
#[repr(C)]
pub(super) struct ReaderState {
    reader: *const std::ffi::c_char,
    user_data: *mut std::ffi::c_void,
    current_state: PcscDword,
    pub(super) event_state: PcscDword,
    atr_len: PcscDword,
    atr: [u8; 36],
}

impl ReaderState {
    pub(super) fn new(reader: *const std::ffi::c_char) -> Self {
        Self {
            reader,
            user_data: std::ptr::null_mut(),
            current_state: SCARD_STATE_UNAWARE,
            event_state: 0,
            atr_len: 0,
            atr: [0u8; 36],
        }
    }
}
