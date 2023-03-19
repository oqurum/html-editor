use std::cell::RefCell;

use gloo_utils::body;
use wasm_bindgen::{prelude::Closure, JsCast, JsValue, UnwrapThrowExt};
use web_sys::{Element, HtmlElement};

use crate::{ComponentFlag, Result};

use super::{Component, Context};

pub struct List;

impl Component for List {
    // No flag needed. We don't store anything in the DOM.
    const FLAG: ComponentFlag = ComponentFlag::LIST;
    const TITLE: &'static str = "List";

    type Data = ();

    fn on_click_button(&self, ctx: &Context<Self>) -> Result<()> {
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

fn show_popup(ctx: Context<List>) -> Result<(), JsValue> {
    let cancel_fn = Closure::once(|| {
        DISPLAYING.with(|popup| {
            popup.take().unwrap_throw().close();
        });
    });

    let modal = ctx.document.create_element("div")?;
    modal.class_list().add_2("modal", "d-block")?;
    modal.set_attribute("tabindex", "-1")?;

    let content = ctx.document.create_element("div")?;
    content.class_list().add_1("modal-dialog")?;
    modal.append_child(&content)?;

    let inner = ctx.document.create_element("div")?;
    inner.class_list().add_1("modal-content")?;

    {
        let header = ctx.document.create_element("div")?;
        header.class_list().add_1("modal-header")?;
        inner.append_child(&header)?;

        let title: HtmlElement = ctx.document.create_element("h3")?.unchecked_into();
        title.set_inner_text("List");
        header.append_child(&title)?;

        let cancel: HtmlElement = ctx
            .document
            .create_element("editor-button")?
            .unchecked_into();
        cancel.set_attribute("type", "editor-button")?;
        cancel.set_attribute("aria-label", "Close")?;
        cancel.class_list().add_1("btn-close")?;

        header.append_child(&cancel)?;
        cancel.add_event_listener_with_callback("click", cancel_fn.as_ref().unchecked_ref())?;
    }

    {
        let body = ctx.document.create_element("div")?;
        body.class_list().add_1("modal-body")?;
        inner.append_child(&body)?;

        let notes_and_highlights = ctx.document.create_element("div")?;
        body.append_child(&notes_and_highlights)?;

        // Notes and Highlights (X)

        {
            let nodes = ctx.nodes.borrow();
            let data = nodes.data.upgrade().unwrap();
            let data = data.borrow();

            for text_cont in data.get_flagged_text() {
                let flagged_container: HtmlElement =
                    ctx.document.create_element("div")?.unchecked_into();
                flagged_container
                    .class_list()
                    .add_1("editor-flagged-item")?;

                // TODO: Allow for handling custom data.

                let content: HtmlElement = ctx.document.create_element("p")?.unchecked_into();
                content.class_list().add_1("editor-content")?;
                content.set_inner_text(&text_cont.content);
                flagged_container.append_child(&content)?;

                let footer = ctx.document.create_element("div")?;
                footer.class_list().add_1("editor-footer")?;
                flagged_container.append_child(&footer)?;

                match text_cont.flag {
                    ComponentFlag::HIGHLIGHT => {
                        let remove: HtmlElement =
                            ctx.document.create_element("span")?.unchecked_into();
                        remove.class_list().add_1("editor-clickable")?;
                        remove.set_inner_text("Remove");
                        footer.append_child(&remove)?;
                    }

                    ComponentFlag::NOTE => {
                        let remove: HtmlElement =
                            ctx.document.create_element("span")?.unchecked_into();
                        remove.class_list().add_1("editor-clickable")?;
                        remove.set_inner_text("Remove");
                        footer.append_child(&remove)?;

                        let edit: HtmlElement =
                            ctx.document.create_element("span")?.unchecked_into();
                        edit.class_list().add_1("editor-clickable")?;
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
    body().append_child(&modal)?;

    let popup = Popup {
        content: modal,
        cancel_fn,
    };

    // TODO: Replace with set once stable.
    DISPLAYING.with(move |v| {
        *v.borrow_mut() = Some(popup);
    });

    Ok(())
}
