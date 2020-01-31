use std::collections::LinkedList;
use std::rc::Rc;
use std::cell::RefCell;

use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen::closure::Closure;
use web_sys::{
    Document,
    HtmlCanvasElement,
    CanvasRenderingContext2d
};

/// A struct that represents a point of the world.
#[derive(Debug)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}

/// Used to respond the events sent from the DOM element.
pub trait Responder {
    fn on_mouse_down(&self, point: Point);
    fn on_mouse_move(&self, point: Point);
    fn on_mouse_up(&self);
}

/// A wrapper of the [`Responder`] object.
struct ResponderHolder {
    responder: Option<Box<dyn Responder>>,
}

impl ResponderHolder {
    fn on_mouse_down(&self, point: Point) {
        if let Some(responder) = self.responder.as_ref() {
            responder.on_mouse_down(point);
        }
    }

    fn on_mouse_move(&self, point: Point) {
        if let Some(responder) = self.responder.as_ref() {
            responder.on_mouse_move(point);
        }
    }

    fn on_mouse_up(&self) {
        if let Some(responder) = self.responder.as_ref() {
            responder.on_mouse_up();
        }
    }
}

const DEFAULT_WIDTH: i32 = 500;
const DEFAULT_HEIGHT: i32 = 500;

/// An object that acts as the controller of the canvas DOM elememnt.
#[allow(dead_code)]
pub struct Canvas {
    el: HtmlCanvasElement,
    ctx: CanvasRenderingContext2d,
    responder_holder: Rc<RefCell<ResponderHolder>>,
    event_handlers: LinkedList<Box<dyn Drop>>,
    width: f64,
    height: f64,
    cell_width: f64,
    cell_height: f64,
}

impl Canvas {
    /// Creates a new instance with the given [`web_sys::Document`] and world size.
    /// 
    /// Calling this method has a side-effect that manipulate the DOM to initialize
    /// the canvas and related elements.
    pub fn new(document: &Document, rows: i32, cols: i32) -> Canvas {
        let width = DEFAULT_WIDTH as f64;
        let height = DEFAULT_HEIGHT as f64;
        let cell_width = width / (rows as f64);
        let cell_height = height / (cols as f64);

        let body = document.body().unwrap();
    
        let canvas_el = document.create_element("canvas")
            .unwrap()
            .dyn_into::<web_sys::HtmlCanvasElement>()
            .unwrap();
        canvas_el.set_width(DEFAULT_WIDTH as u32);
        canvas_el.set_height(DEFAULT_HEIGHT as u32);
        body.prepend_with_node_1(&canvas_el).unwrap();

        // Install the event listeners.
        let responder_holder = Rc::new(RefCell::new(ResponderHolder {
            responder: None,
        }));
        let mut event_handlers: LinkedList<Box<dyn Drop>> = LinkedList::new();
        {
            let event_target: &web_sys::EventTarget = &canvas_el;
            // The mouse down event:
            {
                let responder_holder_clone = responder_holder.clone();
                let mouse_down_cb = 
                Box::new(Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
                    let point = Point {
                        x: (event.offset_x() as f64 / cell_width) as i32,
                        y: (event.offset_y() as f64 / cell_height) as i32,
                    };

                    responder_holder_clone.borrow().on_mouse_down(point);
                }) as Box<dyn FnMut(_)>));
                event_target.add_event_listener_with_callback(
                    "mousedown",
                    mouse_down_cb.as_ref().as_ref().unchecked_ref()
                ).unwrap();
                event_handlers.push_back(mouse_down_cb);
            }

            // The mouse move event:
            {
                let responder_holder_clone = responder_holder.clone();
                let mouse_move_cb = 
                Box::new(Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
                    let point = Point {
                        x: (event.offset_x() as f64 / cell_width) as i32,
                        y: (event.offset_y() as f64 / cell_height) as i32,
                    };

                    responder_holder_clone.borrow().on_mouse_move(point);
                }) as Box<dyn FnMut(_)>));
                event_target.add_event_listener_with_callback(
                    "mousemove",
                    mouse_move_cb.as_ref().as_ref().unchecked_ref()
                ).unwrap();
                event_handlers.push_back(mouse_move_cb);
            }

            // The mouse up event:
            {
                let responder_holder_clone = responder_holder.clone();
                let mouse_up_cb = 
                Box::new(Closure::wrap(Box::new(move || {
                    responder_holder_clone.borrow().on_mouse_up();
                }) as Box<dyn FnMut()>));
                event_target.add_event_listener_with_callback(
                    "mouseup",
                    mouse_up_cb.as_ref().as_ref().unchecked_ref()
                ).unwrap();
                event_handlers.push_back(mouse_up_cb);
            }
        }

        // Then, let's just add some style.
        {
            let style = body.style();
            style.set_property("display", "flex").unwrap();
            style.set_property("align-items", "center").unwrap();
            style.set_property("justify-content", "center").unwrap();
            style.set_property("height", "100vh").unwrap();
            style.set_property("margin", "0").unwrap();
            style.set_property("background-color", "#f5f5f5").unwrap();
        }
        {
            let style = canvas_el.style();
            style.set_property("margin", "30px").unwrap();
            style.set_property("box-shadow", "0 10px 30px #00000026").unwrap();
            style.set_property("background-color", "#fff").unwrap();
        }

        let context = canvas_el.get_context("2d")
            .unwrap()  // Result<_, _>
            .unwrap()  // Option<_>
            .dyn_into::<web_sys::CanvasRenderingContext2d>()
            .unwrap();
        return Canvas {
            el: canvas_el,
            ctx: context,
            responder_holder: responder_holder,
            event_handlers: event_handlers,
            width: width,
            height: height,
            cell_width: cell_width,
            cell_height: cell_height,
        };
    }

    pub fn install_responder(&mut self, responder: Box<dyn Responder>) {
        self.responder_holder.borrow_mut().responder = Some(responder);
    }

    pub fn clear(&self) {
        self.ctx.clear_rect(0.0, 0.0, self.width, self.height);
    }

    pub fn draw_cell(&self, x: i32, y: i32) {
        self.ctx.set_fill_style(&JsValue::from_str("#000"));
        self.ctx.fill_rect(
            self.cell_width * (x as f64),
            self.cell_height * (y as f64),
            self.cell_width,
            self.cell_height
        );
    }
}
