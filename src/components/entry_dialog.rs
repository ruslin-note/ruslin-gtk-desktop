use adw::prelude::*;
use gtk::glib;
use relm4::{gtk, ComponentParts, ComponentSender, RelmWidgetExt, SimpleComponent};

pub struct EntryDialogModel {
    visible: bool,
    entry: gtk::EntryBuffer,
    button_sensitive: bool,
}

pub struct EntryDialogInit {
    pub title: String,
    pub button_label: String,
}

#[derive(Debug)]
pub enum EntryDialogInput {
    Show,
    Hide,
    ChangeButtonSensitive(bool),
    ConfirmText,
}

#[derive(Debug)]
pub enum EntryDialogOutput {
    Text(String),
}

#[relm4::component(pub)]
impl SimpleComponent for EntryDialogModel {
    type Init = EntryDialogInit;
    type Input = EntryDialogInput;
    type Output = EntryDialogOutput;

    view! {
        adw::Window {
            #[watch]
            set_visible: model.visible,
            set_modal: true,

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,

                adw::HeaderBar {
                    #[wrap(Some)]
                    set_title_widget = &adw::WindowTitle {
                        set_title: &init.title,
                        add_css_class: "flat",
                    },
                },

                gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                    set_spacing: 2,

                    gtk::Entry {
                        set_valign: gtk::Align::Center,
                        set_margin_all: 10,
                        set_hexpand: true,
                        set_vexpand: true,
                        set_input_hints: gtk::InputHints::NO_SPELLCHECK,
                        set_buffer: &model.entry,
                    },

                    gtk::Button {
                        set_label: &init.button_label,
                        set_halign: gtk::Align::End,
                        set_margin_end: 10,
                        set_margin_bottom: 10,
                        #[watch]
                        set_sensitive: model.button_sensitive,
                        add_css_class: "suggested-action",

                        connect_clicked[sender] => move |_| {
                            sender.input(EntryDialogInput::ConfirmText);
                        },
                    }

                }
            },

            connect_close_request[sender] => move |_| {
                sender.input(EntryDialogInput::Hide);
                gtk::Inhibit(false)
            },
        }
    }

    fn init(
        init: Self::Init,
        root: &Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = EntryDialogModel {
            visible: false,
            entry: gtk::EntryBuffer::new(None),
            button_sensitive: false,
        };

        model
            .entry
            .connect_text_notify(glib::clone!(@strong sender => move |e| {
                sender.input(EntryDialogInput::ChangeButtonSensitive(e.text().len() > 0));
            }));

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, input: Self::Input, sender: ComponentSender<Self>) {
        match input {
            EntryDialogInput::Show => self.visible = true,
            EntryDialogInput::Hide => self.visible = false,
            EntryDialogInput::ChangeButtonSensitive(sensitive) => self.button_sensitive = sensitive,
            EntryDialogInput::ConfirmText => {
                sender
                    .output(EntryDialogOutput::Text(self.entry.text()))
                    .unwrap();
                self.visible = false;
                self.entry.set_text("");
            }
        }
    }
}
