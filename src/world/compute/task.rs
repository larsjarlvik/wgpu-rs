pub struct Task<'a> {
    pub label: String,
    pub pipeline: &'a wgpu::ComputePipeline,
    pub run_times: u32,
    pub stage_count: u32,
}

impl<'a> Task<'a> {
    pub fn new(label: &str, pipeline: &'a wgpu::ComputePipeline, run_times: u32, stage_count: u32) -> Self {
        Self {
            label: label.to_string(),
            pipeline,
            run_times,
            stage_count,
        }
    }
}
