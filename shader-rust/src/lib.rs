#![no_std]

use spirv_std::spirv;
use spirv_std::glam::{UVec3, Vec3};

#[spirv(compute(threads(1)))]
pub fn cs_main(
    #[spirv(workgroup_id)] workgroup_id: UVec3,
    #[spirv(num_workgroups)] num_workgroups: UVec3,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 0)] color: &mut [u32],
    #[spirv(uniform, descriptor_set = 0, binding = 1)] _look_at: &Vec3,
) {
    let index = (workgroup_id.y * num_workgroups.x + workgroup_id.x) as usize;
    color[index] = 0xff00ffffu32;
}
