use bevy::prelude::*;

const SCOREBOARD_FONT_SIZE: f32 = 24.0;
const SCOREBOARD_TEXT_PADDING: Val = Val::Px(36.0);
const TEXT_COLOR: Color = Color::rgb(1.0, 1.0, 1.0);

pub fn spawn_scoreboard(commands: &mut Commands, font: Handle<Font>) {
    commands.spawn((
        Scoreboard,
        TextBundle::from_sections([
            TextSection::new(
                "Score: ",
                TextStyle {
                    font_size: SCOREBOARD_FONT_SIZE,
                    font: font.clone(),
                    color: TEXT_COLOR,
                    ..default()
                },
            ),
            TextSection::from_style(TextStyle {
                font_size: SCOREBOARD_FONT_SIZE,
                font,
                color: TEXT_COLOR,
                ..default()
            }),
        ])
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: SCOREBOARD_TEXT_PADDING,
            left: SCOREBOARD_TEXT_PADDING,
            ..default()
        }),
    ));
}

/// marker component for the scoreboard
#[derive(Component, Default)]
pub struct Scoreboard;

#[derive(Resource)]
pub struct Score(pub u32);

pub fn update_sys(score: Res<Score>, mut query: Query<(With<Scoreboard>, &mut Text)>) {
    let mut text = query.single_mut().1;
    text.sections[1].value = score.0.to_string();
}
