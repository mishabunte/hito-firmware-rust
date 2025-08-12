use std::time::{Duration, Instant};
use egui::{Color32, Painter, Context};
use crate::common::{LedDriver, LedColor, BlinkSpeed};
use std::any::Any;

pub struct DesktopLedDriver {
    current_color: Option<LedColor>,
    painter: Option<Painter>,
    is_on: bool,
    blink_speed: Duration,
    last_toggle: Instant,
}

impl DesktopLedDriver {
    pub fn new() -> Self {
        Self {
            current_color: None,
            painter: None,
            is_on: false,
            blink_speed: Duration::from_millis(800),
            last_toggle: Instant::now(),
        }
    }

    pub fn set_painter(&mut self, painter: Painter) {
        self.painter = Some(painter);
    }

    pub fn paint_led(&self) {
        if let Some(painter) = &self.painter {
            let color = match (self.current_color, self.is_on) {
                (Some(LedColor::Red), true) => Color32::RED,
                (Some(LedColor::Green), true) => Color32::GREEN,
                (Some(LedColor::Blue), true) => Color32::BLUE,
                _ => Color32::DARK_GRAY,
            };
            painter.circle_filled(
                egui::pos2(84.0, 60.0),
                4.0,
                color,
            );
        }
    }
}

impl LedDriver for DesktopLedDriver {
    fn init(&mut self) {
        self.current_color = None;
    }

    fn turn_on(&mut self, color: LedColor) {
        self.current_color = Some(color.clone());
        self.is_on = true;
    }

    fn turn_off(&mut self) {
        self.current_color = None;
        self.is_on = false;
    }

    fn blink(&mut self, color: LedColor, speed: BlinkSpeed, ctx: &Context) {
        println!("Starting to blink the LED with color {:?}", color);

        self.current_color = Some(color);
        self.blink_speed = speed.duration();

        let now = Instant::now();
        if now.duration_since(self.last_toggle) >= self.blink_speed {
            self.is_on = !self.is_on; // toggle LED state
            self.last_toggle = now;

            // Request GUI redraw to reflect the state change
            ctx.request_repaint();
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

