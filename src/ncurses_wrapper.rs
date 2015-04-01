#![allow(unstable)]

extern crate libc;

use std::ffi::CString;
use std::io::timer;
use std::time::duration::Duration;
use self::libc::{c_int};


mod ext {
    extern crate libc;
    use self::libc::{c_int, c_char};

    #[repr(C)]
    pub struct WINDOW;

    #[link(name="ncurses")]
    extern {
        pub fn initscr() -> *mut WINDOW;
        pub fn endwin() -> c_int;
        pub fn wrefresh(win: *mut WINDOW) -> c_int;
        pub fn mvwaddstr(win: *mut WINDOW, y: c_int, x: c_int, text: *const c_char) -> c_int;
        pub fn getmaxy(win: *mut WINDOW) -> c_int;
        pub fn getmaxx(win: *mut WINDOW) -> c_int;
    }
}


/// Safe wrapper for the ncurses endwin function
pub fn endwin() -> Result<(), c_int> {
    let result = unsafe{ ext::endwin() };
    if result == 0 {
        return Ok(())
    }
    else {
        return Err(result)
    }
}


/// Turns an ncurses error code into a result so we can toss errors up the stack
fn handle_err(result: c_int) -> Result<c_int, c_int> {
    if result < 0 {
        Err(result)
    } else {
        Ok(result)
    }
}


/// Wraps an ncruses WINDOW struct with the basic functions for manipulating
/// the window.
pub struct Window {
    w: *mut ext::WINDOW
}


impl Window {
    /// Initialize the screen and get a window
    pub fn initscr() -> Result<Window, c_int> {
        let window = unsafe{ ext::initscr() };
        if window.is_null() {
            Err(-1)
        }
        else {
            Ok(Window{w: window})
        }
    }

    /// Add a string to the screen starting at the given location
    pub fn addstr(&mut self, y: c_int, x: c_int, text: &str) -> Result<c_int, c_int> {
        let c_text = CString::from_slice(text.as_bytes());
        handle_err(unsafe{
            ext::mvwaddstr(self.w, y, x, c_text.as_ptr())
        })
    }

    /// Refresh the output on the display
    pub fn refresh(&mut self) -> Result<c_int, c_int> {
        handle_err(unsafe{ ext::wrefresh(self.w) })
    }

    /// Gets the maximum y on the screen
    pub fn get_max_y(&mut self) -> Result<c_int, c_int> {
        handle_err(unsafe{ ext::getmaxy(self.w) })
    }

    /// Gets the maximum x on the screen
    pub fn get_max_x(&mut self) -> Result<c_int, c_int> {
        handle_err(unsafe{ ext::getmaxx(self.w) })
    }

    // Gets a tuple containing the maximum y and x on the screen
    pub fn get_max_yx(&mut self) -> Result<(c_int, c_int), c_int> {
        Ok((try!(self.get_max_y()), try!(self.get_max_x())))
    }

}


fn test() -> Result<(), c_int> {
    let mut win = try!(Window::initscr());
    try!(win.addstr(0, 0, "test1"));
    try!(win.addstr(1, 0, "test2"));
    try!(win.addstr(2, 0, "test3"));
    try!(win.addstr(5, 10, "test5"));
    try!(win.refresh());
    let (y, x) =  try!(win.get_max_yx());
    println!("y={}, x={}", y, x);
    timer::sleep(Duration::milliseconds(10000));

    try!(endwin());
    Ok(())
}



fn main() {
    test().unwrap();

}
