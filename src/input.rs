use specs::{Component, HashMapStorage, Join};
use winit::event::{ElementState, KeyboardInput, VirtualKeyCode};

#[derive(Debug, Component)]
#[storage(HashMapStorage)]
pub struct InputVector(pub glam::Vec2);

/// Input event queue used to store input events for the current frame.
pub struct InputEventQueue {
    queue: std::collections::VecDeque<KeyboardInput>,
}

impl InputEventQueue {
    pub fn push_event(&mut self, input_event: KeyboardInput) {
        self.queue.push_back(input_event);
    }

    pub fn pop_event(&mut self) -> Option<KeyboardInput> {
        self.queue.pop_front()
    }
}

impl Default for InputEventQueue {
    fn default() -> Self {
        Self {
            queue: std::collections::VecDeque::new(),
        }
    }
}

/// Input resource used to track current key presses & their associated state.
pub struct InputTracker {
    keys: Vec<(VirtualKeyCode, ElementState)>,
}

impl InputTracker {
    fn get_key_entry_mut(
        &mut self,
        key: VirtualKeyCode,
    ) -> Option<&mut (VirtualKeyCode, ElementState)> {
        self.keys.iter_mut().find(|(keycode, _)| keycode.eq(&key))
    }

    /// Consumes an input event, updating the internal state tracker with
    /// the event data.
    pub fn consume_input_event(&mut self, event: winit::event::KeyboardInput) {
        if let Some(virtual_keycode) = event.virtual_keycode {
            if let Some((_, key_state)) = self.get_key_entry_mut(virtual_keycode) {
                *key_state = event.state;
            } else {
                self.keys.push((virtual_keycode, event.state));
            }
        }
    }

    /// Maintains the integrity of the input tracker's internal state
    /// by removing any released keys.
    ///
    /// IMPORTANT: Call this method *only* after all key events have been processed.
    pub fn maintain(&mut self) {
        for index in (0..self.keys.len()).rev() {
            if self.keys[index].1.eq(&ElementState::Released) {
                self.keys.remove(index);
            }
        }
    }

    /// Returns an iterator to the underlying `Vec`.
    pub fn iter(&self) -> std::slice::Iter<(VirtualKeyCode, ElementState)> {
        self.keys.iter()
    }
}

impl Default for InputTracker {
    fn default() -> Self {
        Self { keys: Vec::new() }
    }
}

pub struct InputSystem;

impl InputSystem {
    fn process_input((keycode, _): &(VirtualKeyCode, ElementState), vec: &mut glam::Vec2) {
        // TODO process by Pressed/Released?
        match keycode {
            VirtualKeyCode::A => {
                vec.x += 1.0;
            }

            VirtualKeyCode::D => {
                vec.x -= 1.0;
            }

            VirtualKeyCode::W => {
                vec.y += 1.0;
            }

            VirtualKeyCode::S => {
                vec.y -= 1.0;
            }

            _ => {}
        }
    }
}

impl<'a> specs::System<'a> for InputSystem {
    type SystemData = (
        specs::Write<'a, InputEventQueue>,
        specs::Write<'a, InputTracker>,
        specs::WriteStorage<'a, InputVector>,
    );

    fn run(&mut self, (mut input_events, mut input_tracker, mut input_vectors): Self::SystemData) {
        // Process all pending input events.
        while let Some(input_event) = input_events.pop_event() {
            input_tracker.consume_input_event(input_event);
        }

        let mut vec = glam::Vec2::ZERO;

        // Apply current input state to the translation vector.
        for key in input_tracker.iter() {
            Self::process_input(key, &mut vec);
        }

        // Apply translation vector to all input vectors.
        for input_vector in (&mut input_vectors).join() {
            input_vector.0 = vec;
        }

        // Maintain the InputTracker, discarding any Release presses.
        // TODO perhaps figure a way to do this at end-of-frame?
        input_tracker.maintain();
    }
}
