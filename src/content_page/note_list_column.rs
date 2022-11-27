use adw::prelude::*;
use relm4::{
    factory::FactoryVecDeque, gtk, prelude::*, ComponentParts, ComponentSender, SimpleComponent,
};
use ruslin_data::{AbbrNote, FolderID, NoteID};

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
        root = gtk::Label {
            #[watch]
            set_label: &self.abbr_note.title,
        }
    }

    fn init_model(init: Self::Init, _index: &DynamicIndex, _sender: FactorySender<Self>) -> Self {
        Self { abbr_note: init }
    }
}

pub struct NoteListColumnModel {
    ctx: AppContext,
    notes: FactoryVecDeque<NoteItemModel>,
    folder_id: Option<FolderID>,
}

pub struct NoteListColumInit {
    pub ctx: AppContext,
}

#[derive(Debug)]
pub enum NoteListColumnInput {
    RefreshNotes(Option<FolderID>),
    SelectNote(usize),
    CreateNote,
}

#[derive(Debug)]
pub enum NoteListColumnOutput {
    SelectNote(NoteID),
    CreateNote(Option<FolderID>),
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

            #[name = "sidebar_header"]
            adw::HeaderBar {
                #[wrap(Some)]
                set_title_widget = &adw::WindowTitle {
                },

                pack_start = &gtk::Button {
                    set_icon_name: icons::view_sidebar_start_symbolic(),
                    // connect_clicked[leaflet] => move |_| {
                    //     leaflet.navigate(adw::NavigationDirection::Back);
                    // }
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
            NoteListColumnInput::RefreshNotes(folder_id) => {
                self.reload_notes(folder_id);
            }
            NoteListColumnInput::SelectNote(index) => {
                sender
                    .output(NoteListColumnOutput::SelectNote(
                        self.notes.get(index).unwrap().abbr_note.id.clone(),
                    ))
                    .unwrap();
            }
            NoteListColumnInput::CreateNote => {
                sender
                    .output(NoteListColumnOutput::CreateNote(self.folder_id.clone()))
                    .unwrap();
            }
        }
    }
}

impl NoteListColumnModel {
    fn reload_notes(&mut self, folder_id: Option<FolderID>) {
        let mut notes_guard = self.notes.guard();
        notes_guard.clear();
        for note in self
            .ctx
            .data
            .db
            .load_abbr_notes(folder_id.as_ref())
            .unwrap()
        {
            notes_guard.push_back(note);
        }
        self.folder_id = folder_id;
    }
}
