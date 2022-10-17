use yew::prelude::*;
use yew_router::prelude::*;

use crate::routes::{switch, AppRoute};

#[function_component(App)]
pub fn app() -> Html {
    html! {
        <BrowserRouter>
            <Switch<AppRoute> render={ Switch::render(switch) } />
        </BrowserRouter>
    }
}
