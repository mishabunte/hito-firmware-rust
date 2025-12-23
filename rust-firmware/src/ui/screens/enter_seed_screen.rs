//! Screen creation functions for the firmware UI
//! 
//! This module contains functions to create and configure each screen in the application.
//! Each function sets up the UI items, header, and callbacks for its respective screen.

extern crate alloc;
use alloc::vec;
use alloc::rc::Rc;

use slint::ModelRc;
use slint::VecModel;
use core::cell::RefCell;

use crate::slint_generatedMainWindow::{MainWindow, ScreenItem, ScreenButton, ScreenImage};
use crate::log_info;
use crate::state;
use crate::ui::router::{navigate_to, go_back};

use crate::ui::screens::generic_question_screen::create_generic_question_screen;

use super::Screen;
use slint::format;

use core::ffi::CStr;
use alloc::vec::Vec;
use alloc::string::String;

use crate::firmware;
use crate::vault;


const ABC_X: f32 = 0.0;
const ABC_Y: f32 = 80.0;
const ABC_W: f32 = 80.0;
const ABC_H: f32 = 50.0;

const FIRST_WORD_X: f32 = 0.0;
const FIRST_WORD_Y: f32 = 90.0;
const WORD_GAP_X: f32 = 160.0;
const WORD_GAP_Y: f32 = 50.0;
const WORD_W: f32 = 160.0;
const WORD_H: f32 = 50.0;

// Navigation button positions
const NAV_BUTTON_Y: f32 = 190.0;
const NAV_BUTTON_W: f32 = 130.0;
const NAV_BUTTON_H: f32 = 40.0;
const PREV_BUTTON_X: f32 = 20.0;
const NEXT_BUTTON_X: f32 = 170.0;

// Timeout in milliseconds for returning to LetterSeq mode
const LETTER_MODE_TIMEOUT_MS: u64 = 1_250_000;
const WORD_MODE_TIMEOUT_MS: u64 = 1_000_000;

static mut SEED_ENTERED: Vec<u16> = Vec::new();

#[derive(Clone, Copy, PartialEq, Eq)]
enum EnterSeedMode {
    LetterSeq,
    Letter,
    WordEntered,
}

const LETTER_SEQUENCES: [&str; 8] = [
    "abc", "def", "ghi", "jkl", 
    "mno", "pqrs", "tuv", "wxyz"
];

const ERASE_BUTTON_COORDS: (f32, f32) = (250.0, 50.0);
const ERASE_BUTTON_SIZE: (f32, f32) = (30.0, 30.0);

const ABC_LOCATIONS: [(f32, f32); 8] = [
    (ABC_X, ABC_Y),
    (ABC_X + ABC_W * 1.0, ABC_Y),
    (ABC_X + ABC_W * 2.0, ABC_Y),
    (ABC_X + ABC_W * 3.0, ABC_Y),
    (ABC_X, ABC_Y + ABC_H),
    (ABC_X + ABC_W * 1.0, ABC_Y + ABC_H),
    (ABC_X + ABC_W * 2.0, ABC_Y + ABC_H),
    (ABC_X + ABC_W * 3.0, ABC_Y + ABC_H),
];

// Global state for loop handler
static mut ENTER_SEED_MODE: EnterSeedMode = EnterSeedMode::LetterSeq;
static mut MODE_CHANGE_TIME: u64 = 0;
static mut CURRENT_WORD_INDEX: usize = 0;

use crate::crypto::ffi::{crypt0_bip39_english, CRYPT0_BIP39_MNEMONIC_ENGLISH_MAXWORDS, crypt0_bip39_mnemonic_to_entropy, crypt0_bip39_entropy_to_seed_en};

fn bip39_word_by_index(index: u16) -> Option<&'static str> {
    let idx = index as usize;
    let max = CRYPT0_BIP39_MNEMONIC_ENGLISH_MAXWORDS as usize;
    if idx >= max {
        return None;
    }

    let ptr = unsafe { crypt0_bip39_english[idx] };
    if ptr.is_null() {
        return None;
    }

    unsafe { CStr::from_ptr(ptr) }.to_str().ok()
}

fn bip39_index_by_word(word: &str) -> Option<u16> {
    let word_lc = word.to_ascii_lowercase();
    let word_bytes = word_lc.as_bytes();

    let max = CRYPT0_BIP39_MNEMONIC_ENGLISH_MAXWORDS as usize;

    for i in 0..max {
        let ptr = unsafe { crypt0_bip39_english[i] };
        if ptr.is_null() {
            break;
        }

        let w = unsafe { CStr::from_ptr(ptr) }.to_bytes();

        if w == word_bytes {
            return Some(i as u16);
        }
    }

    None
}


pub fn find_bip39_matches(prefix: &str) -> Option<Vec<u16>> {
    if prefix.is_empty() {
        return None;
    }

    let prefix_lc = prefix.to_ascii_lowercase();
    let prefix_bytes = prefix_lc.as_bytes();

    let max = CRYPT0_BIP39_MNEMONIC_ENGLISH_MAXWORDS as usize;
    let mut indices: Vec<u16> = Vec::new();

    for i in 0..max {
        let ptr = unsafe { crypt0_bip39_english[i] };
        if ptr.is_null() {
            break;
        }

        let word = unsafe { CStr::from_ptr(ptr) }.to_bytes();

        if word.starts_with(prefix_bytes) {
            indices.push(i as u16);
        }
    }

    if indices.is_empty() {
        None
    } else {
        Some(indices)
    }
}

fn letters_from_sequence(item: ScreenButton) -> ModelRc<ScreenButton> {
    let mut items: Vec<ScreenButton> = Vec::new();
    let button_gap = 80.0;
    let mut button_x = 45.0;
    if item.text == "pqrs" || item.text == "wxyz" {
        button_x = 5.0;
    }
    let button_y = 115.0;
    let button_w = 70.0;
    let button_h = 40.0;
    for (i, ch) in item.text.chars().enumerate() {
        let item = ScreenButton {
            text: ch.to_ascii_lowercase().into(),
            width: button_w,
            height: button_h,
            x: button_x + i as f32 * button_gap,
            y: button_y,
            has_border: true,
            inverted: true,
        };
        items.push(item);
    }
    ModelRc::new(VecModel::from(items))
}

/// Create navigation buttons (Prev word / Next word or Done)
fn create_nav_buttons(current_index: usize, seed_len: usize, entered_seed_len: usize) -> Vec<ScreenButton> {
    let mut buttons = Vec::new();
    
    // Prev word button (if we have a previous word)
    if current_index > 0 {
        buttons.push(ScreenButton {
            text: "Prev word".into(),
            width: NAV_BUTTON_W,
            height: NAV_BUTTON_H,
            x: PREV_BUTTON_X,
            y: NAV_BUTTON_Y,
            has_border: true,
            inverted: true,
        });
    }

    if entered_seed_len >= seed_len && current_index + 1 >= seed_len {
        // All words entered, show Done
        buttons.push(ScreenButton {
            text: "Done".into(),
            width: NAV_BUTTON_W,
            height: NAV_BUTTON_H,
            x: NEXT_BUTTON_X,
            y: NAV_BUTTON_Y,
            has_border: true,
            inverted: true,
        });
    } else if current_index < entered_seed_len {
        // Next word button (if we have a next word to enter)
        buttons.push(ScreenButton {
            text: "Next word".into(),
            width: NAV_BUTTON_W,
            height: NAV_BUTTON_H,
            x: NEXT_BUTTON_X,
            y: NAV_BUTTON_Y,
            has_border: true,
            inverted: true,
        });
    }

    
    // // Next word or Done button
    // if current_index < seed_words.len() {
    //     log_info!("Now we have {} words entered, current index: {}, seed_len {}", seed_words.len(), current_index, seed_len);
    //     // We have a word at this position already, show next navigation
    //     if current_index < seed_words.len() {
    //         // There's a next word
    //         buttons.push(ScreenButton {
    //             text: "Next word".into(),
    //             width: NAV_BUTTON_W,
    //             height: NAV_BUTTON_H,
    //             x: NEXT_BUTTON_X,
    //             y: NAV_BUTTON_Y,
    //             has_border: true,
    //             inverted: true,
    //         });
    //     } else if seed_words.len() >= seed_len {
    //         // All words entered, show Done
    //         log_info!("All seed words entered, showing Done button");
    //         buttons.push(ScreenButton {
    //             text: "Done".into(),
    //             width: NAV_BUTTON_W,
    //             height: NAV_BUTTON_H,
    //             x: NEXT_BUTTON_X,
    //             y: NAV_BUTTON_Y,
    //             has_border: true,
    //             inverted: true,
    //         });
    //     }
    // }
    
    buttons
}

fn create_abc_buttons() -> ModelRc<ScreenButton> {
    let mut abc_buttons = LETTER_SEQUENCES.iter().enumerate().map(|(i, seq)| {
        let (x, y) = ABC_LOCATIONS[i];
        ScreenButton { 
            text: (*seq).into(),  
            width: ABC_W, 
            height: ABC_H,
            has_border: false, 
            x, 
            y,
            inverted: false,
        }
    }).collect::<Vec<ScreenButton>>();
    
    abc_buttons.push(ScreenButton {
        text: "<".into(),
        width: ERASE_BUTTON_SIZE.0,
        height: ERASE_BUTTON_SIZE.1,
        x: ERASE_BUTTON_COORDS.0,
        y: ERASE_BUTTON_COORDS.1,
        has_border: false,
        inverted: true,
    });
    ModelRc::new(VecModel::from(abc_buttons))
}

/// Create ABC buttons with navigation
fn create_abc_buttons_with_nav(current_index: usize, seed_len: usize, seed_words: &[u16]) -> ModelRc<ScreenButton> {
    let mut abc_buttons = LETTER_SEQUENCES.iter().enumerate().map(|(i, seq)| {
        let (x, y) = ABC_LOCATIONS[i];
        ScreenButton { 
            text: (*seq).into(),  
            width: ABC_W, 
            height: ABC_H,
            has_border: false, 
            x, 
            y,
            inverted: false,
        }
    }).collect::<Vec<ScreenButton>>();
    
    abc_buttons.push(ScreenButton {
        text: "<".into(),
        width: ERASE_BUTTON_SIZE.0,
        height: ERASE_BUTTON_SIZE.1,
        x: ERASE_BUTTON_COORDS.0,
        y: ERASE_BUTTON_COORDS.1,
        has_border: false,
        inverted: true,
    });
    
    // Add navigation buttons
    abc_buttons.extend(create_nav_buttons(current_index, seed_len, seed_words.len()));
    
    ModelRc::new(VecModel::from(abc_buttons))
}

/// Get current time in milliseconds using the platform timer
fn get_time_ms() -> u64 {
    crate::now_us() // now_us already returns milliseconds (divided by 1000)
}

/// Reset global state for enter seed screen
fn reset_enter_seed_state() {
    unsafe {
        ENTER_SEED_MODE = EnterSeedMode::LetterSeq;
        MODE_CHANGE_TIME = 0;
        CURRENT_WORD_INDEX = 0;
    }
}

/// Set mode and update timestamp
fn set_mode(mode: EnterSeedMode) {
    unsafe {
        ENTER_SEED_MODE = mode;
        if mode != EnterSeedMode::LetterSeq {
            MODE_CHANGE_TIME = get_time_ms();
        }
    }
}

/// Check if we should return to LetterSeq mode due to timeout
pub fn handle_enter_seed_loop(ui: &Rc<MainWindow>) {
    unsafe {
        if ENTER_SEED_MODE != EnterSeedMode::LetterSeq && MODE_CHANGE_TIME > 0 {
            let elapsed = get_time_ms() - MODE_CHANGE_TIME;
            let timeout_ms = match ENTER_SEED_MODE {
                EnterSeedMode::Letter => LETTER_MODE_TIMEOUT_MS,
                EnterSeedMode::WordEntered => WORD_MODE_TIMEOUT_MS,
                _ => 0,
            };
            if elapsed >= timeout_ms {
                ENTER_SEED_MODE = EnterSeedMode::LetterSeq;
                MODE_CHANGE_TIME = 0;
                
                // Refresh UI to show ABC buttons
                let seed_len = state().lock().get_seed_length().unwrap_or(24);
                ui.set_buttons(create_abc_buttons_with_nav(CURRENT_WORD_INDEX, seed_len, SEED_ENTERED.as_slice()));
            }
        }
    }
}

/// Cleanup function for enter seed screen
pub fn cleanup_enter_seed_screen() {
    reset_enter_seed_state();
}

/// Create the "Enter Seed" screen
pub fn create_enter_seed_screen(ui: &Rc<MainWindow>) {
    // Reset state
    reset_enter_seed_state();
    
    let seed_len = state().lock().get_seed_length().unwrap_or(24);
    let seed_words = unsafe {SEED_ENTERED.as_slice()};
    let current_word_index = seed_words.len(); // Start at the next word to enter
    
    unsafe {
        CURRENT_WORD_INDEX = current_word_index;
    }

    ui.set_header_title(format!("SEED, word {}/{}", current_word_index + 1, seed_len));

    ui.set_center_text(true);
    ui.set_buttons(create_abc_buttons_with_nav(current_word_index, seed_len, &seed_words));

    let word_entered = Rc::new(RefCell::new(String::new()));
    let current_index = Rc::new(RefCell::new(current_word_index));

    let set_word_display = |ui: &MainWindow, word_entered: &Rc<RefCell<String>>| {
        let w = word_entered.borrow();
        let word_displayed: slint::SharedString = if w.is_empty() {
            "........".into()
        } else {
            w.clone().into()
        };

        let items = ModelRc::new(VecModel::from(vec![
            ScreenItem {
                text: word_displayed,
                width: 220.0,
                height: 40.0,
                x: 50.0,
                y: 50.0,
            },
        ]));
        ui.set_items(items);
    };

    set_word_display(ui, &word_entered);

    let ui_weak = Rc::downgrade(ui);
    let word_entered_rc = word_entered.clone();
    let current_index_rc = current_index.clone();

    ui.on_pressed(move |item| {
        if let Some(ui) = ui_weak.upgrade() {
            let seed_len = state().lock().get_seed_length().unwrap_or(24);
            let seed_words = unsafe {SEED_ENTERED.as_slice()};
            let idx = *current_index_rc.borrow();
            
            // Handle Done button
            if item.text == "Done" {
                log_info!("Done pressed - all seed words entered");
                // TODO: Navigate to verification or next step
                navigate_to(Screen::EncryptingSeed);
                return;
            }
            
            // Handle Prev word button
            if item.text == "Prev word" {
                if idx > 0 {
                    let new_idx = idx - 1;
                    *current_index_rc.borrow_mut() = new_idx;
                    unsafe { CURRENT_WORD_INDEX = new_idx; }
                    
                    // Show the previous word
                    {
                        let mut w = word_entered_rc.borrow_mut();
                        w.clear();
                        if new_idx < seed_words.len() {
                            w.push_str(bip39_word_by_index(seed_words[new_idx]).unwrap_or(""));
                        }
                    }
                    set_word_display(&ui, &word_entered_rc);
                    
                    ui.set_header_title(format!("SEED, word {}/{}", new_idx + 1, seed_len));
                    let updated_words = unsafe {SEED_ENTERED.as_slice()};
                    ui.set_buttons(create_abc_buttons_with_nav(new_idx, seed_len, &updated_words));
                    set_mode(EnterSeedMode::LetterSeq);
                }
                return;
            }
            
            // Handle Next word button
            if item.text == "Next word" {
                if idx < seed_words.len() {
                    let new_idx = idx + 1;
                    *current_index_rc.borrow_mut() = new_idx;
                    unsafe { CURRENT_WORD_INDEX = new_idx; }
                    
                    // Show the next word
                    {
                        let mut w = word_entered_rc.borrow_mut();
                        w.clear();
                        if new_idx < seed_words.len() {
                            w.push_str(bip39_word_by_index(seed_words[new_idx]).unwrap_or(""));
                        }
                    }
                    set_word_display(&ui, &word_entered_rc);
                    
                    ui.set_header_title(format!("SEED, word {}/{}", new_idx + 1, seed_len));
                    let updated_words = unsafe {SEED_ENTERED.as_slice()};
                    ui.set_buttons(create_abc_buttons_with_nav(new_idx, seed_len, updated_words));
                    set_mode(EnterSeedMode::LetterSeq);
                }
                return;
            }
            
            // Handle backspace
            if item.text == "<" {
                let mut w = word_entered_rc.borrow_mut();
                w.pop();
                drop(w);
                set_word_display(&ui, &word_entered_rc);
                return;
            }
            
            let current_mode = unsafe { ENTER_SEED_MODE };

            match current_mode {
                EnterSeedMode::WordEntered => {
                    // A word was selected
                    {
                        let mut w = word_entered_rc.borrow_mut();
                        w.clear();
                        w.push_str(&item.text);
                        
                        // Update or append word at current index
                        let mut words = unsafe { SEED_ENTERED.clone() };
                        if idx < words.len() {
                            // Replace existing word
                            words[idx] = bip39_index_by_word(w.clone().as_str()).unwrap_or(0);
                            unsafe { SEED_ENTERED = words; }
                        } else {
                            // Append new word
                            unsafe { SEED_ENTERED.push(bip39_index_by_word(w.clone().as_str()).unwrap_or(0)); }
                        }
                        log_info!("Entered seed word {}: {}", idx + 1, *w);
                    }
                    
                    let seed_words = unsafe { SEED_ENTERED.as_slice() };
                    let next_index = idx + 1;

                    if next_index >= seed_len {
                        log_info!("All seed words entered: {:?}", seed_words);
                        // Show Done button
                        *current_index_rc.borrow_mut() = seed_len - 1;
                        unsafe { CURRENT_WORD_INDEX = seed_len - 1; }
                        
                        {
                            let mut w = word_entered_rc.borrow_mut();
                            w.clear();
                            w.push_str(bip39_word_by_index(seed_words[seed_len - 1]).unwrap_or(""));
                        }
                        set_word_display(&ui, &word_entered_rc);
                        
                        ui.set_header_title(format!("SEED, word {}/{}", seed_len, seed_len));
                        ui.set_buttons(create_abc_buttons_with_nav(seed_len - 1, seed_len, &seed_words));
                        set_mode(EnterSeedMode::LetterSeq);
                        return;
                    }

                    // Move to next word
                    *current_index_rc.borrow_mut() = next_index;
                    unsafe { CURRENT_WORD_INDEX = next_index; }
                    
                    {
                        let mut w = word_entered_rc.borrow_mut();
                        w.clear();
                        if next_index < seed_words.len() {
                            w.push_str(bip39_word_by_index(seed_words[next_index]).unwrap_or(""));
                        }
                    }
                    set_word_display(&ui, &word_entered_rc);

                    ui.set_header_title(format!("SEED, word {}/{}", next_index + 1, seed_len));
                    ui.set_buttons(create_abc_buttons_with_nav(next_index, seed_len, &seed_words));
                    set_mode(EnterSeedMode::LetterSeq);
                }
                EnterSeedMode::LetterSeq => {
                    ui.set_buttons(letters_from_sequence(item));
                    set_mode(EnterSeedMode::Letter);
                }
                EnterSeedMode::Letter => {
                    {
                        let mut w = word_entered_rc.borrow_mut();
                        w.push_str(&item.text);
                        let matches_found = find_bip39_matches(&w);
                        if let Some(matches) = matches_found {
                          if matches.len() <= 4 {
                              // Show matching words
                              let mut word_buttons: Vec<ScreenButton> = Vec::new();
                              for (i, index) in matches.iter().enumerate() {
                                  if let Some(word) = bip39_word_by_index(*index) {
                                      let col = (i % 2) as f32;
                                      let row = (i / 2) as f32;      
                                      let x = FIRST_WORD_X + WORD_GAP_X * col;
                                      let y = FIRST_WORD_Y + WORD_GAP_Y * row;

                                      let button = ScreenButton {
                                          text: word.into(),
                                          width: WORD_W,
                                          height: WORD_H,
                                          x,
                                          y,
                                          has_border: false,
                                          inverted: false,
                                      };
                                      word_buttons.push(button);
                                  }
                              }
                              ui.set_buttons(ModelRc::new(VecModel::from(word_buttons)));
                              drop(w);
                              set_mode(EnterSeedMode::WordEntered);
                              return;
                          }
                        }
                    }
                    set_word_display(&ui, &word_entered_rc);
                    let seed_words = unsafe {SEED_ENTERED.as_slice()};
                    ui.set_buttons(create_abc_buttons_with_nav(idx, seed_len, &seed_words));
                    set_mode(EnterSeedMode::LetterSeq);
                }
            }
        }
    });
}
pub fn create_wallet_setup_screen(ui: &Rc<MainWindow>) {
  let button_y = 50.0;
  let seed_12_y = button_y + 45.0;
  let y_gap = 40.0;
  ui.set_header_title("WALLET SETUP".into());
  let buttons = ModelRc::new(VecModel::from(vec![
      ScreenButton {
          text: "Generate new seed".into(),  
          width: 240.0, 
          height: 40.0,
          has_border: false, 
          x: 40.0, 
          y: button_y,
          inverted: false,
      },
      ScreenButton { 
          text: "Import seed 12 words".into(),  
          width: 240.0, 
          height: 40.0,
          has_border: false, 
          x: 40.0, 
          y: seed_12_y,
          inverted: false,
      },
      ScreenButton { 
          text: "Import seed 18 words".into(),  
          width: 240.0, 
          height: 40.0,
          has_border: false, 
          x: 40.0, 
          y: seed_12_y + y_gap,
          inverted: false,
      },
      ScreenButton { 
          text: "Import seed 24 words".into(),  
          width: 240.0, 
          height: 40.0,
          has_border: false, 
          x: 40.0, 
          y: seed_12_y + y_gap * 2.0,
          inverted: false,
      },
  ]));
  ui.set_buttons(buttons);
  ui.on_pressed(move |item| {
      match item.text.as_str() {
          "Generate new seed" => {
            navigate_to(Screen::GenerateSeed);
          },
          "Import seed 12 words" => {
              log_info!("Navigate to IMPORT SEED 12 WORDS screen");
              state().lock().set_seed_length(12);
              navigate_to(Screen::EnterSeed);
          },
          "Import seed 18 words" => {
              log_info!("Navigate to IMPORT SEED 18 WORDS screen");
              state().lock().set_seed_length(18);
              navigate_to(Screen::EnterSeed);
          },
          "Import seed 24 words" => {
              log_info!("Navigate to IMPORT SEED 24 WORDS screen");
              state().lock().set_seed_length(24);
              navigate_to(Screen::EnterSeed);
          },
          _ => {},
      }
  });
}

pub fn create_generate_seed_screen(ui: &Rc<MainWindow>) {
    create_generic_question_screen(
        ui,
        "SEED BACKUP",
        "Make sure to backup \\\\ your seed phrase \\\\ We'll verify \\\\ a few words next",
        "Continue",
        "Show seed",
        || {
          navigate_to(Screen::EnterSeed)
        },
        || {
          navigate_to(Screen::ShowSeedBackup);
        }
    );
}

pub fn create_encrypting_seed_screen(ui: &Rc<MainWindow>) {
    ui.set_header_title("ENCRYPTING SEED".into());

    let items = ModelRc::new(VecModel::from(vec![
        ScreenItem {
            text: "Encrypting your seed phrase...".into(),
            width: 300.0,
            height: 40.0,
            x: 10.0,
            y: 80.0,
        },
    ]));
    ui.set_items(items);
    let vault_arc = firmware().vault.clone();

    let buttons = ModelRc::new(VecModel::from(vec![
      ScreenButton {
          text: "Encrypt".into(),  
          width: 240.0, 
          height: 40.0,
          has_border: true, 
          x: 40.0, 
          y: 150.0,
          inverted: true,
      },
    ]));

    ui.set_buttons(buttons);

    ui.on_pressed(move |item| {
        if item.text == "Encrypt" {
            log_info!("Starting seed encryption...");
        }
    });
}