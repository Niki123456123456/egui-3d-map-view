use three_d::*;

pub fn handle_events(
    camera: &mut Camera,
    ctx: &egui::Context,
    target: Vec3,
    min_distance: f32,
    max_distance: f32,
    yaw: &mut f32,
) {
    let mut pointer_down = false;
    let mut secondary_down = false;
    let mut delta = egui::Vec2::ZERO;
    let mut zoom_delta = 0.;
    let mut pinch_zoom = 0.;
    ctx.input(|i| {
        zoom_delta = i.smooth_scroll_delta.y;
        pointer_down = i.pointer.primary_down();
        secondary_down = i.pointer.secondary_down();
        delta = i.pointer.delta();
        pinch_zoom = i.zoom_delta();
    });

    if zoom_delta != 0. {
        let speed = 0.01 * (target.distance(camera.position()) - min_distance) + 0.001;
        camera.zoom_towards(target, speed * zoom_delta, min_distance, max_distance);
    }

    if pinch_zoom != 1. {
        let speed = (target.distance(camera.position()) - min_distance) + 0.001;
        camera.zoom_towards(
            target,
            speed * (pinch_zoom - 1.),
            min_distance,
            max_distance,
        );
    }

    if pointer_down {
        let delta = delta;
        let speed = 0.01
            * ((target.distance(camera.position()) - min_distance) / (max_distance - min_distance));
        camera.rotate_around_with_fixed_up(target, speed * delta.x, speed * delta.y);
    }

    if secondary_down {
        *yaw += 0.01 * delta.x;

        let target = camera.target();
        let p = camera.position();
        let mut up = three_d::Matrix3::from_angle_x(three_d::Rad(0.01 * delta.x)) * camera.up();
        // rot_roll(&mut up, three_d::Rad(0.01 * delta.x));
        camera.set_view(p, target, up);

        // let mut dir = three_d::Matrix3::from_angle_y(three_d::Rad(0.01 * delta.y)) * camera.view_direction();
        // // rot_pitch(&mut dir, three_d::Rad(0.01 * delta.y));

        // camera.view = Mat4::look_to_rh(Point3::from_vec(camera.position()), dir, camera.up());
    }
}

pub fn rot2(theta: three_d::Rad<f32>) -> three_d::Matrix2<f32> {
    let (s, c) = theta.0.sin_cos();
    three_d::Matrix2::new(c, -s, s, c)
}

pub fn rot_y(theta: three_d::Rad<f32>) -> three_d::Matrix3<f32> {
    let (s, c) = theta.0.sin_cos();
    three_d::Matrix3::from_angle_y(theta)
}


pub fn rot_roll(v: &mut three_d::Vec3, theta: three_d::Rad<f32>) {
    let a = rot2(theta) * three_d::vec2(v.y, v.z);
    v.y = a.x;
    v.z = a.y;
}

pub fn rot_pitch(v: &mut three_d::Vec3, theta: three_d::Rad<f32>) {
    let a = rot2(theta) * three_d::vec2(v.x, v.z);
    v.x = a.x;
    v.z = a.y;
}
