#[rustfmt::skip]
mod config;
mod app;
mod components;
mod content_page;
mod icons;
mod modals;
mod properties;
mod setup;

use gtk::prelude::ApplicationExt;
use relm4::{
    actions::{AccelsPlus, RelmAction, RelmActionGroup},
    gtk, main_application, RelmApp,
};

use app::App;
use setup::setup;

use crate::config::APP_ID;

relm4::new_action_group!(AppActionGroup, "app");
relm4::new_stateless_action!(QuitAction, AppActionGroup, "quit");

fn main() {
    #[cfg(debug_assertions)]
    let max_level = tracing::Level::DEBUG;

    #[cfg(not(debug_assertions))]
    let max_level = tracing::Level::INFO;

    // Enable logging
    tracing_subscriber::fmt()
        .with_span_events(tracing_subscriber::fmt::format::FmtSpan::FULL)
        .with_max_level(max_level)
        .init();

    setup();

    let app = main_application();
    app.set_application_id(Some(APP_ID));
    app.set_resource_base_path(Some("/org/dianqk/ruslin/"));

    let actions = RelmActionGroup::<AppActionGroup>::new();

    let quit_action = {
        let app = app.clone();
        RelmAction::<QuitAction>::new_stateless(move |_| {
            app.quit();
        })
    };
    actions.add_action(&quit_action);

    app.set_accelerators_for_action::<QuitAction>(&["<Control>q"]);

    app.set_action_group(Some(&actions.into_action_group()));

    let app = RelmApp::with_app(app);

    app.run::<App>(());
}
