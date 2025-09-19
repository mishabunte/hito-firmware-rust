
//pub mod display;
//
use std::cell::RefCell;
use std::println;
use std::rc::Rc;
use once_cell::sync::OnceCell;
use std::thread_local;
use std::vec;
use std::vec::Vec;

use minifb::{Key, MouseButton, Window, WindowOptions};
use std::fs;

use resvg::tiny_skia::Pixmap;
use resvg::usvg::{Options, Tree};
use resvg::render;
use resvg::tiny_skia::{Transform};
use resvg::usvg::fontdb::Database;

/* ---------- global holder ---------- */
/*
thread_local! {
static SIMULATOR_WINDOW: RefCell<SimulatorWindow> = 
    RefCell::new(SimulatorWindow::new());
}

fn simulator_window_options() -> minifb::WindowOptions {
    WindowOptions {
        resize: false,
        scale: minifb::Scale::X1,
        ..WindowOptions::default()
    }
}
fn simulator_window() -> &'static RefCell<SimulatorWindow> {
    //SIMULATOR_WINDOW.get_or_init( || RefCell::new(SimulatorWindow::new()));
    SIMULATOR_WINDOW
}
*/

/*
type SharedWin = Rc<RefCell<SimulatorWindow>>;

static mut SINGLETON: Option<SharedWin> = None;

pub fn simulator_window() -> SharedWin {
    unsafe {
        SINGLETON.get_or_insert_with(|| {
            let win = SimulatorWindow::new();
            Rc::new(RefCell::new(win))
        }).clone()
    }
}
*/

thread_local! {
    static WINDOW: OnceCell<RefCell<SimulatorWindow>> = OnceCell::new();
}

pub fn simulator_window_init() {
    WINDOW.with(|cell| {
        cell.set(RefCell::new(SimulatorWindow::new())).expect("init called twice");
    });
}

pub fn simulator_window_update() {
    WINDOW.with(|cell| {
        let win_cell = cell.get().expect("simulator_window_init() not called");
        let mut win = win_cell.borrow_mut();

        win.update();
    });
}

pub fn simulator_window_has_touch() -> bool {
    WINDOW.with(|cell| {
        let win_cell = cell.get().expect("simulator_window_init() not called");
        let mut window = win_cell.borrow_mut();

        window.mouse_clicked
    })
}

pub fn simulator_window_touch_position() -> (u16, u16) {
    WINDOW.with(|cell| {
        let win_cell = cell.get().expect("simulator_window_init() not called");
        let mut window = win_cell.borrow_mut();

        window.mouse_position_screen()
    })
}

pub fn simulator_window_draw_rect(x: u16, y: u16, w: u16, h: u16, color: u32) {
    WINDOW.with(|cell| {
        let win_cell = cell.get().expect("simulator_window_init() not called");
        let mut window = win_cell.borrow_mut();

        window.draw_rect(x, y, w, h, color);
    });
}

pub fn simulator_window_fill_rect(x: u16, y: u16, w: u16, h: u16, color: u32) {
    WINDOW.with(|cell| {
        let win_cell = cell.get().expect("simulator_window_init() not called");
        let mut window = win_cell.borrow_mut();

        window.fill_rect(x, y, w, h, color);
    });
}

pub fn simulator_window_draw_led(color: u32) {
    WINDOW.with(|cell| {
        let win_cell = cell.get().expect("simulator_window_init() not called");
        let mut window = win_cell.borrow_mut();

        window.draw_led(color);
    });
}

pub fn simulator_window_draw_line(y: u16, x_start: u16, x_end: u16, pixels: &[u16]) {
    WINDOW.with(|cell| {
        let win_cell = cell.get().expect("simulator_window_init() not called");
        let mut window = win_cell.borrow_mut();

        window.draw_line(y, x_start, x_end, pixels);
    });
}

pub fn simulator_window_is_mouse_pressed() -> Option<bool> {
    WINDOW.with(|cell| {
        let win_cell = cell.get().expect("simulator_window_init() not called");
        let mut window = win_cell.borrow_mut();

        let is_pressed = window.is_mouse_pressed();
        is_pressed
    })
}

pub fn simulator_window_draw_image() {
}

const WINDOW_WIDTH: u32 = 480;
const WINDOW_HEIGHT: u32 = 720;
const SCREEN_WIDTH: u32 = 320;
const SCREEN_HEIGHT: u32 = 240;
const SCREEN_OFFSET_X: u32 = 80;
const SCREEN_OFFSET_Y: u32 = 80;
const NAME: &str = "Hito Simulator";

struct SimulatorWindow {
    window: Window,
    buffer: Vec<u32>,
    mouse_clicked: bool,
}

use std::fmt;

use crate::log_info;

impl fmt::Debug for SimulatorWindow {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        //write!(f, "HitoSimulator", self.width, self.height)
        write!(f, "HitoSimulator")
    }
}

impl SimulatorWindow {

    pub fn new() -> Self {

        let opts = WindowOptions {
            resize: false,
            scale: minifb::Scale::X1,
            ..WindowOptions::default()
        };
        let window = Window::new(NAME, WINDOW_WIDTH as usize, WINDOW_HEIGHT as usize, opts).unwrap();
        let mut this = Self { 
            //window: Rc::new(window),
            window: window,
            buffer: vec![0u32; WINDOW_WIDTH as usize * WINDOW_HEIGHT as usize],
            mouse_clicked: false,
        };
        this.init_background_png();
        this
    }

    fn init_background_svg(&mut self) {
        let svg_data = fs::read("./res/simulator/hito_simulator_background.svg").unwrap();

        let options = resvg::usvg::Options::default();
        let mut fontdb = Database::new();
        let tree = resvg::usvg::Tree::from_data(&svg_data, &options, &fontdb).unwrap();

        let mut pixmap = Pixmap::new(WINDOW_WIDTH as u32, WINDOW_HEIGHT as u32).unwrap();
        render(&tree, Transform::identity(), &mut pixmap.as_mut());

        for y in 0..WINDOW_HEIGHT {
            for x in 0..WINDOW_WIDTH {
                let pixel = pixmap.pixel(x, y).unwrap();
                //println!("({}, {}): r={}, g={}, b={}, a={}", x, y, pixel.red(), pixel.green(), pixel.blue(), pixel.alpha());
                let i = y * WINDOW_WIDTH + x;
                self.buffer[i as usize] = ((pixel.red() as u32) << 16) | ((pixel.green() as u32) << 8) | (pixel.blue() as u32);

                //self.buffer[i as usize] = 0xFF0000;
            }
        }
    }

    fn init_background_png(&mut self) {
        let img = image::open("./res/simulator/hito_simulator_background.png").expect("Failed to open image");

        let rgba = img.to_rgba8(); // converts to ImageBuffer<Rgba<u8>, Vec<u8>>

        for (x, y, pixel) in rgba.enumerate_pixels() {
            let i = (y as usize) * WINDOW_WIDTH as usize + (x as usize);
            if i < self.buffer.len() {
                let i = (y as usize) * WINDOW_WIDTH as usize + (x as usize);
                let rgba = pixel.0;
                self.buffer[i] = ((rgba[0] as u32) << 16) | ((rgba[1] as u32) << 8) | (rgba[2] as u32);
            }
        }
    }

    fn draw_rect(&mut self, x: u16, y: u16, w: u16, h: u16, color: u32) {
        for i in 0..h {
            for j in 0..w {
                let px = (x + j) as u32 + (SCREEN_OFFSET_X as u32);
                let py = (y + i) as u32 + (SCREEN_OFFSET_X as u32);
                if px < WINDOW_WIDTH && py < WINDOW_HEIGHT {
                    let index = (py as usize) * (WINDOW_WIDTH as usize) + (px as usize);
                    self.buffer[index] = color;
                }
            }
        }
    }

    fn fill_rect(&mut self, x: u16, y: u16, w: u16, h: u16, color: u32) {
        for i in 0..h {
            for j in 0..w {
                let px = (x + j) as u32 + (SCREEN_OFFSET_X as u32);
                let py = (y + i) as u32 + (SCREEN_OFFSET_X as u32);
                if px < WINDOW_WIDTH && py < WINDOW_HEIGHT {
                    let index = (py as usize) * (WINDOW_WIDTH as usize) + (px as usize);
                    self.buffer[index] = color;
                }
            }
        }
    }

    fn draw_line(&mut self, y: u16, x_start: u16, x_end: u16, pixels: &[u16]) {
        let py = (y as u32) + SCREEN_OFFSET_Y;
        if py >= WINDOW_HEIGHT {
            println!("draw_line: y={} is out of bounds", y);
            return;
        }

        let line_len = (x_end - x_start) as usize;
        for (i, &rgb565_pixel) in pixels.iter().take(line_len).enumerate() {
            let px = (x_start as u32 + i as u32) + SCREEN_OFFSET_X;
            if px < WINDOW_WIDTH {
                let index = (py as usize) * (WINDOW_WIDTH as usize) + (px as usize);
                // Convert RGB565 to RGB888
                let r8 = ((rgb565_pixel >> 11) & 0x1F) << 3;
                let g8 = ((rgb565_pixel >> 5) & 0x3F) << 2;
                let b8 = (rgb565_pixel & 0x1F) << 3;
                let color32 = ((r8 as u32) << 16) | ((g8 as u32) << 8) | (b8 as u32);
                self.buffer[index] = color32;
            }
        }
    }

    fn draw_led(&mut self, color: u32) {
        // LED position in the simulator window (outside the screen area)
        let led_center_x = 82i32;
        let led_center_y = 58i32;
        let led_radius = 4i32;

        // Draw a filled circle for the LED
        for dy in -led_radius..=led_radius {
            for dx in -led_radius..=led_radius {
                let distance_squared = dx * dx + dy * dy;
                if distance_squared <= led_radius * led_radius {
                    let px = led_center_x + dx;
                    let py = led_center_y + dy;
                    if px >= 0 && py >= 0 && (px as u32) < WINDOW_WIDTH && (py as u32) < WINDOW_HEIGHT {
                        let index = (py as usize) * (WINDOW_WIDTH as usize) + (px as usize);
                        self.buffer[index] = color;
                    }
                }
            }
        }
    }

    pub fn is_mouse_pressed(&mut self) -> Option<bool> {
        if self.window.get_mouse_down(MouseButton::Left) && !self.mouse_clicked {
            // Mouse button just pressed
            //println!("Mouse pressed");
            self.mouse_clicked = true;
            return Some(true);
        } else if !self.window.get_mouse_down(MouseButton::Left) && self.mouse_clicked {
            // Mouse button just released
            //println!("Mouse released");
            self.mouse_clicked = false;
            log_info!("Mouse released");
            return Some(false);
        }
        None
    }

    pub fn mouse_position_screen(&self) -> (u16, u16) {
        if let Some((x, y)) = self.window.get_mouse_pos(minifb::MouseMode::Discard) {
            // Adjust for the screen offset
            let adjusted_x = (x as u32).saturating_sub(SCREEN_OFFSET_X);
            let adjusted_y = (y as u32).saturating_sub(SCREEN_OFFSET_Y);
            return (adjusted_x as u16, adjusted_y as u16);
        }
        (0, 0)
        /*
        if self.window.get_mouse_down(MouseButton::Left) {
            if let Some((x, y)) = self.window.get_mouse_pos(minifb::MouseMode::Discard) {
                return Some((x as u16, y as u16));
            }
        }
        None
        */
    }

    pub fn update(&mut self) {

        if self.window.is_open() {
            self.window.update_with_buffer(&self.buffer, WINDOW_WIDTH as usize, WINDOW_HEIGHT as usize).unwrap();
        }

        std::thread::sleep(std::time::Duration::from_millis(40));
    }

}



