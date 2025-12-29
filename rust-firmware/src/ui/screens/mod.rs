extern crate alloc;

use alloc::rc::Rc;

use crate::{QR_CODE, slint_generatedMainWindow::MainWindow, ui::{navigate_to, screens::enter_passcode_screen::{create_confirm_passcode_screen, create_finalize_passcode_change_screen, create_set_passcode_screen}}};
mod pair_with_the_app_screen;
mod menu_screen;
mod home_screen;
mod factory_reset_screen;
mod enter_passcode_screen;
mod lock_screen;
mod receive_screen;
mod send_screen;
mod send_stellar_screen;
mod show_seed_screen;
mod enter_seed_screen;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use alloc::vec;
use slint::ModelRc;
use slint::VecModel;
use crate::slint_generatedMainWindow::ScreenItem;

// Re-export loop handlers and cleanup functions
pub use send_screen::{handle_send_screen_loop, cleanup_send_screen};
pub use show_seed_screen::{handle_show_seed_loop};
pub use enter_seed_screen::{handle_enter_seed_loop, cleanup_enter_seed_screen, init_seed_check};
mod generic_question_screen;
mod device_info_screen;

static mut ERROR_MESSAGE: Option<String> = None;

/// Enum representing all available screens in the application
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Screen {
    Lock,
    EnterPasscode,
    Home,
    Menu,
    PairWithApp,
    FactoryReset,
    FactoryResetPasscode,
    FactoryResetErase,
    Receive,
    Send,
    SendStellar,
    ShowSeedPasscode,
    ShowSeed,
    ShowSeedBackup,
    EnterSeed,
    SeedCheck,
    WalletSetup,
    EncryptingSeed,
    GenerateSeed,
    EnterCurrentPasscode,
    SetNewPasscode,
    ChangePasscodeConfirm,
    FinalizePasscodeChange,
    Alert,
    DeviceInfo,
}

pub fn show_alert(alert: &str) {
  unsafe {
      ERROR_MESSAGE = Some(alert.to_string());
  }
  navigate_to(Screen::Alert); // Ensure we are on a known screen before showing alert
}

pub fn create_alert_screen(ui: &Rc<MainWindow>, alert: &str) {
    let button_x = 0.0;
    let button_y = 62.0;
    let button_gap = 28.0;
    let button_w = 130.0;
    let button_h = 40.0;

    let cancel_x = 20.0;
    let cancel_y = 190.0;
    let confirm_x = 170.0;
    let confirm_y = 190.0;
    
    ui.set_center_text(true);
    // parse question into lines if too long, \\ is line break
    let alert_lines: Vec<&str> = alert.split("\\\\").collect();

    let lines = alert_lines.len();

    // Create screen items based on number of lines
    let mut items_vec = vec![];
    for (i, line) in alert_lines.iter().enumerate() {
        items_vec.push(ScreenItem { 
            text: (*line).into(), 
            width: 320.0,
            height: 25.0,
            x: button_x, 
            // add a 10.0 gap between 2. and 3. line
            y: if i > 1 { button_y + (i as f32) * button_gap + 10.0 } else { button_y + (i as f32) * button_gap },
        });
    }
    
    let items = ModelRc::new(VecModel::from(items_vec));
    
    ui.set_items(items);
    ui.set_header_title(slint::SharedString::from("ALERT"));
}


fn clear_qr_buffer() {
    if let Some(ref mut _qr) = unsafe { crate::common::QR_CODE.as_mut() } {
        unsafe {
          QR_CODE = None;
        }
    }
}

fn clear_screen_data(ui: &Rc<MainWindow>) {
    ui.invoke_clear_screen();
    clear_qr_buffer();
    ui.on_press(|_| {});
    ui.on_pressed(|_| {});
    ui.set_is_lockscreen(false);
}

pub fn create_screen(ui: &Rc<MainWindow>, screen: Screen) {
    clear_screen_data(ui);
    match screen {
        Screen::WalletSetup => enter_seed_screen::create_wallet_setup_screen(ui),
        Screen::EnterSeed => enter_seed_screen::create_enter_seed_screen(ui, false),
        Screen::SeedCheck => enter_seed_screen::create_enter_seed_screen(ui, true),
        Screen::GenerateSeed => enter_seed_screen::create_generate_seed_screen(ui),
        Screen::ShowSeed => show_seed_screen::create_show_seed_screen(ui, false),
        Screen::ShowSeedBackup => show_seed_screen::create_show_seed_screen(ui, true),
        Screen::SendStellar => send_stellar_screen::create_send_stellar_screen(ui),
        Screen::Send => send_screen::create_send_screen(ui),
        Screen::Lock => lock_screen::create_lock_screen(ui),
        Screen::EnterPasscode => enter_passcode_screen::create_enter_passcode_screen(ui, Screen::Home),
        Screen::Menu => menu_screen::create_menu_screen(ui),
        Screen::Home => home_screen::create_home_screen(ui),
        Screen::Alert => {
            let alert_message = unsafe {
                ERROR_MESSAGE.take().unwrap_or_else(|| "Unknown error".to_string())
            };
            create_alert_screen(ui, &alert_message);
        },
        Screen::PairWithApp => pair_with_the_app_screen::create_pair_with_app_screen(ui),
        Screen::FactoryReset => factory_reset_screen::create_factory_reset_screen(ui),
        Screen::Receive => receive_screen::create_receive_screen(ui),
        Screen::ShowSeedPasscode => enter_passcode_screen::create_enter_passcode_screen(ui, Screen::ShowSeed),
        Screen::EncryptingSeed => enter_seed_screen::create_encrypting_seed_screen(ui),
        Screen::FactoryResetPasscode => enter_passcode_screen::create_enter_passcode_screen(ui, Screen::FactoryResetErase),
        Screen::FactoryResetErase => 
        factory_reset_screen::create_erase_screen(ui),
        Screen::DeviceInfo => device_info_screen::create_device_info_screen(ui),
        Screen::EnterCurrentPasscode => enter_passcode_screen::create_enter_passcode_screen(ui, Screen::SetNewPasscode),
        Screen::SetNewPasscode => create_set_passcode_screen(ui),
        Screen::FinalizePasscodeChange => create_finalize_passcode_change_screen(ui),
        Screen::ChangePasscodeConfirm => create_confirm_passcode_screen(ui),
    }
}