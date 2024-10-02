//! Module containing definitions and manipulation resources for unique per thread and/or core
//! local stored structures
use state::BootState;
use core::sync::atomic::{AtomicUsize, Ordering};

// Initialize the first core id. This will be incremented atomically for each of the following
// cores that come online afterwards
static CORE_ID: AtomicUsize = AtomicUsize::new(0x1337);

/// Contains unique per core informations that can only be accessed by it's corresponding core.
// We need the address pointer as the first field of the structure. In order to make sure Rust does
// not suffle fields around, we `repr(C)` it
#[repr(C)]
pub struct Core {
    // Represents the address to this current structure
    core_ptr: usize,
    // Represents the unique identifier for the core
    id: usize,
    // Represents a systems state passed on from the bootloader to the kernel
    pub state: &'static BootState,
}

impl Core {
    pub fn id(&self) -> usize {
        self.id
    }
}

pub unsafe fn get_core_state() -> &'static Core {
    let ptr: usize;
    core::arch::asm!("mov {0}, gs:[0]", out(reg) ptr);
    &*(ptr as *const Core)
}

// Get the current core structure
#[macro_export]
macro_rules! core {
    () => {
        $crate::tls::get_core_state()
    };
}

pub fn init(state: &'static BootState) -> Option<()> {
    // Acquire a lock to the memory
    let mut mmu_lock = state.mmu.lock();
    let mmu = mmu_lock.as_mut()?;

    // Allocate memory that will hold the `Core` structure
    let core_ptr = mmu
        .allocate(
            u64::try_from(core::mem::size_of::<Core>()).ok()?,
            u64::try_from(core::mem::align_of::<Core>()).ok()?,
        )?;

    // Get a new core id for the current core
    let id = CORE_ID.fetch_add(1, Ordering::Relaxed);

    // Create the core structure
    let core = Core { core_ptr, id, state };

    unsafe {
        // Write the structure in the newly allocated address
        core::ptr::write(core_ptr as *mut Core, core);
        // Set the gs base to reflect the new structure
        cpu::x86::write_gs_base(u64::try_from(core_ptr).ok()?);
    }

    Some(())
}
