//! Screen creation functions for the firmware UI
//! 
//! This module contains functions to create and configure each screen in the application.
//! Each function sets up the UI items, header, and callbacks for its respective screen.

extern crate alloc;
use alloc::string::String;

use alloc::vec;
use alloc::rc::Rc;

use slint::ModelRc;
use slint::VecModel;

use crate::slint_generatedMainWindow::{MainWindow, ScreenItem, ScreenButton, ScreenImage};
use crate::log_info;
use crate::ui::router::{navigate_to, go_back};

use super::Screen;

#[cfg(feature = "zephyr")]
pub fn shuffle_digits(mut arr: [u8; 10]) -> [u8; 10] {
    use crate::crypto::ffi::hito_sys_rand32_get;
    fn rand10() -> usize {
        unsafe { (hito_sys_rand32_get() % 10) as usize }
    }
    for _ in 0..128 {
        let i = rand10();
        let j = rand10();

        if i != j {
            arr.swap(i, j);
        }
    }
    arr
}

#[cfg(feature = "minifb")]
pub fn shuffle_digits(mut arr: [u8; 10]) -> [u8; 10] {
    use rand::Rng;
    let mut rng = rand::thread_rng();

    for _ in 0..128 {
        let i = rng.gen_range(0..10);
        let j = rng.gen_range(0..10);

        if i != j {
            arr.swap(i, j);
        }
    }

    arr
}

/// Create the "Enter Passcode" screen
pub fn create_enter_passcode_screen(ui: &Rc<MainWindow>) {
    let char_button_x = 10.0;
    let char_button_y = 115.0;

    let char_button_w = 60.0;
    let char_button_h = 60.0;

    ui.set_header_title(slint::SharedString::from("Enter Passcode"));

    let arr = [b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9'];
    let shuffled = shuffle_digits(arr);

    let mut passcode_entered: String = String::new();

    let buttons = ModelRc::new(VecModel::from(vec![
        ScreenButton { 
            text: (shuffled[0] as char).to_ascii_lowercase().into(),  
            width: char_button_w, 
            height: char_button_h,
            has_border: false, 
            x: char_button_x, 
            y: char_button_y,
            inverted: true,
        },
        ScreenButton { 
            text: (shuffled[1] as char).to_ascii_lowercase().into(),  
            width: char_button_w, 
            height: char_button_h,
            has_border: false, 
            x: char_button_x + char_button_w, 
            y: char_button_y,
            inverted: true,
        },
        ScreenButton { 
            text: (shuffled[2] as char).to_ascii_lowercase().into(),  
            width: char_button_w, 
            height: char_button_h,
            has_border: false, 
            x: char_button_x + char_button_w * 2.0, 
            y: char_button_y,
            inverted: true,
        },
        ScreenButton { 
            text: (shuffled[3] as char).to_ascii_lowercase().into(),  
            width: char_button_w, 
            height: char_button_h,
            has_border: false, 
            x: char_button_x + char_button_w * 3.0, 
            y: char_button_y,
            inverted: true,
        },
        ScreenButton { 
            text: (shuffled[4] as char).to_ascii_lowercase().into(),  
            width: char_button_w, 
            height: char_button_h,
            has_border: false, 
            x: char_button_x + char_button_w * 4.0, 
            y: char_button_y,
            inverted: true,
        },
        ScreenButton { 
            text: (shuffled[5] as char).to_ascii_lowercase().into(),  
            width: char_button_w, 
            height: char_button_h,
            has_border: false, 
            x: char_button_x, 
            y: char_button_y + char_button_h,
            inverted: true,
        },
        ScreenButton { 
            text: (shuffled[6] as char).to_ascii_lowercase().into(),  
            width: char_button_w, 
            height: char_button_h,
            has_border: false, 
            x: char_button_x + char_button_w, 
            y: char_button_y + char_button_h,
            inverted: true,
        },
        ScreenButton { 
            text: (shuffled[7] as char).to_ascii_lowercase().into(),  
            width: char_button_w, 
            height: char_button_h,
            has_border: false, 
            x: char_button_x + char_button_w * 2.0, 
            y: char_button_y + char_button_h,
            inverted: true,
        },
        ScreenButton { 
            text: (shuffled[8] as char).to_ascii_lowercase().into(),  
            width: char_button_w, 
            height: char_button_h,
            has_border: false, 
            x: char_button_x + char_button_w * 3.0, 
            y: char_button_y + char_button_h,
            inverted: true,
        },
        ScreenButton { 
            text: (shuffled[9] as char).to_ascii_lowercase().into(),  
            width: char_button_w, 
            height: char_button_h,
            has_border: false, 
            x: char_button_x + char_button_w * 4.0, 
            y: char_button_y + char_button_h,
            inverted: true,
        },
        ScreenButton { 
            text: "<".into(),  
            width: char_button_w, 
            height: char_button_h,
            has_border: false, 
            x: char_button_x + char_button_w * 4.0 - char_button_w / 3.0, 
            y: char_button_y - char_button_h,
            inverted: true,
        },
    ]));
    ui.set_buttons(buttons);
    let ui_weak = Rc::downgrade(ui);
    
    ui.on_pressed(move |item| {
        let Some(ui) = ui_weak.upgrade() else { return };
        if item.text.parse::<u8>().is_ok() {
          if passcode_entered.len() >= 5 {
              navigate_to(Screen::Home);
          }
            passcode_entered.push_str(&item.text.to_ascii_lowercase());
        }
        if item.text == "<" {
            passcode_entered.pop();
        }
        let passcode_displayed = "* ".repeat(passcode_entered.len());
        let items = ModelRc::new(VecModel::from(vec![
            ScreenItem { 
                text: passcode_displayed.clone().into(), 
                width: 320.0 - char_button_x * 2.0 - char_button_w * 2.0 - char_button_w * 2.0 / 3.0, 
                height: 25.0, 
                x: char_button_x + char_button_w + char_button_w / 3.0, 
                y: char_button_y - char_button_h / 2.0 - 10.0, 
            },
        ]));
        ui.set_items(items);
    });
}
