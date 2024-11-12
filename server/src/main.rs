// SPDX-FileCopyrightText: 2024 Janet Blackquill <uhhadd@gmail.com>
//
// SPDX-License-Identifier: MPL-2.0

use core::str;
use std::{collections::{HashMap, HashSet, VecDeque}, sync::{Arc, Mutex}, time::Duration};

use logic::{field::Field, hooks::{Cubes, Sounds}, input::{Input, InputProvider, Inputs}};
use nanoserde::{DeJson, SerJson};
use logic::proto::{ClientToServer, ServerToClient};
use quad_net::quad_socket::server::{listen, Settings};

#[derive(Default)]
struct ClientState {
    id: Option<u32>,
}

struct WorldClientState {
    field: Field,
    queued_messages: VecDeque<ServerToClient>,
    inputs: Inputs,
    provider: NetworkInputProvider,
    tick: u64,
}

struct World {
    clients: HashMap<u32, WorldClientState>,
}

struct NetworkInputProvider {
    just_pressed: HashSet<Input>,
    current: HashSet<Input>,
}
impl InputProvider for NetworkInputProvider {
    fn peek(&mut self) {
    }
    fn consume(&mut self) {
        self.just_pressed.clear();
    }
    fn key_just_pressed(&self, input: Input) -> bool {
        self.just_pressed.contains(&input)
    }
    fn key_down(&self, input: Input) -> bool {
        self.current.contains(&input)
    }
}

#[derive(Clone, Copy)]
struct DummyImpl;
impl Cubes for DummyImpl {
    fn spawn_cube(&mut self, _x: i32, _y: i32, _color: logic::well::Block) {
    }
}
impl Sounds for DummyImpl {
    fn block_spawn(&mut self, _color: logic::well::Block) {
    }
    fn line_clear(&mut self) {
    }
    fn lock(&mut self) {
    }
    fn land(&mut self) {
    }
}

impl World {
    fn enqueue_message_excluding(&mut self, id: u32, message: ServerToClient) {
        for (client, state) in &mut self.clients {
            if *client == id {
                continue;
            }
            state.queued_messages.push_back(message.clone());
        }
    }
    fn enqueue_message_to(&mut self, id: u32, message: ServerToClient) {
        if let Some(state) = self.clients.get_mut(&id) {
            state.queued_messages.push_back(message.clone());
        }
    }
    fn dequeue_messages_for(&mut self, id: u32) -> VecDeque<ServerToClient> {
        if let Some(state) = self.clients.get_mut(&id) {
            let ret = state.queued_messages.clone();
            state.queued_messages.clear();
            ret
        } else {
            VecDeque::new()
        }
    }
    fn join(&mut self, client_id: u32) {
        self.clients.insert(client_id, WorldClientState {
            field: Field::new(),
            queued_messages: VecDeque::new(),
            inputs: Inputs::new(),
            provider: NetworkInputProvider { just_pressed: HashSet::new(), current: HashSet::new() },
            tick: 0,
        });

        let clients = self.clients.iter().map(|client| { (client.0.clone(), client.1.field.clone()) } ).collect::<Vec<_>>();
        for (client, field) in clients {
            if client == client_id {
                continue;
            }
            self.enqueue_message_to(client_id, ServerToClient::Join { client_id: client, field });
        }
        self.enqueue_message_excluding(client_id, ServerToClient::Join { client_id, field: self.clients[&client_id].field.clone() });
    }
    fn leave(&mut self, client_id: u32) {
        self.clients.remove(&client_id);
        self.enqueue_message_excluding(client_id, ServerToClient::Leave { client_id });
    }
    fn input(&mut self, client_id: u32, input: Input, up: bool) {
        self.enqueue_message_excluding(client_id, ServerToClient::Input { client_id, input, up });
        if let Some(state) = self.clients.get_mut(&client_id) {
            if up {
                state.provider.just_pressed.insert(input);
                state.provider.current.insert(input);
            } else {
                state.provider.just_pressed.remove(&input);
                state.provider.current.remove(&input);
            }
        }
    }
    fn tick(&mut self, client_id: u32) {
        let mut a = DummyImpl;
        let mut b = DummyImpl;
        if let Some(state) = self.clients.get_mut(&client_id) {
            state.inputs.tick(state.tick, &mut state.provider);
            state.field.update(&state.inputs, &mut a, &mut b);
            state.tick += 1;
            self.enqueue_message_excluding(client_id, ServerToClient::Tick { client_id });
        }
    }
}

fn main() {
    let world = Arc::new(Mutex::new(World {
        clients: HashMap::new(),
    }));
    listen(
        "0.0.0.0:8088",
        "0.0.0.0:6507",
        Settings {
            on_message: {
                let world = world.clone();
                move |_out, state: &mut ClientState, msg| {
                    let msg = ClientToServer::deserialize_json(str::from_utf8(&msg).unwrap()).unwrap();

                    match msg {
                    ClientToServer::Join { client_id } => {
                        if state.id.is_none() {
                            state.id = Some(client_id);
                            world.lock().unwrap().join(client_id);
                        }
                    }
                    ClientToServer::Input { input, up } => {
                        if let Some(id) = state.id {
                            world.lock().unwrap().input(id, input, up);
                        }
                    }
                    ClientToServer::Tick {} => {
                        if let Some(id) = state.id {
                            world.lock().unwrap().tick(id);
                        }
                    }
                    }
                }
            },
            on_timer: {
                let world = world.clone();
                move |out, state| {
                    if let Some(id) = state.id {
                        let messages = world.lock().unwrap().dequeue_messages_for(id);
                        for msg in messages {
                            out.send(msg.serialize_json().as_bytes()).unwrap();
                        }
                    }
                }
            },
            on_disconnect: {
                let world = world.clone();
                move |state| {
                    if let Some(id) = state.id {
                        world.lock().unwrap().leave(id);
                    }
                }
            },
            timer: Some(Duration::from_secs_f64(1. / 60.)),
            _marker: std::marker::PhantomData,
        },
    );
}