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

use crate::common::ui_set_progress_bar_properties;
use crate::crypto::crypt0::mnemonic_to_indices;
use crate::drivers::Battery;
use crate::slint_generatedMainWindow::{MainWindow, ScreenItem, ScreenButton, ScreenImage};
use crate::log_info;
use crate::state;
use crate::ui::router::{navigate_to, go_back};

use crate::ui::screens::show_alert;
use crate::ui::screens::generic_question_screen::create_generic_question_screen;
use crate::ui_report_progress;

use super::Screen;
use slint::format;

use core::ffi::CStr;
use alloc::vec::Vec;
use alloc::string::String;

use crate::firmware;
use crate::vault;
use crate::crypto;

const SEED_CHECK_WORDS_AMOUNT: usize = 3;

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

static mut SEED_WORDS_ENTERED: Vec<String> = Vec::new();

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
static mut SEED_CHECK_MODE: bool = false;
// For seed check mode: indices of words to verify (e.g., [2, 6, 14] for words 3, 7, 15)
static mut SEED_CHECK_INDICES: [usize; 3] = [0, 0, 0];
// For seed check mode: the expected seed to compare against
static mut EXPECTED_SEED_WORDS: Vec<String> = Vec::new();

use crate::crypto::ffi::{crypt0_bip39_english, CRYPT0_BIP39_MNEMONIC_ENGLISH_MAXWORDS};

use crate::crypto::crypt0::{ bip39_index_by_word, bip39_word_by_index };


pub fn find_bip39_matches(prefix: &str) -> Option<Vec<String>> {
    if prefix.is_empty() {
        return None;
    }

    let prefix_lc = prefix.to_ascii_lowercase();
    let prefix_bytes = prefix_lc.as_bytes();

    let max = CRYPT0_BIP39_MNEMONIC_ENGLISH_MAXWORDS as usize;
    let mut matches: Vec<String> = Vec::new();

    for i in 0..max {
        let ptr = unsafe { crypt0_bip39_english[i] };
        if ptr.is_null() {
            break;
        }

        let word = unsafe { CStr::from_ptr(ptr) }.to_bytes();

        if word.starts_with(prefix_bytes) {
            matches.push(String::from_utf8_lossy(word).into_owned());
        }
    }

    if matches.is_empty() {
        None
    } else {
        Some(matches)
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
fn create_abc_buttons_with_nav(current_index: usize, seed_len: usize, entered_seed_len: usize) -> ModelRc<ScreenButton> {
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
    abc_buttons.extend(create_nav_buttons(current_index, seed_len, entered_seed_len));
    
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
        SEED_CHECK_MODE = false;
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
                ui.set_buttons(create_abc_buttons_with_nav(CURRENT_WORD_INDEX, seed_len, SEED_WORDS_ENTERED.len()));
            }
        }
    }
}

/// Cleanup function for enter seed screen
pub fn cleanup_enter_seed_screen() {
    reset_enter_seed_state();
}

/// Get the header title based on mode
fn get_header_title(word_index: usize, seed_len: usize, seed_check: bool) -> slint::SharedString {
    if seed_check {
        // In seed check mode, show which word number we're asking for
        let actual_word_num = unsafe { SEED_CHECK_INDICES[word_index] + 1 };
        log_info!("Seed check mode: requesting word index {} (word number {})", word_index, actual_word_num);
        format!("TYPE WORD {}", actual_word_num)
    } else {
        format!("SEED, word {}/{}", word_index + 1, seed_len)
    }
}

/// Initialize seed check mode with specific word indices to verify
pub fn init_seed_check(expected_seed: &[String], indices: [usize; 3]) {
    unsafe {
        SEED_CHECK_MODE = true;
        SEED_CHECK_INDICES = indices;
        EXPECTED_SEED_WORDS = expected_seed.to_vec();
        SEED_WORDS_ENTERED.clear();
    }
}

fn on_encrypt_clicked(seed_check_mode: bool) {
  let mnemonic = if seed_check_mode {
    unsafe {
      log_info!("EXPECTED_SEED: {:?}", EXPECTED_SEED_WORDS);
      EXPECTED_SEED_WORDS.as_slice()   
    }
  } else {
    unsafe {
      log_info!("SEED_WORDS_ENTERED: {:?}", SEED_WORDS_ENTERED);
      &SEED_WORDS_ENTERED.as_slice()
    }
  };
  let vault_arc = firmware().vault.clone();
  let mut vault = vault_arc.lock();

  let mut seed_indices: Vec<u16> = Vec::with_capacity(mnemonic.len());
  for word in mnemonic.iter() {
    match bip39_index_by_word(word) {
      Some(idx) => seed_indices.push(idx),
      None => {
        show_alert("Error converting mnemonic to indices.\\\\Please try again.");
        return;
      }
    }
  }
  
  // Generate entropy from mnemonic indices
  if let Ok((entropy, entropy_len)) = crypto::crypt0::mnemonic_indices_to_entropy(seed_indices) {
    let pass = state().lock().get_pin();
    match vault.init(&entropy, entropy_len, pass.as_bytes(), Some(ui_report_progress)) {
        Ok(()) => {
            log_info!("Vault initialized and seed encrypted successfully.");
            navigate_to(Screen::FinalizeSeed);
            //firmware().battery.lock().reboot();
        },
        Err(e) => {
          log_info!("Error initializing vault: {:?}", e);
          firmware().battery.lock().reboot();
        }
    }
  } else {
    show_alert("Error converting mnemonic to entropy. Please try again.");
  }
}

/// Generate random word indices for seed verification (like C's word_ids_init)
/// Uses crypt0_rng to generate random indices, ensuring no duplicates
fn generate_random_word_indices(seed_len: usize) -> [usize; SEED_CHECK_WORDS_AMOUNT] {
    let mut indices = [0usize; SEED_CHECK_WORDS_AMOUNT];
    let mut random_bytes = [0u8; SEED_CHECK_WORDS_AMOUNT];
    
    loop {
        // Generate random bytes
        unsafe {
            crypto::ffi::crypt0_rng(random_bytes.as_mut_ptr(), SEED_CHECK_WORDS_AMOUNT);
        }
        
        // Convert to word indices (0 to seed_len-1 range)
        let mut has_duplicates = false;
        for i in 0..SEED_CHECK_WORDS_AMOUNT {
            indices[i] = (random_bytes[i] as usize) % seed_len;
            
            // Check for duplicates with previous indices
            if i > 0 {
                for j in 0..i {
                    if indices[i] == indices[j] {
                        has_duplicates = true;
                        break;
                    }
                }
            }
            if has_duplicates {
                break;
            }
        }
        
        if !has_duplicates {
            break;
        }
    }
    
    indices
}

/// Verify the entered words against the expected seed
fn verify_seed_check() -> bool {
    unsafe {
        if SEED_WORDS_ENTERED.len() != 3 {
            return false;
        }
        for (i, &check_idx) in SEED_CHECK_INDICES.iter().enumerate() {
            if check_idx >= EXPECTED_SEED_WORDS.len() {
                return false;
            }
            if SEED_WORDS_ENTERED.get(i) != EXPECTED_SEED_WORDS.get(check_idx) {
                return false;
            }
        }
        true
    }
}

fn set_word_display(ui: &MainWindow, word_entered: &Rc<RefCell<String>>) {
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
}

/// Create the "Enter Seed" screen (also used for seed check mode)
pub fn create_enter_seed_screen(ui: &Rc<MainWindow>, seed_check: bool) {
    // Reset state
    reset_enter_seed_state();
    
    unsafe {
        SEED_CHECK_MODE = seed_check;
        if !seed_check {
            // Clear entered seed for fresh import
            SEED_WORDS_ENTERED.clear();
        }
        // Note: For seed check mode, init_seed_check should have been called 
        // before navigating to this screen (e.g., from create_generate_seed_screen)
    }
    
    let seed_len = if seed_check { 3 } else { state().lock().get_seed_length().unwrap_or(24) };
    let seed_words = unsafe { SEED_WORDS_ENTERED.as_slice() };
    let current_word_index = 0; // Start at the next word to enter
    
    unsafe {
        CURRENT_WORD_INDEX = current_word_index;
    }

    ui.set_header_title(get_header_title(current_word_index, seed_len, seed_check));

    ui.set_center_text(true);
    ui.set_buttons(create_abc_buttons_with_nav(current_word_index, seed_len, seed_words.len()));

    let word_entered = Rc::new(RefCell::new(String::new()));
    let current_index = Rc::new(RefCell::new(current_word_index));

    set_word_display(ui, &word_entered);

    let ui_weak = Rc::downgrade(ui);
    let word_entered_rc = word_entered.clone();
    let current_index_rc = current_index.clone();

    ui.on_pressed(move |item| {
        if let Some(ui) = ui_weak.upgrade() {
            let is_seed_check = unsafe { SEED_CHECK_MODE };
            let seed_len = if is_seed_check { 3 } else { state().lock().get_seed_length().unwrap_or(24) };
            let seed_words = unsafe { SEED_WORDS_ENTERED.as_slice() };
            let idx = *current_index_rc.borrow();
            
            // Handle Done button
            if item.text == "Done" {
                log_info!("Done pressed - all seed words entered");
                ui.set_small_text_items(ModelRc::new(VecModel::from(vec![])));
                if is_seed_check {
                    // Verify the entered words
                    if verify_seed_check() {
                        log_info!("Seed check passed!");
                        navigate_to(Screen::EncryptingSeed);
                    } else {
                        log_info!("Seed check failed!");
                        show_alert("Seed check failed! Please try again.");
                    }
                } else {
                    navigate_to(Screen::EncryptingSeed);
                }
                return;
            }
            
            // Handle Prev word button
            if item.text == "Prev word" {
                if idx > 0 {
                    let new_idx = idx - 1;
                    *current_index_rc.borrow_mut() = new_idx;
                    unsafe { CURRENT_WORD_INDEX = new_idx; }
                    
                    // Show the previous word
                    log_info!("Moving to previous word index {}", new_idx);
                    {
                        log_info!("Mutating word_entered for previous word");
                        let mut w = word_entered_rc.borrow_mut();
                        w.clear();
                        log_info!("Cleared word_entered, now setting to previous word");
                        if new_idx < seed_words.len() {
                            w.push_str(&seed_words[new_idx]);
                        }
                        log_info!("Set word_entered to previous word: {}", *w);
                    }
                    set_word_display(&ui, &word_entered_rc);
                    ui.set_small_text_items(ModelRc::new(VecModel::from(vec![])));
                    ui.set_header_title(get_header_title(new_idx, seed_len, is_seed_check));
                    ui.set_buttons(create_abc_buttons_with_nav(new_idx, seed_len, seed_words.len()));
                    set_mode(EnterSeedMode::LetterSeq);
                    log_info!("Moved to previous word index {}", new_idx);
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
                            w.push_str(&seed_words[new_idx]);
                        }
                    }
                    set_word_display(&ui, &word_entered_rc);
                    ui.set_small_text_items(ModelRc::new(VecModel::from(vec![])));
                    ui.set_header_title(get_header_title(new_idx, seed_len, is_seed_check));
                    ui.set_buttons(create_abc_buttons_with_nav(new_idx, seed_len, seed_words.len()));
                    set_mode(EnterSeedMode::LetterSeq);
                }
                return;
            }
            
            // Handle backspace
            if item.text == "<" {
                let mut w = word_entered_rc.borrow_mut();
                w.pop();
                ui.set_small_text_items(ModelRc::new(VecModel::from(vec![])));
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
                        let mut words = unsafe { SEED_WORDS_ENTERED.clone() };
                        if idx < seed_words.len() {
                            // Replace existing word
                            words[idx] = w.clone();
                            unsafe { SEED_WORDS_ENTERED = words; }
                        } else {
                            // Append new word
                            unsafe { SEED_WORDS_ENTERED.push(w.clone()); }
                        }
                        log_info!("Entered seed word {}: {}", idx + 1, *w);
                    }
                    
                    let words = unsafe { SEED_WORDS_ENTERED.as_slice() };
                    let next_index = idx + 1;

                    if next_index >= seed_len {
                        log_info!("All seed words entered: {:?}", seed_words);
                        // Show Done button
                        *current_index_rc.borrow_mut() = seed_len - 1;
                        unsafe { CURRENT_WORD_INDEX = seed_len - 1; }
                        
                        {
                            let mut w = word_entered_rc.borrow_mut();
                            w.clear();
                            w.push_str(&words[seed_len - 1]);
                        }
                        set_word_display(&ui, &word_entered_rc);
                        
                        ui.set_header_title(get_header_title(seed_len - 1, seed_len, is_seed_check));
                        ui.set_buttons(create_abc_buttons_with_nav(seed_len - 1, seed_len, words.len()));
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
                            w.push_str(&words[next_index]);
                        }
                    }
                    set_word_display(&ui, &word_entered_rc);

                    ui.set_header_title(get_header_title(next_index, seed_len, is_seed_check));
                    ui.set_buttons(create_abc_buttons_with_nav(next_index, seed_len, words.len()));
                    set_mode(EnterSeedMode::LetterSeq);
                }
                EnterSeedMode::LetterSeq => {
                    ui.set_small_text_items(ModelRc::new(VecModel::from(vec![])));
                    ui.set_buttons(letters_from_sequence(item));
                    set_mode(EnterSeedMode::Letter);
                }
                EnterSeedMode::Letter => {
                    let mut w = word_entered_rc.borrow_mut();
                    w.push_str(&item.text);
                    let matches_found = find_bip39_matches(&w);
                    if let Some(matches) = matches_found {
                      if matches.len() <= 4 {
                          // Show matching words
                          let mut word_buttons: Vec<ScreenButton> = Vec::new();
                          for (i, word) in matches.iter().enumerate() {
                                // Exact match - select this word
                                if word == w.as_str() {
                                    log_info!("Exact match found for word: {}", word);
                                    // Update or append word at current index
                                    let mut word_list = unsafe { SEED_WORDS_ENTERED.clone() };
                                    if idx < seed_words.len() {
                                        // Replace existing word
                                        word_list[idx] = word.to_ascii_lowercase();
                                        unsafe { SEED_WORDS_ENTERED = word_list; }
                                    } else {
                                        // Append new word
                                        unsafe { SEED_WORDS_ENTERED.push(word.to_ascii_lowercase()); }
                                    }
                                    
                                    let seed_words = unsafe { SEED_WORDS_ENTERED.as_slice() };
                                    let next_index = idx + 1;

                                    if next_index >= seed_len {
                                        log_info!("All seed words entered: {:?}", seed_words);
                                        // Show Done button
                                        *current_index_rc.borrow_mut() = seed_len - 1;
                                        unsafe { CURRENT_WORD_INDEX = seed_len - 1; }
                                        
                                        {
                                            let mut w = word_entered_rc.borrow_mut();
                                            w.clear();
                                            w.push_str(&seed_words[seed_len - 1]);
                                        }
                                        set_word_display(&ui, &word_entered_rc);
                                        
                                        ui.set_header_title(get_header_title(seed_len - 1, seed_len, is_seed_check));
                                        ui.set_buttons(create_abc_buttons_with_nav(seed_len - 1, seed_len, seed_words.len()));
                                        set_mode(EnterSeedMode::LetterSeq);
                                        return;
                                    }
                                    drop(w);

                                    // Move to next word
                                    *current_index_rc.borrow_mut() = next_index;
                                    unsafe { CURRENT_WORD_INDEX = next_index; }
                                    
                                    {
                                        let mut w = word_entered_rc.borrow_mut();
                                        w.clear();
                                        if next_index < seed_words.len() {
                                            w.push_str(&seed_words[next_index]);
                                        }
                                    }
                                    set_word_display(&ui, &word_entered_rc);

                                    ui.set_header_title(get_header_title(next_index, seed_len, is_seed_check));
                                    ui.set_buttons(create_abc_buttons_with_nav(next_index, seed_len, seed_words.len()));
                                    set_mode(EnterSeedMode::LetterSeq);
                                    return;
                                }
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
                          ui.set_buttons(ModelRc::new(VecModel::from(word_buttons)));
                          drop(w);
                          set_mode(EnterSeedMode::WordEntered);
                          set_word_display(&ui, &word_entered_rc);
                          return;
                      }
                    } else {
                        let small_text_items = ModelRc::new(VecModel::from(vec![
                            ScreenItem {
                                text: "Word not found".into(),
                                x: 100.0,
                                y: 70.0,
                                width: 120.0,
                                height: 20.0,
                            },
                        ]));
                        ui.set_small_text_items(small_text_items);
                        // set_mode(EnterSeedMode::LetterSeq);
                        // set_word_display(&ui, &word_entered_rc);
                        // return;       
                    }
                    drop(w);
                    set_word_display(&ui, &word_entered_rc);
                    let seed = unsafe {SEED_WORDS_ENTERED.as_slice()};
                    ui.set_buttons(create_abc_buttons_with_nav(idx, seed_len, seed.len()));
                    set_mode(EnterSeedMode::LetterSeq);
                }
            }
        } else {
            log_info!("UI reference lost in enter seed screen callback");
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
      #[cfg(feature = "minifb")]
      ScreenButton { 
          text: "zero zoo".into(),  
          width: 240.0, 
          height: 40.0,
          has_border: false, 
          x: 40.0, 
          y: seed_12_y + y_gap * 3.0,
          inverted: false,
      },
  ]));
  ui.set_buttons(buttons);
  let vault_arc = firmware().vault.clone();
  ui.on_pressed(move |item| {
      match item.text.as_str() {
          "Generate new seed" => {
            if let Ok(()) = vault_arc.lock().generate_mnemonic() {
                log_info!("Navigate to GENERATE SEED screen");
                navigate_to(Screen::GenerateSeed);
            } else {
                log_info!("Error generating new mnemonic seed");
            }
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
          "zero zoo" => {
              state().lock().set_seed_length(12);
              unsafe {
                  SEED_CHECK_MODE = false;
                  let mnemonic = "zero zero zero zero zero zero zero zero zero zero zero zoo";
                  SEED_WORDS_ENTERED = mnemonic.split_whitespace().map(|w| w.to_ascii_lowercase()).collect();
              }
              navigate_to(Screen::EncryptingSeed);
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
        Some("Show seed"),
        || {
            let vault_arc = firmware().vault.clone();
            let vault = vault_arc.lock();
            
            // Get the actual mnemonic from vault and convert to word indices
            let mnemonic = vault.get_mnemonic();
            let mnemonic = if let Ok(mnemonic) = &mnemonic {
                log_info!("Generated mnemonic: {}", mnemonic);
                mnemonic.clone()
            } else {
                log_info!("Error retrieving generated mnemonic");
                return;
            };
            let array = mnemonic.split_whitespace().map(|w| w.to_ascii_lowercase()).collect::<Vec<String>>();

            // if array.is_err() {
            //     show_alert("Error converting mnemonic to indices for verification");
            //     return;
            // }
            // let array = array.unwrap();

            if let Ok(seed_len) = vault.get_mnemonic_len() {
              log_info!("Generated seed for verification, length: {}", seed_len);
              
              // Generate random word indices to verify (no duplicates)
              let indices = generate_random_word_indices(seed_len);
              log_info!("Verifying seed words at indices: {:?}", indices);
              
              // Initialize seed check mode with the expected seed
              init_seed_check(&array, indices);
              
              navigate_to(Screen::SeedCheck);
            }
          
        },
        || {
            navigate_to(Screen::ShowSeedBackup);
        }
    );
}

pub fn create_encrypting_seed_screen(ui: &Rc<MainWindow>) {
    let button_x = 0.0;
    let button_y = 62.0;
    let button_gap = 28.0;
    let message = "Your seed phrase\\\\is being ciphered";
    let header_title = "ENCRYPTING";
    
    ui.set_center_text(true);
    // parse message into lines if too long, \\ is line break
    let message_lines: Vec<&str> = message.split("\\\\").collect();

    let lines = message_lines.len();
    log_info!("GenericProgressBar: Message has {} lines", lines);
    for (i, line) in message_lines.iter().enumerate() {
        log_info!("Message line {}: {}", i, line);
        if line.len() > 25 {
            log_info!("Warning: line {} is too long ({} characters)", i, line.len());
            // fail here
            panic!("Line {} is too long ({} characters)", i, line.len());
        }
    }

    // Create screen items based on number of lines
    let mut items_vec = vec![];
    for (i, line) in message_lines.iter().enumerate() {
        items_vec.push(ScreenItem { 
            text: (*line).into(), 
            width: 320.0,
            height: 25.0,
            x: button_x, 
            // add a 10.0 gap between 2. and 3. line
            y: if i > 1 { button_y + (i as f32) * button_gap + 10.0 } else { button_y + (i as f32) * button_gap },
        });
    }

    let last_y = if lines > 0 {
        button_y + ((lines - 1) as f32) * button_gap + 10.0
    } else {
        button_y
    };

    let buttons = ModelRc::new(VecModel::from(vec![
        ScreenButton {
            text: "ENCRYPT".into(),  
            has_border: true, 
            x: 84.0, 
            y: 158.0,
            width: 152.0, 
            height: 25.0,
            inverted: true,
        },
    ]));
    
    let items = ModelRc::new(VecModel::from(items_vec));
    
    ui.set_items(items);
    ui.set_buttons(buttons);
    ui.set_header_title(slint::SharedString::from(header_title));
    ui_set_progress_bar_properties(84, 158, 152, 25, 1);

    ui.on_pressed(move |item| {
        if item.text == "ENCRYPT" {
          let seed_check_mode = unsafe { SEED_CHECK_MODE };
          on_encrypt_clicked(seed_check_mode);
        }
    });

}