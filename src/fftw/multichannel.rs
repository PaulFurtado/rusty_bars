use fftw::plan::*;


/// An FFTW Plan wrapper for multiple channels of data.
pub struct MultiChannelFft {
    /// The size of the FFTs to be run
    pub size: usize,
    /// The number of channels
    pub channel_count: usize,
    /// The plans for each channel
    pub channel_plans: Vec<FftwPlan>
}


impl MultiChannelFft {
    //// Create and initialize a new MultiChannelFft
    pub fn new(size: usize, channel_count: usize) -> MultiChannelFft {
        let mut channel_plans: Vec<FftwPlan> = Vec::with_capacity(channel_count);

        for _ in 0..channel_count {
            channel_plans.push(FftwPlan::new(size));
        }
        channel_plans.shrink_to_fit();

        MultiChannelFft {
            size: size,
            channel_count: channel_count,
            channel_plans: channel_plans,
        }
    }

    /// Return a borrowed reference to the plan for a channel
    pub fn get_channel<'a>(&'a self, index: usize) -> Option<&'a FftwPlan> {
        self.channel_plans.get(index)
    }

    /// Return a mutable borrowed reference to the plan for a channel
    pub fn get_channel_mut<'a>(&'a mut self, index: usize) -> Option<&'a mut FftwPlan> {
        self.channel_plans.get_mut(index)
    }

    /// Execute all of the FFT channels
    pub fn execute(&mut self) {
        for plan in self.channel_plans.iter_mut() {
            plan.execute();
        }
    }

    /// Gets a vector of all of the input slices
    pub fn get_inputs<'a>(&'a mut self) -> Vec<&'a mut [f64]> {
        let mut inputs: Vec<&mut [f64]> = Vec::with_capacity(self.channel_count);
        for channel in self.channel_plans.iter_mut() {
            inputs.push(channel.get_input_slice());
        }
        inputs
    }
}
