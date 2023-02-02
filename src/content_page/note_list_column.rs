use adw::prelude::*;
use relm4::{
    factory::FactoryVecDeque, gtk, prelude::*, ComponentParts, ComponentSender, SimpleComponent,
};
use ruslin_data::AbbrNote;

use crate::{icons, AppContext};

struct NoteItemModel {
    abbr_note: AbbrNote,
}

#[derive(Debug)]
enum NoteItemInput {}

#[derive(Debug)]
enum NoteItemOutput {}

#[relm4::factory]
impl FactoryComponent for NoteItemModel {
    type Init = AbbrNote;
    type Input = NoteItemInput;
    type Output = NoteItemOutput;
    type CommandOutput = ();
    type Widgets = NoteItemWidgets;
    type ParentInput = NoteListColumnInput;
    type ParentWidget = gtk::ListBox;

    view! {
        root = gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_spacing: 5,
            gtk::Box {
                set_margin_top: 5,

                gtk::Label {
                    #[watch]
                    set_label: &self.abbr_note.title,
                    add_css_class: "heading",
                },
            },
            gtk::Box {
                set_margin_bottom: 7,

                gtk::Label {
                    #[watch]
                    set_label: &self.abbr_note.user_updated_time.format_ymd_hms(),
                    add_css_class: "caption",
                },
            }
        }
    }

    fn init_model(init: Self::Init, _index: &DynamicIndex, _sender: FactorySender<Self>) -> Self {
        Self { abbr_note: init }
    }
}

pub struct NoteListColumnModel {
    ctx: AppContext,
    notes: FactoryVecDeque<NoteItemModel>,
    folder_id: Option<String>,
}

pub struct NoteListColumInit {
    pub ctx: AppContext,
}

#[derive(Debug)]
pub enum NoteListColumnInput {
    RefreshNotes { folder_id: Option<String> },
    SelectNote(usize),
    CreateNote,
}

#[derive(Debug)]
pub enum NoteListColumnOutput {
    SelectNote { id: String },
    CreateNote { folder_id: Option<String> },
}

#[relm4::component(pub)]
impl SimpleComponent for NoteListColumnModel {
    type Init = NoteListColumInit;
    type Input = NoteListColumnInput;
    type Output = NoteListColumnOutput;
    type Widgets = ComponentWidgets;

    view! {
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_width_request: 220,

            #[name = "sidebar_header"]
            adw::HeaderBar {
                #[wrap(Some)]
                set_title_widget = &adw::WindowTitle {
                },

                #[name = "change_siderbar_button"]
                pack_start = &gtk::ToggleButton {
                    set_icon_name: icons::view_sidebar_start_symbolic(),
                },

                pack_end = &gtk::Button {
                    set_icon_name: icons::list_add_symbolic(),
                    connect_clicked[sender] => move |_| {
                        sender.input(NoteListColumnInput::CreateNote);
                    }
                }
            },

            #[local_ref]
            note_list_box -> gtk::ListBox {
                set_selection_mode: gtk::SelectionMode::Single,
                add_css_class: "navigation-sidebar",

                connect_row_selected[sender] => move |_, row| {
                    if let Some(row) = row {
                        sender.input(NoteListColumnInput::SelectNote(row.index() as usize))
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
        let notes = FactoryVecDeque::new(gtk::ListBox::default(), sender.input_sender());
        let model = NoteListColumnModel {
            ctx: init.ctx,
            notes,
            folder_id: None,
        };

        let note_list_box = model.notes.widget();

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, input: Self::Input, sender: ComponentSender<Self>) {
        match input {
            NoteListColumnInput::RefreshNotes { folder_id } => {
                self.reload_notes(folder_id);
            }
            NoteListColumnInput::SelectNote(index) => {
                sender
                    .output(NoteListColumnOutput::SelectNote {
                        id: self.notes.get(index).unwrap().abbr_note.id.clone(),
                    })
                    .unwrap();
            }
            NoteListColumnInput::CreateNote => {
                sender
                    .output(NoteListColumnOutput::CreateNote {
                        folder_id: self.folder_id.clone(),
                    })
                    .unwrap();
            }
        }
    }
}

impl NoteListColumnModel {
    fn reload_notes(&mut self, folder_id: Option<String>) {
        let mut notes_guard = self.notes.guard();
        notes_guard.clear();
        for note in self
            .ctx
            .data
            .db
            .load_abbr_notes(folder_id.as_deref())
            .unwrap()
        {
            notes_guard.push_back(note);
        }
        self.folder_id = folder_id;
    }
}
