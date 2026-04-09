use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use super::pcsclib::PcscLib;

/// Signals sent from the smartcard polling thread to the TUI main loop
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum CardEvent {
    Inserted,
    Removed,
}

/// Result of probing the OS for PC/SC reader availability
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum ReaderProbe {
    /// At least one reader was found
    Available,
    /// PC/SC subsystem exists but no readers are connected
    NoReaders,
    /// PC/SC library could not be loaded at all
    Unavailable,
}

/// probe we can load the PC/SC library and see at least one reader
pub(crate) fn probe_readers() -> ReaderProbe {
    let lib = match PcscLib::load() {
        Some(l) => l,
        None => return ReaderProbe::Unavailable,
    };
    let ctx = match lib.establish_context() {
        Some(c) => c,
        None => return ReaderProbe::Unavailable,
    };
    let result = if lib.first_reader(ctx).is_some() {
        ReaderProbe::Available
    } else {
        ReaderProbe::NoReaders
    };
    lib.release_context(ctx);
    result
}

/// Handle to the background smartcard watch process
///
/// Dropping it signals the thread to stop
pub(crate) struct SmartcardWatchProcess {
    pub rx: mpsc::Receiver<CardEvent>,
    stop: std::sync::Arc<std::sync::atomic::AtomicBool>,
    _handle: thread::JoinHandle<()>,
}

impl SmartcardWatchProcess {
    /// Spawn a background thread that polls the first available reader and
    /// sends [`CardEvent`]s when the card state changes
    pub(crate) fn spawn() -> Self {
        let (tx, rx) = mpsc::channel();
        let stop = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let stop_flag = stop.clone();

        let handle = thread::spawn(move || {
            Self::poll_loop(&tx, &stop_flag);
        });

        Self {
            rx,
            stop,
            _handle: handle,
        }
    }

    /// Tell the background thread to exit
    pub(crate) fn stop(&self) {
        self.stop.store(true, std::sync::atomic::Ordering::Relaxed);
    }

    /// The actual polling loop running on the background thread
    fn poll_loop(tx: &mpsc::Sender<CardEvent>, stop: &std::sync::atomic::AtomicBool) {
        const POLL_INTERVAL: Duration = Duration::from_millis(500);

        let Some(lib) = PcscLib::load() else { return };
        let Some(ctx) = lib.establish_context() else {
            return;
        };

        let mut card_present = false;

        while !stop.load(std::sync::atomic::Ordering::Relaxed) {
            // Re-discover the reader each iteration so we survive reader
            // reconnects (e.g. USB unplug/replug)
            if let Some(reader) = lib.first_reader(ctx) {
                let present = lib.is_card_present(ctx, &reader);
                if present && !card_present {
                    let _ = tx.send(CardEvent::Inserted);
                } else if !present && card_present {
                    let _ = tx.send(CardEvent::Removed);
                }
                card_present = present;
            }
            thread::sleep(POLL_INTERVAL);
        }

        lib.release_context(ctx);
    }
}

impl Drop for SmartcardWatchProcess {
    fn drop(&mut self) {
        self.stop();
    }
}
