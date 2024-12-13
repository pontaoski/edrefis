// SPDX-FileCopyrightText: 2024 Janet Blackquill <uhhadd@gmail.com>
//
// SPDX-License-Identifier: MPL-2.0

use std::{collections::HashSet, sync::Arc, time::Duration};
use futures::channel::oneshot::Receiver;
use logic::{field::{Field, GameState}, hooks::{Cubes, Sounds}, input::{Input, InputProvider, Inputs}};
use wasm_bindgen::JsValue;
use web_sys::HtmlCanvasElement;
use winit::{application::ApplicationHandler, dpi::PhysicalSize, event::WindowEvent, event_loop::{ActiveEventLoop, EventLoop}, platform::web::EventLoopExtWebSys, window::Window};

use crate::{gpu::State, graphics_gpu::{self, Graphics}};

#[derive(Clone, Copy)]
struct DummyImpl;
impl Cubes for DummyImpl {
    fn spawn_cube(&mut self, _x: i32, _y: i32, _color: logic::well::Block) {}
}
impl Sounds for DummyImpl {
    fn block_spawn(&mut self, color: logic::well::Block) {}
    fn line_clear(&mut self) {}
    fn lock(&mut self) {}
    fn land(&mut self) {}
}

struct App {
    canvas: HtmlCanvasElement,
    field: Field,
    ticks: u64,
    inputs: Inputs,
    input_provider: WebInputs,
    window: Option<Arc<Window>>,
    renderer: Option<(State<'static>, Graphics)>,

    renderer_receiver: Option<Receiver<(State<'static>, Graphics)>>
}

fn input_to_web_key(keycode: Input) -> winit::keyboard::PhysicalKey {
    match keycode {
        Input::Up => winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::ArrowUp),
        Input::Down => winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::ArrowDown),
        Input::Left => winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::ArrowLeft),
        Input::Right => winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::ArrowRight),
        Input::CW => winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::KeyX),
        Input::CCW => winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::KeyZ),
    }
}

struct WebInputs {
    just_pressed_key: HashSet<winit::keyboard::PhysicalKey>,
    current_key: HashSet<winit::keyboard::PhysicalKey>,
}
impl WebInputs {
    fn new() -> WebInputs {
        WebInputs {
            just_pressed_key: HashSet::new(),
            current_key: HashSet::new(),
        }
    }
    fn push_key(&mut self, keycode: winit::keyboard::PhysicalKey) {
        self.just_pressed_key.insert(keycode.clone());
        self.current_key.insert(keycode);
    }
    fn release_key(&mut self, keycode: winit::keyboard::PhysicalKey) {
        self.just_pressed_key.remove(&keycode);
        self.current_key.remove(&keycode);
    }
}

impl InputProvider for WebInputs {
    fn peek(&mut self) {}

    fn consume(&mut self) {
        self.just_pressed_key.clear();
    }

    fn key_just_pressed(&self, input: Input) -> bool {
        self.just_pressed_key.contains(&input_to_web_key(input))
    }

    fn key_down(&self, input: Input) -> bool {
        self.current_key.contains(&input_to_web_key(input))
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        use winit::platform::web::WindowAttributesExtWebSys;

        let window = Arc::new(event_loop.create_window(Window::default_attributes().with_canvas(Some(self.canvas.clone()))).unwrap());

        let (sender, receiver) = futures::channel::oneshot::channel();
        let width = self.canvas.width();
        let height = self.canvas.height();
        self.renderer_receiver = Some(receiver);
        let threaded = window.clone();
        wasm_bindgen_futures::spawn_local(async move {
            let mut gpu = crate::gpu::State::new(width, height, |instance| {
                instance.create_surface(threaded).map_err(|e| e.to_string())
            }).await.unwrap();
            let graphics = graphics_gpu::Graphics::new(&mut gpu).unwrap();

            sender.send((gpu, graphics));
        });
        self.window = Some(window);
    }
    fn new_events(&mut self, event_loop: &ActiveEventLoop, cause: winit::event::StartCause) {
        if let winit::event::StartCause::ResumeTimeReached { .. } = cause {
            let mut sounds = DummyImpl {};
            let mut cubes = DummyImpl {};

            self.ticks += 1;
            self.inputs.tick(self.ticks, &mut self.input_provider);
            self.field.update(&mut self.inputs, &mut sounds, &mut cubes);

            event_loop.set_control_flow(winit::event_loop::ControlFlow::wait_duration(Duration::from_secs_f64(1. / 60.)));
            self.window.as_mut().unwrap().request_redraw();
        }
    }
    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        let mut renderer_received = false;
        if let Some(receiver) = self.renderer_receiver.as_mut() {
            if let Ok(Some(renderer)) = receiver.try_recv() {
                self.renderer = Some(renderer);
                renderer_received = true;
            }
        }
        if renderer_received {
            self.renderer_receiver = None;
        }

        let Some((gpu, graphics)) = self.renderer.as_mut() else {
            return;
        };

        match event {
            WindowEvent::RedrawRequested => {
                match self.field.state {
                    GameState::ActivePiece { piece, .. } => {
                        graphics.render(&self.field, &self.field.well, Some(&piece), &self.field.next, gpu);
                    }
                    _ => {
                        graphics.render(&self.field, &self.field.well, None, &self.field.next, gpu);
                    }
                }
            }
            WindowEvent::Resized(PhysicalSize { width, height }) => {
                let (width, height) = ((width).max(1), (height).max(1));
                gpu.resize(width, height);
            }
            WindowEvent::KeyboardInput { event, .. } => {
                if event.state.is_pressed() {
                    self.input_provider.push_key(event.physical_key);
                } else {
                    self.input_provider.release_key(event.physical_key);
                }
            }
            _ => {

            }
        }
    }
}

pub fn main() -> Result<(), String> {
    use wasm_bindgen::JsCast;

    std::panic::set_hook(Box::new(console_error_panic_hook::hook));

    let event_loop = EventLoop::new().map_err(|e| e.to_string())?;

    let canvas = web_sys::window()
        .ok_or("No window found")?
        .document()
        .ok_or("No window.document found")?
        .get_element_by_id("canvas")
        .ok_or("No window.document canvas found")?
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .map_err(|e| e.to_string())?;

    let field = Field::new();

    let app: App = App {
        canvas,
        field,
        ticks: 0,
        inputs: Inputs::new(),
        input_provider: WebInputs::new(),
        window: None,
        renderer: None,
        renderer_receiver: None,
    };

    event_loop.set_control_flow(winit::event_loop::ControlFlow::wait_duration(Duration::from_secs_f64(1. / 60.)));
    event_loop.spawn_app(app);

    Ok(())
}
