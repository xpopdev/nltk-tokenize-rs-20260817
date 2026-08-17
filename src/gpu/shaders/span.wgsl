// span.wgsl — parallel string search + byte_to_char prefix scan
@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {}
