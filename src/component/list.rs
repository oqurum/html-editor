use std::cell::RefCell;

use gloo_utils::{body, document};
use wasm_bindgen::{prelude::Closure, JsCast, JsValue, UnwrapThrowExt};
use web_sys::{Element, HtmlElement};

use crate::{ComponentFlag, Result};

use super::{Component, Context};

pub struct List;

impl Component for List {
    // No flag needed. We don't store anything in the DOM.
    const FLAG: ComponentFlag = ComponentFlag::empty();
    const TITLE: &'static str = "List";

    type Data = ();

    fn on_click_button(&self, ctx: &Context) -> Result<()> {
        log::debug!("List");

        show_popup(ctx.clone())?;

        Ok(())
    }
}

thread_local! {
    static DISPLAYING: RefCell<Option<Popup>> = RefCell::default();
}

#[allow(dead_code)]
struct Popup {
    content: Element,
    cancel_fn: Closure<dyn FnMut()>,
}

impl Popup {
    pub fn close(self) {
        self.content.remove();
    }
}

fn show_popup(ctx: Context) -> Result<(), JsValue> {
    let cancel_fn = Closure::once(|| {
        DISPLAYING.with(|popup| {
            popup.take().unwrap_throw().close();
        });
    });

    let content = document().create_element("div")?;
    content.class_list().add_1("popup")?;

    let inner = document().create_element("div")?;
    inner.class_list().add_1("popup-container")?;

    {
        let header = document().create_element("div")?;
        header.class_list().add_1("popup-header")?;
        inner.append_child(&header)?;

        let title: HtmlElement = document().create_element("h3")?.unchecked_into();
        title.set_inner_text("List");
        header.append_child(&title)?;

        let cancel: HtmlElement = document().create_element("span")?.unchecked_into();
        cancel.set_inner_text("X");
        header.append_child(&cancel)?;
        cancel.add_event_listener_with_callback("click", cancel_fn.as_ref().unchecked_ref())?;
    }

    {
        let body = document().create_element("div")?;
        body.class_list().add_1("popup-body")?;
        inner.append_child(&body)?;

        let notes_and_highlights = document().create_element("div")?;
        body.append_child(&notes_and_highlights)?;

        // Notes and Highlights (X)

        {
            let nodes = ctx.nodes.borrow();
            let data = nodes.data.upgrade().unwrap();
            let data = data.borrow();

            for text_cont in data.get_flagged_text() {
                let flagged_container: HtmlElement =
                    document().create_element("div")?.unchecked_into();
                flagged_container.class_list().add_1("flagged-item")?;

                // TODO: Allow for handling custom data.

                let content: HtmlElement = document().create_element("p")?.unchecked_into();
                content.class_list().add_1("content")?;
                content.set_inner_text(&text_cont.content);
                flagged_container.append_child(&content)?;

                let footer = document().create_element("div")?;
                footer.class_list().add_1("footer")?;
                flagged_container.append_child(&footer)?;

                match text_cont.flag {
                    ComponentFlag::HIGHLIGHT => {
                        let remove: HtmlElement =
                            document().create_element("span")?.unchecked_into();
                        remove.class_list().add_1("clickable")?;
                        remove.set_inner_text("Remove");
                        footer.append_child(&remove)?;
                    }

                    ComponentFlag::NOTE => {
                        let remove: HtmlElement =
                            document().create_element("span")?.unchecked_into();
                        remove.class_list().add_1("clickable")?;
                        remove.set_inner_text("Remove");
                        footer.append_child(&remove)?;

                        let edit: HtmlElement = document().create_element("span")?.unchecked_into();
                        edit.class_list().add_1("clickable")?;
                        edit.set_inner_text("Edit");
                        footer.append_child(&edit)?;
                    }

                    _ => (),
                }

                notes_and_highlights.append_child(&flagged_container)?;
            }
        }
    }

    content.append_child(&inner)?;
    body().append_child(&content)?;

    let popup = Popup { content, cancel_fn };

    // TODO: Replace with set once stable.
    DISPLAYING.with(move |v| {
        *v.borrow_mut() = Some(popup);
    });

    Ok(())
}
