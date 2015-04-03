#![allow(unstable)]

extern crate libc;

use self::libc::{c_int, c_char};
use ncurses_wrapper::{Window,endwin};
use analyze_spectrum::scale_fft_output;


/// Loops through an iterator of f64 and gets the min and max values.
/// The min/max functions in the standard library don't work on floats.
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



/// Resize the row buffer to width
fn resize_rowbuf(row: &mut Vec<c_char>, width: usize) {
    while row.len() < width {
        row.push('#' as c_char);
    }
    while row.len() > width {
        row.pop().unwrap();
    }
    row.shrink_to_fit();
}


pub struct Visualizer{
   // The ncurses Window object
   win: Window,
   // A buffer of characters for a row on the screen (used to reduce calls to
   // the ncurses addstr function)
   rows: Vec<Vec<c_char>>,
   // The width of the window the last time the animation was called
   width: usize,
   // The height of the window the last time the animation was called
   height: usize
}


impl Visualizer {
    /// Instantiate a new visualizer. Takes over the terminal with ncurses.
    pub fn new() -> Visualizer {
        let mut win = match Window::initscr() {
            Err(_) => panic!("Failed to initialize screen!"),
            Ok(win) => win
        };

        // Disable the cursor so it's not moving all around the screen when the
        // animation is rendering.
        match win.curs_set(0) {
            Err(_) => panic!("Failed to disable cursor!"),
            Ok(_) => {}
        }

        Visualizer{
            win: win,
            rows: Vec::new(),
            width: 0,
            height: 0
        }
    }

    /// Get the width of the scren in columns. Callers can use this to
    /// determine the minimum amount of data the animation needs to fill the
    /// screen.
    pub fn get_width(&self) -> usize {
        self.win.get_max_x().unwrap() as usize - 1
    }

    /// Adds or removes rows if the window size is changed.
    fn update_row_count(&mut self, height: usize) {
        while self.rows.len() < height {
            self.rows.push(Vec::new());
        }
        while self.rows.len() > height {
            self.rows.pop();
        }
    }

    /// Resizes each of hte row buffers to the given width
    fn resize_rowbufs(&mut self, width: usize) {
        for row in self.rows.iter_mut() {
            resize_rowbuf(row, width);
        }
    }

    /// Do any necessary adjustments for a window size change. This gets
    /// called when we fetch the max_yx
    fn update_size(&mut self) {
        let (max_y, max_x) = self.win.get_max_yx().unwrap();
        let height: usize = max_y as usize;
        let width: usize = max_x as usize - 1;

        if self.width != width || self.height != height {
            self.update_row_count(height);
            self.resize_rowbufs(width);
            self.width = width;
            self.height = height;
        }
    }

    /// Render a single frame of the animation
    pub fn render_frame(&mut self, data: &Vec<f64>) -> Result<(), c_int> {
        self.update_size();

        let data = scale_fft_output(data, self.width as usize);
        let (_, max_val) = get_min_max(&mut data.iter());
        let scaled: Vec<usize> = data.iter()
            .map(|&x| {
                if x < 1.0 {
                    0
                } else {
                    ((x / max_val) * (self.height as f64 - 1.0)) as usize
                }
            })
            .collect();

        for (y, row) in self.rows.iter_mut().enumerate().rev() {
            for (x, val) in row.iter_mut().enumerate() {
                *val = (if x >= scaled.len() {
                    'X'
                } else {
                    let val = scaled[x];
                    if val >= y {
                        '|'
                    } else {
                        ' '
                    }
                }) as c_char;
            }

            match self.win.addbytes((self.height - y -1) as c_int, 0, row) {
                Err(_) => {
                    // Happens when window is resized. Skip the frame.
                    return Ok(());
                },
                Ok(_) => { }
            }
        }

        // Add some info so you can see the decisions it's making
        let debuginfo = format!(" width: {}, height: {}, bars: {} ", self.width, self.height, scaled.len());
        let _ = self.win.addstr(0, (self.width - debuginfo.len()) as c_int, debuginfo.as_slice());

        // Calling refresh makes it actually take effect
        try!(self.win.refresh());

        Ok(())
    }
}


impl Drop for Visualizer {
    /// Call endwin to clean up the termninal when the Visualizer is dallocated
    fn drop(&mut self) {
        match endwin() {
            _ => {}
        };
    }
}
