use std::sync::Arc;

use adw::prelude::*;
use gtk::prelude::{ApplicationExt, ApplicationWindowExt, GtkWindowExt, SettingsExt, WidgetExt};
use gtk::{gio, glib};
use relm4::{
    actions::{ActionGroupName, RelmAction, RelmActionGroup},
    gtk, main_application, Component, ComponentController, ComponentParts, ComponentSender,
    Controller, SimpleComponent,
};

use crate::config::{APP_ID, PROFILE};
use crate::content_page::{ContentPageInit, ContentPageModel};
use crate::login_page::{LoginPageModel, LoginPageOutput};
use crate::modals::about::AboutDialog;
use ruslin_data::RuslinData;

pub struct App {
    about_dialog: Controller<AboutDialog>,
    content_page: Controller<ContentPageModel>,
    login_page: Controller<LoginPageModel>,
    ctx: AppContext,
}

#[derive(Debug)]
pub enum AppMsg {
    Quit,
    RefreshPageStack,
}

relm4::new_action_group!(pub(super) WindowActionGroup, "win");
relm4::new_stateless_action!(PreferencesAction, WindowActionGroup, "preferences");
relm4::new_stateless_action!(pub(super) ShortcutsAction, WindowActionGroup, "show-help-overlay");
relm4::new_stateless_action!(AboutAction, WindowActionGroup, "about");

#[derive(Debug, Clone)]
pub struct AppContext {
    pub data: Arc<RuslinData>,
}

#[derive(Debug)]
pub struct AppInit {
    pub ctx: AppContext,
}

#[relm4::component(pub)]
impl SimpleComponent for App {
    type Init = AppInit;
    type Input = AppMsg;
    type Output = ();
    type Widgets = AppWidgets;

    menu! {
        primary_menu: {
            section! {
                "_Preferences" => PreferencesAction,
                "_Keyboard" => ShortcutsAction,
                "_About Ruslin" => AboutAction,
            }
        }
    }

    view! {
        main_window = adw::ApplicationWindow::new(&main_application()) {
            connect_close_request[sender] => move |_| {
                sender.input(AppMsg::Quit);
                gtk::Inhibit(true)
            },

            #[wrap(Some)]
            set_help_overlay: shortcuts = &gtk::Builder::from_resource(
                    "/org/dianqk/ruslin/gtk/help-overlay.ui"
                )
                .object::<gtk::ShortcutsWindow>("help_overlay")
                .unwrap() -> gtk::ShortcutsWindow {
                    set_transient_for: Some(&main_window),
                    set_application: Some(&main_application()),
            },

            add_css_class?: if PROFILE == "Devel" {
                    Some("devel")
                } else {
                    None
                },

            #[wrap(Some)]
            set_content: stack = &gtk::Stack {
                set_transition_type: gtk::StackTransitionType::None,
                add_child: model.login_page.widget(),
                add_child: model.content_page.widget(),
            },
        }
    }

    fn init(
        init: Self::Init,
        root: &Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let about_dialog = AboutDialog::builder()
            .transient_for(root)
            .launch(())
            .detach();

        let content_page = ContentPageModel::builder()
            .launch(ContentPageInit {
                ctx: init.ctx.clone(),
            })
            .detach();

        let login_page = LoginPageModel::builder().launch(init.ctx.clone()).forward(
            sender.input_sender(),
            |msg| match msg {
                LoginPageOutput::LoginSuccess => AppMsg::RefreshPageStack,
            },
        );

        let model = Self {
            about_dialog,
            content_page,
            login_page,
            ctx: init.ctx,
        };

        let widgets = view_output!();

        let actions = RelmActionGroup::<WindowActionGroup>::new();

        let shortcuts_action = {
            let shortcuts = widgets.shortcuts.clone();
            RelmAction::<ShortcutsAction>::new_stateless(move |_| {
                shortcuts.present();
            })
        };

        let about_action = {
            let sender = model.about_dialog.sender().clone();
            RelmAction::<AboutAction>::new_stateless(move |_| {
                sender.send(()).unwrap();
            })
        };

        actions.add_action(&shortcuts_action);
        actions.add_action(&about_action);

        widgets
            .main_window
            .insert_action_group(WindowActionGroup::NAME, Some(&actions.into_action_group()));

        widgets.load_window_size();

        sender.input(AppMsg::RefreshPageStack);

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        match message {
            AppMsg::Quit => main_application().quit(),
            AppMsg::RefreshPageStack => {}
        }
    }

    fn shutdown(&mut self, widgets: &mut Self::Widgets, _output: relm4::Sender<Self::Output>) {
        widgets.save_window_size().unwrap();
    }

    fn pre_view() {
        if model.ctx.data.sync_exists() {
            stack.set_visible_child(model.content_page.widget());
        } else {
            stack.set_visible_child(model.login_page.widget());
        }
    }

    fn post_view() {
        stack.set_transition_type(gtk::StackTransitionType::SlideLeft);
    }
}

impl AppWidgets {
    fn save_window_size(&self) -> Result<(), glib::BoolError> {
        let settings = gio::Settings::new(APP_ID);
        let (width, height) = self.main_window.default_size();

        settings.set_int("window-width", width)?;
        settings.set_int("window-height", height)?;

        settings.set_boolean("is-maximized", self.main_window.is_maximized())?;

        Ok(())
    }

    fn load_window_size(&self) {
        let settings = gio::Settings::new(APP_ID);

        let width = settings.int("window-width");
        let height = settings.int("window-height");
        let is_maximized = settings.boolean("is-maximized");

        self.main_window.set_default_size(width, height);

        if is_maximized {
            self.main_window.maximize();
        }
    }
}
