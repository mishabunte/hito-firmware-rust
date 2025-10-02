use alloc::rc::{Rc};
use core::cell::{Cell};
use alloc::vec::Vec;
use slint::ComponentHandle;
use crate::log_info;

use crate::{LockScreen, TestScreen, TestScreen2, Router, BrightnessController, EnterPinScreen, HomeScreen, ScreenEnum, SendScreen, ReceiveScreen, SettingsScreen};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScreenType { 
    Lock,
    TestScreen,
    EnterPin, 
    TestScreen2, 
    Home,
    Send,
    Receive,
    Settings,
}

const MAX_HISTORY: usize = 16; // max history length for back navigation

/// Screen types available in the application
pub enum Screen {
    Lock(LockScreen),
    Test(TestScreen),
    EnterPin(EnterPinScreen),
    Test2(TestScreen2),
    Home(HomeScreen),
    Send(SendScreen),
    Receive(ReceiveScreen),
    Settings(SettingsScreen),
}

impl Screen {
    fn show(&self) -> Result<(), slint::PlatformError> {
        match self {
            Screen::Lock(c)     => c.show(),
            Screen::Test(c)     => c.show(),
            Screen::EnterPin(c) => c.show(),
            Screen::Test2(c)    => c.show(),
            Screen::Home(c)     => c.show(),
            Screen::Send(c)     => c.show(),
            Screen::Receive(c)  => c.show(),
            Screen::Settings(c) => c.show(),
        }
    }
    fn hide(&self) -> Result<(), slint::PlatformError> {
        match self {
            Screen::Lock(c)     => c.hide(),
            Screen::Test(c)     => c.hide(),
            Screen::EnterPin(c) => c.hide(),
            Screen::Test2(c)    => c.hide(),
            Screen::Home(c)     => c.hide(),
            Screen::Send(c)     => c.hide(),
            Screen::Receive(c)  => c.hide(),
            Screen::Settings(c) => c.hide(),
        }
    }
}

fn slint_screen_to_type(screen: ScreenEnum) -> ScreenType {
    match screen {
        ScreenEnum::Lock => ScreenType::Lock,
        ScreenEnum::TestScreen => ScreenType::TestScreen,
        ScreenEnum::EnterPin => ScreenType::EnterPin,
        ScreenEnum::TestScreen2 => ScreenType::TestScreen2,
        ScreenEnum::Home => ScreenType::Home,
        ScreenEnum::Send => ScreenType::Send,
        ScreenEnum::Receive => ScreenType::Receive,
        ScreenEnum::Settings => ScreenType::Settings,
    }
}

fn make_screen(kind: ScreenType, to_next: &Rc<Cell<Option<ScreenType>>>, to_prev: &Rc<Cell<bool>>, brightness_request: &Rc<Cell<Option<u8>>>, battery_request: &Rc<Cell<bool>>) -> Result<Screen, slint::PlatformError>
{
    match kind {
        ScreenType::Lock => {
            let c = LockScreen::new()?;
            {
                let to_next_request = Rc::clone(to_next);
                c.global::<Router>().on_navigate(move |screen| {
                    // Convert slint Screen enum to ScreenType
                    let screen_type = slint_screen_to_type(screen);
                    to_next_request.set(Some(screen_type));
                });

                let to_prev_flag = Rc::clone(to_prev);
                c.global::<Router>().on_back(move || to_prev_flag.set(true));

                let br = brightness_request.clone();
                c.global::<BrightnessController>().on_brightness_changed(move |brightness_value| {
                    br.set(Some(brightness_value as u8));
                });
            }
            Ok(Screen::Lock(c))
        }
        ScreenType::TestScreen => {
            let c = TestScreen::new()?;
            {
                let to_next_request = Rc::clone(to_next);
                c.global::<Router>().on_navigate(move |screen| {
                    // Convert slint Screen enum to ScreenType
                    let screen_type = slint_screen_to_type(screen);
                    to_next_request.set(Some(screen_type));
                });

                let to_prev_flag = Rc::clone(to_prev);
                c.global::<Router>().on_back(move || to_prev_flag.set(true));

                let br = brightness_request.clone();
                c.global::<BrightnessController>().on_brightness_changed(move |brightness_value| {
                    br.set(Some(brightness_value as u8));
                });
            }
            Ok(Screen::Test(c))
        }
        ScreenType::EnterPin => {
            let c = EnterPinScreen::new()?;
            {
                let to_next_request = Rc::clone(to_next);
                c.global::<Router>().on_navigate(move |screen| {
                    // Convert slint Screen enum to ScreenType
                    let screen_type = slint_screen_to_type(screen);
                    to_next_request.set(Some(screen_type));
                });

                let to_prev_flag = Rc::clone(to_prev);
                c.global::<Router>().on_back(move || to_prev_flag.set(true));

                let br = brightness_request.clone();
                c.global::<BrightnessController>().on_brightness_changed(move |brightness_value| {
                    br.set(Some(brightness_value as u8));
                });
            }
            Ok(Screen::EnterPin(c))
        }
        ScreenType::TestScreen2 => {
            let c = TestScreen2::new()?;
            {
                let to_next_request = Rc::clone(to_next);
                c.global::<Router>().on_navigate(move |screen| {
                    // Convert slint Screen enum to ScreenType
                    let screen_type = slint_screen_to_type(screen);
                    to_next_request.set(Some(screen_type));
                });

                let to_prev_flag = Rc::clone(to_prev);
                c.global::<Router>().on_back(move || to_prev_flag.set(true));

                let br = brightness_request.clone();
                c.global::<BrightnessController>().on_brightness_changed(move |brightness_value| {
                    br.set(Some(brightness_value as u8));
                });
            }
            Ok(Screen::Test2(c))
        }
        ScreenType::Home => {
            let c = HomeScreen::new()?;
            {
                let to_next_request = Rc::clone(to_next);
                c.global::<Router>().on_navigate(move |screen| {
                    // Convert slint Screen enum to ScreenType
                    let screen_type = slint_screen_to_type(screen);
                    to_next_request.set(Some(screen_type));
                });

                let to_prev_flag = Rc::clone(to_prev);
                c.global::<Router>().on_back(move || to_prev_flag.set(true));

                let br = brightness_request.clone();
                c.global::<BrightnessController>().on_brightness_changed(move |brightness_value| {
                    br.set(Some(brightness_value as u8));
                });
            }
            Ok(Screen::Home(c))
        }
        ScreenType::Receive => {
            let c = ReceiveScreen::new()?;
            {
                let to_next_request = Rc::clone(to_next);
                c.global::<Router>().on_navigate(move |screen| {
                    // Convert slint Screen enum to ScreenType
                    let screen_type = slint_screen_to_type(screen);
                    to_next_request.set(Some(screen_type));
                });

                let to_prev_flag = Rc::clone(to_prev);
                c.global::<Router>().on_back(move || to_prev_flag.set(true));

                let br = brightness_request.clone();
                c.global::<BrightnessController>().on_brightness_changed(move |brightness_value| {
                    br.set(Some(brightness_value as u8));
                });
            }
            Ok(Screen::Receive(c))
        }
        ScreenType::Send => {
            let c = SendScreen::new()?;
            {
                let to_next_request = Rc::clone(to_next);
                c.global::<Router>().on_navigate(move |screen| {
                    // Convert slint Screen enum to ScreenType
                    let screen_type = slint_screen_to_type(screen);
                    to_next_request.set(Some(screen_type));
                });

                let to_prev_flag = Rc::clone(to_prev);
                c.global::<Router>().on_back(move || to_prev_flag.set(true));

                let br = brightness_request.clone();
                c.global::<BrightnessController>().on_brightness_changed(move |brightness_value| {
                    br.set(Some(brightness_value as u8));
                });
            }
            Ok(Screen::Send(c))
        }
        ScreenType::Settings => {
            let c = SettingsScreen::new()?;
            {
                let to_next_request = Rc::clone(to_next);
                c.global::<Router>().on_navigate(move |screen| {
                    // Convert slint Screen enum to ScreenType
                    let screen_type = slint_screen_to_type(screen);
                    to_next_request.set(Some(screen_type));
                });

                let to_prev_flag = Rc::clone(to_prev);
                c.global::<Router>().on_back(move || to_prev_flag.set(true));

                let br = brightness_request.clone();
                c.global::<BrightnessController>().on_brightness_changed(move |brightness_value| {
                    br.set(Some(brightness_value as u8));
                });
            }
            Ok(Screen::Settings(c))
        }
    }
}

pub struct ScreenManager {
    pub current: Option<Screen>,      // small handle on stack; real memory lives in Slint internals
    to_next: Rc<Cell<Option<ScreenType>>>,
    to_prev: Rc<Cell<bool>>,
    pub brightness_request: Rc<Cell<Option<u8>>>,
    pub battery_request: Rc<Cell<bool>>,
    current_type: Option<ScreenType>,
    history: Vec<ScreenType>, // for back navigation
}
    

impl ScreenManager {
    pub fn new() -> Result<Self, slint::PlatformError> {
        let to_next = Rc::new(Cell::new(None));
        let to_prev = Rc::new(Cell::new(false));
        let brightness_request = Rc::new(Cell::new(None));
        let battery_request = Rc::new(Cell::new(false));

        Ok(Self {
            current: None,
            current_type: None,
            to_next,
            to_prev,
            brightness_request,
            battery_request,
            history: Vec::new()
        })
    }
    /// Internal core switch: does NOT push to history.
    fn switch_to(&mut self, target: ScreenType) -> Result<(), slint::PlatformError> {
        if self.current_type == Some(target) { return Ok(()); }
        if let Some(old) = self.current.take() {
            let _ = old.hide();
            drop(old);
        }

        let new = make_screen(
            target,
            &self.to_next,
            &self.to_prev,
            &self.brightness_request,
            &self.battery_request,
        )?;
        new.show()?;
        self.current = Some(new);
        self.current_type = Some(target);
        Ok(())
    }

    /// Forward navigation: push current onto history, then switch.
    pub fn navigate_to(&mut self, target: ScreenType, on_screen_changed: impl FnOnce()) -> Result<(), slint::PlatformError> {
        if self.current_type != Some(target) {
            // cap history length on embedded if you want (e.g., 16)
            if self.history.len() == MAX_HISTORY { self.history.remove(0); }
            if self.current_type.is_some() {
                self.history.push(self.current_type.unwrap());
            }
        }
        on_screen_changed();
        self.switch_to(target)
    }

    /// Back navigation: pop previous route and switch without pushing again.
    pub fn navigate_back(&mut self) -> Result<(), slint::PlatformError> {
        if let Some(prev) = self.history.pop() {
            self.switch_to(prev)
        } else {
            // No history — optional: stay put or define a fallback
            Ok(())
        }
    }

    pub fn handle_navigation(&mut self, on_screen_changed: impl FnOnce()) -> Result<(), slint::PlatformError> {
        // Atomically read & reset
        let back = self.to_prev.replace(false);
        let next_screen = self.to_next.replace(None);

        if !back && next_screen.is_none() {
            return Ok(()); // no navigation requested
        }

        if back {
            log_info!("I'm going back to 505");
            self.navigate_back()?;
            return Ok(());
        }

        if let Some(target) = next_screen {
            log_info!("Navigating to {:?}", target);
            self.navigate_to(target, on_screen_changed)?;
        }

        Ok(())
    }
}