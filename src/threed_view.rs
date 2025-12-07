use three_d::HasContext;

#[derive(Clone)]
pub struct View {
    pub textures: Option<TexturesContainer>,
    pub size: egui::Vec2,
}

#[derive(Clone)]
pub struct TexturesContainer {
    pub texture: three_d::Texture2D,
    pub depth_texture: three_d::DepthTexture2D,
    pub texture_id: egui::TextureId,
    pub program: eframe::glow::NativeProgram,
    pub vao: eframe::glow::NativeVertexArray,
    pub vbo: eframe::glow::NativeBuffer,
}

fn create_program(gl: &egui_glow::glow::Context) -> eframe::glow::NativeProgram {
    unsafe {
        // Vertex shader
        let vert_shader = gl
            .create_shader(egui_glow::glow::VERTEX_SHADER)
            .expect("Failed to create vertex shader");
        gl.shader_source(
            vert_shader,
            r#"#version 330 core
               layout (location = 0) in vec2 a_pos;

               out vec2 v_TexCoords;

               void main() {
                   gl_Position = vec4(a_pos, 0.0, 1.0);
                   v_TexCoords = a_pos * 0.5 + 0.5;
               }
            "#,
        );
        gl.compile_shader(vert_shader);
        if !gl.get_shader_compile_status(vert_shader) {
            panic!(
                "Vertex shader error: {}",
                gl.get_shader_info_log(vert_shader)
            );
        }

        // Fragment shader
        let frag_shader = gl
            .create_shader(egui_glow::glow::FRAGMENT_SHADER)
            .expect("Failed to create fragment shader");
        gl.shader_source(
            frag_shader,
            r#"#version 330 core

               uniform sampler2D u_Texture;

               in vec2 v_TexCoords;
               out vec4 FragColor;

               void main() {
                   // A reddish triangle
                   FragColor = texture(u_Texture, v_TexCoords);
               }
            "#,
        );
        gl.compile_shader(frag_shader);
        if !gl.get_shader_compile_status(frag_shader) {
            panic!(
                "Fragment shader error: {}",
                gl.get_shader_info_log(frag_shader)
            );
        }

        // Link program
        let program: eframe::glow::NativeProgram =
            gl.create_program().expect("Failed to create GL program");
        gl.attach_shader(program, vert_shader);
        gl.attach_shader(program, frag_shader);
        gl.link_program(program);
        if !gl.get_program_link_status(program) {
            panic!("Program link error: {}", gl.get_program_info_log(program));
        }

        // Shaders can be deleted after linking
        gl.delete_shader(vert_shader);
        gl.delete_shader(frag_shader);

        return program;
    }
}

fn create_buffers(
    gl: &egui_glow::glow::Context,
) -> (eframe::glow::NativeVertexArray, eframe::glow::NativeBuffer) {
    unsafe {
        let vertices: [f32; 12] = [
            -1., -1.,// left bottom
            1., -1., // right bottom
            -1., 1., // left top
            1., -1., // right bottom
            1., 1.,  // right top
            -1., 1., // left top
        ];

        // Upload vertex data
        let vao: eframe::glow::NativeVertexArray =
            gl.create_vertex_array().expect("Failed to create VAO");
        gl.bind_vertex_array(Some(vao));

        let vbo: eframe::glow::NativeBuffer = gl.create_buffer().expect("Failed to create VBO");
        gl.bind_buffer(eframe::glow::ARRAY_BUFFER, Some(vbo));

        let bytes: &[u8] = std::slice::from_raw_parts(
            vertices.as_ptr() as *const u8,
            vertices.len() * std::mem::size_of::<f32>(),
        );
        gl.buffer_data_u8_slice(eframe::glow::ARRAY_BUFFER, bytes, eframe::glow::STATIC_DRAW);

        // Configure vertex attribute 0 as vec2
        gl.enable_vertex_attrib_array(0);
        gl.vertex_attrib_pointer_f32(
            0,
            2, // vec2
            eframe::glow::FLOAT,
            false,
            (2 * std::mem::size_of::<f32>()) as i32,
            0,
        );

        gl.bind_vertex_array(None);
        gl.bind_buffer(eframe::glow::ARRAY_BUFFER, None);

        return (vao, vbo);
    }
}

fn create_textures(
    context: &three_d::Context,
    size: egui::Vec2,
    frame: &mut eframe::Frame,
) -> TexturesContainer {
    let texture: three_d::Texture2D = three_d::Texture2D::new_empty::<[u8; 4]>(
        context,
        size.x as u32,
        size.y as u32,
        three_d::Interpolation::Linear,
        three_d::Interpolation::Linear,
        None,
        three_d::Wrapping::ClampToEdge,
        three_d::Wrapping::ClampToEdge,
    );
    let depth_texture: three_d::DepthTexture2D = three_d::DepthTexture2D::new::<f32>(
        context,
        size.x as u32,
        size.y as u32,
        three_d::Wrapping::ClampToEdge,
        three_d::Wrapping::ClampToEdge,
    );
    let texture_id = frame.register_native_glow_texture(texture.id());
    let gl = frame.gl().unwrap().as_ref();
    let program = create_program(gl);
    let (vao, vbo) = create_buffers(gl);
    return TexturesContainer {
        texture,
        depth_texture,
        texture_id: texture_id,
        program,
        vao,
        vbo,
    };
}

impl Default for View {
    fn default() -> Self {
        Self {
            textures: None,
            size: egui::Vec2::ZERO,
        }
    }
}
impl View {
    pub fn render(
        &mut self,
        frame: &mut eframe::Frame,
        context: &three_d::Context,
        size: egui::Vec2,
        clear_color: egui::Color32,
        clear_depth: f32,
        render: impl FnOnce(three_d::Viewport),
    ) {
        if size != self.size || self.textures.is_none() {
            self.size = size;
            self.textures = Some(create_textures(context, size, frame));
        }

        if let Some(tex) = &mut self.textures {
            let render_target = three_d::RenderTarget::new(
                tex.texture.as_color_target(None),
                tex.depth_texture.as_depth_target(),
            );

            render_target.clear(three_d::ClearState::color_and_depth(
                clear_color.r() as f32 / 255.0,
                clear_color.g() as f32 / 255.0,
                clear_color.b() as f32 / 255.0,
                clear_color.a() as f32 / 255.0,
                clear_depth,
            ));

            let _ = render_target.write(|| {
                let result: Result<(), three_d::CoreError> = Ok(());
                let viewport =
                    three_d::Viewport::new_at_origo(self.size.x as u32, self.size.y as u32);
                context.set_viewport(viewport);
                (render)(viewport);
                return result;
            });
        }
    }

    pub fn show(&self, ui: &mut egui::Ui) {
        if let Some(tex) = &self.textures {
            // let image = egui::Image::new(egui::load::SizedTexture::new(tex.texture_id, self.size))
            //     .uv(egui::Rect::from_min_max(
            //         egui::pos2(1.0, 0.0), // U axis reversed
            //         egui::pos2(0.0, 1.0),
            //     ));

            // ui.add(image);

            let t = tex.clone();

            let callback_fn =
                egui_glow::CallbackFn::new(move |info: egui::PaintCallbackInfo, painter| unsafe {
                    let gl = painter.gl();
                    gl.bind_vertex_array(Some(t.vao));
                    gl.bind_buffer(eframe::glow::ARRAY_BUFFER, Some(t.vbo));
                    gl.enable_vertex_attrib_array(0);
                    gl.use_program(Some(t.program));
                    gl.bind_texture(eframe::glow::TEXTURE_2D, Some(t.texture.id()));

                    gl.draw_arrays(eframe::glow::TRIANGLES, 0, 6);


                    gl.bind_texture(eframe::glow::TEXTURE_2D, None);
                    gl.disable_vertex_attrib_array(0);
                    gl.bind_buffer(eframe::glow::ARRAY_BUFFER, None);
                    gl.bind_vertex_array(None);
                    gl.use_program(None);
                });

            let pos = ui.next_widget_position();
            let rect = egui::Rect::from_two_pos(pos, pos + self.size);

            let callback = egui::PaintCallback {
                rect: rect,
                callback: std::sync::Arc::new(callback_fn),
            };

            ui.painter().add(egui::Shape::Callback(callback));
        }
    }
}

pub fn show(
    id_source: impl std::hash::Hash,
    ui: &mut egui::Ui,
    frame: &mut eframe::Frame,
    size: egui::Vec2,
    render: impl FnOnce(three_d::Viewport),
) {
    show_advanced(
        id_source,
        ui,
        frame,
        size,
        egui::Color32::TRANSPARENT,
        1.,
        render,
    );
}

pub fn show_advanced(
    id_source: impl std::hash::Hash,
    ui: &mut egui::Ui,
    frame: &mut eframe::Frame,
    size: egui::Vec2,
    clear_color: egui::Color32,
    clear_depth: f32,
    render: impl FnOnce(three_d::Viewport),
) {
    let id = egui::Id::new(id_source);
    let ctx = ui.ctx().clone();

    let context = get_or_insert_context(ui.ctx(), frame.gl().unwrap());

    let mut view: View = ctx.data(|d| d.get_temp(id)).unwrap_or_default();
    view.render(frame, &context, size, clear_color, clear_depth, render);
    view.show(ui);
    ctx.data_mut(|d| d.insert_temp(id, view));
}

pub fn get_or_insert_context(
    ctx: &egui::Context,
    gl: &std::sync::Arc<egui_glow::glow::Context>,
) -> three_d::Context {
    let ctx = ctx.clone();
    let mut context: Option<three_d::Context> =
        ctx.data(|d| d.get_temp(egui::Id::new("three_d::Context")));
    if context.is_none() {
        context = Some(three_d::Context::from_gl_context(gl.clone()).unwrap());
        ctx.data_mut(|d| {
            d.insert_temp(egui::Id::new("three_d::Context"), context.clone().unwrap())
        });
    }
    return context.unwrap();
}
