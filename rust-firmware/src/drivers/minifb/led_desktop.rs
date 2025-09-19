use std::{println, time::{Duration, Instant}};
use crate::drivers::{indicator::{BlinkSpeed, Indicator as LedDriver, LedColor}, minifb::simulator_window::simulator_window_draw_led};
use std::any::Any;

pub struct DesktopLedDriver {
    current_color: Option<LedColor>,
    is_on: bool,
    blink_speed: Duration,
    last_toggle: Instant,
}

impl DesktopLedDriver {
    pub fn new() -> Self {
        Self {
            current_color: None,
            is_on: false,
            blink_speed: Duration::from_millis(800),
            last_toggle: Instant::now(),
        }
    }

    fn update_led_display(&self) {
        let color = match (self.current_color, self.is_on) {
            (Some(LedColor::Red), true) => 0xFF0000,      // Red
            (Some(LedColor::Green), true) => 0x00FF00,    // Green
            (Some(LedColor::Blue), true) => 0x0000FF,     // Blue
            _ => 0x404040,                                // Dark gray (off)
        };
        simulator_window_draw_led(color);

        //println!("LED Color: {:?}, State: {}", self.current_color, if self.is_on { "ON" } else { "OFF" });
    }
}

impl LedDriver for DesktopLedDriver {
    fn init(&mut self) {
        self.current_color = None;
        self.is_on = false;
        self.update_led_display();
    }

    fn turn_on(&mut self, color: LedColor) {
        self.current_color = Some(color);
        self.is_on = true;
        self.update_led_display();
    }

    fn turn_off(&mut self) {
        self.current_color = None;
        self.is_on = false;
        self.update_led_display();
    }

    fn blink(&mut self, color: LedColor, speed: BlinkSpeed) {
        self.current_color = Some(color);
        self.blink_speed = speed.duration();

        let now = Instant::now();
        if now.duration_since(self.last_toggle) >= self.blink_speed {
            self.is_on = !self.is_on; // toggle LED state
            self.last_toggle = now;
            self.update_led_display();
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

