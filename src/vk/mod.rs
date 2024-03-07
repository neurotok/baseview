use std::ffi::c_void;
use std::marker::PhantomData;

// On X11 creating the context is a two step process
#[cfg(not(target_os = "linux"))]
use raw_window_handle::HasRawWindowHandle;

#[cfg(target_os = "windows")]
mod win;
#[cfg(target_os = "windows")]
use win as platform;

// We need to use this directly within the X11 window creation to negotiate the correct visual
#[cfg(target_os = "linux")]
pub(crate) mod x11;
#[cfg(target_os = "linux")]
pub(crate) use self::x11 as platform;

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "macos")]
use macos as platform;

#[derive(Clone, Debug)]
pub struct VkConfig {
    pub version: (u8, u8),
}

impl Default for VkConfig {
    fn default() -> Self {
        VkConfig {
            version: (1, 2),
        }
    }
}

#[derive(Debug)]
pub enum VkError {
    InvalidWindowHandle,
    VersionNotSupported,
    CreationFailed(platform::CreationFailedError),
}

pub struct VkContext {
    context: platform::VkContext,
    phantom: PhantomData<*mut ()>,
}

impl VkContext {
    #[cfg(not(target_os = "linux"))]
    pub(crate) unsafe fn create(
        parent: &impl HasRawWindowHandle, config: VkConfig,
    ) -> Result<VkContext, VkError> {
        platform::VkContext::create(parent, config)
            .map(|context| VkContext { context, phantom: PhantomData })
    }

    /// The X11 version needs to be set up in a different way compared to the Windows and macOS
    /// versions. So the platform-specific versions should be used to construct the context within
    /// baseview, and then this object can be passed to the user.
    #[cfg(target_os = "linux")]
    pub(crate) fn new(context: platform::VkContext) -> VkContext {
        VkContext { context, phantom: PhantomData }
    }

    pub unsafe fn make_current(&self) {
        self.context.make_current();
    }

    pub unsafe fn make_not_current(&self) {
        self.context.make_not_current();
    }

    pub fn get_proc_address(&self, symbol: &str) -> *const c_void {
        self.context.get_proc_address(symbol)
    }

    pub fn swap_buffers(&self) {
        self.context.swap_buffers();
    }

    /// On macOS the `NSOpenGLView` needs to be resized separtely from our main view.
    #[cfg(target_os = "macos")]
    pub(crate) fn resize(&self, size: cocoa::foundation::NSSize) {
        self.context.resize(size);
    }
}
