use super::state::ReaderState;

/// SCARDCONTEXT: opaque handle returned by SCardEstablishContext
#[cfg(target_os = "linux")]
pub(super) type SCardContext = std::ffi::c_long;
#[cfg(target_os = "windows")]
pub(super) type SCardContext = usize;

/// LONG / PCSC_LONG: return type of every SCard* function
#[cfg(target_os = "linux")]
pub(super) type PcscLong = std::ffi::c_long;
#[cfg(target_os = "windows")]
pub(super) type PcscLong = std::ffi::c_long;

/// DWORD of PC/SC API
#[cfg(target_os = "linux")]
pub(super) type PcscDword = u32;
#[cfg(target_os = "windows")]
pub(super) type PcscDword = u32;

pub(super) const SCARD_S_SUCCESS: PcscLong = 0;
pub(super) const SCARD_SCOPE_USER: PcscDword = 0;
pub(super) const SCARD_STATE_UNAWARE: PcscDword = 0x0000;
pub(super) const SCARD_STATE_PRESENT: PcscDword = 0x0020;

// Function-pointer types of PC/SC C API

type FnEstablishContext = unsafe extern "C" fn(
    PcscDword,               // dwScope
    *const std::ffi::c_void, // pvReserved1
    *const std::ffi::c_void, // pvReserved2
    *mut SCardContext,       // phContext
) -> PcscLong;

type FnReleaseContext = unsafe extern "C" fn(SCardContext) -> PcscLong;

type FnListReaders = unsafe extern "C" fn(
    SCardContext,            // hContext
    *const std::ffi::c_char, // mszGroups
    *mut std::ffi::c_char,   // mszReaders (multi-string out)
    *mut PcscDword,          // pcchReaders (in/out)
) -> PcscLong;

type FnGetStatusChange = unsafe extern "C" fn(
    SCardContext,     // hContext
    PcscDword,        // dwTimeout (ms)
    *mut ReaderState, // rgReaderStates
    PcscDword,        // cReaders
) -> PcscLong;

/// Thin wrapper around the dynamically loaded PC/SC library
pub(super) struct PcscLib {
    _lib: libloading::Library,
    establish: FnEstablishContext,
    release: FnReleaseContext,
    list_readers: FnListReaders,
    get_status_change: FnGetStatusChange,
}

impl PcscLib {
    /// Try to load the platform-specific PC/SC shared library
    pub(super) fn load() -> Option<Self> {
        #[cfg(target_os = "linux")]
        let lib_name = "libpcsclite.so.1";
        #[cfg(target_os = "windows")]
        let lib_name = "WinSCard.dll";
        #[cfg(not(any(target_os = "linux", target_os = "windows")))]
        return None;

        let lib = unsafe { libloading::Library::new(lib_name).ok()? };

        unsafe {
            let establish: FnEstablishContext = *lib.get(b"SCardEstablishContext\0").ok()?;
            let release: FnReleaseContext = *lib.get(b"SCardReleaseContext\0").ok()?;
            let list_readers: FnListReaders = *lib.get(b"SCardListReaders\0").ok()?;

            #[cfg(target_os = "linux")]
            let get_status_change: FnGetStatusChange = *lib.get(b"SCardGetStatusChange\0").ok()?;
            #[cfg(target_os = "windows")]
            let get_status_change: FnGetStatusChange = *lib.get(b"SCardGetStatusChangeA\0").ok()?;

            Some(Self {
                _lib: lib,
                establish,
                release,
                list_readers,
                get_status_change,
            })
        }
    }

    /// Establish a PC/SC context
    ///
    /// Returns `None` on failure
    pub(super) fn establish_context(&self) -> Option<SCardContext> {
        let mut ctx: SCardContext = 0;
        let rc = unsafe {
            (self.establish)(
                SCARD_SCOPE_USER,
                std::ptr::null(),
                std::ptr::null(),
                &mut ctx,
            )
        };
        if rc == SCARD_S_SUCCESS {
            Some(ctx)
        } else {
            None
        }
    }

    pub(super) fn release_context(&self, ctx: SCardContext) {
        unsafe { (self.release)(ctx) };
    }

    /// Return the name of the first reader | `None`
    pub(super) fn first_reader(&self, ctx: SCardContext) -> Option<std::ffi::CString> {
        // First call: ask for required buffer size.
        let mut len: PcscDword = 0;
        let rc =
            unsafe { (self.list_readers)(ctx, std::ptr::null(), std::ptr::null_mut(), &mut len) };
        if rc != SCARD_S_SUCCESS || len == 0 {
            return None;
        }

        let mut buf: Vec<u8> = vec![0; len as usize];
        let rc = unsafe {
            (self.list_readers)(
                ctx,
                std::ptr::null(),
                buf.as_mut_ptr() as *mut std::ffi::c_char,
                &mut len,
            )
        };
        if rc != SCARD_S_SUCCESS {
            return None;
        }

        // The buffer is a multi-string
        // (sequences separated by \0, terminated by double \0)
        // We only need the first one
        let first_end = buf.iter().position(|&b| b == 0).unwrap_or(buf.len());
        if first_end == 0 {
            return None;
        }
        std::ffi::CString::new(&buf[..first_end]).ok()
    }

    /// Queries whether a card is present in the given reader currently
    /// (non-blocking, timeout = 0)
    pub(super) fn is_card_present(&self, ctx: SCardContext, reader: &std::ffi::CStr) -> bool {
        let mut state = ReaderState::new(reader.as_ptr());
        let rc = unsafe { (self.get_status_change)(ctx, 0, &mut state, 1) };
        if rc != SCARD_S_SUCCESS {
            return false;
        }
        (state.event_state & SCARD_STATE_PRESENT) != 0
    }
}
