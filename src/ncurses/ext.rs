extern crate libc;
use self::libc::{c_int, c_char};

/// Module for external ncurses functions and types

// An opaque pointer to an ncurses window representation.
pub enum Window {}


#[link(name="ncurses")]
extern {
    pub fn initscr() -> *mut Window;
    pub fn endwin() -> c_int;
    pub fn wrefresh(win: *mut Window) -> c_int;
    pub fn mvwaddstr(win: *mut Window, y: c_int, x: c_int, text: *const c_char) -> c_int;
    pub fn mvwaddnstr(win: *mut Window, y: c_int, x: c_int, text: *const c_char, n: c_int) -> c_int;
    pub fn getmaxy(win: *mut Window) -> c_int;
    pub fn getmaxx(win: *mut Window) -> c_int;
    pub fn curs_set(visibility: c_int) -> c_int;
}
