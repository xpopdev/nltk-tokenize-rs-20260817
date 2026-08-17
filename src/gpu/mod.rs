use std::sync::LazyLock;

pub mod batch;

// ── GPU state ──

#[derive(Debug)]
pub enum GpuBackend { Vulkan, Metal, Dx12, Gl, Cuda, BrowserWebGpu }

#[derive(Debug)]
pub enum GpuState {
    Unavailable(String),
    Ready { backend: String, device_name: String },
}

fn init_gpu_state() -> GpuState {
    #[cfg(feature = "gpu")]
    {
        return init_wgpu();
    }
    #[cfg(not(feature = "gpu"))]
    {
        GpuState::Unavailable("gpu feature not enabled — build with --features gpu".to_string())
    }
}

#[cfg(feature = "gpu")]
fn init_wgpu() -> GpuState {
    use wgpu::{Instance, InstanceDescriptor, RequestAdapterOptions, DeviceDescriptor, Features, Limits, MemoryHints};
    let instance = Instance::new(InstanceDescriptor::default());
    let adapter = pollster::block_on(instance.request_adapter(&RequestAdapterOptions::default()));
    match adapter {
        None => GpuState::Unavailable("no wgpu adapter found".to_string()),
        Some(adapter) => {
            let info = adapter.get_info();
            let backend = format!("{:?}", info.backend).to_lowercase();
            let name = if info.name.is_empty() { backend.clone() } else { info.name.clone() };
            match pollster::block_on(adapter.request_device(
                &DeviceDescriptor {
                    label: Some("ported_lib"),
                    required_features: Features::empty(),
                    required_limits: Limits::default(),
                    memory_hints: MemoryHints::Performance,
                },
                None,
            )) {
                Ok(_) => GpuState::Ready { backend, device_name: name },
                Err(e) => GpuState::Unavailable(format!("device request failed: {e}")),
            }
        }
    }
}

static GPU_STATE: LazyLock<GpuState> = LazyLock::new(init_gpu_state);

pub fn is_available() -> bool {
    matches!(*GPU_STATE, GpuState::Ready { .. })
}

pub fn gpu_info() -> String {
    match &*GPU_STATE {
        GpuState::Ready { backend, device_name } => {
            format!(r#"{{"available": true, "backend": "{backend}", "device": "{device_name}"}}"#)
        }
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
