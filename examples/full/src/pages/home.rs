use std::{cell::RefCell, rc::Rc};

use editor::{load_and_register, ListenerEvent, ListenerId, MouseListener, SaveState};
use wasm_bindgen::UnwrapThrowExt;
use web_sys::{window, HtmlElement};
use yew::{
    function_component, functional::use_node_ref, html, use_mut_ref, use_state_eq, Callback,
};

#[function_component(Home)]
pub fn home() -> Html {
    let node = use_node_ref();
    let listener = use_mut_ref(|| None);
    let last_save = use_mut_ref(|| Option::<SaveState>::None);

    let debug = use_state_eq(String::new);

    let register = {
        let node = node.clone();
        let debug = debug.clone();
        let last_save = last_save.clone();
        let listener = listener.clone();

        Rc::new(move || {
            // Unset the last listener to reset the HTML
            std::mem::drop(listener.take());

            // TODO: For some reason it converts the fn to FnOnce if these clones aren't here.
            let last_save = last_save.clone();
            let debug = debug.clone();

            let handle = editor::register(
                node.cast::<HtmlElement>().unwrap_throw(),
                MouseListener::All,
                None,
                Some(Rc::new(RefCell::new(move |id: ListenerId| {
                    let save = id.try_save();

                    debug.set(format!("{save:#?}"));

                    *last_save.borrow_mut() = save;
                })) as ListenerEvent),
            )
            .expect_throw("Registering");

            *listener.borrow_mut() = Some(handle);
        }) as Rc<dyn Fn()>
    };

    {
        let register = register.clone();
        yew_hooks::use_mount(move || register());
    }

    let on_click_save = {
        let last_save = last_save.clone();

        Callback::from(move |_| {
            if let Some(state) = last_save.borrow().as_ref() {
                let store = window().unwrap().local_storage().unwrap().unwrap();
                store
                    .set_item("home", &serde_json::to_string(state).unwrap())
                    .unwrap();
            }
        })
    };

    let on_click_load = {
        let node = node.clone();
        let debug = debug.clone();
        let last_save = last_save.clone();
        let listener = listener.clone();

        Callback::from(move |_| {
            // TODO: For some reason it converts the fn to FnOnce if these clones aren't here.
            let debug = debug.clone();

            let store = window().unwrap().local_storage().unwrap().unwrap();

            match serde_json::from_str::<Option<SaveState>>(
                &store.get_item("home").unwrap().unwrap(),
            ) {
                Ok(Some(v)) => {
                    // Unset the last listener to reset the HTML
                    std::mem::drop(listener.take());

                    let last_save2 = last_save.clone();
                    let handle = match load_and_register(
                        node.cast::<HtmlElement>().unwrap_throw(),
                        v.clone(),
                        MouseListener::All,
                        None,
                        Some(Rc::new(RefCell::new(move |id: ListenerId| {
                            let save = id.try_save();

                            debug.set(format!("{save:#?}"));

                            *last_save2.borrow_mut() = save;
                        })) as ListenerEvent),
                    ) {
                        Ok(v) => v,
                        Err(e) => {
                            log::error!("{e:?}");
                            return;
                        }
                    };

                    *listener.borrow_mut() = Some(handle);
                    *last_save.borrow_mut() = Some(v);
                }
                Ok(None) => (),
                Err(e) => log::error!("Loading {e:?}"),
            }
        })
    };

    let on_click_reset = {
        Callback::from(move |_| {
            *last_save.borrow_mut() = None;
            *listener.borrow_mut() = None;

            register();
        })
    };

    html! {
        <>
            <div ref={ node } style="height: 100%; overflow-y: auto; padding: 0 10px;">
                <h2>{ "Home" }</h2>
                <h2>{ "Lorem Ipsum" }</h2>
                <h4>{ "Neque porro quisquam est qui dolorem ipsum quia dolor sit amet, consectetur, adipisci velit..." }</h4>
                <h5>{ "There is no one who loves pain itself, who seeks after it and wants to have it, simply because it is pain..." }</h5>

                <p>{ "Duis ut velit nulla. Morbi luctus mollis nunc, a convallis elit porta tristique. Praesent elementum nec magna quis interdum. Praesent consectetur vitae risus vitae facilisis. Integer felis enim, scelerisque at dui a, aliquam tempus justo. Ut sed nibh sit amet ipsum elementum dapibus sit amet at metus. Duis maximus odio vel fermentum volutpat. Integer porta id diam id pretium." }</p>
                <p>{ "Morbi tincidunt faucibus nibh. Nullam id justo nec nisi mattis dignissim. Nunc blandit justo ac consequat sodales. Quisque ultrices orci in magna egestas lacinia. Lorem ipsum dolor sit amet, consectetur adipiscing elit. Nulla sit amet sem venenatis, pulvinar ipsum vitae, dapibus odio. Fusce tristique ligula nisi. Phasellus at molestie nunc, nec eleifend felis. Duis sollicitudin, arcu quis accumsan condimentum, odio elit egestas odio, vel molestie felis urna eu tellus. Nam condimentum fermentum libero, vehicula mollis magna ultrices id." }</p>
                <p>{ "Pellentesque diam dui, egestas ut consectetur sit amet, rhoncus id nunc. Mauris facilisis semper ligula, in accumsan tellus rhoncus a. Sed pretium augue id semper euismod. Phasellus vitae mollis nunc, vel laoreet eros. Integer sed nulla ut arcu mollis malesuada non nec nisl. Cras accumsan ultrices nibh, id iaculis nunc lacinia eu. Vivamus scelerisque placerat metus vitae fringilla. Nulla facilisi. Maecenas convallis massa eu pharetra laoreet. Etiam tincidunt elit non rhoncus laoreet. Suspendisse faucibus dapibus ipsum, et suscipit urna sagittis a. Nunc venenatis, magna nec interdum auctor, diam nibh vehicula lorem, ut imperdiet mi lectus id elit. Curabitur nec felis non massa euismod pellentesque." }</p>
                <p>{ "Phasellus aliquet sapien tincidunt, tempus felis non, placerat nisi. Ut vel mauris pharetra, rutrum sem quis, egestas neque. Mauris eget tempor arcu. Quisque nec ante nunc. Nunc velit neque, efficitur vitae vehicula id, tristique volutpat odio. Aliquam ultrices tincidunt justo, ac viverra leo porttitor sagittis. Phasellus elementum nibh purus, mattis porttitor magna molestie eu. Aliquam sed sodales ipsum. Aenean semper lacus id metus pulvinar, vel porta odio porta." }</p>
                <p>{ "Pellentesque vel est eget leo varius rhoncus bibendum non velit. Nam auctor dictum justo, eget pretium ante. Quisque volutpat erat eros, a sollicitudin risus fringilla porttitor. Ut pharetra diam et elit pellentesque consectetur. Fusce nec mattis sem, commodo imperdiet purus. Suspendisse venenatis, risus vitae semper dictum, ante ex consectetur massa, vel posuere mauris ante a leo. Aenean iaculis, massa dapibus porttitor porttitor, dolor tellus fermentum urna, eu fermentum justo tellus et orci. Aenean a volutpat nibh. Donec malesuada accumsan nisi, nec maximus metus tempor eget. Nulla facilisi. Sed mauris leo, ornare at ligula in, tincidunt vehicula nulla. " }</p>

                <div class="asdf">
                    <h2>{ "Multi-Tags in div" }</h2>

                    <span>{ "Hello, " }</span>
                    <span>{ "how are " }</span>
                    <span>{ "you?" }</span>

                    <p>{ "Going into a paragraph" }</p>
                </div>

                <div>
                    <h2>{ "New Div sibling" }</h2>
                    <span>{ "Jello" }</span>
                </div>
            </div>

            <div style="background: black; color: whitesmoke; overflow: auto; max-height: 100%; width: 97em; padding: 0 5px;">
                <div>
                    <button onclick={ on_click_save }>{ "Save State" }</button>
                    <button onclick={ on_click_load }>{ "Load State" }</button>
                    <button onclick={ on_click_reset }>{ "Reset View" }</button>
                </div>
                <p style="white-space: pre-wrap;">{ debug.to_string() }</p>
            </div>
        </>
    }
}
