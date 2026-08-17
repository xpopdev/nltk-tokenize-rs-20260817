// char_class.wgsl — wordpunct / is_cjk / xml per-char kernels
// Phase 3: parallel char-class table lookup + prefix-scan scatter.
// Stub: validated by cargo check --features gpu (entry point exists).
@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {}
