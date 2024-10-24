use std::cell::RefCell;

use gloo_utils::body;
use wasm_bindgen::{prelude::Closure, JsCast, JsValue, UnwrapThrowExt};
use web_sys::{Element, HtmlElement, HtmlTextAreaElement, MouseEvent};

use crate::{helper::TargetCast, util::ElementEvent, ComponentFlag, Result};

use super::{Component, Context};

// TODO: Note should be the only component set for the nodes its' on.

pub struct Note;

impl Component for Note {
    const FLAG: ComponentFlag = ComponentFlag::NOTE;
    const TITLE: &'static str = "Note";

    const ALLOWED_SIBLINGS: ComponentFlag = ComponentFlag::empty();
    const OVERWRITE_INVALID: bool = true;

    type Data = ();

    fn on_click_button(&self, ctx: &Context<Self>) -> Result<()> {
        debug!("Note - Selected {}", ctx.get_selection_data_ids().len());

        show_popup(None, ctx.clone())?;

        Ok(())
    }

    fn on_click(&self, ctx: &Context<Self>) -> Result<()> {
        info!("on_click");

        for (clicked_flag, id) in ctx.get_selection_data_ids() {
            if clicked_flag == Self::FLAG {
                show_popup(Some(id), ctx.clone())?;

                break;
            }
        }

        Ok(())
    }
}

thread_local! {
    static DISPLAYING: RefCell<Option<Popup>> = RefCell::default();
}

#[allow(dead_code)]
struct Popup {
    events: Vec<ElementEvent>,

    text_area: HtmlTextAreaElement,
    content: Element,
}

impl Popup {
    pub fn close(self) {
        self.content.remove();
    }

    pub fn value(&self) -> String {
        self.text_area.value()
    }
}

fn show_popup(editing_id: Option<u32>, ctx: Context<Note>) -> Result<(), JsValue> {
    let close_popup_fn: Closure<dyn FnMut(MouseEvent)> = Closure::new(|e: MouseEvent| {
        if e.target_unchecked_into::<HtmlElement>()
            .class_list()
            .contains("modal")
        {
            // Stop propagation so we don't open the popup again
            e.stop_propagation();

            DISPLAYING.with(|popup| {
                popup.take().unwrap_throw().close();
            });
        }
    });

    let cancel_fn = || {
        Closure::new(|e: MouseEvent| {
            // Stop propagation so we don't open the popup again
            e.stop_propagation();

            DISPLAYING.with(|popup| {
                popup.take().unwrap_throw().close();
            });
        }) as Closure<dyn FnMut(MouseEvent)>
    };

    let delete_fn = {
        let ctx = ctx.clone();

        Closure::once(move || {
            DISPLAYING.with(move |popup| {
                let popup = popup.take().unwrap_throw();

                ctx.remove_selection(editing_id).unwrap_throw();
                ctx.remove_data(editing_id.unwrap());

                popup.close();

                ctx.save();
            });
        })
    };

    let save_fn = {
        let ctx = ctx.clone();

        Closure::once(move || {
            DISPLAYING.with(move |popup| {
                let popup = popup.take().unwrap_throw();

                if let Some(editing_id) = editing_id {
                    ctx.update_data(editing_id, &popup.value());
                } else {
                    ctx.reload_section().unwrap_throw();
                    let data_pos = ctx.store_data(&popup.value());
                    if let Err(_e) = ctx.insert_selection(Some(data_pos)).unwrap_throw() {
                        // Remove Inserted data if we're unable to insert
                        ctx.remove_data(data_pos);

                        // TODO: Display Error Message
                    }
                }

                popup.close();

                ctx.save();
            });
        })
    };

    let mut element_events = Vec::new();

    let modal = ctx.document.create_element("div")?;
    modal.class_list().add_2("modal", "d-block")?;
    modal.set_attribute("tabindex", "-1")?;
    element_events.push(ElementEvent::link(
        modal.clone().unchecked_into(),
        close_popup_fn,
        |t, f| t.add_event_listener_with_callback("click", f),
        Box::new(|t, f| t.remove_event_listener_with_callback("click", f)),
    ));

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
        title.set_inner_text("Note");
        header.append_child(&title)?;

        let cancel: HtmlElement = ctx
            .document
            .create_element("editor-button")?
            .unchecked_into();
        cancel.set_attribute("type", "editor-button")?;
        cancel.set_attribute("aria-label", "Close")?;
        cancel.class_list().add_1("btn-close")?;

        header.append_child(&cancel)?;
        element_events.push(ElementEvent::link(
            cancel.unchecked_into(),
            cancel_fn(),
            |t, f| t.add_event_listener_with_callback("click", f),
            Box::new(|t, f| t.remove_event_listener_with_callback("click", f)),
        ));
    }

    let text_area = {
        let body = ctx.document.create_element("div")?;
        body.class_list().add_1("modal-body")?;
        inner.append_child(&body)?;

        let text_area: HtmlTextAreaElement =
            ctx.document.create_element("textarea")?.unchecked_into();
        text_area.class_list().add_1("form-control")?;
        text_area.set_max_length(500);
        text_area.set_spellcheck(true);
        body.append_child(&text_area)?;

        if let Some(editing_id) = editing_id {
            text_area.set_value(&ctx.get_data(editing_id).parse::<String>())
        }

        text_area
    };

    {
        let footer = ctx.document.create_element("div")?;
        footer.class_list().add_1("modal-footer")?;
        inner.append_child(&footer)?;

        let save: HtmlElement = ctx
            .document
            .create_element("editor-button")?
            .unchecked_into();
        save.class_list().add_2("btn", "btn-success")?;
        save.set_inner_text("Save");
        footer.append_child(&save)?;
        element_events.push(ElementEvent::link(
            save.unchecked_into(),
            save_fn,
            |t, f| t.add_event_listener_with_callback("click", f),
            Box::new(|t, f| t.remove_event_listener_with_callback("click", f)),
        ));

        let cancel: HtmlElement = ctx
            .document
            .create_element("editor-button")?
            .unchecked_into();
        cancel.class_list().add_2("btn", "btn-danger")?;
        cancel.set_inner_text("Cancel");
        footer.append_child(&cancel)?;
        element_events.push(ElementEvent::link(
            cancel.unchecked_into(),
            cancel_fn(),
            |t, f| t.add_event_listener_with_callback("click", f),
            Box::new(|t, f| t.remove_event_listener_with_callback("click", f)),
        ));

        let delete: HtmlElement = ctx
            .document
            .create_element("editor-button")?
            .unchecked_into();
        delete.class_list().add_2("btn", "btn-danger")?;
        delete.set_inner_text("Delete");
        footer.append_child(&delete)?;

        if editing_id.is_some() {
            element_events.push(ElementEvent::link(
                delete.unchecked_into(),
                delete_fn,
                |t, f| t.add_event_listener_with_callback("click", f),
                Box::new(|t, f| t.remove_event_listener_with_callback("click", f)),
            ));
        } else {
            delete.set_attribute("disabled", "true")?;
        }
    }

    content.append_child(&inner)?;
    body().append_child(&modal)?;

    let popup = Popup {
        events: element_events,

        text_area,
        content: modal,
    };

    // TODO: Replace with set once stable.
    DISPLAYING.with(move |v| {
        *v.borrow_mut() = Some(popup);
    });

    Ok(())
}
