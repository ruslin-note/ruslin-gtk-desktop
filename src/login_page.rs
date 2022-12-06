use adw::prelude::*;
use relm4::{gtk, Component, ComponentParts, ComponentSender, RelmWidgetExt, SimpleComponent};
use ruslin_data::sync::{SyncConfig, SyncResult};

use crate::AppContext;

pub struct LoginPageModel {
    url: String,
    email: String,
    password: String,
    ctx: AppContext,
}

#[derive(Debug)]
pub enum LoginPageInput {
    ChangeUrl(String),
    ChangeEmail(String),
    ChangePassword(String),
    Login,
}

#[derive(Debug)]
pub enum LoginPageCommandOutput {
    LoginResult(SyncResult<()>),
}

#[derive(Debug)]
pub enum LoginPageOutput {
    LoginSuccess,
}

#[relm4::component(pub)]
impl Component for LoginPageModel {
    type Init = AppContext;
    type Input = LoginPageInput;
    type Output = LoginPageOutput;
    type Widgets = ComponentWidgets;
    type CommandOutput = LoginPageCommandOutput;

    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,

            adw::HeaderBar {
                #[wrap(Some)]
                set_title_widget = &adw::WindowTitle {
                    set_title: "Ruslin"
                },
            },

            adw::Clamp {
                set_maximum_size: 500,
                set_tightening_threshold: 300,
                set_margin_all: 5,
                set_vexpand: true,
                set_valign: gtk::Align::Center,

                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,

                    gtk::Label {
                        set_justify: gtk::Justification::Center,
                        set_ellipsize: gtk::pango::EllipsizeMode::End,
                        set_margin_top: 10,
                        set_margin_bottom: 30,
                        add_css_class: "title-1",
                        set_lines: 2,
                        set_label: "Please log into Joplin Server and enjoy using Ruslin",
                    },

                    adw::PreferencesGroup {
                        adw::EntryRow {
                            set_title: "URL",
                            connect_text_notify[sender] => move |entry_row| {
                                sender.input(LoginPageInput::ChangeUrl(entry_row.text().to_string()));
                            }
                        },
                        adw::EntryRow {
                            set_title: "Email",
                            connect_text_notify[sender] => move |entry_row| {
                                sender.input(LoginPageInput::ChangeEmail(entry_row.text().to_string()));
                            }
                        },
                        adw::PasswordEntryRow {
                            set_title: "Password",
                            connect_text_notify[sender] => move |entry_row| {
                                sender.input(LoginPageInput::ChangePassword(entry_row.text().to_string()));
                            }
                        },
                    },

                    #[name = "login_button"]
                    gtk::Button {
                        set_margin_top: 30,
                        set_margin_bottom: 30,
                        add_css_class: "suggested-action",
                        set_label: "Login",
                        #[watch]
                        set_sensitive: !model.url.is_empty() && !model.email.is_empty() && !model.password.is_empty(),
                        connect_clicked[sender] => move |_| {
                            sender.input(LoginPageInput::Login);
                        }
                    }
                }
            }
        }
    }

    fn init(
        init: Self::Init,
        root: &Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = LoginPageModel {
            url: String::new(),
            email: String::new(),
            password: String::new(),
            ctx: init,
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, input: Self::Input, sender: ComponentSender<Self>, root: &Self::Root) {
        match input {
            LoginPageInput::ChangeUrl(url) => {
                self.url = url;
            }
            LoginPageInput::ChangeEmail(email) => {
                self.email = email;
            }
            LoginPageInput::ChangePassword(password) => {
                self.password = password;
            }
            LoginPageInput::Login => {
                let sync_config = SyncConfig::JoplinServer {
                    host: self.url.clone(),
                    email: self.email.clone(),
                    password: self.password.clone(),
                };
                let data = self.ctx.data.clone();
                sender.oneshot_command(async move {
                    LoginPageCommandOutput::LoginResult(data.save_sync_config(sync_config).await)
                });
            }
        }
    }

    fn update_cmd(
        &mut self,
        message: Self::CommandOutput,
        sender: ComponentSender<Self>,
        root: &Self::Root,
    ) {
        match message {
            LoginPageCommandOutput::LoginResult(result) => match result {
                Ok(_) => {
                    sender.output(LoginPageOutput::LoginSuccess).unwrap();
                }
                Err(e) => {
                    log::error!("login failed: {:?}", e);
                }
            },
        }
    }
}
