// MP-103.b — per-vertex closest-point distance shader.
//
// Brute-force kernel cross-vendor (NVIDIA + AMD + Intel + Apple). Usa `array<f32>`
// con stride 4-byte explícito en vez de `array<vec3<f32>>` para evitar quirks de
// alignment vec3 (que en algunos backends Metal/Vulkan toma 16 bytes y en otros 12).
//
// Buffers:
//   * `src_xyz` (read)         — flat f32 array packed as [x0,y0,z0, x1,y1,z1, ...].
//   * `dst_xyz` (read)         — same layout.
//   * `distances` (read_write) — f32 output, longitud = src_count.
//   * `params`   uniform        — { src_count: u32, dst_count: u32 }.

struct Params {
    src_count: u32,
    dst_count: u32,
};

@group(0) @binding(0) var<storage, read>       src_xyz:   array<f32>;
@group(0) @binding(1) var<storage, read>       dst_xyz:   array<f32>;
@group(0) @binding(2) var<storage, read_write> distances: array<f32>;
@group(0) @binding(3) var<uniform>             params:    Params;

const FAR_AWAY: f32 = 1.0e18;

@compute @workgroup_size(64, 1, 1)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i: u32 = gid.x;
    if (i >= params.src_count) {
        return;
    }
    let base_i: u32 = i * 3u;
    let sx: f32 = src_xyz[base_i + 0u];
    let sy: f32 = src_xyz[base_i + 1u];
    let sz: f32 = src_xyz[base_i + 2u];

    if (params.dst_count == 0u) {
        distances[i] = 0.0;
        return;
    }

    var best_d2: f32 = FAR_AWAY;
    for (var j: u32 = 0u; j < params.dst_count; j = j + 1u) {
        let base_j: u32 = j * 3u;
        let dx: f32 = sx - dst_xyz[base_j + 0u];
        let dy: f32 = sy - dst_xyz[base_j + 1u];
        let dz: f32 = sz - dst_xyz[base_j + 2u];
        let d2: f32 = dx*dx + dy*dy + dz*dz;
        if (d2 < best_d2) {
            best_d2 = d2;
        }
    }
    distances[i] = sqrt(best_d2);
}
