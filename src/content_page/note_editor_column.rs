use adw::prelude::*;
use relm4::{gtk, prelude::*, ComponentParts, ComponentSender, SimpleComponent};
use ruslin_data::{Note, UpdateSource};
use sourceview5::{prelude::*, LanguageManager, StyleSchemeManager};

use crate::AppContext;

#[tracker::track]
pub struct NoteEditorColumnModel {
    #[tracker::do_not_track]
    pub ctx: AppContext,
    pub current_note: Option<Note>,
}

pub struct NoteEditorColumnInit {
    pub ctx: AppContext,
}

#[derive(Debug)]
pub enum NoteEditorColumnInput {
    OpenNote { id: String },
    CreateNote { folder_id: Option<String> },
    UpdateTitle(String),
    UpdateBody(String),
}

#[derive(Debug)]
pub enum NoteEditorColumnOutput {}

#[relm4::component(pub)]
impl SimpleComponent for NoteEditorColumnModel {
    type Init = NoteEditorColumnInit;
    type Input = NoteEditorColumnInput;
    type Output = NoteEditorColumnOutput;
    type Widgets = NoteEditorColumnWidgets;

    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_hexpand: true,

            #[name = "content_header"]
            adw::HeaderBar {
                #[name = "back_button"]
                pack_start = &gtk::Button {
                    set_icon_name: "go-previous-symbolic",
                    // connect_clicked[leaflet] => move |_| {
                    //     leaflet.navigate(adw::NavigationDirection::Back);
                    // }
                },

                #[wrap(Some)]
                set_title_widget = &adw::WindowTitle {
                    set_title: "Content",
                },
            },

            gtk::Box {
                set_orientation: gtk::Orientation::Vertical,
                add_css_class: "content-view",
                #[watch]
                set_visible: model.current_note.is_some(),

                gtk::TextView {
                    set_height_request: 30,
                    set_margin_all: 15,
                    set_hexpand: true,
                    set_input_hints: gtk::InputHints::NO_SPELLCHECK,
                    #[wrap(Some)]
                    set_buffer: title_buf = &gtk::TextBuffer {
                        #[track = "model.changed(NoteEditorColumnModel::current_note())"]
                        set_text: model.current_note.as_ref().map(|n| n.get_title()).unwrap_or_default(),
                        connect_end_user_action[sender] => move |buf| {
                            let (start, end) = buf.bounds();
                            sender.input(NoteEditorColumnInput::UpdateTitle(buf.text(&start, &end, true).to_string()));
                        }
                    },
                    add_css_class: "title-1",
                },

                gtk::Separator {

                },

                gtk::ScrolledWindow {
                    set_vexpand: true,

                    sourceview5::View {
                        set_vexpand: true,
                        set_editable: true,
                        set_monospace: true,
                        set_margin_top: 10,
                        set_margin_bottom: 10,
                        set_margin_start: 15,
                        set_margin_end: 15,
                        set_wrap_mode: gtk::WrapMode::Word,
                        set_tab_width: 4,
                        set_auto_indent: true,
                        set_insert_spaces_instead_of_tabs: true,
                        // set_highlight_current_line: true,
                        #[wrap(Some)]
                        set_buffer: body_buf = &sourceview5::Buffer {
                            set_language: LanguageManager::new().language("markdown").as_ref(),
                            // ["Adwaita", "Adwaita-dark", "classic", "classic-dark", "cobalt", "cobalt-light", "kate", "kate-dark", "oblivion", "solarized-dark", "solarized-light", "tango"]
                            set_style_scheme: StyleSchemeManager::new().scheme("classic").as_ref(),
                            #[track = "model.changed(NoteEditorColumnModel::current_note())"]
                            set_text: model.current_note.as_ref().map(|n| n.body.as_ref()).unwrap_or_default(),
                            connect_end_user_action[sender] => move |x| {
                                let (start, end) = x.bounds();
                                let text = x.text(&start, &end, true).to_string();
                                sender.input(NoteEditorColumnInput::UpdateBody(text));
                            }
                        }
                    },
                },
            }
        }
    }

    fn init(
        init: Self::Init,
        root: &Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = NoteEditorColumnModel {
            ctx: init.ctx,
            current_note: None,
            tracker: 0,
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, input: Self::Input, _sender: ComponentSender<Self>) {
        match input {
            NoteEditorColumnInput::OpenNote { id } => {
                self.reset();
                let note = self.ctx.data.db.load_note(&id).unwrap();
                self.set_current_note(Some(note));
            }
            NoteEditorColumnInput::CreateNote { folder_id } => {
                self.reset();
                self.set_current_note(Some(Note::new(folder_id, String::new(), String::new())));
            }
            NoteEditorColumnInput::UpdateTitle(title) => {
                if self.changed(NoteEditorColumnModel::current_note()) {
                    self.reset();
                    return;
                }
                if let Some(note) = self.current_note.as_mut() {
                    if note.get_title() != title {
                        note.set_title(&title);
                        self.ctx
                            .data
                            .db
                            .replace_note(note, UpdateSource::LocalEdit)
                            .unwrap();
                    }
                }
            }
            NoteEditorColumnInput::UpdateBody(body) => {
                if self.changed(NoteEditorColumnModel::current_note()) {
                    self.reset();
                    return;
                }
                if let Some(note) = self.current_note.as_mut() {
                    if note.body != body {
                        note.body = body;
                        self.ctx
                            .data
                            .db
                            .replace_note(note, UpdateSource::LocalEdit)
                            .unwrap();
                    }
                }
            }
        }
    }
}
