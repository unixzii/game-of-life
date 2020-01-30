use std::rc::Rc;
use std::cell::{RefCell, RefMut};

use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen::closure::Closure;
use web_sys::{console, window};

use crate::ui;
use crate::engine;

struct UiResponder {
    state: State,
}

impl ui::Responder for UiResponder {
    fn on_mouse_down(&self, point: ui::Point) {
        js_log!("on_mouse_down: {:?}", point);
        let mut state_inner = self.state.get_inner();
        state_inner.is_mouse_down = true;

        // TODO: Maybe we should not use inner here.
        drop(state_inner);

        self.state.put_cell(point.x, point.y);
    }

    fn on_mouse_move(&self, point: ui::Point) {
        if !self.state.get_inner().is_mouse_down {
            return;
        }

        js_log!("on_mouse_move: {:?}", point);
        self.state.put_cell(point.x, point.y);
    }

    fn on_mouse_up(&self) {
        js_log!("on_mouse_up");
        self.state.get_inner().is_mouse_down = false;
    }
}

pub struct Config {
    pub update_interval: i32,
}

pub struct State {
    inner: Rc<RefCell<StateInner>>,
}

struct StateInner {
    canvas: ui::Canvas,
    world: engine::World,
    config: Config,
    is_mouse_down: bool,
    timer_closure: Option<Box<dyn Drop>>,
    timer_id: i32,
}

impl State {
    pub fn new(canvas: ui::Canvas, world: engine::World, config: Config) -> State {
        let state = State {
            inner: Rc::new(RefCell::new(StateInner {
                canvas: canvas,
                world: world,
                config: config,
                is_mouse_down: false,
                timer_closure: None,
                timer_id: -1,
            })),
        };

        let responder = Box::new(UiResponder { state: state.clone() });
        state.get_inner().canvas.install_responder(responder);

        return state;
    }

    pub fn clone(&self) -> State {
        return State {
            inner: self.inner.clone(),
        };
    }

    pub fn resume(&self) {
        let mut inner = self.get_inner();
        if inner.timer_closure.is_some() {
            return;
        }

        let state_clone = self.clone();
        let closure = Box::new(Closure::wrap(Box::new(move || {
            state_clone.tick();
        }) as Box<dyn FnMut()>));
        let timer_id =
        window().unwrap().set_interval_with_callback_and_timeout_and_arguments_0(
            closure.as_ref().as_ref().unchecked_ref(),
            inner.config.update_interval
        ).unwrap();
        inner.timer_closure = Some(closure);
        inner.timer_id = timer_id;
    }

    pub fn pause(&self) {
        let mut inner = self.get_inner();
        if inner.timer_closure.is_none() {
            return;
        }

        window().unwrap().clear_interval_with_handle(inner.timer_id);
        inner.timer_closure = None;
        inner.timer_id = -1;
    }

    fn put_cell(&self, x: i32, y: i32) {
        self.get_inner().world.set_cell(x, y, engine::Cell::Alive);
        self.update_canvas();
    }

    fn tick(&self) {
        // js_log!("tick!");

        self.get_inner().world.next_gen();
        self.update_canvas();
    }

    fn update_canvas(&self) {
        let inner = self.get_inner();
        inner.canvas.clear();
        for col in 0..(inner.world.height()) {
            for row in 0..(inner.world.width()) {
                if inner.world.cell_at(row, col) == engine::Cell::Alive {
                    inner.canvas.draw_cell(row, col);
                }
            }
        }
    }

    fn get_inner<'a>(&'a self) -> RefMut<'a, StateInner> {
        return self.inner.borrow_mut();
    }
}

impl Drop for State {
    fn drop(&mut self) {
        self.pause();
    }
}
