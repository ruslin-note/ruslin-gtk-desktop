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
use sidebar_column::SidebarColumnModel;

use crate::{content_page::sidebar_column::SidebarColumnOutput, properties};

use self::note_list_column::NoteListColumInput;

pub struct ContentPageModel {
    note_list_column: Controller<NoteListColumnModel>,
    note_editor_column: Controller<NoteEditorColumnModel>,
    sidebar_column: Controller<SidebarColumnModel>,
}

#[derive(Debug)]
pub enum ContentPageInput {
    OpenFolder(Vec<String>),
}

#[relm4::component(pub)]
impl SimpleComponent for ContentPageModel {
    type Init = ();
    type Input = ContentPageInput;
    type Output = ();
    type Widgets = ComponentWidgets;

    view! {
        adw::Flap {
            set_flap_position: gtk::PackType::Start,
            set_fold_threshold_policy: adw::FoldThresholdPolicy::Natural,
            set_swipe_to_open: true,
            set_swipe_to_close: true,

            set_flap: Some(model.sidebar_column.widget()),

            #[wrap(Some)]
            set_content = &gtk::Box {
                #[name = "leaflet"]
                adw::Leaflet {
                    set_can_navigate_back: true,
                    set_fold_threshold_policy: adw::FoldThresholdPolicy::Minimum,

                    append: model.note_list_column.widget(),
                    append: model.note_editor_column.widget(),
                }
            },
        }
    }

    fn init(
        _init: Self::Init,
        root: &Self::Root,
        sender: ComponentSender<Self>,
    ) -> ComponentParts<Self> {
        let note_editor_column = note_editor_column::NoteEditorColumnModel::builder()
            .launch(())
            .detach();

        let note_list_column = note_list_column::NoteListColumnModel::builder()
            .launch(())
            .forward(note_editor_column.sender(), |msg| match msg {
                // Is it ok to forward a subcomponent directly to another subcomponent?
                note_list_column::NoteListColumnOutput::SelectNote(note_id) => {
                    note_editor_column::NoteEditorColumnInput::OpenNote(note_id)
                }
            });

        let sidebar_column = sidebar_column::SidebarColumnModel::builder()
            .launch(())
            .forward(sender.input_sender(), |msg| match msg {
                SidebarColumnOutput::OpenFolder(notes) => ContentPageInput::OpenFolder(notes),
            });
        let model = ContentPageModel {
            note_editor_column,
            note_list_column,
            sidebar_column,
        };

        let widgets = view_output!();

        let leaflet: &adw::Leaflet = &widgets.leaflet;

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

        ComponentParts { model, widgets }
    }

    fn update(&mut self, input: Self::Input, _sender: ComponentSender<Self>) {
        match input {
            ContentPageInput::OpenFolder(notes) => {
                self.note_list_column
                    .sender()
                    .send(NoteListColumInput::RefreshNotes(notes))
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
