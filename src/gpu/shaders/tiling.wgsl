// tiling.wgsl — TextTiling TF cosine + depth valleys matmul
@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {}
