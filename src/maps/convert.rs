
pub fn glam_to_three_d(mat: &glam::Mat4) -> three_d::Mat4 {
    let cols = mat.to_cols_array();
    three_d::Mat4::new(
        cols[0], cols[1], cols[2], cols[3], cols[4], cols[5], cols[6], cols[7], cols[8], cols[9],
        cols[10], cols[11], cols[12], cols[13], cols[14], cols[15],
    )
}

pub fn dglam_to_three_d(mat: &glam::DMat4) -> three_d::Mat4 {
    let cols = mat.to_cols_array();
    three_d::Mat4::new(
        cols[0] as f32,
        cols[1] as f32,
        cols[2] as f32,
        cols[3] as f32,
        cols[4] as f32,
        cols[5] as f32,
        cols[6] as f32,
        cols[7] as f32,
        cols[8] as f32,
        cols[9] as f32,
        cols[10] as f32,
        cols[11] as f32,
        cols[12] as f32,
        cols[13] as f32,
        cols[14] as f32,
        cols[15] as f32,
    )
}

pub fn three_d_to_glam(mat: &three_d::Mat4) -> glam::DMat4 {
    glam::DMat4::from_cols_array(&[
        mat[0][0] as f64,
        mat[0][1] as f64,
        mat[0][2] as f64,
        mat[0][3] as f64,
        mat[1][0] as f64,
        mat[1][1] as f64,
        mat[1][2] as f64,
        mat[1][3] as f64,
        mat[2][0] as f64,
        mat[2][1] as f64,
        mat[2][2] as f64,
        mat[2][3] as f64,
        mat[3][0] as f64,
        mat[3][1] as f64,
        mat[3][2] as f64,
        mat[3][3] as f64,
    ])
}

pub fn three_d_vec3_to_glam_d(vec: &three_d::Vec3) -> glam::DVec3 {
    glam::DVec3::new(vec.x as f64, vec.y as f64, vec.z as f64)
}

pub fn three_d_vec3_to_glam(vec: &three_d::Vec3) -> glam::Vec3 {
    glam::Vec3::new(vec.x, vec.y, vec.z)
}

pub fn glam_d_vec3_to_three_d(vec: &glam::DVec3) -> three_d::Vec3 {
    three_d::Vec3::new(vec.x as f32, vec.y as f32, vec.z as f32)
}