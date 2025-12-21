#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([800.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "test",
        options,
        Box::new(|cc: &eframe::CreationContext<'_>| Ok(Box::new(App::new(cc, "".into())))),
    )
}
use eframe::egui;
use egui::Color32;
use three_d::{EuclideanSpace, InnerSpace, MetricSpace, SquareMatrix, Zero};

struct App {
    tile_cache: Option<egui_3d_map_view::maps::TileCache>,
    camera: three_d::Camera,
    light: three_d::AmbientLight,
    key: String,
    view: egui_3d_map_view::threed_view::View,
    context: three_d::Context,
    settings_open: bool,
    show_bounding_boxes: bool,
    search_promise: Option<poll_promise::Promise<Vec<egui_3d_map_view::search::Place>>>,
    search: String,
    show_search: bool,
    gpx_promise: Option<poll_promise::Promise<egui_3d_map_view::gpx::GpxRoute>>,
    gpx_routes: Vec<egui_3d_map_view::gpx::GpxRouteGPU>,
    m: three_d::ColorMaterial,
    rotation : three_d::Vec2,
}

impl App {
    fn new(cc: &eframe::CreationContext<'_>, key: String) -> Self {
        let context = three_d::Context::from_gl_context(cc.gl.as_ref().unwrap().clone()).unwrap();
        let camera = three_d::Camera::new_perspective(
            three_d::Viewport::new_at_origo(512, 512),
            three_d::vec3(47702560.0, 0.0, -9691560.0),
            three_d::vec3(0.0, 0.0, 0.0),
            three_d::vec3(0., 0., 1.),
            three_d::degrees(45.0),
            100.,        //0.1,
            1000000000., //1000.0,
        );

        let light: three_d::AmbientLight =
            three_d::AmbientLight::new(&context, 0.5, three_d::Srgba::WHITE);
        let tile_cache = if key != "" {
            Some(egui_3d_map_view::maps::TileCache::new(
                &context,
                key.clone(),
            ))
        } else {
            None
        };

        let m = three_d::ColorMaterial::new(
            &context,
            &three_d::CpuMaterial {
                albedo: three_d::Srgba::RED,
                ..Default::default()
            },
        );
        Self {
            tile_cache,
            camera,
            light,
            key,
            view: Default::default(),
            context,
            settings_open: false,
            show_bounding_boxes: false,
            search_promise: None,
            search: Default::default(),
            show_search: false,
            gpx_promise: None,
            gpx_routes: vec![],
            m,
            rotation : three_d::Vector2::zero(),
        }
    }

    fn key_edit(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("google api key ");
            let mut key = self.key.clone();
            egui::TextEdit::singleline(&mut key)
                .password(true)
                .desired_width(ui.available_width())
                .show(ui);
            if key != self.key {
                self.key = key;
                self.tile_cache = Some(egui_3d_map_view::maps::TileCache::new(
                    &self.context,
                    self.key.clone(),
                ));
            }
        });
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let mut search_rect = egui::Rect::ZERO;

        let min_distance = 6_378_000. - 15_000.;
        let max_distance = 50_000_000.;
        let max_pitch_distance = min_distance + 10_000.;

        egui::CentralPanel::default()
            .frame(egui::Frame::default().inner_margin(egui::Margin::ZERO))
            .show(ctx, |ui| {
                egui::Frame::central_panel(ui.style()).show(ui, |ui| {
                    egui::Sides::new().show(
                        ui,
                        |ui| {
                            ui.horizontal(|ui| {
                                ui.label("🔍");
                                let resp = egui::TextEdit::singleline(&mut self.search)
                                    .return_key(Some(egui::KeyboardShortcut::new(
                                        egui::Modifiers::NONE,
                                        egui::Key::Enter,
                                    )))
                                    .hint_text("search")
                                    .return_key(Some(egui::KeyboardShortcut::new(
                                        egui::Modifiers::NONE,
                                        egui::Key::Enter,
                                    )))
                                    .show(ui);
                                search_rect = resp.response.rect;
                                if resp.response.lost_focus()
                                    && ui.input(|i| i.key_pressed(egui::Key::Enter))
                                {
                                    self.search_promise =
                                        Some(egui_3d_map_view::search::search(self.search.clone()));
                                    self.show_search = true;
                                }
                                if ui.button("🔍").clicked() {
                                    self.search_promise =
                                        Some(egui_3d_map_view::search::search(self.search.clone()));
                                    self.show_search = true;
                                }

                                if ui.button("upload gpx").clicked() {
                                    self.gpx_promise = Some(egui_3d_map_view::gpx::open());
                                }
                            });
                        },
                        |ui| {
                            if ui.button("⚙").clicked() {
                                self.settings_open = !self.settings_open;
                            }
                        },
                    );
                    if self.tile_cache.is_none() {
                        ui.vertical(|ui| {
                            ui.heading("Please insert your google api key");
                            self.key_edit(ui);
                        });
                    }
                });

                if !self.tile_cache.is_none() {
                    let rect = ui.available_rect_before_wrap();

                    let resp = ui.interact(rect, ui.next_auto_id(), egui::Sense::all());
                    if resp.contains_pointer() {
                        let target = self.camera.target();
                        egui_3d_map_view::orbitcontrol::handle_events(
                            &mut self.camera,
                            ctx,
                            target,
                            min_distance,
                            max_distance,
                            &mut self.rotation,
                        );
                    }
                    self.view.render(
                        &self.context,
                        rect.size(),
                        Color32::TRANSPARENT,
                        1.0,
                        |viewport| {
                            let height = self.camera.target().distance(self.camera.position())
                                - min_distance;
                            let height_relativ = height / (max_pitch_distance - min_distance);
                            let max_pitch = std::f32::consts::TAU / 4.;
                            let pitch = if height_relativ < 1. {
                                (1. - height_relativ) * max_pitch
                            } else {
                                0.
                            };

                            let mut cam: three_d::Camera = self.camera.clone();

                            // rotate_around_with_fixed_right_direction(&mut cam, self.rotation);
                            let pos = cam.position();
                            // // cam.rotate_around_with_fixed_up(pos, self.rotation.x, self.rotation.y);
                            // cam.rotate_around_with_fixed_up(pos, 0., std::f32::consts::TAU / 4.);
                            // cam.rotate_around_with_fixed_up(pos, self.rotation.x, 0.);
                            // cam.rotate_around_with_fixed_up(pos, 0., -std::f32::consts::TAU / 4.);
                            rotate_around_with_fixed_right_direction(&mut cam, three_d::vec2(0., self.rotation.x));
                            //cam.pitch(three_d::radians(pitch));

                            // let mut dir = three_d::Matrix3::from_angle_y(three_d::Rad(self.yaw))
                            //     * cam.view_direction();
                            // // // rot_pitch(&mut dir, three_d::Rad(0.01 * delta.y));

                            // cam.view =
                            //     three_d::Mat4::look_to_rh(three_d::Point3::from_vec(cam.position()), dir, cam.up());

                            cam.set_viewport(viewport);
                            if let Some(tile_cache) = &mut self.tile_cache {
                                tile_cache.load(&self.context);
                                tile_cache.render(&cam, &[&self.light], self.show_bounding_boxes);
                            }
                            for route in self.gpx_routes.iter() {
                                three_d::Geometry::render_with_material(
                                    &route.mesh,
                                    &self.m,
                                    &cam,
                                    &[&self.light],
                                );
                            }
                        },
                    );
                    self.view.show(ui);
                }
            });

        if self.show_search {
            if let Some(search_promise) = &self.search_promise {
                if let Some(places) = search_promise.ready() {
                    egui::Window::new("search_results")
                        .frame(egui::Frame::central_panel(&ctx.style()).corner_radius(
                            egui::CornerRadius {
                                nw: 0,
                                ne: 0,
                                sw: 8,
                                se: 8,
                            },
                        ))
                        .fixed_pos(search_rect.left_bottom())
                        .title_bar(false)
                        .resizable(false)
                        .show(ctx, |ui| {
                            for place in places {
                                ui.horizontal(|ui| {
                                    ui.label(&place.name);
                                    if ui.button("visit").clicked() {
                                        let coodinates = egui_3d_map_view::maps::latlon_to_xyz(
                                            place.lat, place.lon, 1000.,
                                        );
                                        self.camera = three_d::Camera::new_perspective(
                                            three_d::Viewport::new_at_origo(512, 512),
                                            egui_3d_map_view::maps::glam_d_vec3_to_three_d(
                                                &coodinates,
                                            ),
                                            three_d::vec3(0.0, 0.0, 0.0),
                                            three_d::vec3(0.0, 0.0, 1.0),
                                            three_d::degrees(45.0),
                                            100.,        //0.1,
                                            1000000000., //1000.0,
                                        );
                                        self.show_search = false;
                                    }
                                });
                                ui.label(format!("{:.1}° {:.1}°", place.lat, place.lon));
                            }
                        });
                }
            }
        }

        if let Some(gpx_promise) = &self.gpx_promise {
            if let Some(route) = gpx_promise.ready() {
                self.gpx_routes
                    .push(egui_3d_map_view::gpx::GpxRouteGPU::new(
                        route.clone(),
                        &self.context,
                    ));
                self.gpx_promise = None;
            }
        }
        if self.settings_open {
            egui::Window::new("⚙ settings").show(ctx, |ui| {
                let dt = ctx.input(|i| i.stable_dt);
                let fps = if dt > 0. { 1. / dt } else { 0. };
                ui.label(format!("FPS: {:.1}", fps));

                ui.checkbox(&mut self.show_bounding_boxes, "show bounding boxes");
                self.key_edit(ui);

                let height = self.camera.target().distance(self.camera.position()) - min_distance;
                let height_relativ = height / (max_distance - min_distance);
                ui.label(format!(
                    "height: {:.0} km {:.2} %",
                    height / 1000.,
                    height_relativ * 100.
                ));
                ui.label(format!("yaw: {:.3}", self.rotation.x));
                ui.label(format!("pitch: {:.3}", self.rotation.y));
                
                // egui::ScrollArea::vertical().show(ui, |ui| {
                //     egui_ltreeview::TreeView::new(ui.make_persistent_id("Names tree view")).show(
                //         ui,
                //         |builder| {
                //             if let Some(tile_cache) = &mut self.tile_cache {
                //                 calc_visiblity(tile_cache, &self.camera);
                //                 show_tile_tree(&tile_cache.roots, builder, tile_cache, 0);
                //             }
                //         },
                //     );
                // });
            });
        }

        ctx.request_repaint();
    }
}

fn calc_visiblity(tile_cache: &mut egui_3d_map_view::maps::TileCache, camera: &three_d::Camera) {
    let s = egui_3d_map_view::maps::get_view_state(camera);
    for (_, t) in tile_cache.cache.iter_mut() {
        t.is_visible = t.bv.is_visible(s.position) && t.bv.intersects_frustum(&s.frustum);
        t.meets_sse = s.does_tile_meet_sse(t);
    }
}

fn any_child_rendered(
    key: &String,
    tile_cache: &egui_3d_map_view::maps::TileCache,
    level: usize,
) -> Option<usize> {
    if let Some(t) = tile_cache.cache.get(key) {
        if !t.is_visible {
            return None;
        }
        if t.is_visible && t.meets_sse {
            return Some(level);
        }
        for c in t.children.iter() {
            if let Some(l) = any_child_rendered(&c, tile_cache, level + 1) {
                return Some(l);
            }
        }
    }
    return None;
}

fn show_tile_tree<'a>(
    roots: &Vec<String>,
    builder: &mut egui_ltreeview::TreeViewBuilder<'a, String>,
    tile_cache: &egui_3d_map_view::maps::TileCache,
    level: usize,
) {
    for key in roots.iter() {
        if let Some(t) = tile_cache.cache.get(key) {
            let mut job = egui::text::LayoutJob::default();
            let font = egui::FontId::monospace(10.);
            if t.is_visible {
                job.append(
                    "👁",
                    0.0,
                    egui::TextFormat::simple(font.clone(), Color32::DARK_GREEN),
                );
            } else {
                job.append(
                    "🙈",
                    0.0,
                    egui::TextFormat::simple(font.clone(), Color32::DARK_RED),
                );
            }
            if t.meets_sse {
                job.append(
                    "✅",
                    0.0,
                    egui::TextFormat::simple(font.clone(), Color32::DARK_GREEN),
                );
            } else {
                job.append(
                    "❌",
                    0.0,
                    egui::TextFormat::simple(font.clone(), Color32::DARK_RED),
                );
            }
            if let Some(l) = any_child_rendered(key, tile_cache, 0) {
                let text = format!("{}", l);
                job.append(
                    &text,
                    0.0,
                    egui::TextFormat::simple(font.clone(), Color32::GREEN),
                );
            }

            builder.dir(key.clone(), job);
            show_tile_tree(&t.children, builder, tile_cache, level + 1);
            builder.close_dir();
        }
    }
}

pub fn rotate_around_with_fixed_right_direction(
    camera: &mut three_d::Camera,
    rotation : three_d::Vec2
) {
    use three_d::{Camera, Vec3};
    let point = camera.position();
    // Rotations are about the origin -> translate to pivot, rotate, translate back
    let position = camera.position() - point;
    let target = camera.target() - point;

    let right = camera.right_direction().normalize();
    let view_dir = (target - position).normalize();

    // Two-axis orbit:
    // - rotate around fixed `right` (pitch-like)
    // - rotate around "vertical" axis derived from view & right (yaw-like), still orthogonal to right
    let k_pitch = right;
    let k_yaw = right.cross(view_dir).normalize();

    // Prepare cos/sin terms, inverted because controls rotate left/up while rotations follow RH rule
    let cos_x = (-rotation.x).cos();
    let sin_x = (-rotation.x).sin();
    let cos_y = (-rotation.y).cos();
    let sin_y = (-rotation.y).sin();

    // Rodrigues rotation
    let rodrigues = |v: Vec3, k: Vec3, cos: f32, sin: f32| -> Vec3 {
        v * cos + k.cross(v) * sin + k * k.dot(v) * (1.0 - cos)
    };

    // Apply rotations
    let position_x = rodrigues(position, k_pitch, cos_x, sin_x);
    let target_x = rodrigues(target, k_pitch, cos_x, sin_x);

    let position_xy = rodrigues(position_x, k_yaw, cos_y, sin_y);
    let target_xy = rodrigues(target_x, k_yaw, cos_y, sin_y);

    // Avoid the singularity where the camera looks exactly along ±right (yaw axis becomes unstable)
    let new_dir = (target_xy - position_xy).normalize();
    if new_dir.dot(right).abs() < 0.999 {
        // Recompute an "up" consistent with fixed right and the new forward direction
        let new_up = right.cross(new_dir).normalize();
        camera.set_view(position_xy + point, target_xy + point, new_up);
    } else {
        // Fall back to pitch-only rotation
        let new_dir_x = (target_x - position_x).normalize();
        let new_up_x = right.cross(new_dir_x).normalize();
        camera.set_view(position_x + point, target_x + point, new_up_x);
    }
}