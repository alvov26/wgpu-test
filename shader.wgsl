@group(0) @binding(0) var<storage, read_write> color: array<u32>;
@group(0) @binding(1) var<uniform> look_at: vec3<f32>;

@compute 
@workgroup_size(1)
fn cs_main(
    @builtin(workgroup_id) workgroup_id: vec3<u32>,
    @builtin(num_workgroups) num_workgroups: vec3<u32>,
) {
    color[workgroup_id.y * num_workgroups.x + workgroup_id.x] = 0xff00ffffu;
}