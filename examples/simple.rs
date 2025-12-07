#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use eframe::egui;
use three_d::Object;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([800.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "test",
        options,
        Box::new(|cc: &eframe::CreationContext<'_>| Ok(Box::new(App::new(cc)))),
    )
}

struct App {
    cube: three_d::Gm<three_d::Mesh, three_d::PhysicalMaterial>,
    camera: three_d::Camera,
    light: three_d::AmbientLight,
    fps: f32,
    last_frame: std::time::Instant,
}

impl App {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let context: three_d::Context =
            three_d::Context::from_gl_context(cc.gl.as_ref().unwrap().clone()).unwrap();
        let camera: three_d::Camera = three_d::Camera::new_perspective(
            three_d::Viewport {
                x: 0,
                y: 0,
                width: 1024,
                height: 1024,
            },
            three_d::vec3(5.0, 2.0, 2.5),
            three_d::vec3(0.0, 0.0, -0.5),
            three_d::vec3(0.0, 1.0, 0.0),
            three_d::degrees(45.0),
            0.1,
            1000.0,
        );

        let cube: three_d::Gm<three_d::Mesh, three_d::PhysicalMaterial> = three_d::Gm::new(
            three_d::Mesh::new(&context, &three_d::CpuMesh::cube()),
            three_d::PhysicalMaterial::new_transparent(
                &context,
                &three_d::CpuMaterial {
                    albedo: three_d::Srgba {
                        r: 0,
                        g: 0,
                        b: 255,
                        a: 255,
                    },
                    ..Default::default()
                },
            ),
        );

        let light: three_d::AmbientLight =
            three_d::AmbientLight::new(&context, 0.5, three_d::Srgba::WHITE);
        Self {
            camera,
            cube,
            light,
            last_frame: std::time::Instant::now(),
            fps: 0.0,
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let dt = ctx.input(|i| i.stable_dt);

        let now = std::time::Instant::now();
        let dt = now.duration_since(self.last_frame).as_secs_f32();
        self.last_frame = now;

        if dt > 0.0 {
            self.fps = 1.0 / dt;
        }

        let target = self.camera.target();
        egui_3d_map_view::orbitcontrol::handle_events(&mut self.camera, ctx, target, 0.1, 1000.);
        egui::CentralPanel::default().show(ctx, |ui| {
            let size = ui.available_size_before_wrap();

            ui.label(format!("FPS: {:.1}", self.fps));

            egui_3d_map_view::threed_view::show("main", ui, frame, size, |viewport| {
                self.camera.set_viewport(viewport);
                self.cube.render(&self.camera, &[&self.light]);
            });
        });

        egui::Window::new("w1").show(ctx, |ui| {
            let size = ui.available_size_before_wrap();

            egui_3d_map_view::threed_view::show_advanced(
                "w1-view",
                ui,
                frame,
                size,
                egui::Color32::DARK_RED,
                1.,
                |viewport| {
                    self.camera.set_viewport(viewport);
                    self.cube.render(&self.camera, &[&self.light]);
                },
            );
        });

        egui::Window::new("w2")
            .frame(egui::Frame::NONE)
            .show(ctx, |ui| {
                let size = ui.available_size_before_wrap();

                egui_3d_map_view::threed_view::show_advanced(
                    "w2-view",
                    ui,
                    frame,
                    size,
                    egui::Color32::TRANSPARENT,
                    1.,
                    |viewport| {
                        self.camera.set_viewport(viewport);
                        self.cube.render(&self.camera, &[&self.light]);
                    },
                );
            });

         ctx.request_repaint();
    }
}
