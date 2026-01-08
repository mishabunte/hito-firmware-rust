//! Screen creation functions for the firmware UI
//! 
//! This module contains functions to create and configure each screen in the application.
//! Each function sets up the UI items, header, and callbacks for its respective screen.

extern crate alloc;
use alloc::vec;
use alloc::rc::Rc;

use slint::ModelRc;
use slint::VecModel;

use crate::firmware;
use crate::slint_generatedMainWindow::{MainWindow, ScreenItem};
use crate::log_info;
use crate::ui::router::{navigate_to, go_back};
use crate::ui::screens::show_alert;
use crate::vault::firmware_version;
use slint::format;

use super::Screen;

/// Create the main menu screen with navigation options
pub fn create_device_info_screen(ui: &Rc<MainWindow>) {
    ui.set_header_title(slint::SharedString::from("DEVICE INFO"));

    let item_x = 0.0;
    let item_y = 42.0;
    let item_w = 320.0 - item_x * 2.0;
    let item_h = 35.0;

    let info = firmware().vault.lock().get_device_info();

    if let Err(e) = info {
        log_info!("Error getting device info: {:?}", e);
        show_alert("\\\\Failed to get device info");
    } else {
        let info = info.unwrap();

        ui.set_center_text(true);

        let factory_reset_string = match info.factory_reset_count {
            n if n < 0 => "Factory reset no data".into(),
            0 => "Factory setup".into(),
            n if n > 30 => "Factory reset 30 plus".into(),
            n => format!("Factory reset {} times", n-1),
        };

        let items = ModelRc::new(VecModel::from(vec![
            ScreenItem { 
                text: format!("{}", info.firmware_version), 
                width: item_w, 
                height: item_h, 
                x: item_x, 
                y: 60.0,
            },
            ScreenItem { 
                text: format!("Bootloader: {}", info.bootloader_version), 
                width: item_w, 
                height: item_h, 
                x: item_x, 
                y: 85.0,
            },
            ScreenItem { 
                text: "Serial number".into(), 
                width: item_w, 
                height: item_h, 
                x: item_x, 
                y: 120.0,
            },
            ScreenItem { 
                text: format!("{}", info.serial_number),
                width: item_w, 
                height: item_h, 
                x: item_x, 
                y: 145.0,
            },
            ScreenItem { 
                text: factory_reset_string, 
                width: item_w, 
                height: item_h, 
                x: item_x, 
                y: 180.0,
            },
        ]));
        
        ui.set_items(items);
    }
}