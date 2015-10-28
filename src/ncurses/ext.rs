extern crate libc;
use self::libc::{c_int, c_char};

/// Module for external ncurses functions and types

#[repr(C)]
pub struct WINDOW;

#[link(name="ncurses")]
extern {
    pub fn initscr() -> *mut WINDOW;
    pub fn endwin() -> c_int;
    pub fn wrefresh(win: *mut WINDOW) -> c_int;
    pub fn mvwaddstr(win: *mut WINDOW, y: c_int, x: c_int, text: *const c_char) -> c_int;
    pub fn mvwaddnstr(win: *mut WINDOW, y: c_int, x: c_int, text: *const c_char, n: c_int) -> c_int;
    pub fn getmaxy(win: *mut WINDOW) -> c_int;
    pub fn getmaxx(win: *mut WINDOW) -> c_int;
    pub fn curs_set(visibility: c_int) -> c_int;
}
