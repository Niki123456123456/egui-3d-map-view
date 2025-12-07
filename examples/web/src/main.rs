

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

        let start_result = eframe::WebRunner::new()
            .start(
                canvas,
                web_options,
                Box::new(|cc| Ok(Box::new(App::new(cc)))),
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
    view : egui_3d_map_view::threed_view::View,
    context: three_d::Context
}

impl App {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        
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
        Self {
            tile_cache: None,
            camera,
            light,
            key: "".to_string(),
            view : Default::default(), context,
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let dt = ctx.input(|i| i.stable_dt);
        let fps = if dt > 0. { 1. / dt } else { 0. };

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
                        &self.context,
                        self.key.clone(),
                    ));
                }
            });

            let size = ui.available_size_before_wrap();

            self.view.render(frame, &self.context, size, Color32::BLACK,
                1.0,
                |viewport| {
                    self.camera.set_viewport(viewport);
                    if let Some(tile_cache) = &mut self.tile_cache {
                        tile_cache.load(&self.context);
                        tile_cache.render(&self.camera, &[&self.light]);
                    }
                },);

            self.view.show(ui);
        });

        ctx.request_repaint();
    }
}
