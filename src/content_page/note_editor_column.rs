use adw::prelude::*;
use relm4::{gtk, prelude::*, ComponentParts, ComponentSender, SimpleComponent};
use sourceview5::{prelude::*, LanguageManager, StyleSchemeManager};

pub struct NoteEditorColumnModel {
    pub current_note_id: Option<String>,
}

#[derive(Debug)]
pub enum NoteEditorColumnInput {
    OpenNote(String),
}

#[derive(Debug)]
pub enum NoteEditorColumnOutput {}

#[relm4::component(pub)]
impl SimpleComponent for NoteEditorColumnModel {
    type Init = ();
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

            // gtk::Label {
            //     set_vexpand: true,

            //     #[watch]
            //     set_text: &format!("Page {}", model.current_note_id.as_deref().unwrap_or_default()),
            // }
            gtk::ScrolledWindow {
                sourceview5::View {
                    set_vexpand: true,
                    set_editable: true,
                    set_monospace: true,
                    set_margin_all: 5,
                    set_wrap_mode: gtk::WrapMode::Word,
                    set_tab_width: 4,
                    set_auto_indent: true,
                    set_insert_spaces_instead_of_tabs: true,
                    set_highlight_current_line: true,
                    #[wrap(Some)]
                    set_buffer: valuebuf = &sourceview5::Buffer {
                        set_language: LanguageManager::new().language("markdown").as_ref(),
                        set_style_scheme: StyleSchemeManager::new().scheme("classic").as_ref(),
                        connect_changed[_sender] => move |x| {
                            let (start, end) = x.bounds();
                            log::debug!("valuebuf changed to {:?}", x.text(&start, &end, true));
                            // let text = x.text(&start, &end, true).to_string();
                            // sender.input(OptPageMsg::UpdateConf(text))
                        }
                    }
                },
            },
        }
    }

    fn init(
        _init: Self::Init,
        root: &Self::Root,
        _sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let model = NoteEditorColumnModel {
            current_note_id: None,
        };

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, input: Self::Input, _sender: ComponentSender<Self>) {
        match input {
            NoteEditorColumnInput::OpenNote(note_id) => self.current_note_id = Some(note_id),
        }
    }
}
