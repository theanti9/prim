use bevy_ecs::{
    prelude::Component,
    query::With,
    schedule::{ShouldRun, SystemSet},
    system::{Commands, Query, Res, ResMut},
};
use glam::Vec4;
use libprim::{
    initialization::InitializeCommand,
    input::{Keyboard, Mouse},
    run,
    state::RenderState,
    text::{InitializeFont, TextSection},
};
use wgpu_text::section::{HorizontalAlign, Layout, OwnedText, Section, Text, VerticalAlign};

pub struct Spawned;

pub struct HasRunMarker<T>(bool, T)
where
    T: Send + Sync + 'static;

fn run_only_once<T>(mut marker: ResMut<HasRunMarker<T>>) -> ShouldRun
where
    T: Send + Sync + 'static,
{
    if !marker.0 {
        marker.0 = true;
        return ShouldRun::Yes;
    }
    ShouldRun::No
}

#[derive(Component)]
pub struct InputDisplay;

pub fn input_display(
    mut query: Query<&mut TextSection, With<InputDisplay>>,
    keyboard: Res<Keyboard>,
    mouse: Res<Mouse>,
) {
    if let Ok(mut text_section) = query.get_single_mut() {
        if let Some(key) = keyboard.currently_pressed().iter().next() {
            text_section.section.text[0] = OwnedText::default()
                .with_text(format!("{:?}", key))
                .with_color(Vec4::ONE)
                .with_scale(64.0);
            return;
        }

        if let Some(button) = mouse.currently_pressed().iter().next() {
            text_section.section.text[0] = OwnedText::default()
                .with_text(format!("{:?}", button))
                .with_color(Vec4::ONE)
                .with_scale(64.0);
            return;
        }

        if !text_section.section.text[0].text.is_empty() {
            text_section.section.text[0] = OwnedText::default()
                .with_text("")
                .with_color(Vec4::ONE)
                .with_scale(64.0);
        }
    }
}

fn spawn_world(mut commands: Commands, render_state: Res<RenderState>) {
    commands.spawn().insert(InputDisplay).insert(TextSection {
        font_id: 0,
        section: Section::default()
            .with_text(vec![Text::default()
                .with_text("")
                .with_color(Vec4::new(1.0, 1.0, 1.0, 1.0))
                .with_scale(64.0)])
            .with_screen_position((
                render_state.config.width as f32 / 2.0,
                render_state.config.height as f32 / 2.0,
            ))
            .with_layout(
                Layout::default_single_line()
                    .h_align(HorizontalAlign::Center)
                    .v_align(VerticalAlign::Center),
            )
            .to_owned(),
    });
}

pub fn show_input() {
    run(|state| {
        state.add_initializer(InitializeCommand::InitializeFont(InitializeFont::new(
            "RobotoMono".to_string(),
            include_bytes!("../assets/fonts/RobotoMono-Regular.ttf"),
        )));
        {
            let world = state.borrow_world();
            world.insert_resource(HasRunMarker(false, Spawned));
        }
        let schedule = state.borrow_schedule();
        schedule.add_system_set_to_stage(
            "pre_update",
            SystemSet::new()
                .with_run_criteria(run_only_once::<Spawned>)
                .with_system(spawn_world),
        );

        schedule.add_system_to_stage("update", input_display);
    });
}

fn main() {
    show_input();
}
