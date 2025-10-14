use std::cell::{Ref, RefCell};
use std::rc::Rc;

use wasm_bindgen::closure::Closure;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{Element, Event, HtmlButtonElement};

use crate::{Colour, SudokuModel};

#[wasm_bindgen(start)]
pub fn main() -> Result<(), JsValue> {
    let sudoku_model = Rc::new(RefCell::new(SudokuModel::example()));

    // Use `web_sys`'s global `window` function to get a handle on the global
    // window object.
    let window = web_sys::window().expect("no global `window` exists");
    let document = window.document().expect("should have a document on window");
    let body = document.body().expect("document should have a body");

    // Manufacture the element we're gonna append
    let table = document.create_element("table")?;
    table.set_class_name("sudoku-table");
    let table_body = document.create_element("tbody")?;
    let buttons = Rc::new(RefCell::new(Vec::new()));
    for y in 0..9 {
        let tr = document.create_element("tr")?;
        tr.set_class_name("sudoku-row");
        for x in 0..9 {
            let td = document.create_element("td")?;
            td.set_class_name("sudoku-col");

            let button = document.create_element("button")?;
            set_button_values(sudoku_model.borrow(), &button, x, y);
            button.set_attribute("x", &x.to_string())?;
            button.set_attribute("y", &y.to_string())?;
            if !sudoku_model.borrow().get(x, y).enabled {
                button.set_attribute("disabled", "")?;
            } else {
                let model = sudoku_model.clone();
                let buttons = buttons.clone();
                let cb = Closure::wrap(Box::new(move |e: Event| {
                    let button = e
                        .current_target()
                        .unwrap()
                        .dyn_into::<HtmlButtonElement>()
                        .unwrap();
                    let x = button.get_attribute("x").unwrap().parse::<usize>().unwrap();
                    let y = button.get_attribute("y").unwrap().parse::<usize>().unwrap();
                    model.borrow_mut().add(x, y, 1);
                    for x in 0..9 {
                        for y in 0..9 {
                            set_button_values(
                                model.borrow(),
                                buttons.borrow().get(x + y * 9).unwrap(),
                                x,
                                y,
                            );
                        }
                    }
                }) as Box<dyn FnMut(_)>);

                button.add_event_listener_with_callback("click", &cb.as_ref().unchecked_ref())?;
                cb.forget();
            }

            td.append_child(&button)?;
            tr.append_child(&td)?;
            buttons.borrow_mut().push(button);
        }
        table_body.append_child(&tr)?;
    }
    table.append_child(&table_body)?;
    body.append_child(&table)?;

    Ok(())
}

fn set_button_values(model: Ref<SudokuModel>, button: &Element, x: usize, y: usize) {
    button.set_inner_html(model.text(x, y));
    match model.colour(x, y) {
        Colour::Black => {
            button.set_class_name("sudoku-cell");
        }
        Colour::Red => {
            button.set_class_name("sudoku-cell red");
        }
        Colour::Green => {
            button.set_class_name("sudoku-cell green");
        }
    }
}
