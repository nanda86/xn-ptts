//! Carry the calling thread's quality of service over to the worker threads the driver starts.
//!
//! On Apple platforms a new pthread starts at the default QoS class, whatever its creator runs
//! at. The driver's caller waits on those workers (the utterance thread blocks on Mimi's PCM
//! channel), so a user-initiated caller -- the Swift package runs every call on a
//! `.userInitiated` queue -- would wait on a lower-priority thread: a priority inversion, which
//! Xcode's Thread Performance Checker reports and the scheduler can stretch under load.

/// The QoS class and relative priority of the current thread.
#[derive(Clone, Copy, Debug)]
pub struct Qos {
    #[cfg_attr(not(target_vendor = "apple"), allow(dead_code))]
    class: u32,
    #[cfg_attr(not(target_vendor = "apple"), allow(dead_code))]
    relative_priority: i32,
}

#[cfg(target_vendor = "apple")]
mod sys {
    // <pthread/qos.h>; libSystem is always linked on Apple targets.
    unsafe extern "C" {
        pub fn qos_class_self() -> u32;
        pub fn pthread_set_qos_class_self_np(qos_class: u32, relative_priority: i32) -> i32;
    }
    /// QOS_CLASS_UNSPECIFIED: the thread has no class of its own.
    pub const UNSPECIFIED: u32 = 0;
}

impl Qos {
    /// The current thread's QoS, to hand to a thread it starts.
    pub fn current() -> Self {
        #[cfg(target_vendor = "apple")]
        {
            // SAFETY: plain libSystem queries of the calling thread.
            let class = unsafe { sys::qos_class_self() };
            Self { class, relative_priority: 0 }
        }
        #[cfg(not(target_vendor = "apple"))]
        {
            Self { class: 0, relative_priority: 0 }
        }
    }

    /// Apply this QoS to the calling thread. Call it first thing in the started thread.
    pub fn apply(self) {
        #[cfg(target_vendor = "apple")]
        if self.class != sys::UNSPECIFIED {
            // SAFETY: sets only the calling thread's own QoS. A failure leaves the default
            // class, which is no worse than not calling it.
            unsafe { sys::pthread_set_qos_class_self_np(self.class, self.relative_priority) };
        }
    }
}
