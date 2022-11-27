use adw::prelude::*;
use relm4::{
    factory::FactoryVecDeque, gtk, prelude::*, ComponentParts, ComponentSender, SimpleComponent,
};
use ruslin_data::{AbbrNote, FolderID};

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
    type ParentInput = NoteListColumInput;
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
}

pub struct NoteListColumInit {
    pub ctx: AppContext,
}

#[derive(Debug)]
pub enum NoteListColumInput {
    RefreshNotes(Option<FolderID>),
}

#[derive(Debug)]
pub enum NoteListColumnOutput {
    SelectNote(String),
}

#[relm4::component(pub)]
impl SimpleComponent for NoteListColumnModel {
    type Init = NoteListColumInit;
    type Input = NoteListColumInput;
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
            },

            #[local_ref]
            note_list_box -> gtk::ListBox {
                set_selection_mode: gtk::SelectionMode::Single,

                connect_row_selected[sender] => move |_, row| {
                    if let Some(row) = row {
                        sender.output(NoteListColumnOutput::SelectNote(format!("{}", row.index() + 1))).unwrap();
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
        };

        let note_list_box = model.notes.widget();

        let widgets = view_output!();

        ComponentParts { model, widgets }
    }

    fn update(&mut self, input: Self::Input, _sender: ComponentSender<Self>) {
        match input {
            NoteListColumInput::RefreshNotes(folder_id) => {
                self.reload_notes(folder_id);
            }
        }
    }
}

impl NoteListColumnModel {
    fn reload_notes(&mut self, folder_id: Option<FolderID>) {
        let mut notes_guard = self.notes.guard();
        for note in self
            .ctx
            .data
            .db
            .load_abbr_notes(folder_id.as_ref())
            .unwrap()
        {
            notes_guard.push_back(note);
        }
    }
}
