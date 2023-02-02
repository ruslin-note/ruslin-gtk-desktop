use std::convert::identity;

use adw::prelude::*;
use relm4::{
    actions::{RelmAction, RelmActionGroup},
    factory::FactoryVecDeque,
    gtk,
    prelude::*,
    ComponentParts, ComponentSender, Worker, WorkerController,
};
use ruslin_data::{
    sync::{SyncInfo, SyncResult},
    Folder, UpdateSource,
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

struct FoldersHandler {
    ctx: AppContext,
}

impl FoldersHandler {
    fn load_folders(&self) -> Vec<Folder> {
        self.ctx.data.db.load_folders().unwrap()
    }
}

#[derive(Debug)]
enum FoldersHandlerInput {
    InsertFolder { title: String },
    ReloadFolers,
}

impl Worker for FoldersHandler {
    type Init = AppContext;
    type Input = FoldersHandlerInput;
    type Output = SidebarColumnInput;

    fn init(init: Self::Init, sender: ComponentSender<Self>) -> Self {
        sender.input(FoldersHandlerInput::ReloadFolers);
        Self { ctx: init }
    }

    fn update(&mut self, message: Self::Input, sender: ComponentSender<Self>) {
        match message {
            FoldersHandlerInput::InsertFolder { title } => {
                let folder = Folder::new(title, None);
                self.ctx
                    .data
                    .db
                    .replace_folder(&folder, UpdateSource::LocalEdit)
                    .unwrap();
                let folders = self.load_folders();
                sender
                    .output(SidebarColumnInput::ReloadFolers(folders))
                    .unwrap();
            }
            FoldersHandlerInput::ReloadFolers => {
                let folders = self.load_folders();
                sender
                    .output(SidebarColumnInput::ReloadFolers(folders))
                    .unwrap();
            }
        }
    }
}

pub struct SidebarColumnModel {
    folders: FactoryVecDeque<FolderItemModel>,
    add_note_dialog: Controller<EntryDialogModel>,
    worker: WorkerController<FoldersHandler>,
    ctx: AppContext,
}

pub struct SidebarColumnInit {
    pub ctx: AppContext,
}

#[derive(Debug)]
pub enum SidebarColumnInput {
    SelectFolderIndex(u32),
    SelectAllFolders,
    ShowAddNoteDialog,
    ReloadFolers(Vec<Folder>),
    SyncRemote,
}

#[derive(Debug)]
pub enum SidebarColumnCommand {
    SyncRemote(SyncResult<SyncInfo>),
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
        let worker = FoldersHandler::builder()
            .detach_worker(init.ctx.clone())
            .forward(sender.input_sender(), identity);
        let folders = FactoryVecDeque::new(gtk::ListBox::default(), sender.input_sender());
        let add_note_dialog = EntryDialogModel::builder()
            .transient_for(root)
            .launch(EntryDialogInit {
                title: "Add Note".to_string(),
                button_label: "Add".to_string(),
            })
            .forward(worker.sender(), |msg| match msg {
                EntryDialogOutput::Text(title) => FoldersHandlerInput::InsertFolder { title },
            });
        let model = SidebarColumnModel {
            folders,
            add_note_dialog,
            worker,
            ctx: init.ctx,
        };

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

    fn update(&mut self, input: Self::Input, sender: ComponentSender<Self>, _root: &Self::Root) {
        match input {
            SidebarColumnInput::ReloadFolers(folders) => {
                let mut folders_guard = self.folders.guard();
                folders_guard.clear();
                for folder in folders.into_iter() {
                    folders_guard.push_back(folder);
                }
            }
            SidebarColumnInput::SelectAllFolders => {
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
            SidebarColumnInput::ShowAddNoteDialog => {
                self.add_note_dialog.emit(EntryDialogInput::Show);
            }
            SidebarColumnInput::SyncRemote => {
                let data = self.ctx.data.clone();
                sender.oneshot_command(async move {
                    SidebarColumnCommand::SyncRemote(data.synchronize(false).await)
                });
            }
        }
    }

    fn update_cmd(
        &mut self,
        message: Self::CommandOutput,
        _sender: ComponentSender<Self>,
        _root: &Self::Root,
    ) {
        match message {
            SidebarColumnCommand::SyncRemote(result) => match result {
                Ok(_) => {
                    log::info!("sync success!");
                    self.worker.emit(FoldersHandlerInput::ReloadFolers);
                }
                Err(e) => {
                    log::error!("sync fail: {:?}", e);
                }
            },
        }
    }
}
