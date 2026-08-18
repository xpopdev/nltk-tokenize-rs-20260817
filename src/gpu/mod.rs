use std::sync::LazyLock;

pub mod batch;

#[derive(Debug)]
pub enum GpuState {
    Unavailable(String),
}

static GPU_STATE: LazyLock<GpuState> =
    LazyLock::new(|| GpuState::Unavailable("gpu feature removed in 1.0.0 — batch uses rayon on CPU".to_string()));

pub fn is_available() -> bool {
    false
}

pub fn gpu_info() -> String {
    match &*GPU_STATE {
        GpuState::Unavailable(reason) => {
            let esc = reason.replace('"', "'").replace('\\', "/");
            format!(r#"{{"available": false, "reason": "{esc}"}}"#)
        }
    }
}

pub fn warmup() {
    let _ = &*GPU_STATE;
    let _ = &*batch::PAR_CHUNK;
}
