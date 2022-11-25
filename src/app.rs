use adw::prelude::*;
use gtk::prelude::{ApplicationExt, ApplicationWindowExt, GtkWindowExt, SettingsExt, WidgetExt};
use gtk::{gio, glib};
use relm4::{
    actions::{ActionGroupName, RelmAction, RelmActionGroup},
    gtk, main_application, Component, ComponentController, ComponentParts, ComponentSender,
    Controller, SimpleComponent,
};

use crate::config::{APP_ID, PROFILE};
use crate::modals::about::AboutDialog;
use crate::properties;

pub(super) struct App {
    about_dialog: Controller<AboutDialog>,
    current_note_index: u32,
}

#[derive(Debug)]
pub(super) enum AppMsg {
    Quit,
    SelectNote(u32),
}

relm4::new_action_group!(pub(super) WindowActionGroup, "win");
relm4::new_stateless_action!(PreferencesAction, WindowActionGroup, "preferences");
relm4::new_stateless_action!(pub(super) ShortcutsAction, WindowActionGroup, "show-help-overlay");
relm4::new_stateless_action!(AboutAction, WindowActionGroup, "about");

#[relm4::component(pub)]
impl SimpleComponent for App {
    type Init = ();
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
        main_window = gtk::ApplicationWindow::new(&main_application()) {
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
            set_titlebar = &adw::HeaderBar {
                pack_end = &gtk::MenuButton {
                    set_icon_name: "open-menu-symbolic",
                    set_menu_model: Some(&primary_menu),
                }
            },

            #[name = "leaflet"]
            adw::Leaflet {
                set_can_navigate_back: true,

                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,

                    #[name = "sidebar_header"]
                    adw::HeaderBar {
                        #[wrap(Some)]
                        set_title_widget = &adw::WindowTitle {
                            set_title: "Notes",
                        },
                    },

                    gtk::ListBox {
                        set_selection_mode: gtk::SelectionMode::Single,

                        adw::ActionRow {
                            set_title: "Note 1",
                        },

                        adw::ActionRow {
                            set_title: "Note 2",
                        },

                        adw::ActionRow {
                            set_title: "Note 3",
                        },

                        connect_row_selected[sender] => move |_, row| {
                            if let Some(row) = row {
                                sender.input(AppMsg::SelectNote((row.index() + 1) as u32));
                            }
                        }
                    }
                },

                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_hexpand: true,

                    #[name = "content_header"]
                    adw::HeaderBar {
                        #[name = "back_button"]
                        pack_start = &gtk::Button {
                            set_icon_name: "go-previous-symbolic",
                            connect_clicked[leaflet] => move |_| {
                                leaflet.navigate(adw::NavigationDirection::Back);
                            }
                        },

                        #[wrap(Some)]
                        set_title_widget = &adw::WindowTitle {
                            set_title: "Content",
                        },
                    },

                    gtk::Label {
                        set_vexpand: true,

                        #[watch]
                        set_text: &format!("Page {}", model.current_note_index),
                    }
                },
            }
        }
    }

    fn init(
        _init: Self::Init,
        root: &Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let about_dialog = AboutDialog::builder()
            .transient_for(root)
            .launch(())
            .detach();

        let model = Self {
            about_dialog,
            current_note_index: 0,
        };

        let widgets = view_output!();

        widgets
            .leaflet
            .bind_property(
                properties::folded(),
                &widgets.sidebar_header,
                properties::show_end_title_buttons(),
            )
            .flags(glib::BindingFlags::SYNC_CREATE)
            .build();
        widgets
            .leaflet
            .bind_property(
                properties::folded(),
                &widgets.content_header,
                properties::show_start_title_buttons(),
            ) // ? https://gnome.pages.gitlab.gnome.org/libadwaita/doc/main/class.HeaderBar.html
            .flags(glib::BindingFlags::SYNC_CREATE)
            .build();
        widgets
            .leaflet
            .bind_property(
                properties::folded(),
                &widgets.back_button,
                properties::visible(),
            )
            .flags(glib::BindingFlags::SYNC_CREATE)
            .build();

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
                sender.send(());
            })
        };

        actions.add_action(&shortcuts_action);
        actions.add_action(&about_action);

        widgets
            .main_window
            .insert_action_group(WindowActionGroup::NAME, Some(&actions.into_action_group()));

        widgets.load_window_size();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, message: Self::Input, _sender: ComponentSender<Self>) {
        match message {
            AppMsg::Quit => main_application().quit(),
            AppMsg::SelectNote(index) => self.current_note_index = index,
        }
    }

    fn pre_view() {
        /*
           https://relm4.org/book/next/component_macro_reference.html?highlight=pre_view#manual-view
           You can also implement your own view logic, which will be added to the view code that the view macro generates.
           Code inside pre_view() will run before the view update, while post_view() will run after.
        */
        widgets.leaflet.navigate(adw::NavigationDirection::Forward);
    }

    fn shutdown(&mut self, widgets: &mut Self::Widgets, _output: relm4::Sender<Self::Output>) {
        widgets.save_window_size().unwrap();
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
