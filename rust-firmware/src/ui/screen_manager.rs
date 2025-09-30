use alloc::boxed::Box;
use alloc::rc::{Rc, Weak};
use core::cell::{RefCell, Cell};
use core::any::Any;
use slint::ComponentHandle;
use crate::log_info;

use crate::{LockScreen, TestScreen, TestScreen2, Router, BrightnessController, BatteryController, EnterPinScreen};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScreenType { Lock, TestScreen, EnterPin, TestScreen2 }

/// Screen types available in the application
pub enum Screen {
    Lock(LockScreen),
    Test(TestScreen),
    EnterPin(EnterPinScreen),
    Test2(TestScreen2),
}

impl Screen {
    fn show(&self) -> Result<(), slint::PlatformError> {
        match self {
            Screen::Lock(c)     => c.show(),
            Screen::Test(c)     => c.show(),
            Screen::EnterPin(c) => c.show(),
            Screen::Test2(c)    => c.show(),
        }
    }
    fn hide(&self) -> Result<(), slint::PlatformError> {
        match self {
            Screen::Lock(c)     => c.hide(),
            Screen::Test(c)     => c.hide(),
            Screen::EnterPin(c) => c.hide(),
            Screen::Test2(c)    => c.hide(),
        }
    }
}

fn make_screen(kind: ScreenType, to_next: &Rc<Cell<bool>>, brightness_request: &Rc<Cell<Option<u8>>>, battery_request: &Rc<Cell<bool>>) -> Result<Screen, slint::PlatformError>
{
    match kind {
        ScreenType::Lock => {
            let c = LockScreen::new()?;
            {
                let flag = Rc::clone(to_next);
                c.global::<Router>().on_navigate(move || flag.set(true));
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
                let flag = Rc::clone(to_next);
                c.global::<Router>().on_navigate(move || flag.set(true));
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
                let flag = Rc::clone(to_next);
                c.global::<Router>().on_navigate(move || flag.set(true));
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
                let flag = Rc::clone(to_next);
                c.global::<Router>().on_navigate(move || flag.set(true));
                let br = brightness_request.clone();
                c.global::<BrightnessController>().on_brightness_changed(move |brightness_value| {
                    br.set(Some(brightness_value as u8));
                });
            }
            Ok(Screen::Test2(c))
        }
    }
}

pub struct ScreenManager {
    pub current: Option<Screen>,      // small handle on stack; real memory lives in Slint internals
    to_next: Rc<Cell<bool>>,
    pub brightness_request: Rc<Cell<Option<u8>>>,
    pub battery_request: Rc<Cell<bool>>,
    current_type: ScreenType,
}
    

impl ScreenManager {
    pub fn new() -> Result<Self, slint::PlatformError> {
        let to_next = Rc::new(Cell::new(false));
        let brightness_request = Rc::new(Cell::new(None));
        let battery_request = Rc::new(Cell::new(false));
        let first = make_screen(ScreenType::Lock, &to_next, &brightness_request, &battery_request)?;
        first.show()?;

        Ok(Self {
            current: Some(first),
            current_type: ScreenType::Lock,
            to_next,
            brightness_request,
            battery_request,
        })
    }

    pub fn navigate_to(&mut self, target: ScreenType) -> Result<(), slint::PlatformError> {
        if self.current_type == target { return Ok(()); }
        if let Some(old) = self.current.take() {
            let _ = old.hide();
            drop(old); // free memory
        }
        let new = make_screen(target, &self.to_next, &self.brightness_request, &self.battery_request)?;
        new.show()?;
        self.current = Some(new);
        self.current_type = target;
        Ok(())
    }

    pub fn handle_navigation(&mut self) -> Result<(), slint::PlatformError> {
        // Atomically read & reset
        if !self.to_next.replace(false) {
            return Ok(());
        }

        // Decide routing solely from the current screen type
        let next = match self.current_type {
            ScreenType::Lock        => Some(ScreenType::TestScreen),
            ScreenType::TestScreen  => Some(ScreenType::EnterPin),
            ScreenType::EnterPin    => Some(ScreenType::TestScreen2),
            ScreenType::TestScreen2 => None, // or loop back
        };

        if let Some(target) = next {
            log_info!("Navigating to {:?}", target);
            self.navigate_to(target)?;
        }
        Ok(())
    }
}