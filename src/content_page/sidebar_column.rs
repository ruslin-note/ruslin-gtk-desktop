use adw::prelude::*;
use relm4::{
    actions::{RelmAction, RelmActionGroup},
    factory::FactoryVecDeque,
    gtk,
    prelude::*,
    ComponentParts, ComponentSender,
};
use ruslin_data::{
    sync::{SyncError, SyncInfo},
    Folder,
};

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
                add_css_class: "heading",
            }
        }
    }

    fn init_model(init: Self::Init, _index: &DynamicIndex, _sender: FactorySender<Self>) -> Self {
        Self { folder: init }
    }
}

pub struct SidebarColumnModel {
    folders: FactoryVecDeque<FolderItemModel>,
    add_note_dialog: Controller<EntryDialogModel>,
    ctx: AppContext,
}

pub struct SidebarColumnInit {
    pub ctx: AppContext,
}

#[derive(Debug)]
pub enum SidebarColumnInput {
    SelectFolderIndex(u32),
    SelectAllNotes,
    ShowCreateFolderDialog,
    SyncRemote,
    InsertFolder { title: String },
    ReloadFolders,
}

#[derive(Debug)]
pub enum SidebarColumnCommand {
    SyncSuccess(SyncInfo),
    ReloadFolders(Vec<Folder>),
    AddedFolder,
    ToastError(SyncError),
}

#[derive(Debug)]
pub enum SidebarColumnOutput {
    OpenFolder { folder_id: Option<String> },
}

relm4::new_action_group!(pub(super) WindowActionGroup, "win");
relm4::new_stateless_action!(PreferencesAction, WindowActionGroup, "preferences");
relm4::new_stateless_action!(pub(super) ShortcutsAction, WindowActionGroup, "show-help-overlay");
relm4::new_stateless_action!(AboutAction, WindowActionGroup, "about");
relm4::new_stateless_action!(AddFolderAction, WindowActionGroup, "add-folder");

#[relm4::component(pub)]
impl Component for SidebarColumnModel {
    type Init = SidebarColumnInit;
    type Input = SidebarColumnInput;
    type Output = SidebarColumnOutput;
    type Widgets = ComponentWidgets;
    type CommandOutput = SidebarColumnCommand;

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
                pack_start = &gtk::Button {
                    set_icon_name: icons::view_refresh_symbolic(),
                    connect_clicked[sender] => move |_| {
                        sender.input(SidebarColumnInput::SyncRemote);
                    }
                },

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
                        set_label: "All Notes",
                        add_css_class: "heading",
                    },
                },

                connect_row_selected[sender, folder_list_box] => move |_, row| {
                    if row.is_some() {
                        folder_list_box.unselect_all();
                        sender.input(SidebarColumnInput::SelectAllNotes);
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
        let folders: FactoryVecDeque<FolderItemModel> =
            FactoryVecDeque::new(gtk::ListBox::default(), sender.input_sender());
        let add_note_dialog = EntryDialogModel::builder()
            .transient_for(&root)
            .launch(EntryDialogInit {
                title: "Create Folder".to_string(),
                button_label: "Create".to_string(),
            })
            .forward(sender.input_sender(), |msg| match msg {
                EntryDialogOutput::Text(title) => SidebarColumnInput::InsertFolder { title },
            });
        let model = SidebarColumnModel {
            folders,
            add_note_dialog,
            ctx: init.ctx,
        };
        sender.input(SidebarColumnInput::ReloadFolders);

        let folder_list_box = model.folders.widget();
        let widgets = view_output!();

        let add_group = RelmActionGroup::<WindowActionGroup>::new();
        let add_folder_action: RelmAction<AddFolderAction> = RelmAction::new_stateless(move |_| {
            sender.input(SidebarColumnInput::ShowCreateFolderDialog);
        });
        add_group.add_action(&add_folder_action);
        let add_actions = add_group.into_action_group();
        widgets
            .main_view
            .insert_action_group("win", Some(&add_actions));
        ComponentParts { model, widgets }
    }

    fn update(&mut self, input: Self::Input, sender: ComponentSender<Self>, _root: &Self::Root) {
        match input {
            SidebarColumnInput::SelectAllNotes => {
                sender
                    .output(SidebarColumnOutput::OpenFolder { folder_id: None })
                    .unwrap();
            }
            SidebarColumnInput::SelectFolderIndex(index) => {
                sender
                    .output(SidebarColumnOutput::OpenFolder {
                        folder_id: Some(
                            self.folders.get(index as usize).unwrap().folder.id.clone(),
                        ),
                    })
                    .unwrap();
            }
            SidebarColumnInput::ShowCreateFolderDialog => {
                self.add_note_dialog.emit(EntryDialogInput::Show);
            }
            SidebarColumnInput::SyncRemote => {
                let data = self.ctx.data.clone();
                sender.oneshot_command(async move {
                    match data.synchronize(false).await {
                        Ok(info) => SidebarColumnCommand::SyncSuccess(info),
                        Err(e) => SidebarColumnCommand::ToastError(e),
                    }
                });
            }
            SidebarColumnInput::InsertFolder { title } => {
                let data = self.ctx.data.clone();
                sender.spawn_oneshot_command(move || match data.db.insert_root_folder(title) {
                    Ok(_) => SidebarColumnCommand::AddedFolder,
                    Err(e) => SidebarColumnCommand::ToastError(e.into()),
                })
            }
            SidebarColumnInput::ReloadFolders => {
                let data = self.ctx.data.clone();
                sender.spawn_oneshot_command(move || match data.db.load_folders() {
                    Ok(folders) => SidebarColumnCommand::ReloadFolders(folders),
                    Err(e) => SidebarColumnCommand::ToastError(e.into()),
                })
            }
        }
    }

    fn update_cmd(
        &mut self,
        message: Self::CommandOutput,
        sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match message {
            SidebarColumnCommand::SyncSuccess(_) => {
                sender.input(SidebarColumnInput::ReloadFolders);
            }
            SidebarColumnCommand::ReloadFolders(folders) => {
                let mut folders_guard = self.folders.guard();
                folders_guard.clear();
                for folder in folders.into_iter() {
                    folders_guard.push_back(folder);
                }
            }
            SidebarColumnCommand::AddedFolder => {
                sender.input(SidebarColumnInput::ReloadFolders);
            }
            SidebarColumnCommand::ToastError(e) => {
                todo!("{e}")
            }
        }
    }
}
