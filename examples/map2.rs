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

struct App {
    tile_cache: Option<egui_3d_map_view::maps::TileCache>,
    camera: three_d::Camera,
    light: three_d::AmbientLight,
    key: String,
    view: egui_3d_map_view::threed_view::View,
    context: three_d::Context,
    settings_open: bool,
    show_bounding_boxes: bool,
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
        Self {
            tile_cache,
            camera,
            light,
            key,
            view: Default::default(),
            context,
            settings_open: false,
            show_bounding_boxes: false,
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
        let target = self.camera.target();
        egui_3d_map_view::orbitcontrol::handle_events(
            &mut self.camera,
            ctx,
            target,
            6_378_000.0 - 15_000.,
            50_000_000.0,
        );
        egui::CentralPanel::default()
            .frame(egui::Frame::default().inner_margin(egui::Margin::ZERO))
            .show(ctx, |ui| {
                egui::Frame::central_panel(ui.style()).show(ui, |ui| {
                    egui::Sides::new().show(
                        ui,
                        |ui| {
                            ui.horizontal(|ui| {});
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
                    let size = ui.available_size();
                    self.view.render(
                        frame,
                        &self.context,
                        size,
                        Color32::BLACK,
                        1.0,
                        |viewport| {
                            self.camera.set_viewport(viewport);
                            if let Some(tile_cache) = &mut self.tile_cache {
                                tile_cache.load(&self.context);
                                tile_cache.render(
                                    &self.camera,
                                    &[&self.light],
                                    self.show_bounding_boxes,
                                );
                            }
                        },
                    );
                    self.view.show(ui);
                }
            });

        if self.settings_open {
            egui::Window::new("⚙ settings").show(ctx, |ui| {
                let dt = ctx.input(|i| i.stable_dt);
                let fps = if dt > 0. { 1. / dt } else { 0. };
                ui.label(format!("FPS: {:.1}", fps));

                ui.checkbox(&mut self.show_bounding_boxes, "show bounding boxes");
                self.key_edit(ui);
            });
        }

        ctx.request_repaint();
    }
}
