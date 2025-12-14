use super::*;

pub struct ViewState {
    pub frustum: Frustum,
    pub planes: [Plane; 6],
    pub position: glam::DVec3,
    pub viewport_size: glam::DVec2,
    pub culling_volume: CullingVolume,
    pub projection_matrix: glam::DMat4,
}

impl ViewState {
    pub fn compute_screen_space_error(&self, geometric_error: f64, distance: f64) -> f64 {
        let distance = distance.max(1e-7);
        let mut center_ndc = self.projection_matrix * glam::dvec4(0., 0., -distance, 1.);
        center_ndc /= center_ndc.w;
        let mut error_offset_ndc =
            self.projection_matrix * glam::dvec4(0., geometric_error, -distance, 1.);
        error_offset_ndc /= error_offset_ndc.w;
        let ndc_error = (error_offset_ndc - center_ndc).y;

        return -ndc_error * self.viewport_size.y / 2.;
    }

    pub fn does_tile_meet_sse(&self, tile: &Tile) -> bool {
        let distance = tile
            .bounding
            .compute_distance_squared_to_position(self.position)
            .sqrt();
        let sse = self
            .compute_screen_space_error(tile.geometric_error, distance)
            .abs();
        // println!("sse {}", sse);
        let maximum_screen_space_error = 16.0;
        return sse < maximum_screen_space_error;
    }
}

pub fn get_view_state(camera: &three_d::Camera) -> ViewState {
    let position = three_d_vec3_to_glam_d(&camera.position());
    let frustum = Frustum::from_view_proj_with_origin_far(
        three_d_to_glam(&(camera.projection() * camera.view())),
        position,
    );
    let s = ViewState {
        frustum,
        planes: extract_planes(&three_d_to_glam(&(camera.projection() * camera.view()))),
        position: three_d_vec3_to_glam_d(&camera.position()),
        viewport_size: glam::dvec2(
            camera.viewport().width as f64,
            camera.viewport().height as f64,
        ),
        culling_volume: CullingVolume::new_matrix(three_d_to_glam(
            &(camera.projection() * camera.view()),
        )),
        projection_matrix: three_d_to_glam(&camera.projection()),
    };
    return s;
}