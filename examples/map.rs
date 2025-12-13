#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use eframe::egui;
use egui::Color32;

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
    tile_cache: Option<egui_3d_map_view::maps::TileCache>,
    camera: three_d::Camera,
    light: three_d::AmbientLight,
    key: String,
}

impl App {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let context = egui_3d_map_view::threed_view::get_or_insert_context(
            &cc.egui_ctx,
            cc.gl.as_ref().unwrap(),
        );
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
        Self {
            tile_cache: None,
            camera,
            light,
            key: "".to_string(),
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let dt = ctx.input(|i| i.stable_dt);
        let fps = if dt > 0. { 1. / dt } else { 0. };

        let context =
            egui_3d_map_view::threed_view::get_or_insert_context(ctx, frame.gl().unwrap());

        let target = self.camera.target();
        egui_3d_map_view::orbitcontrol::handle_events(
            &mut self.camera,
            ctx,
            target,
            6_378_000.0 - 15_000.,
            50_000_000.0,
        );
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(format!("FPS: {:.1}", fps));
                let mut key = self.key.clone();
                egui::TextEdit::singleline(&mut key).password(true).show(ui);
                if key != self.key {
                    self.key = key;
                    self.tile_cache = Some(egui_3d_map_view::maps::TileCache::new(
                        &context,
                        self.key.clone(),
                    ));
                }
            });

            let size = ui.available_size_before_wrap();

            egui_3d_map_view::threed_view::show_advanced(
                "main",
                ui,
                frame,
                size,
                Color32::BLACK,
                1.0,
                |viewport| {
                    self.camera.set_viewport(viewport);
                    if let Some(tile_cache) = &mut self.tile_cache {
                        tile_cache.load(&context);
                        tile_cache.render(&self.camera, &[&self.light], false);
                    }
                },
            );
        });

        ctx.request_repaint();
    }
}
