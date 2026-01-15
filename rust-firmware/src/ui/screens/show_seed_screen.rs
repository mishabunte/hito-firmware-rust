//! Show Seed Screen - displays mnemonic seed phrase with touch-to-reveal
//! 
//! Words are masked by default and revealed when touched or slid over.
//! Uses loop handler to continuously check touch position for slide-to-reveal.

extern crate alloc;
use alloc::string::String;
use alloc::vec;
use alloc::rc::Rc;
use alloc::vec::Vec;
use alloc::format;

use slint::ModelRc;
use slint::VecModel;

use crate::slint_generatedMainWindow::{MainWindow, ScreenItem, ScreenButton};
use crate::log_info;
use crate::firmware;
use crate::drivers::Touch;
use crate::ui::navigate_to;
use crate::ui::router::set_back_screen;
use crate::ui::screens::show_alert;

static mut BACKUP_MODE: bool = false;

use super::Screen;

const WORDS_PER_PAGE: usize = 8;
const MASKED_WORD: &str = "xx.xxxxxxxx";

// Layout constants
const X_LEFT: f32 = 22.0;
const X_RIGHT: f32 = 172.0;
const Y_TOP: f32 = 55.0;
const Y_GAP: f32 = 30.0;
const TEXT_H: f32 = 25.0;
const TEXT_W: f32 = 140.0;

// Word positions for 8 words per page (4 on left, 4 on right)
const WORD_COORDS: [(f32, f32); 8] = [
    (X_LEFT, Y_TOP), (X_LEFT, Y_TOP + Y_GAP), (X_LEFT, Y_TOP + Y_GAP * 2.0), (X_LEFT, Y_TOP + Y_GAP * 3.0),
    (X_RIGHT, Y_TOP), (X_RIGHT, Y_TOP + Y_GAP), (X_RIGHT, Y_TOP + Y_GAP * 2.0), (X_RIGHT, Y_TOP + Y_GAP * 3.0),
];

// Global state for show seed screen
static mut CURRENT_PAGE: usize = 0;
static mut MNEMONIC_WORDS: Option<Vec<String>> = None;
// Bitmask of revealed slots (bit i = slot i is revealed)
static mut REVEALED_SLOTS: u8 = 0;

/// Format word number with leading zero for single digits
fn format_word_number(index: usize, word: &str) -> String {
    let num = index + 1;
    if num < 10 {
        format!("0{}.{}", num, word)
    } else {
        format!("{}.{}", num, word)
    }
}

/// Generate items for a mnemonic page
/// `revealed_slots` is a bitmask where bit i means slot i is revealed
fn generate_page_items(page_index: usize, mnemonic: &[String], revealed_slots: u8) -> ModelRc<ScreenItem> {
    ModelRc::new(VecModel::from(
        (0..WORDS_PER_PAGE).map(|slot| {
            let word_index = page_index * WORDS_PER_PAGE + slot;
            if word_index < mnemonic.len() {
                let is_revealed = (revealed_slots & (1 << slot)) != 0;
                let text = if is_revealed {
                    format_word_number(word_index, &mnemonic[word_index])
                } else if unsafe { BACKUP_MODE } {
                    format_word_number(word_index, &mnemonic[word_index])
                } else {
                    MASKED_WORD.into()
                };
                ScreenItem {
                    text: text.into(),
                    width: TEXT_W,
                    height: TEXT_H,
                    x: WORD_COORDS[slot].0,
                    y: WORD_COORDS[slot].1,
                }
            } else {
                ScreenItem {
                    text: "".into(),
                    width: TEXT_W,
                    height: TEXT_H,
                    x: WORD_COORDS[slot].0,
                    y: WORD_COORDS[slot].1,
                }
            }
        }).collect::<VecModel<ScreenItem>>()
    ))
}

pub fn create_show_seed_screen(ui: &Rc<MainWindow>, backup_mode: bool) {
    if !backup_mode {
        set_back_screen(Screen::Home);
    }
    unsafe {
        CURRENT_PAGE = 0;
        REVEALED_SLOTS = 0;
    }
    
    // Get mnemonic data in a single lock scope to avoid deadlock
    let mnemonic_result = {
        let vault = firmware().vault.lock();
        let len = vault.get_mnemonic_len();
        let mnemonic = vault.get_mnemonic();
        (len, mnemonic)
    };

    unsafe {
        BACKUP_MODE = backup_mode;
    }
    
    let (mnemonic_len, mnemonic_result) = mnemonic_result;
    let total_pages = if let Ok(len) = mnemonic_len {
        len / WORDS_PER_PAGE + if len % WORDS_PER_PAGE > 0 { 1 } else { 0 }
    } else {
        0
    };

    ui.set_header_title(slint::SharedString::from(format!("SEED, page 1/{}", total_pages)));

    if let Ok(mnemonic) = mnemonic_result {
        let mnemonic_array: Vec<String> = mnemonic.split_whitespace().map(String::from).collect();
        
        unsafe {
            MNEMONIC_WORDS = Some(mnemonic_array.clone());
        }
        
        let buttons = ModelRc::new(VecModel::from(vec![
            ScreenButton {
                text: "Next page".into(),
                width: 130.0,
                height: 40.0,
                x: 170.0,
                y: 190.0,
                has_border: false,
                inverted: false,
            },
        ]));
        ui.set_buttons(buttons);
        
        let items = generate_page_items(0, &mnemonic_array, 0);
        ui.set_items(items);

        // Handle "Next page" button press
        let ui_weak = Rc::downgrade(ui);
        let total_pages_copy = total_pages;
        
        ui.on_pressed(move |item| {
            if item.text == "Next page" {
                unsafe {
                    if CURRENT_PAGE + 1 < total_pages_copy {
                        CURRENT_PAGE += 1;
                    } else {
                        CURRENT_PAGE = 0;
                    }
                    REVEALED_SLOTS = 0;
                    log_info!("New current page: {}", CURRENT_PAGE);
                    
                    if let Some(ui) = ui_weak.upgrade() {
                        if let Some(ref words) = MNEMONIC_WORDS {
                            let items = generate_page_items(CURRENT_PAGE, words, 0);
                            ui.set_items(items);
                            ui.set_header_title(slint::SharedString::from(
                                format!("SEED, PAGE {}/{}", CURRENT_PAGE + 1, total_pages_copy)
                            ));
                        }
                    }
                }
                log_info!("Next page button pressed");
            }
        });
    } else if let Err(e) = mnemonic_result {
      show_alert(format!("Failed to retrieve seed phrase: {}. Please try again", e).as_str());
    }
}

/// Check if a touch position is within a word slot area
fn get_touched_slot(touch_x: u16, touch_y: u16) -> Option<usize> {
    let tx = touch_x as f32;
    let ty = touch_y as f32;
    
    for (slot, &(x, y)) in WORD_COORDS.iter().enumerate() {
        // Check if touch is within the word's bounding box
        if tx >= x && tx <= x + TEXT_W && ty >= y && ty <= y + TEXT_H {
            return Some(slot);
        }
    }
    None
}

/// Handle touch events for revealing words - called from main loop
/// This enables slide-to-reveal: sliding your finger reveals words as you pass over them
pub fn handle_show_seed_loop(ui: &MainWindow) {
    let fw = firmware();
    let touch = fw.touch.lock();
    
    if touch.has_touch() {
        let (tx, ty) = touch.get_position();
        
        if let Some(slot) = get_touched_slot(tx, ty) {
            unsafe {
                let slot_bit = 1u8 << slot;
                // Only update if this slot wasn't already revealed
                if (REVEALED_SLOTS & slot_bit) == 0 {
                    REVEALED_SLOTS |= slot_bit;
                    
                    if let Some(ref words) = MNEMONIC_WORDS {
                        let items = generate_page_items(CURRENT_PAGE, words, REVEALED_SLOTS);
                        ui.set_items(items);
                    }
                }
            }
        }
    }
    // No action needed when touch is released - words stay revealed
}
