#![allow(unstable)]

extern crate libc;

use self::libc::c_int;
use ncurses_wrapper::{Window,endwin};
use analyze_spectrum::scale_fft_output;


fn get_min_max<'a, I: Iterator<Item=&'a f64>>(iter: &'a mut I) -> (f64, f64) {
    let mut min: f64 = 0.0;
    let mut max: f64 = 0.0;
    for &x in *iter {
        if x < min {
            min = x;
        }
        if x > max {
            max = x;
        }
    }
    (min, max)
}



pub struct Visualizer {
    win: Window
}

impl Visualizer {
    pub fn new() -> Visualizer {
        Visualizer{win: Window::initscr().unwrap()}
    }

    pub fn render_frame(&mut self, data: &Vec<f64>) -> Result<(), c_int> {
        let (max_y, max_x) = try!(self.win.get_max_yx());
        let data = scale_fft_output(data, max_x as usize);
        let (min_val, max_val) = get_min_max(&mut data.iter());
        let scaled: Vec<usize> = data.iter()
            .map(|&x| ((x / max_val) * (max_y as f64) + 0.5) as usize)
            .collect();

        for y in (0..max_y) {
            for (x, &val) in scaled.iter().enumerate() {
                if (val as c_int) > (max_y - (y as c_int)) {
                    self.win.addstr(y as c_int, x as c_int, "|");
                } else {
                    self.win.addstr(y as c_int, x as c_int, " ");
                }
            }
        }
        self.win.refresh();


        Ok(())
    }

}







impl Drop for Visualizer {
    fn drop(&mut self) {
        endwin();
    }
}
