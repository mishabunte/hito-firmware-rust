extern crate alloc;
use alloc::rc::Rc;

use crate::{QR_CODE, slint_generatedMainWindow::MainWindow};
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

// Re-export loop handlers and cleanup functions
pub use send_screen::{handle_send_screen_loop, cleanup_send_screen};
pub use show_seed_screen::{handle_show_seed_loop};
mod generic_question_screen;

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
    ShowSeed,
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
        Screen::ShowSeed => show_seed_screen::create_show_seed_screen(ui),
        Screen::SendStellar => send_stellar_screen::create_send_stellar_screen(ui),
        Screen::Send => send_screen::create_send_screen(ui),
        Screen::Lock => lock_screen::create_lock_screen(ui),
        Screen::EnterPasscode => enter_passcode_screen::create_enter_passcode_screen(ui, Screen::Home), // Handled separately in lock screen
        Screen::Menu => menu_screen::create_menu_screen(ui),
        Screen::Home => home_screen::create_home_screen(ui),
        Screen::PairWithApp => pair_with_the_app_screen::create_pair_with_app_screen(ui),
        Screen::FactoryReset => factory_reset_screen::create_factory_reset_screen(ui),
        Screen::Receive => receive_screen::create_receive_screen(ui),
        Screen::FactoryResetPasscode => enter_passcode_screen::create_enter_passcode_screen(ui, Screen::FactoryResetErase),
        Screen::FactoryResetErase => 
        factory_reset_screen::create_erase_screen(ui),
    }
}