use std::{fs, mem};
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::ptr::null_mut;

use winapi::shared::minwindef::DWORD;
use winapi::shared::windef::POINTL;
use winapi::um::wingdi::{DEVMODE_u1, DEVMODEW, DISPLAY_DEVICEW, DMDO_270, DMDO_DEFAULT};
use winapi::um::winnt::LONG;
use winapi::um::winuser::{CDS_TEST, CDS_UPDATEREGISTRY, ChangeDisplaySettingsExW, DISP_CHANGE_BADMODE, DISP_CHANGE_FAILED, DISP_CHANGE_SUCCESSFUL, ENUM_CURRENT_SETTINGS, EnumDisplayDevicesW, EnumDisplaySettingsExW};

// DMDO_DEFAULT    0
// DMDO_90         1
// DMDO_180        2
// DMDO_270        3 // <- portrait flipped


fn main() {
    // define handle for display device
    let mut display_device: DISPLAY_DEVICEW = unsafe { mem::zeroed() };
    display_device.cb = mem::size_of::<DISPLAY_DEVICEW>() as u32;

    // Enumerate display devices
    let device_index: DWORD = 1;
    let _ = unsafe {
        EnumDisplayDevicesW(
            null_mut(),
            device_index,
            &mut display_device,
            0,
        )
    };

    if (display_device.StateFlags & 1) == 0 {
        panic!("Enumeration complete, no secondary monitor found");
    }

    // Get the current display settings
    let mut dev_mode: DEVMODEW = unsafe { mem::zeroed() };
    dev_mode.dmSize = mem::size_of::<DEVMODEW>() as u16;

    // get current display settings for given device
    let success = unsafe {
        EnumDisplaySettingsExW(
            display_device.DeviceName.as_ptr(),
            ENUM_CURRENT_SETTINGS,
            &mut dev_mode,
            CDS_TEST)
    };

    if success == 0 {
        panic!("Failed to get current display settings.");
    }

    let current_orientation = unsafe { dev_mode.u1.s2_mut().dmDisplayOrientation };
    let new_orientation = if current_orientation == 3 { DMDO_DEFAULT } else { DMDO_270 };

    // save previous position settings
    save_position_for_current_orientation(&mut dev_mode, current_orientation);

    // swap width and height if portrait mode
    (dev_mode.dmPelsWidth, dev_mode.dmPelsHeight) = (dev_mode.dmPelsHeight, dev_mode.dmPelsWidth);

    // Update the orientation in the dev_mode struct
    let mut u1: DEVMODE_u1 = unsafe { mem::zeroed() };
    unsafe { u1.s2_mut().dmDisplayOrientation = new_orientation; }
    dev_mode.u1 = u1;

    // read previous position for given orientation if exists
    read_position_for_new_orientation(&mut dev_mode, new_orientation);

    // Apply the new display settings
    let result = unsafe {
        ChangeDisplaySettingsExW(
            display_device.DeviceName.as_ptr(),
            &mut dev_mode,
            null_mut(),
            CDS_UPDATEREGISTRY,
            null_mut(),
        )
    };

    match result {
        DISP_CHANGE_SUCCESSFUL => {
            println!("Display orientation changed successfully.");
        }
        DISP_CHANGE_BADMODE => {
            println!("Invalid display mode specified.");
        }
        DISP_CHANGE_FAILED => {
            println!("Failed to change display settings.");
        }
        _ => {
            println!("Unexpected result from ChangeDisplaySettingsExW: {}", result);
        }
    }
}

fn read_position_for_new_orientation(dev_mode: &mut DEVMODEW, orientation: u32) {
    let file_name = format!("orientation{}.txt", orientation);
    let path = Path::new(&file_name);

    if path.exists() {
        let mut file = File::open(file_name).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).ok();

        let mut split = contents.trim()
            .split(" ")
            .map(|x| x.parse::<i64>().unwrap())
            .map(|x1| x1 as LONG);
        let (x, y) = (split.next().unwrap(), split.next().unwrap());

        unsafe { dev_mode.u1.s2_mut().dmPosition = POINTL { x, y } };
    }
}

fn save_position_for_current_orientation(dev_mode: &mut DEVMODEW, orientation: u32) {
    let current_position: POINTL = unsafe { dev_mode.u1.s2().dmPosition };

    let string = format!("{} {}", current_position.x, current_position.y);
    let file_name = format!("orientation{}.txt", orientation);

    fs::write(file_name, string).ok();
}
