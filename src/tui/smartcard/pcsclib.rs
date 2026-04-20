/// SCARDCONTEXT: opaque handle returned by SCardEstablishContext
#[cfg(target_os = "linux")]
pub(super) type SCardContext = std::ffi::c_long;
#[cfg(target_os = "windows")]
pub(super) type SCardContext = usize;

/// SCARDHANDLE: opaque handle returned by SCardConnect
#[cfg(target_os = "linux")]
type SCardHandle = std::ffi::c_long;
#[cfg(target_os = "windows")]
type SCardHandle = usize;

/// LONG / PCSC_LONG: return type of every SCard* function
#[cfg(target_os = "linux")]
pub(super) type PcscLong = std::ffi::c_long;
#[cfg(target_os = "windows")]
pub(super) type PcscLong = std::ffi::c_long;

/// DWORD of PC/SC API
///
/// pcsclite on Linux defines DWORD as `unsigned long` (8 bytes on x86_64)
/// while on Windows DWORD is always `u32` (LLP64 data model)
///
/// Using the wrong width corrupts the stack in release builds
#[cfg(target_os = "linux")]
pub(super) type PcscDword = std::ffi::c_ulong;
#[cfg(target_os = "windows")]
pub(super) type PcscDword = u32;

pub(super) const SCARD_S_SUCCESS: PcscLong = 0;
const SCARD_SCOPE_USER: PcscDword = 0;
const SCARD_SHARE_SHARED: PcscDword = 2;
const SCARD_PROTOCOL_ANY: PcscDword = 0x0003; // T=0 | T=1
const SCARD_LEAVE_CARD: PcscDword = 0;

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

type FnConnect = unsafe extern "C" fn(
    SCardContext,            // hContext
    *const std::ffi::c_char, // szReader
    PcscDword,               // dwShareMode
    PcscDword,               // dwPreferredProtocols
    *mut SCardHandle,        // phCard (out)
    *mut PcscDword,          // pdwActiveProtocol (out)
) -> PcscLong;

type FnDisconnect = unsafe extern "C" fn(
    SCardHandle, // hCard
    PcscDword,   // dwDisposition
) -> PcscLong;

/// Thin wrapper around the dynamically loaded PC/SC library
pub(super) struct PcscLib {
    _lib: libloading::Library,
    establish: FnEstablishContext,
    release: FnReleaseContext,
    list_readers: FnListReaders,
    connect: FnConnect,
    disconnect: FnDisconnect,
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
            #[cfg(target_os = "linux")]
            let list_readers: FnListReaders = *lib.get(b"SCardListReaders\0").ok()?;
            #[cfg(target_os = "windows")]
            let list_readers: FnListReaders = *lib.get(b"SCardListReadersA\0").ok()?;

            #[cfg(target_os = "linux")]
            let connect: FnConnect = *lib.get(b"SCardConnect\0").ok()?;
            #[cfg(target_os = "windows")]
            let connect: FnConnect = *lib.get(b"SCardConnectA\0").ok()?;

            let disconnect: FnDisconnect = *lib.get(b"SCardDisconnect\0").ok()?;

            Some(Self {
                _lib: lib,
                establish,
                release,
                list_readers,
                connect,
                disconnect,
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

    /// Attempt to connect to a card in the given reader
    ///
    /// Returns `true` if a card is present and responsive
    /// The connection is released immediately
    pub(super) fn is_card_present(&self, ctx: SCardContext, reader: &std::ffi::CStr) -> bool {
        let mut handle: SCardHandle = 0;
        let mut protocol: PcscDword = 0;
        let rc = unsafe {
            (self.connect)(
                ctx,
                reader.as_ptr(),
                SCARD_SHARE_SHARED,
                SCARD_PROTOCOL_ANY,
                &mut handle,
                &mut protocol,
            )
        };
        if rc == SCARD_S_SUCCESS {
            unsafe { (self.disconnect)(handle, SCARD_LEAVE_CARD) };
            true
        } else {
            false
        }
    }
}
