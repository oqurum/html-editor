use wasm_bindgen::UnwrapThrowExt;
use web_sys::HtmlElement;
use yew::{function_component, functional::use_node_ref, html};

#[function_component(Home)]
pub fn home() -> Html {
    let node = use_node_ref();

    {
        let node = node.clone();
        yew_hooks::use_mount(move || {
            editor::register(node.cast::<HtmlElement>().unwrap_throw()).unwrap_throw();
        });
    }

    html! {
        <div ref={ node }>
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
    }
}
