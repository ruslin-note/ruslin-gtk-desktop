use adw::prelude::*;
use relm4::{
    actions::{RelmAction, RelmActionGroup},
    factory::FactoryVecDeque,
    gtk,
    prelude::*,
    ComponentParts, ComponentSender, SimpleComponent,
};
use ruslin_data::{Folder, FolderID};

use crate::{
    components::{EntryDialogInit, EntryDialogInput, EntryDialogModel, EntryDialogOutput},
    icons, AppContext,
};

struct FolderItemModel {
    folder: Folder,
}

#[derive(Debug)]
enum FolderItemInput {
    // ChangeName(String),
}

#[derive(Debug)]
enum FolderItemOutput {}

#[relm4::factory]
impl FactoryComponent for FolderItemModel {
    type Init = Folder;
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
                set_label: &self.folder.title,
            }
        }
    }

    fn init_model(init: Self::Init, _index: &DynamicIndex, _sender: FactorySender<Self>) -> Self {
        Self { folder: init }
    }
}

pub struct SidebarColumnModel {
    ctx: AppContext,
    note_count: i32,
    folders: FactoryVecDeque<FolderItemModel>,
    folder_notes: Vec<Vec<String>>,
    add_note_dialog: Controller<EntryDialogModel>,
}

pub struct SidebarColumnInit {
    pub ctx: AppContext,
}

#[derive(Debug)]
pub enum SidebarColumnInput {
    InsertFolder(String),
    SelectFolderIndex(u32),
    SelectAllFolders,
    ShowAddNoteDialog,
}

#[derive(Debug)]
pub enum SidebarColumnOutput {
    OpenFolder(Option<FolderID>),
}

relm4::new_action_group!(pub(super) WindowActionGroup, "win");
relm4::new_stateless_action!(PreferencesAction, WindowActionGroup, "preferences");
relm4::new_stateless_action!(pub(super) ShortcutsAction, WindowActionGroup, "show-help-overlay");
relm4::new_stateless_action!(AboutAction, WindowActionGroup, "about");
relm4::new_stateless_action!(AddFolderAction, WindowActionGroup, "add-folder");

#[relm4::component(pub)]
impl SimpleComponent for SidebarColumnModel {
    type Init = SidebarColumnInit;
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
        init: Self::Init,
        root: &Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let folders = FactoryVecDeque::new(gtk::ListBox::default(), sender.input_sender());
        let add_note_dialog = EntryDialogModel::builder()
            .transient_for(root)
            .launch(EntryDialogInit {
                title: "Add Note".to_string(),
                button_label: "Add".to_string(),
            })
            .forward(sender.input_sender(), |msg| match msg {
                EntryDialogOutput::Text(s) => SidebarColumnInput::InsertFolder(s),
            });
        let mut model = SidebarColumnModel {
            ctx: init.ctx,
            note_count: 0,
            folders,
            folder_notes: Vec::new(),
            add_note_dialog,
        };
        model.reload_folders();

        let folder_list_box = model.folders.widget();
        let widgets = view_output!();

        let add_group = RelmActionGroup::<WindowActionGroup>::new();
        let add_folder_action: RelmAction<AddFolderAction> = RelmAction::new_stateless(move |_| {
            sender.input(SidebarColumnInput::ShowAddNoteDialog);
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
            SidebarColumnInput::InsertFolder(name) => {
                self.note_count += 1;
                self.folder_notes.push(vec![
                    format!("[{}] Note 1", name),
                    format!("[{}] Note 2", name),
                    format!("[{}] Note 3", name),
                ]);
                let folder = ruslin_data::Folder::new(name, None);
                self.ctx.data.db.replace_folder(&folder).unwrap();
                self.folders.guard().push_back(folder);
            }
            SidebarColumnInput::SelectAllFolders => {
                sender
                    .output(SidebarColumnOutput::OpenFolder(None))
                    .unwrap();
            }
            SidebarColumnInput::SelectFolderIndex(index) => {
                sender
                    .output(SidebarColumnOutput::OpenFolder(Some(
                        self.folders.get(index as usize).unwrap().folder.id.clone(),
                    )))
                    .unwrap();
            }
            SidebarColumnInput::ShowAddNoteDialog => {
                self.add_note_dialog.emit(EntryDialogInput::Show);
            }
        }
    }
}

impl SidebarColumnModel {
    fn reload_folders(&mut self) {
        // TODO: async & result
        let mut folders_guard = self.folders.guard();
        folders_guard.clear();
        for folder in self.ctx.data.db.load_folders().unwrap() {
            folders_guard.push_back(folder);
        }
    }
}
