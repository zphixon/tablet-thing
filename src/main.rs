use glow::{Context, HasContext};
use glutin::{
    dpi::PhysicalSize,
    event::{Event, KeyboardInput, MouseButton, MouseScrollDelta, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
    ContextBuilder,
};
use std::mem::size_of;
use tablet_thing::{input::InputHandler, State, StrokeStyle};

fn main() {
    let (gl, context, ev) = {
        let event_loop = EventLoop::new();
        let builder = WindowBuilder::new().with_title("hi! <3");
        let context = unsafe {
            ContextBuilder::new()
                .with_vsync(true)
                .with_gl(glutin::GlRequest::Latest)
                .with_multisampling(4)
                .build_windowed(builder, &event_loop)
                .unwrap()
                .make_current()
                .unwrap()
        };
        let gl = unsafe {
            Context::from_loader_function(|name| context.get_proc_address(name) as *const _)
        };

        (gl, context, event_loop)
    };

    let strokes_program;
    let circle_program;

    let view_uniform;
    let stroke_color_uniform;
    let brush_size_uniform;

    let circle_view_uniform;
    let erasing_uniform;
    let pen_down_uniform;

    unsafe {
        gl.enable(glow::MULTISAMPLE);
        gl.enable(glow::VERTEX_PROGRAM_POINT_SIZE);
        gl.enable(glow::DEBUG_OUTPUT);
        gl.disable(glow::CULL_FACE);
        gl.clear_color(0.0, 0.0, 0.0, 1.0);

        circle_program = gl.create_program().expect("create program");
        let (circles_vs_source, circles_fs_source) = (
            include_str!("shaders/circle.vert"),
            include_str!("shaders/circle.frag"),
        );
        let circles_vs = gl
            .create_shader(glow::VERTEX_SHADER)
            .expect("create shader");
        gl.shader_source(circles_vs, circles_vs_source);
        gl.compile_shader(circles_vs);
        if !gl.get_shader_compile_status(circles_vs) {
            panic!("{}", gl.get_shader_info_log(circles_vs));
        }
        gl.attach_shader(circle_program, circles_vs);
        let circles_fs = gl
            .create_shader(glow::FRAGMENT_SHADER)
            .expect("create shader");
        gl.shader_source(circles_fs, circles_fs_source);
        gl.compile_shader(circles_fs);
        if !gl.get_shader_compile_status(circles_fs) {
            panic!("{}", gl.get_shader_info_log(circles_fs));
        }
        gl.attach_shader(circle_program, circles_fs);
        gl.link_program(circle_program);
        if !gl.get_program_link_status(circle_program) {
            panic!("{}", gl.get_program_info_log(circle_program));
        }
        gl.detach_shader(circle_program, circles_vs);
        gl.delete_shader(circles_vs);
        gl.detach_shader(circle_program, circles_fs);
        gl.delete_shader(circles_fs);

        gl.use_program(Some(circle_program));
        erasing_uniform = gl.get_uniform_location(circle_program, "erasing").unwrap();
        pen_down_uniform = gl.get_uniform_location(circle_program, "penDown").unwrap();
        circle_view_uniform = gl.get_uniform_location(circle_program, "view").unwrap();
        gl.uniform_1_f32(Some(&erasing_uniform), 0.0);
        gl.uniform_1_f32(Some(&pen_down_uniform), 0.0);
        gl.uniform_matrix_4_f32_slice(
            Some(&circle_view_uniform),
            false,
            &glam::Mat4::IDENTITY.to_cols_array(),
        );

        strokes_program = gl.create_program().expect("create program");
        let (strokes_vs_source, strokes_fs_source) = (
            include_str!("shaders/points.vert"),
            include_str!("shaders/points.frag"),
        );

        let strokes_vs = gl
            .create_shader(glow::VERTEX_SHADER)
            .expect("create shader");
        gl.shader_source(strokes_vs, strokes_vs_source);
        gl.compile_shader(strokes_vs);
        if !gl.get_shader_compile_status(strokes_vs) {
            panic!("{}", gl.get_shader_info_log(strokes_vs));
        }
        gl.attach_shader(strokes_program, strokes_vs);

        let strokes_fs = gl
            .create_shader(glow::FRAGMENT_SHADER)
            .expect("create shader");
        gl.shader_source(strokes_fs, strokes_fs_source);
        gl.compile_shader(strokes_fs);
        if !gl.get_shader_compile_status(strokes_fs) {
            panic!("{}", gl.get_shader_info_log(strokes_fs));
        }

        gl.attach_shader(strokes_program, strokes_fs);
        gl.link_program(strokes_program);
        if !gl.get_program_link_status(strokes_program) {
            panic!("{}", gl.get_program_info_log(strokes_program));
        }

        gl.detach_shader(strokes_program, strokes_vs);
        gl.delete_shader(strokes_vs);
        gl.detach_shader(strokes_program, strokes_fs);
        gl.delete_shader(strokes_fs);

        gl.use_program(Some(strokes_program));

        view_uniform = gl.get_uniform_location(strokes_program, "view").unwrap();
        stroke_color_uniform = gl
            .get_uniform_location(strokes_program, "strokeColor")
            .unwrap();
        brush_size_uniform = gl
            .get_uniform_location(strokes_program, "brushSize")
            .unwrap();

        gl.uniform_matrix_4_f32_slice(
            Some(&view_uniform),
            false,
            &glam::Mat4::IDENTITY.to_cols_array(),
        );
    };

    let mut cursor_visible = true;
    let mut input_handler = InputHandler::default();
    let mut aa = true;
    let mut stroke_style = glow::LINE_STRIP;

    let mut state = State::default();
    println!("stroke style {:?}", state.stroke_style);

    // gl origin in stroke space
    ev.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        match event {
            Event::WindowEvent {
                event:
                    WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                virtual_keycode: Some(key),
                                state: key_state,
                                ..
                            },
                        ..
                    },
                ..
            } => {
                use VirtualKeyCode::*;
                input_handler.handle_key(key, key_state);

                if input_handler.just_pressed(Escape) {
                    *control_flow = ControlFlow::Exit;
                }

                if input_handler.just_pressed(C) {
                    state.clear_strokes();
                    context.window().request_redraw();
                }

                if input_handler.just_pressed(D) {
                    for stroke in state.strokes.iter() {
                        println!("stroke");
                        for point in stroke.points.iter() {
                            let x = point.x;
                            let y = point.y;
                            let pressure = point.pressure;
                            println!("{x}, {y}, {pressure}");
                        }
                    }
                    println!("brush={}", state.brush_size);
                    println!("zoom={:.02}", state.zoom);
                    println!("origin={}", state.origin);
                }

                if input_handler.just_pressed(A) {
                    aa = !aa;

                    if aa {
                        unsafe { gl.enable(glow::MULTISAMPLE) };
                    } else {
                        unsafe { gl.disable(glow::MULTISAMPLE) };
                    }

                    context.window().request_redraw();
                }

                if input_handler.just_pressed(P) {
                    stroke_style = match stroke_style {
                        glow::LINE_STRIP => glow::POINTS,
                        glow::POINTS => glow::LINE_STRIP,
                        _ => glow::LINE_STRIP,
                    };

                    context.window().request_redraw();
                }

                match (input_handler.control(), input_handler.just_pressed(Z)) {
                    (true, true) => {
                        state.undo_stroke();
                        context.window().request_redraw();
                    }
                    (false, true) => {
                        state.origin = Default::default();
                        state.zoom = tablet_thing::DEFAULT_ZOOM;
                        context.window().request_redraw();
                    }
                    _ => {}
                }

                if input_handler.just_pressed(Key1)
                    || input_handler.just_pressed(Key2)
                    || input_handler.just_pressed(Key3)
                    || input_handler.just_pressed(Key4)
                    || input_handler.just_pressed(Key5)
                    || input_handler.just_pressed(Key6)
                    || input_handler.just_pressed(Key7)
                    || input_handler.just_pressed(Key8)
                    || input_handler.just_pressed(Key9)
                    || input_handler.just_pressed(Key0)
                {
                    state.stroke_style = unsafe {
                        std::mem::transmute(
                            match key {
                                Key1 => 0,
                                Key2 => 1,
                                Key3 => 2,
                                Key4 => 3,
                                Key5 => 4,
                                Key6 => 5,
                                Key7 => 6,
                                Key8 => 7,
                                Key9 => 8,
                                Key0 => 9,
                                _ => unreachable!(),
                            } % StrokeStyle::NUM_VARIANTS,
                        )
                    };
                    context.window().request_redraw();

                    println!("stroke style {:?}", state.stroke_style);
                }

                if input_handler.just_pressed(R) {
                    state.use_individual_style = !state.use_individual_style;
                    context.window().request_redraw();
                }

                if input_handler.shift() && input_handler.just_pressed(S) {
                    let num_string = std::fs::read_to_string("img/num.txt").expect("read num.txt");
                    let num = num_string.trim().parse::<usize>().expect("parse num.txt");
                    let filename = format!("img/strokes{num}.png");

                    let image = unsafe {
                        let PhysicalSize { width, height } = context.window().inner_size();
                        let mut data = std::iter::repeat(0)
                            .take(width as usize * height as usize * 4)
                            .collect::<Vec<_>>();
                        gl.read_pixels(
                            0,
                            0,
                            width as i32,
                            height as i32,
                            glow::RGBA,
                            glow::UNSIGNED_BYTE,
                            glow::PixelPackData::Slice(data.as_mut_slice()),
                        );
                        image::DynamicImage::ImageRgba8(
                            image::RgbaImage::from_raw(width, height, data).unwrap(),
                        )
                    };

                    image.flipv().save(&filename).unwrap();
                    let next_num = num + 1;
                    std::fs::write("img/num.txt", format!("{next_num}")).unwrap();
                    println!("wrote image as {filename}");
                }
            }

            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }

            Event::WindowEvent {
                event: WindowEvent::Touch(touch),
                ..
            } => {
                use tablet_thing::graphics::*;

                cursor_visible = false;

                // TODO handle fingers
                let prev_cursor_y = input_handler.cursor_pos().y as f32;
                input_handler.handle_mouse_move(touch.location);
                let next_cursor_y = input_handler.cursor_pos().y as f32;
                let cursor_dy = next_cursor_y - prev_cursor_y;

                context.window().set_cursor_visible(cursor_visible);

                let PhysicalSize { width, height } = context.window().inner_size();
                let prev_stylus_gl = stroke_to_gl(width, height, state.zoom, state.stylus.point);
                let prev_stylus = gl_to_physical_position(width, height, prev_stylus_gl);

                state.update(width, height, touch);

                let next_stylus_gl = stroke_to_gl(width, height, state.zoom, state.stylus.point);
                let next_stylus = gl_to_physical_position(width, height, next_stylus_gl);

                match (
                    input_handler.button_down(MouseButton::Middle),
                    input_handler.control(),
                ) {
                    (true, false) => {
                        state.move_origin(width, height, prev_stylus, next_stylus);
                    }
                    (true, true) => state.change_zoom(cursor_dy),
                    _ => {}
                }

                context.window().request_redraw();
            }

            Event::WindowEvent {
                event: WindowEvent::MouseWheel { delta, .. },
                ..
            } => {
                let zoom_in = match delta {
                    MouseScrollDelta::LineDelta(_, y) if y.is_sign_positive() => true,
                    MouseScrollDelta::PixelDelta(pos) if pos.y.is_sign_positive() => true,
                    MouseScrollDelta::LineDelta(_, y) if y.is_sign_negative() => false,
                    MouseScrollDelta::PixelDelta(pos) if pos.y.is_sign_negative() => false,
                    _ => unreachable!(),
                };
                const ZOOM_SPEED: f32 = 4.25;

                let dzoom = if zoom_in { ZOOM_SPEED } else { -ZOOM_SPEED };
                state.change_zoom(dzoom);

                context.window().request_redraw();
            }

            Event::WindowEvent {
                event: WindowEvent::ReceivedCharacter(c @ ('[' | ']')),
                ..
            } => {
                match c {
                    '[' => state.decrease_brush(),
                    ']' => state.increase_brush(),
                    _ => unreachable!(),
                };
                context.window().request_redraw();
            }

            Event::WindowEvent {
                event: WindowEvent::MouseInput { state, button, .. },
                ..
            } => {
                input_handler.handle_mouse_button(button, state);
            }

            Event::WindowEvent {
                event: WindowEvent::CursorMoved { position, .. },
                ..
            } => {
                let prev = input_handler.cursor_pos();
                input_handler.handle_mouse_move(position);

                if input_handler.button_down(MouseButton::Left) {
                    let next = input_handler.cursor_pos();
                    let PhysicalSize { width, height } = context.window().inner_size();
                    state.move_origin(width, height, prev, next);
                    context.window().request_redraw();
                }

                if !cursor_visible {
                    cursor_visible = true;
                    context.window().set_cursor_visible(cursor_visible);
                    context.window().request_redraw();
                }
            }

            Event::RedrawRequested(_) => {
                unsafe {
                    gl.use_program(Some(strokes_program));
                    let view = tablet_thing::graphics::view_matrix(
                        state.zoom,
                        context.window().inner_size(),
                        state.origin,
                    );
                    gl.uniform_matrix_4_f32_slice(
                        Some(&view_uniform),
                        false,
                        &view.to_cols_array(),
                    );
                    gl.clear(glow::COLOR_BUFFER_BIT);
                }

                for stroke in state.strokes.iter_mut() {
                    if stroke.points.is_empty() || stroke.erased {
                        continue;
                    }

                    unsafe {
                        let vbo = stroke
                            .vbo
                            .get_or_insert_with(|| gl.create_buffer().unwrap());
                        let vao = stroke
                            .vao
                            .get_or_insert_with(|| gl.create_vertex_array().unwrap());

                        let points_flat = std::slice::from_raw_parts(
                            stroke.points.as_ptr() as *const f32,
                            stroke.points.len() * 3,
                        );

                        let bytes = std::slice::from_raw_parts(
                            points_flat.as_ptr() as *const u8,
                            points_flat.len() * size_of::<f32>(),
                        );

                        gl.bind_buffer(glow::ARRAY_BUFFER, Some(*vbo));
                        gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, &bytes, glow::STATIC_DRAW);

                        gl.bind_vertex_array(Some(*vao));

                        gl.vertex_attrib_pointer_f32(
                            0,
                            2,
                            glow::FLOAT,
                            false,
                            size_of::<f32>() as i32 * 3,
                            0,
                        );
                        gl.enable_vertex_attrib_array(0);

                        gl.vertex_attrib_pointer_f32(
                            1,
                            1,
                            glow::FLOAT,
                            false,
                            size_of::<f32>() as i32 * 3,
                            size_of::<f32>() as i32 * 2,
                        );
                        gl.enable_vertex_attrib_array(1);

                        gl.uniform_3_f32(
                            Some(&stroke_color_uniform),
                            stroke.color[0] as f32 / 255.0,
                            stroke.color[1] as f32 / 255.0,
                            stroke.color[2] as f32 / 255.0,
                        );

                        gl.uniform_1_f32(Some(&brush_size_uniform), stroke.brush_size);

                        gl.draw_arrays(stroke_style, 0, stroke.points.len() as i32);
                    }
                }

                if !cursor_visible {
                    let circle = tablet_thing::graphics::circle_points(state.brush_size, 32);
                    unsafe {
                        gl.use_program(Some(circle_program));
                        let vbo = gl.create_buffer().unwrap();
                        gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
                        let vao = gl.create_vertex_array().unwrap();
                        gl.bind_vertex_array(Some(vao));
                        let bytes = std::slice::from_raw_parts(
                            circle.as_ptr() as *const u8,
                            circle.len() * size_of::<f32>(),
                        );
                        gl.buffer_data_u8_slice(glow::ARRAY_BUFFER, &bytes, glow::STATIC_DRAW);
                        gl.enable_vertex_attrib_array(0);
                        gl.vertex_attrib_pointer_f32(
                            0,
                            2,
                            glow::FLOAT,
                            false,
                            size_of::<f32>() as i32 * 2,
                            0,
                        );

                        gl.uniform_1_f32(
                            Some(&erasing_uniform),
                            if state.stylus.inverted() { 1.0 } else { 0.0 },
                        );
                        gl.uniform_1_f32(
                            Some(&pen_down_uniform),
                            if state.stylus.down() { 1.0 } else { 0.0 },
                        );

                        let PhysicalSize { width, height } = context.window().inner_size();
                        let xform = tablet_thing::graphics::stroke_to_gl(
                            width,
                            height,
                            state.zoom,
                            state.stylus.point,
                        );
                        let view = glam::Mat4::from_scale_rotation_translation(
                            glam::vec3(1.0 / width as f32, 1.0 / height as f32, 1.0),
                            glam::Quat::IDENTITY,
                            glam::vec3(xform.x, xform.y, 0.0),
                        );

                        gl.uniform_matrix_4_f32_slice(
                            Some(&circle_view_uniform),
                            false,
                            &view.to_cols_array(),
                        );
                        gl.draw_arrays(glow::LINE_LOOP, 0, circle.len() as i32 / 2);
                    }
                }

                context.swap_buffers().unwrap();
            }

            Event::WindowEvent {
                event: WindowEvent::Resized(size),
                ..
            } => {
                context.resize(size);
                unsafe {
                    gl.viewport(0, 0, size.width as i32, size.height as i32);
                };
            }

            _ => {}
        }
    });
}
