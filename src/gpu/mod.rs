pub mod batch;

// GPU backend was experimental (wgpu stubs in shaders/*) and never
// used on the tokenizer hot path. Batch parallelism is CPU (rayon).
// Kept as a module for the is_available/gpu_info surface; wgpu deps
// removed from Cargo.toml 1.0.0.

#[derive(Debug)]
pub enum GpuState {
    Unavailable(String),
}

static GPU_STATE: GpuState =
    GpuState::Unavailable("gpu feature removed in 1.0.0 — batch uses rayon on CPU");

pub fn is_available() -> bool {
    false
}

pub fn gpu_info() -> String {
    match &GPU_STATE {
        GpuState::Unavailable(reason) => {
            let esc = reason.replace('"', "'").replace('\\', "/");
            format!(r#"{{"available": false, "reason": "{esc}"}}"#)
        }
    }
}

pub fn warmup() {
    let _ = &*batch::PAR_CHUNK;
}
