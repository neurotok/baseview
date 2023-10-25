use ash::{version::EntryV1_0, version::InstanceV1_0, vk};
use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};
use std::ptr;

pub struct VkContext {
    _entry: ash::Entry,
    instance: ash::Instance,
    surface: vk::SurfaceKHR,
}

impl VkContext {
    pub fn new(window_handle: &impl HasRawWindowHandle) -> Result<Self, String> {
        let entry = ash::Entry::new().map_err(|_| "Unable to create Vulkan entry points.")?;

        let app_name = ash::vk::make_string("App Name");
        let app_info = vk::ApplicationInfo {
            p_application_name: app_name.as_ptr(),
            s_type: vk::StructureType::APPLICATION_INFO,
            p_next: ptr::null(),
            application_version: vk::make_version(1, 0, 0),
            engine_version: vk::make_version(1, 0, 0),
            api_version: vk::API_VERSION_1_0,
            p_engine_name: app_name.as_ptr(),
        };

        let instance_info = vk::InstanceCreateInfo {
            s_type: vk::StructureType::INSTANCE_CREATE_INFO,
            p_next: ptr::null(),
            flags: vk::InstanceCreateFlags::empty(),
            p_application_info: &app_info,
            pp_enabled_extension_names: ptr::null(),
            enabled_extension_count: 0,
            pp_enabled_layer_names: ptr::null(),
            enabled_layer_count: 0,
        };

        let instance = entry
            .create_instance(&instance_info, None)
            .map_err(|_| "Unable to create Vulkan instance.")?;

        let surface = unsafe {
            if let RawWindowHandle::Xlib(handle) = window_handle.raw_window_handle() {
                let create_info = vk::XlibSurfaceCreateInfoKHR {
                    s_type: vk::StructureType::XLIB_SURFACE_CREATE_INFO_KHR,
                    p_next: ptr::null(),
                    flags: vk::XlibSurfaceCreateFlagsKHR::empty(),
                    window: handle.window,
                    dpy: handle.display as *mut _,
                };
                instance
                    .create_xlib_surface_khr(&create_info, None)
                    .map_err(|_| "Unable to create Vulkan Xlib surface.")?
            } else {
                return Err("Invalid window handle".to_string());
            }
        };

        Ok(Self {
            _entry: entry,
            instance,
            surface,
        })
    }
}

impl Drop for VkContext {
    fn drop(&mut self) {
        unsafe {
            self.instance.destroy_surface_khr(self.surface, None);
            self.instance.destroy_instance(None);
        }
    }
}
