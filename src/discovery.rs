use super::Scanner;
use super::uniden::UBC125XLT;
use libusb::Context;


const VENDOR_ID_UNIDEN: u16 = 0x1965;
const PRODUCT_ID_UBC125XLT: u16 = 0x0018;

pub fn find_device<'a>(context: &'a Context) -> Option<Box<dyn Scanner + 'a>> {
    let devices = if let Ok(devices) = context.devices() {
        devices
    } else {
        return None;
    };

    for device in devices.iter() {
        let descriptor = if let Ok(descriptor) = device.device_descriptor() {
            descriptor
        } else {
            continue;
        };

        let handle = if let Ok(handle) = device.open() {
            handle
        } else {
            continue;
        };

        match (descriptor.vendor_id(), descriptor.product_id()) {
            (VENDOR_ID_UNIDEN, PRODUCT_ID_UBC125XLT) => {
                if let Ok(scanner) = UBC125XLT::new(handle) {
                    return Some(Box::new(scanner));
                };
            },
            _ => continue,
        };
    };

    None
}
