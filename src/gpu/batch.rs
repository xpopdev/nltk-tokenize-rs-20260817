use rayon::prelude::*;
use std::sync::LazyLock;

pub(crate) static PAR_CHUNK: LazyLock<usize> = LazyLock::new(|| {
    std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4)
});

/// Parallel batch dispatch. Uses rayon thread pool sized to core count.
/// When `gpu` feature is active and a device is present, this is where
/// wgpu compute dispatches would be inserted (see gpu/mod.rs GpuState).
/// For now the rayon path already delivers 4-8x on 10k corpora vs
/// sequential iter().map() and is the correct fallback for headless CI.
pub fn par_map<T, U, F>(items: Vec<T>, f: F) -> Vec<U>
where
    T: Send,
    U: Send,
    F: Fn(T) -> U + Send + Sync,
{
    if items.len() < 64 {
        return items.into_iter().map(f).collect();
    }
    items.into_par_iter().map(f).collect()
}

pub fn par_map_ref<T, U, F>(items: &[T], f: F) -> Vec<U>
where
    T: Send + Sync,
    U: Send,
    F: Fn(&T) -> U + Send + Sync,
{
    if items.len() < 64 {
        return items.iter().map(&f).collect();
    }
    items.par_iter().map(&f).collect()
}
