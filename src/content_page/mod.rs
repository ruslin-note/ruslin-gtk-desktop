pub mod note_editor_column;
pub mod note_list_column;
pub mod sidebar_column;

use adw::prelude::*;
use gtk::glib;
use relm4::{
    gtk, Component, ComponentController, ComponentParts, ComponentSender, Controller,
    SimpleComponent,
};

use note_editor_column::NoteEditorColumnModel;
use note_list_column::NoteListColumnModel;
use ruslin_data::FolderID;
use sidebar_column::SidebarColumnModel;

use crate::{
    app::AppContext,
    content_page::{
        note_editor_column::{NoteEditorColumnInit, NoteEditorColumnInput},
        note_list_column::{NoteListColumInit, NoteListColumnOutput},
        sidebar_column::{SidebarColumnInit, SidebarColumnOutput},
    },
    properties,
};

use self::note_list_column::NoteListColumnInput;

pub struct ContentPageModel {
    note_list_column: Controller<NoteListColumnModel>,
    note_editor_column: Controller<NoteEditorColumnModel>,
    sidebar_column: Controller<SidebarColumnModel>,
}

pub struct ContentPageInit {
    pub ctx: AppContext,
}

#[derive(Debug)]
pub enum ContentPageInput {
    OpenFolder(Option<FolderID>),
}

#[relm4::component(pub)]
impl SimpleComponent for ContentPageModel {
    type Init = ContentPageInit;
    type Input = ContentPageInput;
    type Output = ();
    type Widgets = ComponentWidgets;

    view! {
        #[name = "flap"]
        adw::Flap {
            set_flap_position: gtk::PackType::Start,
            set_fold_threshold_policy: adw::FoldThresholdPolicy::Natural,
            set_swipe_to_open: true,
            set_swipe_to_close: true,

            set_flap: Some(model.sidebar_column.widget()),

            #[wrap(Some)]
            set_separator = &gtk::Separator {

            },

            #[wrap(Some)]
            set_content = &gtk::Box {
                #[name = "leaflet"]
                adw::Leaflet {
                    set_can_navigate_back: true,
                    set_fold_threshold_policy: adw::FoldThresholdPolicy::Minimum,

                    append: model.note_list_column.widget(),

                    append = &gtk::Separator::new(gtk::Orientation::Horizontal) {

                    } -> {
                        set_navigatable: false,
                    },

                    append: model.note_editor_column.widget(),
                }
            },
        }
    }

    fn init(
        init: Self::Init,
        root: &Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let note_editor_column = note_editor_column::NoteEditorColumnModel::builder()
            .launch(NoteEditorColumnInit {
                ctx: init.ctx.clone(),
            })
            .detach();

        let note_list_column = note_list_column::NoteListColumnModel::builder()
            .launch(NoteListColumInit {
                ctx: init.ctx.clone(),
            })
            .forward(note_editor_column.sender(), |msg| match msg {
                // Is it ok to forward a subcomponent directly to another subcomponent?
                NoteListColumnOutput::SelectNote(note_id) => {
                    NoteEditorColumnInput::OpenNote(note_id)
                }
                NoteListColumnOutput::CreateNote(folder_id) => {
                    NoteEditorColumnInput::CreateNote(folder_id)
                }
            });

        let sidebar_column = sidebar_column::SidebarColumnModel::builder()
            .launch(SidebarColumnInit { ctx: init.ctx })
            .forward(sender.input_sender(), |msg| match msg {
                SidebarColumnOutput::OpenFolder(folder_id) => {
                    ContentPageInput::OpenFolder(folder_id)
                }
            });

        let model = ContentPageModel {
            note_editor_column,
            note_list_column,
            sidebar_column,
        };

        let widgets = view_output!();

        let leaflet: &adw::Leaflet = &widgets.leaflet;
        let flap: &adw::Flap = &widgets.flap;

        leaflet
            .bind_property(
                properties::folded(),
                &model.note_list_column.widgets().sidebar_header,
                properties::show_end_title_buttons(),
            )
            .flags(glib::BindingFlags::SYNC_CREATE)
            .build();
        leaflet
            .bind_property(
                properties::folded(),
                &model.note_editor_column.widgets().content_header,
                properties::show_start_title_buttons(),
            ) // ? https://gnome.pages.gitlab.gnome.org/libadwaita/doc/main/class.HeaderBar.html
            .flags(glib::BindingFlags::SYNC_CREATE)
            .build();
        leaflet
            .bind_property(
                properties::folded(),
                &model.note_editor_column.widgets().back_button,
                properties::visible(),
            )
            .flags(glib::BindingFlags::SYNC_CREATE)
            .build();
        model
            .note_editor_column
            .widgets()
            .back_button
            .connect_clicked(glib::clone!(@strong leaflet => move |_| { // use output?
                leaflet.navigate(adw::NavigationDirection::Back);
            }));
        flap.bind_property(
            properties::folded(),
            &model.note_list_column.widgets().change_siderbar_button,
            "active",
        )
        .flags(glib::BindingFlags::SYNC_CREATE | glib::BindingFlags::INVERT_BOOLEAN)
        .build();
        model
            .note_list_column
            .widgets()
            .change_siderbar_button
            .connect_clicked(glib::clone!(@strong flap => move |_| { // use output?
                match flap.is_folded() {
                    true => flap.set_fold_policy(adw::FlapFoldPolicy::Never),
                    false => flap.set_fold_policy(adw::FlapFoldPolicy::Always),
                }
            }));

        ComponentParts { model, widgets }
    }

    fn update(&mut self, input: Self::Input, _sender: ComponentSender<Self>) {
        match input {
            ContentPageInput::OpenFolder(notes) => {
                self.note_list_column
                    .sender()
                    .send(NoteListColumnInput::RefreshNotes(notes))
                    .unwrap();
            }
        }
    }

    fn pre_view() {
        /*
           https://relm4.org/book/next/component_macro_reference.html?highlight=pre_view#manual-view
           You can also implement your own view logic, which will be added to the view code that the view macro generates.
           Code inside pre_view() will run before the view update, while post_view() will run after.
        */
        widgets.leaflet.navigate(adw::NavigationDirection::Forward);
    }
}
