use adw::prelude::*;
use relm4::{
    actions::{RelmAction, RelmActionGroup},
    factory::FactoryVecDeque,
    gtk,
    prelude::*,
    ComponentParts, ComponentSender, SimpleComponent,
};

use crate::icons;

struct FolderItemModel {
    name: String,
}

#[derive(Debug)]
enum FolderItemInput {
    // ChangeName(String),
}

#[derive(Debug)]
enum FolderItemOutput {}

#[relm4::factory]
impl FactoryComponent for FolderItemModel {
    type Init = String;
    type Input = FolderItemInput;
    type Output = FolderItemOutput;
    type CommandOutput = ();
    type Widgets = FolderItemWidgets;
    type ParentInput = SidebarColumnInput;
    type ParentWidget = gtk::ListBox;

    view! {
        root = gtk::Box {
            #[name(label)]
            gtk::Label {
                #[watch]
                set_label: &self.name,
            }
        }
    }

    fn init_model(init: Self::Init, _index: &DynamicIndex, _sender: FactorySender<Self>) -> Self {
        Self { name: init }
    }
}

pub struct SidebarColumnModel {
    note_count: i32,
    folders: FactoryVecDeque<FolderItemModel>,
    folder_notes: Vec<Vec<String>>,
}

#[derive(Debug)]
pub enum SidebarColumnInput {
    InsertFolder,
    SelectFolderIndex(u32),
    SelectAllFolders,
}

#[derive(Debug)]
pub enum SidebarColumnOutput {
    OpenFolder(Vec<String>),
}

relm4::new_action_group!(pub(super) WindowActionGroup, "win");
relm4::new_stateless_action!(PreferencesAction, WindowActionGroup, "preferences");
relm4::new_stateless_action!(pub(super) ShortcutsAction, WindowActionGroup, "show-help-overlay");
relm4::new_stateless_action!(AboutAction, WindowActionGroup, "about");
relm4::new_stateless_action!(AddFolderAction, WindowActionGroup, "add-folder");

#[relm4::component(pub)]
impl SimpleComponent for SidebarColumnModel {
    type Init = ();
    type Input = SidebarColumnInput;
    type Output = SidebarColumnOutput;
    type Widgets = ComponentWidgets;

    menu! {
        primary_menu: {
            section! {
                "_Preferences" => PreferencesAction,
                "_Keyboard" => ShortcutsAction,
                "_About Ruslin" => AboutAction,
            }
        },
        add_menu: {
            section! {
                "Add Folder" => AddFolderAction,
            }
        }
    }

    view! {
        main_view = gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_width_request: 160,

            #[name = "folder_sidebar_header"]
            adw::HeaderBar {
                #[wrap(Some)]
                set_title_widget = &adw::WindowTitle {
                },

                set_show_end_title_buttons: false,

                pack_end = &gtk::MenuButton {
                    set_icon_name: icons::open_menu_symbolic(),
                    set_menu_model: Some(&primary_menu),
                },

                pack_end = &gtk::MenuButton {
                    set_icon_name: icons::list_add_symbolic(),
                    set_menu_model: Some(&add_menu),
                }
            },

            #[name = "all_notes_list_box"]
            gtk::ListBox {
                set_selection_mode: gtk::SelectionMode::Single,
                add_css_class: "navigation-sidebar",

                gtk::Box {
                    gtk::Label {
                        set_label: "All Folders",
                    },
                },

                connect_row_selected[sender, folder_list_box] => move |_, row| {
                    if row.is_some() {
                        folder_list_box.unselect_all();
                        sender.input(SidebarColumnInput::SelectAllFolders);
                    }
                }
            },

            #[local_ref]
            folder_list_box -> gtk::ListBox {
                set_selection_mode: gtk::SelectionMode::Single,
                add_css_class: "navigation-sidebar",

                connect_row_selected[sender, all_notes_list_box] => move |_, row| {
                    if let Some(row) = row {
                        all_notes_list_box.unselect_all();
                        sender.input(SidebarColumnInput::SelectFolderIndex(row.index() as u32));
                    }
                }
            }
        }
    }

    fn init(
        _init: Self::Init,
        root: &Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let folders = FactoryVecDeque::new(gtk::ListBox::default(), sender.input_sender());
        let model = SidebarColumnModel {
            note_count: 0,
            folders,
            folder_notes: Vec::new(),
        };

        let folder_list_box = model.folders.widget();
        let widgets = view_output!();

        let add_group = RelmActionGroup::<WindowActionGroup>::new();
        let add_folder_action: RelmAction<AddFolderAction> = RelmAction::new_stateless(move |_| {
            sender.input(SidebarColumnInput::InsertFolder);
        });
        add_group.add_action(&add_folder_action);
        let add_actions = add_group.into_action_group();
        widgets
            .main_view
            .insert_action_group("win", Some(&add_actions));

        ComponentParts { model, widgets }
    }

    fn update(&mut self, input: Self::Input, sender: ComponentSender<Self>) {
        match input {
            SidebarColumnInput::InsertFolder => {
                self.note_count += 1;
                self.folder_notes.push(vec![
                    format!("[{}] Note 1", self.note_count),
                    format!("[{}] Note 2", self.note_count),
                    format!("[{}] Note 3", self.note_count),
                ]);
                self.folders.guard().push_back(self.note_count.to_string());
            }
            SidebarColumnInput::SelectAllFolders => {
                let mut folder_notes: Vec<String> = Vec::new();
                for notes in &self.folder_notes {
                    folder_notes.extend_from_slice(notes);
                }
                sender
                    .output(SidebarColumnOutput::OpenFolder(folder_notes))
                    .unwrap();
            }
            SidebarColumnInput::SelectFolderIndex(index) => {
                sender
                    .output(SidebarColumnOutput::OpenFolder(
                        self.folder_notes[index as usize].clone(),
                    ))
                    .unwrap();
            }
        }
    }
}
