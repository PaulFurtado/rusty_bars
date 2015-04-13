#![allow(unstable)]

use std::slice;
use std::num::Float;
use fftw::multichannel::MultiChannelFft;
use fftw::hanning::HanningWindowCalculator;


/// Audio FFT for 16bit little endian audio data (S16LE)
pub struct AudioFft {
    /// The multichannel fft object that does the work for us
    multichan_fft: MultiChannelFft,
    /// The input cursor indicates how much data has been read in. Input is
    /// 16bit integers, inerleaved by channels. The so that means the maximum
    /// value of input_cursor is channel_count * fft_size
    input_cursor: usize,
    /// The number of elements needed to fill an FFT. This is equal to the size
    /// of the FFT times the number of channels
    required_input: usize,
    /// The size of the FFT
    fft_size: usize,
    /// The number of audio channels. Ex: 2 for stereo audio.
    channel_count: usize,
    /// Helper for executing the Hanning window function as data is inserted
    hanning: HanningWindowCalculator,
    /// Holds output for the combined channels
    output: Vec<f64>
}


impl AudioFft {
    /// Create a new AudioFft
    pub fn new(fft_size: usize, channel_count: usize) -> AudioFft {
        let mut out_vec = Vec::with_capacity(fft_size/2);
        for _ in (0..fft_size/2) {
            out_vec.push(0.0);
        }
        AudioFft {
            multichan_fft: MultiChannelFft::new(fft_size, channel_count),
            input_cursor: 0,
            fft_size: fft_size,
            channel_count: channel_count,
            required_input: channel_count * fft_size,
            hanning: HanningWindowCalculator::new(fft_size),
            output: out_vec,
        }
    }


    /// Exeuce the FFT
    pub fn execute(&mut self) {
        self.multichan_fft.execute();
        self.input_cursor = 0;
    }

    /// Allows a client to feed data into the FFT in chunks. This is useful for
    /// ineracting with PulseAudio because its asynchronous API gives audio data
    /// in arbitrary chunk sizes depending on how much data is available.
    ///
    /// Arguments:
    ///     input: A slice pointing at S16LE audio data for the number of
    ///            channels in self.channel_count
    /// Returns:
    ///     The number of bytes it read. If the number of bytes returned is
    ///     less than the input size, the FFT is ready to execute.
    pub fn feed_data(&mut self, input: &[i16]) -> usize {
        let mut bytes_read: usize = 0;

        let mut inputs = self.multichan_fft.get_inputs();

        for value in input.iter() {
            // If there is enough data to run the FFT, return the number of
            // bytes that were read out of this input slice
            if self.input_cursor == self.required_input {
                return bytes_read;
            }

            // The channel number and index the current value is for
            let channel_num = self.input_cursor % self.channel_count;
            let channel_index = self.input_cursor / self.channel_count;

            // Compute the hanning window value for the element set the input
            inputs[channel_num][channel_index] = self.hanning.get_value(channel_index, *value as f64);

            bytes_read += 1;
            self.input_cursor += 1;
        }

        bytes_read
    }

    /// Wrapper for feed_data which takes an &[u8] slice instead of an &[i16]
    /// slice. Simply casts the slice to an i16 slice then calls feed_data on
    /// it.
    pub fn feed_u8_data(&mut self, input: &[u8]) -> usize {
        let i16_ptr: *const i16 = input.as_ptr() as *const i16;
        self.feed_data(unsafe{ slice::from_raw_buf(&i16_ptr, input.len()/2) }) * 2
    }

    /// Computes the combined output of all channels into the output field of
    /// this struct. Every time compute_output is called, it reuses the same
    /// output vector to avoid allocations.
    pub fn compute_output(&mut self) {
        let mut first = true;
        for channel in self.multichan_fft.channel_plans.iter() {
            for (index, &value) in channel.get_output_slice().slice_to(self.fft_size/2).iter().enumerate() {
                // Turn the FFT output value into decibals
                let power: f64 = 20.0 * value.abs().log10();
                // If it's bigger than the biggest value for this channel for
                // this execution, then replace the current value
                if first || power > self.output[index] {
                    self.output[index] = power;
                }
            }
            first = false;
        }
    }

    /// Borrow the combined output vector
    pub fn get_output(&self) -> &[f64] {
        self.output.as_slice()
    }
}
