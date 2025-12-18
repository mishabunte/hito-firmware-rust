//! Navigation router for the firmware UI
//! 
//! This module provides screen navigation functionality, managing transitions
//! between different UI screens and maintaining navigation history.

extern crate alloc;
use alloc::vec::Vec;
use alloc::rc::Rc;
use core::cell::{Cell, RefCell};

use crate::slint_generatedMainWindow::MainWindow;
use super::screens::{create_screen, Screen};

/// Pending navigation action
#[derive(Clone, Copy, Debug)]
enum PendingNavigation {
    None,
    NavigateTo(Screen),
    GoBack,
}

/// Global router storage (single-threaded embedded system)
static mut GLOBAL_ROUTER: Option<Router> = None;

/// Pending navigation request (deferred to avoid callback recursion)
static mut PENDING_NAV: PendingNavigation = PendingNavigation::None;

/// Initialize the global router (must be called once at startup)
pub fn init_global_router(ui: Rc<MainWindow>) {
    unsafe {
        GLOBAL_ROUTER = Some(Router::new(ui));
    }
}

/// Request navigation to a screen (deferred until process_pending_navigation is called)
pub fn navigate_to(screen: Screen) {
    unsafe {
        PENDING_NAV = PendingNavigation::NavigateTo(screen);
    }
}

/// Request to navigate back (deferred until process_pending_navigation is called)
pub fn go_back() {
    unsafe {
        PENDING_NAV = PendingNavigation::GoBack;
    }
}

/// Process any pending navigation requests
/// Call this from the main loop, outside of any callbacks
pub fn process_pending_navigation() {
    unsafe {
        let pending = PENDING_NAV;
        PENDING_NAV = PendingNavigation::None;
        
        if let Some(ref router) = GLOBAL_ROUTER {
            match pending {
                PendingNavigation::None => {}
                PendingNavigation::NavigateTo(screen) => {
                    router.navigate(screen);
                }
                PendingNavigation::GoBack => {
                    router.go_back();
                }
            }
        }
    }
}

/// Router struct that manages screen navigation and history
pub struct Router {
    /// Reference to the main Slint window
    ui: Rc<MainWindow>,
    /// Current active screen
    current_screen: RefCell<Screen>,
    /// Navigation history stack (placeholder for back navigation)
    history: RefCell<Vec<Screen>>,
}

impl Router {
    /// Create a new Router instance
    pub fn new(ui: Rc<MainWindow>) -> Self {
        Self {
            ui,
            current_screen: RefCell::new(Screen::EnterPasscode),
            history: RefCell::new(Vec::new()),
        }
    }

    /// Navigate to a new screen
    /// 
    /// This will:
    /// 1. Push the current screen to history (for back navigation)
    /// 2. Clear existing items in the UI
    /// 3. Call the appropriate screen creation function
    /// 4. Update the current screen state
    pub fn navigate(&self, screen: Screen) {
        // Push current screen to history before navigating
        let current = *self.current_screen.borrow();
        if current != screen {
            self.history.borrow_mut().push(current);
        }

        // Update current screen
        *self.current_screen.borrow_mut() = screen;

        // Create the new screen (clears items and sets up new UI)
        self.create_screen(screen);
    }

    /// Navigate back to the previous screen
    /// 
    /// Returns true if navigation was successful, false if history is empty
    pub fn go_back(&self) -> bool {
        if let Some(previous_screen) = self.history.borrow_mut().pop() {
            *self.current_screen.borrow_mut() = previous_screen;
            self.create_screen(previous_screen);
            true
        } else {
            false
        }
    }

    /// Get the current screen
    pub fn current(&self) -> Screen {
        *self.current_screen.borrow()
    }

    /// Clear the navigation history
    pub fn clear_history(&self) {
        self.history.borrow_mut().clear();
    }

    /// Internal function to create/render a screen based on the Screen enum
    fn create_screen(&self, screen: Screen) {
        create_screen(&self.ui, screen);
    }
}
