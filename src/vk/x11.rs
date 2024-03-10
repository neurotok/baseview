// use std::ffi::{c_void, CString};
// use std::os::raw::{c_int, c_ulong};

// use x11::glx;
// use x11::xlib;

use std::os::raw::c_ulong as other_c_ulong;
use x11::xlib;


use super::{VkConfig, VkError};

use ash::{vk, Entry};
pub use ash::{Device, Instance};
use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};
use std::ptr;

pub enum Queue{
    Graphics(vk::Queue),
    Transfer(vk::Queue),
    VideoDecode(vk::Queue),
    VideoEncode(vk::Queue),
}

pub struct Init
{
    entry: ash::Entry,
    instance: ash::Instance,
    physical_device: vk::PhysicalDevice,
    device: ash::Device,
    surface: vk::SurfaceKHR,
    command_pool: vk::CommandPool,
    queues: Vec<Queue>,
}

pub struct VkContext {
    window: other_c_ulong,
    display: *mut xlib::_XDisplay,
    init: Init,
}

impl VkContext {
    pub unsafe fn create(
        window: other_c_ulong, display: *mut xlib::_XDisplay,
    ) -> Result<VkContext, VkError> {
        if display.is_null() {
            return Err(VkError::InvalidWindowHandle);
        }

        Ok(VkContext {
            window,
            display,
            init: Init::new(window, display)?,
        })
    }
}

impl Drop for VkContext {
    fn drop(&mut self) {}
}


impl Init {
    pub fn new(window_handle: &impl HasRawWindowHandle, display: *mut xlib::_XDisplay) -> Result<Self, String> {
        let entry = ash::Entry::new().map_err(|_| "Unable to create Vulkan entry points.")?;

        let app_name = ash::vk::make_version("App Name");
        let app_info = vk::ApplicationInfo {
            p_application_name: app_name.as_ptr(),
            s_type: vk::StructureType::APPLICATION_INFO,
            p_next: ptr::null(),
            //application_version: vk::make_version(config.version(0), config.version(1), 0),
            application_version: vk::make_version(1, 3, 0),
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
            // TODO
            if let RawWindowHandle::Xlib(handle) = window_handle.raw_window_handle() {
                let create_info = vk::XlibSurfaceCreateInfoKHR {
                    s_type: vk::StructureType::XLIB_SURFACE_CREATE_INFO_KHR,
                    p_next: ptr::null(),
                    flags: vk::XlibSurfaceCreateFlagsKHR::empty(),
                    window: handle.window,
                    dpy: handle.display_handle as *mut _,
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

    // pub unsafe fn make_current(&self) {
    //     errors::XErrorHandler::handle(self.display, |error_handler| {
    //         let res = glx::glXMakeCurrent(self.display, self.window, self.context);
    //         error_handler.check().unwrap();
    //         if res == 0 {
    //             panic!("make_current failed")
    //         }
    //     })
    // }

    // pub unsafe fn make_not_current(&self) {
    //     errors::XErrorHandler::handle(self.display, |error_handler| {
    //         let res = glx::glXakeCurrent(self.display, 0, std::ptr::null_mut());
    //         error_handler.check().unwrap();
    //         if res == 0 {
    //             panic!("make_not_current failed")
    //         }
    //     })
    // }

    // pub fn get_proc_address(&self, symbol: &str) -> *const c_void {
    //     get_proc_address(symbol)
    // }

    // pub fn swap_buffers(&self) {
    //     errors::XErrorHandler::handle(self.display, |error_handler| {
    //         unsafe {
    //             glx::glXSwapBuffers(self.display, self.window);
    //         }
    //         error_handler.check().unwrap();
    //     })
    // }

}

impl Drop for Init {
    fn drop(&mut self) {
        unsafe {
            self.instance.destroy_surface_khr(self.surface, None);
            self.instance.destroy_instance(None);
        }
    }
}
