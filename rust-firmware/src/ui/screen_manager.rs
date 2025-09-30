use alloc::boxed::Box;
use alloc::rc::{Rc, Weak};
use core::cell::{RefCell, Cell};
use alloc::vec::Vec;
use core::any::Any;
use slint::ComponentHandle;
use crate::log_info;

use crate::{LockScreen, TestScreen, TestScreen2, Router, BrightnessController, BatteryController, EnterPinScreen};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScreenType { Lock, TestScreen, EnterPin, TestScreen2 }

const MAX_HISTORY: usize = 16; // max history length for back navigation

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

fn make_screen(kind: ScreenType, to_next: &Rc<Cell<bool>>, to_prev: &Rc<Cell<bool>>, brightness_request: &Rc<Cell<Option<u8>>>, battery_request: &Rc<Cell<bool>>) -> Result<Screen, slint::PlatformError>
{
    match kind {
        ScreenType::Lock => {
            let c = LockScreen::new()?;
            {
                let to_next_flag = Rc::clone(to_next);
                c.global::<Router>().on_navigate(move || to_next_flag.set(true));

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
                let to_next_flag = Rc::clone(to_next);
                c.global::<Router>().on_navigate(move || to_next_flag.set(true));

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
                let flag = Rc::clone(to_next);
                c.global::<Router>().on_navigate(move || flag.set(true));

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
                let to_next_flag = Rc::clone(to_next);
                c.global::<Router>().on_navigate(move || to_next_flag.set(true));

                let to_prev_flag = Rc::clone(to_prev);
                c.global::<Router>().on_back(move || to_prev_flag.set(true));

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
    to_prev: Rc<Cell<bool>>,
    pub brightness_request: Rc<Cell<Option<u8>>>,
    pub battery_request: Rc<Cell<bool>>,
    current_type: ScreenType,
    history: Vec<ScreenType>, // for back navigation
}
    

impl ScreenManager {
    pub fn new() -> Result<Self, slint::PlatformError> {
        let to_next = Rc::new(Cell::new(false));
        let to_prev = Rc::new(Cell::new(false));
        let brightness_request = Rc::new(Cell::new(None));
        let battery_request = Rc::new(Cell::new(false));
        let first = make_screen(ScreenType::Lock, &to_next, &to_prev, &brightness_request, &battery_request)?;
        first.show()?;

        Ok(Self {
            current: Some(first),
            current_type: ScreenType::Lock,
            to_next,
            to_prev,
            brightness_request,
            battery_request,
            history: Vec::new()
        })
    }
    /// Internal core switch: does NOT push to history.
    fn switch_to(&mut self, target: ScreenType) -> Result<(), slint::PlatformError> {
        if self.current_type == target { return Ok(()); }
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
        self.current_type = target;
        Ok(())
    }

    /// Forward navigation: push current onto history, then switch.
    pub fn navigate_to(&mut self, target: ScreenType) -> Result<(), slint::PlatformError> {
        if self.current_type != target {
            // cap history length on embedded if you want (e.g., 16)
            if self.history.len() == MAX_HISTORY { self.history.remove(0); }
            self.history.push(self.current_type);
        }
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

    pub fn handle_navigation(&mut self) -> Result<(), slint::PlatformError> {
        // Atomically read & reset
        let back = self.to_prev.replace(false);
        let fwd  = self.to_next.replace(false);

        if !back && !fwd {
            return Ok(()); // no navigation requested
        }

        if back {
            self.navigate_back()?;
            return Ok(());
        }

        if fwd {
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
        } else {
            Ok(())
        }
    }
}