// use std::ffi::{c_void, CString};
// use std::os::raw::{c_int, c_ulong};

// use x11::glx;
// use x11::xlib;

use std::ffi::CString;
use std::os::raw::c_ulong as other_c_ulong;
use std::os::raw::c_void;
use x11::xlib;

use super::{VkConfig, VkError};

use ash::extensions::{
    ext::DebugUtils,
    khr::{Surface, Swapchain, XlibSurface},
};

use ash::{vk, Entry};
pub use ash::{Device, Instance};
use raw_window_handle::{HasRawWindowHandle, RawWindowHandle};
use std::ptr;

pub enum Queue {
    Graphics(vk::Queue),
    Transfer(vk::Queue),
    VideoDecode(vk::Queue),
    VideoEncode(vk::Queue),
}

pub struct Init {
    entry: Entry,
    instance: Instance,
    //surface: vk::SurfaceKHR,
    surface: XlibSurface,
    pdevice: vk::PhysicalDevice,
    queue_family_index: u32,
    device: Device,
    // command_pool: vk::CommandPool,
    // queues: Vec<Queue>,
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

        Ok(VkContext { window, display, init: Init::new(window, display).unwrap() })
    }
}

impl Drop for VkContext {
    fn drop(&mut self) {}
}

fn check_physical_device_surface_support(
    pdevice: vk::PhysicalDevice, queue_family_index: u32, surface: &XlibSurface,
    display: *mut vk::Display, visual_id: vk::VisualID,
) -> bool {
    unsafe {
        surface.get_physical_device_xlib_presentation_support(
            pdevice,
            queue_family_index,
            &mut (display as *const c_void),
            visual_id,
        )
    }
}

impl Init {
    pub fn new(window: other_c_ulong, display: *mut xlib::_XDisplay) -> Result<Self, String> {
        unsafe {
            let entry = Entry::linked();

            // TODO pass in config
            let app_name = CString::new("Eyecatcher").unwrap();
            let app_info = vk::ApplicationInfo {
                p_application_name: app_name.as_ptr(),
                s_type: vk::StructureType::APPLICATION_INFO,
                p_next: ptr::null(),
                //application_version: vk::make_version(config.version(0), config.version(1), 0),
                application_version: vk::make_api_version(1, 3, 0, 0),
                engine_version: vk::make_api_version(1, 0, 0, 0),
                api_version: vk::API_VERSION_1_0,
                p_engine_name: app_name.as_ptr(),
            };

            let instance_info = vk::InstanceCreateInfo {
                s_type: vk::StructureType::INSTANCE_CREATE_INFO,
                p_next: ptr::null(),
                flags: vk::InstanceCreateFlags::empty(),
                p_application_info: &app_info,
                enabled_extension_count: 1,
                pp_enabled_extension_names: [XlibSurface::name().as_ptr()].as_ptr()
                    as *const *const i8,
                pp_enabled_layer_names: ptr::null(),
                enabled_layer_count: 0,
            };

            let instance = entry
                .create_instance(&instance_info, None)
                .map_err(|_| "Unable to create Vulkan instance.")?;

            let surface = XlibSurface::new(&entry, &instance);

            let surface_info = vk::XlibSurfaceCreateInfoKHR {
                s_type: vk::StructureType::XLIB_SURFACE_CREATE_INFO_KHR,
                p_next: ptr::null(),
                flags: vk::XlibSurfaceCreateFlagsKHR::empty(),
                window,
                dpy: display as *mut _,
            };

            surface
                .create_xlib_surface(&surface_info, None)
                .map_err(|_| "Unable to create Vulkan Xlib surface.")?;

            let pdevices = instance.enumerate_physical_devices().expect("Physical device error");

            let (pdevice, queue_family_index) = pdevices
                .iter()
                .find_map(|pdevice| {
                    instance
                        .get_physical_device_queue_family_properties(*pdevice)
                        .iter()
                        .enumerate()
                        .find_map(|(index, info)| {
                            let supports_graphic =
                                info.queue_flags.contains(vk::QueueFlags::GRAPHICS);
                            let supports_surface = check_physical_device_surface_support(
                                *pdevice,
                                index as u32,
                                &surface,
                                display as *mut *const c_void,
                                0,
                            );
                            if supports_graphic && supports_surface {
                                Some((*pdevice, index))
                            } else {
                                None
                            }
                        })
                })
                .expect("Couldn't find suitable device.");
            let queue_family_index = queue_family_index as u32;

            let device_extension_names_raw = [
                Swapchain::name().as_ptr(),
                #[cfg(any(target_os = "macos", target_os = "ios"))]
                KhrPortabilitySubsetFn::NAME.as_ptr(),
            ];
            let features =
                vk::PhysicalDeviceFeatures { shader_clip_distance: 1, ..Default::default() };
            let priorities = [1.0];

            let queue_info = vk::DeviceQueueCreateInfo::builder()
                .queue_family_index(queue_family_index)
                .queue_priorities(&priorities);

            // let queue_info = vk::DeviceQueueCreateInfo::default()
            //     .queue_family_index(queue_family_index)
            //     .queue_priorities(&priorities);

            let device_create_info = vk::DeviceCreateInfo::builder()
                .queue_create_infos(std::slice::from_ref(&queue_info))
                .enabled_extension_names(&device_extension_names_raw)
                .enabled_features(&features);

            let device: Device =
                instance.create_device(pdevice, &device_create_info, None).unwrap();

            let present_queue = device.get_device_queue(queue_family_index, 0);

            Ok(Self { entry, instance, surface, pdevice, queue_family_index, device })
        }
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
            // TODO
            //self.instance.destroy_surface_khr(self.surface, None);
            //self.instance.destroy_instance(None);
        }
    }
}
