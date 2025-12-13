fn main() {
    use eframe::wasm_bindgen::JsCast as _;

    let web_options = eframe::WebOptions::default();

    wasm_bindgen_futures::spawn_local(async {
        let document = web_sys::window()
            .expect("No window")
            .document()
            .expect("No document");

        let canvas = document
            .get_element_by_id("the_canvas_id")
            .expect("Failed to find the_canvas_id")
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .expect("the_canvas_id was not a HtmlCanvasElement");

        let key = web_sys::window()
            .and_then(|w| w.location().search().ok())
            .and_then(|search| search.strip_prefix("?key=").map(|s| s.to_string()))
            .unwrap_or_default();

        let start_result = eframe::WebRunner::new()
            .start(
                canvas,
                web_options,
                Box::new(move |cc| Ok(Box::new(App::new(cc, key)))),
            )
            .await;

        // Remove the loading text and spinner:
        if let Some(loading_text) = document.get_element_by_id("loading_text") {
            match start_result {
                Ok(_) => {
                    loading_text.remove();
                }
                Err(e) => {
                    loading_text.set_inner_html(
                        "<p> The app has crashed. See the developer console for details. </p>",
                    );
                    panic!("Failed to start eframe: {e:?}");
                }
            }
        }
    });
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
                    let rect = ui.available_rect_before_wrap();

                    let resp = ui.interact(rect, ui.next_auto_id(), egui::Sense::all());
                    if resp.contains_pointer() {
                        let target = self.camera.target();
                        egui_3d_map_view::orbitcontrol::handle_events(
                            &mut self.camera,
                            ctx,
                            target,
                            6_378_000.0 - 15_000.,
                            50_000_000.0,
                        );
                    }
                    self.view.render(
                        frame,
                        &self.context,
                        rect.size(),
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
