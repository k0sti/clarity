// Sampling schedules for Flux diffusion

/// Get the timestep schedule for a given number of steps
pub fn get_schedule(num_steps: usize) -> Vec<f32> {
    let mut timesteps = Vec::with_capacity(num_steps);

    for i in 0..num_steps {
        let t = 1.0 - (i as f32) / (num_steps as f32);
        timesteps.push(t);
    }

    timesteps
}

/// Linear schedule from 1.0 to 0.0
pub fn linear_schedule(num_steps: usize) -> Vec<f32> {
    (0..num_steps)
        .map(|i| 1.0 - (i as f32) / (num_steps as f32))
        .collect()
}

/// Cosine schedule
pub fn cosine_schedule(num_steps: usize) -> Vec<f32> {
    (0..num_steps)
        .map(|i| {
            let t = (i as f32) / (num_steps as f32);
            let cosine_t = ((t * std::f32::consts::PI / 2.0).cos()).powi(2);
            1.0 - cosine_t
        })
        .collect()
}
