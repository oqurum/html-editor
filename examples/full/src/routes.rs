#![allow(clippy::let_unit_value)]

use yew::prelude::*;
use yew_router::Routable;

use crate::pages::*;

#[derive(Routable, Debug, Clone, PartialEq, Eq)]
pub enum AppRoute {
    #[at("/")]
    Home,
}

pub fn switch(routes: &AppRoute) -> Html {
    match routes {
        AppRoute::Home => html! { <home::Home /> },
    }
}
