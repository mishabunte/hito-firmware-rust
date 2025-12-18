extern crate alloc;
use alloc::rc::Rc;

use crate::slint_generatedMainWindow::MainWindow;
mod pair_with_the_app_screen;
mod menu_screen;
mod home_screen;
mod factory_reset_screen;
mod enter_passcode_screen;
mod lock_screen;
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
    FactoryResetErase
}

pub fn create_screen(ui: &Rc<MainWindow>, screen: Screen) {
    ui.invoke_clear_screen();
    match screen {
        Screen::Lock => lock_screen::create_lock_screen(ui),
        Screen::EnterPasscode => enter_passcode_screen::create_enter_passcode_screen(ui, Screen::Home), // Handled separately in lock screen
        Screen::Menu => menu_screen::create_menu_screen(ui),
        Screen::Home => home_screen::create_home_screen(ui),
        Screen::PairWithApp => pair_with_the_app_screen::create_pair_with_app_screen(ui),
        Screen::FactoryReset => factory_reset_screen::create_factory_reset_screen(ui),
        Screen::FactoryResetPasscode => enter_passcode_screen::create_enter_passcode_screen(ui, Screen::FactoryResetErase),
        Screen::FactoryResetErase => 
        factory_reset_screen::create_erase_screen(ui),
    }
}