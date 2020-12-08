// TODO: eradicate all unnecessary `.clone()` functions with copy, reference, or moves.

use ash::vk;
use ash::version::{EntryV1_0, InstanceV1_0, DeviceV1_0};
use ash::extensions::{khr, ext};

use winit::window::Window;

use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_void};
use std::borrow::Cow;

use crate::device::DeviceMTXG;

pub mod device;
pub mod swapchain;
pub mod pipeline;
pub mod command_buffers;
pub mod buffer;
pub mod descriptors;


pub trait CleanupVkObj {
    unsafe fn cleanup(&self, device: &DeviceMTXG);  // frees Vulkan object at end of program; or to denote manual free
    unsafe fn cleanup_recreation(&self, device: &DeviceMTXG);  // frees Vulkan object when swapchain is recreated
}

// blanket trait for all objects supported to cleanup itself to also support for vectors of itself
impl<T: CleanupVkObj> CleanupVkObj for Vec<T> {
    unsafe fn cleanup(&self, device: &DeviceMTXG) {
        for el in self {
            el.cleanup(&device);
        }
    }

    unsafe fn cleanup_recreation(&self, device: &DeviceMTXG) {
        for el in self {
            el.cleanup_recreation(&device);
        }
    }
}


#[derive(Clone)]
pub struct InstanceMTXG {
    pub(crate) debug_mode: bool,
    pub(crate) entry: ash::Entry,
    pub(crate) instance: ash::Instance,
    pub(crate) debug_handler: Option<ext::DebugUtils>,
    pub(crate) debug: Option<vk::DebugUtilsMessengerEXT>,
    pub(crate) surface_handler: khr::Surface,
    pub(crate) surface: vk::SurfaceKHR,
    pub(crate) raw_inst_exts: Vec<*const i8>,
}

impl InstanceMTXG {
    // v1.2.0 only
    pub fn new(window: &Window, app_name: &str, debug_mode: bool, api_version: (u32, u32, u32)) -> Self {
        println!("Debug mode: {:?}", debug_mode);
        if debug_mode {
            println!("creating Vulkan instance...");
        }
        let entry = ash::Entry::new().unwrap();

        let app_name = CString::new(app_name).unwrap();

        let app_info = vk::ApplicationInfo {
            p_application_name: app_name.as_ptr(),
            application_version: 0,
            p_engine_name: std::ptr::null(),
            engine_version: 0,
            api_version: vk::make_version(api_version.0, api_version.1, api_version.2),
            ..Default::default()
        };

        let mut inst_lyrs = vec![];
        let vald_lyr = CString::new("VK_LAYER_KHRONOS_validation").unwrap();
        if debug_mode {
            inst_lyrs.push(vald_lyr.as_c_str());
        }
        let inst_lyrs_raw = inst_lyrs.iter().map(|lyr|lyr.as_ptr()).collect::<Vec<_>>();

        let mut inst_exts = vec![];
        inst_exts.push(ash::extensions::ext::DebugUtils::name());
        inst_exts.append(&mut ash_window::enumerate_required_extensions(window).unwrap());
        let inst_exts_raw = inst_exts.iter().map(|ext|ext.as_ptr()).collect::<Vec<_>>();

        if debug_mode {
            println!("Instance required layers: {:?}", inst_lyrs);
            println!("Instance required extensions: {:?}", inst_exts);
        }

        let debug_cinfo = vk::DebugUtilsMessengerCreateInfoEXT {
            message_severity: vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE |
                vk::DebugUtilsMessageSeverityFlagsEXT::INFO |
                vk::DebugUtilsMessageSeverityFlagsEXT::WARNING |
                vk::DebugUtilsMessageSeverityFlagsEXT::ERROR,
            message_type: vk::DebugUtilsMessageTypeFlagsEXT::GENERAL |
                vk::DebugUtilsMessageTypeFlagsEXT::PERFORMANCE |
                vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION,
            pfn_user_callback: Some(vulkan_validation_debug_callback),
            ..Default::default()
        };

        let inst_cinfo = if debug_mode {
            vk::InstanceCreateInfo {
                p_application_info: &app_info,
                enabled_layer_count: inst_lyrs_raw.len() as u32,
                pp_enabled_layer_names: inst_lyrs_raw.as_ptr(),
                p_next: &debug_cinfo as *const vk::DebugUtilsMessengerCreateInfoEXT as *const c_void,
                enabled_extension_count: inst_exts_raw.len() as u32,
                pp_enabled_extension_names: inst_exts_raw.as_ptr(),
                ..Default::default()
            }
        } else {
            vk::InstanceCreateInfo {
                p_application_info: &app_info,
                enabled_layer_count: 0,
                p_next: std::ptr::null(),
                enabled_extension_count: inst_exts_raw.len() as u32,
                pp_enabled_extension_names: inst_exts_raw.as_ptr(),
                ..Default::default()
            }
        };

        let instance = unsafe { entry.create_instance(&inst_cinfo, None) }.expect("Failed to create a Vulkan instance");

        let debug_handler = if debug_mode {
            Some(ext::DebugUtils::new(&entry, &instance))
        } else {
            None
        };
        let debug = if debug_mode {
            Some(unsafe {
                debug_handler.clone().unwrap().create_debug_utils_messenger(&debug_cinfo, None)
            }.expect("Failed to create debug messenger"))
        } else {
            None
        };

        let surface_handler = khr::Surface::new(&entry, &instance);
        let surface = unsafe { ash_window::create_surface(&entry, &instance, window, None) }.expect("Failed to create a Vulkan surface");

        Self {
            debug_mode: debug_mode,
            entry: entry,
            instance: instance,
            debug_handler: debug_handler,
            debug: debug,
            surface_handler: surface_handler,
            surface: surface,
            raw_inst_exts: inst_lyrs_raw,
        }
    }

    // gets a single device that has Vulkan API & supports the listed extensions in function parameter
    // TODO-CHECKED: Using enum to find device with supported queues without being hardcoded to GRAPHICS | PRESENTATION
    pub fn find_device(&self, devc_ext: Vec<&'static CStr>, feat_anisotropy: bool) -> DeviceMTXG {
        let devc_ext_raw = devc_ext.iter().map(|ext|ext.as_ptr()).collect::<Vec<_>>();

        let phys_devcs = unsafe { self.instance.enumerate_physical_devices() }.unwrap();

        if self.debug_mode {
            println!("Requested device extensions: {:?}", devc_ext);
            println!("Physical devices list: {:?}", phys_devcs);
        }
        if phys_devcs.is_empty() {
            panic!("No compatible physical devices found")
        }

        let mut physical_device = None;
        let mut queue_family_ind_graphics = None;
        let mut queue_family_ind_present = None;

        for devcs in phys_devcs {
            let prop = unsafe { self.instance.get_physical_device_properties(devcs) };
            let feat = unsafe { self.instance.get_physical_device_features(devcs) };
            let exts = unsafe { self.instance.enumerate_device_extension_properties(devcs) }.unwrap();
            let mut exts_raw = exts.iter().map(|ext|ext.extension_name.as_ptr()).collect::<Vec<_>>();
            exts_raw.append(&mut self.raw_inst_exts.clone());

            let queue_fam = unsafe { self.instance.get_physical_device_queue_family_properties(devcs) };

            if self.debug_mode {
                println!("Physical Device: {:?}", devcs);
                println!("\tProperties: {:?}", prop);
                println!("\tFeatures: {:?}", feat);
                println!("\tQueue Families: {:?}", queue_fam);
                println!("\tSupported Extensions: {:?}", exts.iter()
                    .map(|ext| unsafe {
                        CStr::from_ptr(ext.extension_name.as_ptr() as *const c_char)
                    }).collect::<Vec<_>>()
                );
            }

            for r_ext in devc_ext.iter() {  // required extensions
                let mut found = false;
                for s_ext in exts.iter() {  // supported extensions
                    if r_ext == &unsafe { CStr::from_ptr(s_ext.extension_name.as_ptr() as *const c_char) } {
                        found = true;
                    }
                }
                if !found {
                    println!("WARNING: Extension <{:?}> not found", r_ext);
                }
            }

            // if the present queue family index is same as the graphics queue family
            let mut is_same_as_graphics = false;  // tries to use a different queue family
            for (i, qm) in queue_fam.iter().enumerate() {
                let graphics_support = qm.queue_flags.contains(vk::QueueFlags::GRAPHICS);
                let presentation_support = unsafe { self.surface_handler.get_physical_device_surface_support(devcs, i as u32, self.surface) }.expect("Failed to retrieve physical device surface/presentation support");

                if self.debug_mode {
                    println!("Queue Family <{:?}>: Graphics Support <{:?}>, Presentation Support <{:?}>", i, graphics_support, presentation_support);
                }

                if graphics_support && queue_family_ind_graphics.is_none() {
                    queue_family_ind_graphics = Some(i);
                }
                if presentation_support && (queue_family_ind_present.is_none() || is_same_as_graphics) {
                    queue_family_ind_present = Some(i);
                    if let Some(gfx_ind) = queue_family_ind_graphics {
                        is_same_as_graphics = gfx_ind == i;
                    } else {
                        is_same_as_graphics = false;
                    }
                }
            }
            if queue_family_ind_graphics.is_some() && queue_family_ind_present.is_some() {
                physical_device = Some(devcs);
            }
        }

        // panic incompatible queue family first
        let queue_family_ind_graphics = queue_family_ind_graphics.expect("No queue families of the current devices support graphic rendering") as u32;
        let queue_family_ind_present = queue_family_ind_present.expect("No queue families of the current devices support render presentation") as u32;
        let queue_same_ind = queue_family_ind_graphics == queue_family_ind_present;
        let physical_device = physical_device.expect("No physical device found");

        // additional queue count for graphics queue family for transfer queue
        let devc_queue_graphics_cinfo = if queue_same_ind {
            vk::DeviceQueueCreateInfo {
                queue_family_index: queue_family_ind_graphics,
                queue_count: 3,
                p_queue_priorities: [1.0, 1.0, 1.0].as_ptr(),
                ..Default::default()
            }
        } else {
            vk::DeviceQueueCreateInfo {
                queue_family_index: queue_family_ind_graphics,
                queue_count: 2,
                p_queue_priorities: [1.0, 1.0].as_ptr(),
                ..Default::default()
            }
        };

        let devc_queue_present_cinfo = if queue_same_ind {
            None
        } else {
            Some(
                vk::DeviceQueueCreateInfo {
                    queue_family_index: queue_family_ind_present,
                    queue_count: 1,
                    p_queue_priorities: &1.0,
                    ..Default::default()
                }
            )
        };

        let devc_feats = vk::PhysicalDeviceFeatures {
            sampler_anisotropy: if feat_anisotropy {vk::TRUE} else {vk::FALSE},
            ..Default::default()
        };

        let phys_sup_feats = unsafe { self.instance.get_physical_device_features(physical_device) };

        let devc_cinfo = if queue_same_ind {
            vk::DeviceCreateInfo {
                queue_create_info_count: 1,
                p_queue_create_infos: [devc_queue_graphics_cinfo].as_ptr(), // vec![devc_queue_graphics_cinfo].as_ptr(),
                p_enabled_features: &devc_feats,
                enabled_extension_count: devc_ext_raw.len() as u32,
                pp_enabled_extension_names: devc_ext_raw.as_ptr(),
                ..Default::default()
            }
        } else {
            vk::DeviceCreateInfo {
                queue_create_info_count: 2,
                p_queue_create_infos: [devc_queue_graphics_cinfo, devc_queue_present_cinfo.expect("")].as_ptr(), // vec![devc_queue_graphics_cinfo].as_ptr(),
                p_enabled_features: &devc_feats,
                enabled_extension_count: devc_ext_raw.len() as u32,
                pp_enabled_extension_names: devc_ext_raw.as_ptr(),
                ..Default::default()
            }
        };

        // println!("vvvvvvvvvvvvvvvvvvvvvvv [@#]");
        // println!("^^^^^^^^^^^^^^^^^^^^^^^ [@~]");

        let device = unsafe { self.instance.create_device(physical_device, &devc_cinfo, None) }.expect("Failed to create a logical device");

        let queue_graphics = unsafe { device.get_device_queue(queue_family_ind_graphics, 0) };
        let queue_transfer = unsafe { device.get_device_queue(queue_family_ind_graphics, 1) };
        let queue_present = if queue_same_ind {
            unsafe { device.get_device_queue(queue_family_ind_graphics, 2) }
        } else {
            unsafe { device.get_device_queue(queue_family_ind_present, 0) }
        };

        DeviceMTXG {
            instance: self.clone(),
            debug_mode: self.debug_mode,
            physical: physical_device,
            device: device,
            graphics_queue: queue_graphics,
            present_queue: queue_present,
            transfer_queue: queue_transfer,
            graphics_queue_fam_id: queue_family_ind_graphics,
            present_queue_fam_id: queue_family_ind_present,
            transfer_queue_fam_id: queue_family_ind_graphics,  // always under the graphics queue families
            // supported features
            feat_anisotropy: if phys_sup_feats.sampler_anisotropy == vk::TRUE {true} else {false},
        }
    }

    pub unsafe fn cleanup(&self,
                          device: &DeviceMTXG,
                          objects: Vec<&dyn CleanupVkObj>) {
        let device = device.clone();

        device.device.device_wait_idle().unwrap();
        if self.debug_mode {
            println!("Cleanup initiated");
        }

        for obj in objects {
            obj.cleanup(&device);
        }

        // swapchain;pipeline;renderpasses;framebuffer;exec-render. cleanup

        device.device.destroy_device(None);
        if self.debug_mode {
            self.debug_handler.as_ref().unwrap().destroy_debug_utils_messenger(self.debug.unwrap(), None);
        }
        self.surface_handler.destroy_surface(self.surface, None);
        self.instance.destroy_instance(None);
    }
}


unsafe extern "system" fn vulkan_validation_debug_callback(
    msg_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    msg_type: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _p_user_data: *mut c_void,
) -> vk::Bool32 {
    let callback_data = *p_callback_data;
    let message_id_number: i32 = callback_data.message_id_number as i32;

    let message_id_name = if callback_data.p_message_id_name.is_null() {
        Cow::from("")
    } else {
        CStr::from_ptr(callback_data.p_message_id_name).to_string_lossy()
    };

    let message = if callback_data.p_message.is_null() {
        Cow::from("")
    } else {
        CStr::from_ptr(callback_data.p_message).to_string_lossy()
    };

    println!(
        "[{:?}] {:?}: {} ({}):\n{}",
        msg_severity,
        msg_type,
        message_id_name,
        &message_id_number.to_string(),
        message,
    );

    vk::FALSE
}
