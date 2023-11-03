use bevy::prelude::*;
use bevy_ui_dsl::{class_helpers::color::BLACK, *};

use crate::GameState;

#[derive(Component, Debug)]
pub enum MainMenuButtonId {
    SinglePlayer,
    Multiplayer,
    Login,
    Settings,
}

pub fn setup_menu(mut commands: Commands, assets: Res<AssetServer>, mut scale: ResMut<UiScale>) {
    scale.scale = 2.0;

    rooti(c_root, &assets, &mut commands, MenuMarker, |p| {
        node((c_half, c_black), p, |p| {
            text_buttoni(
                "Single Player",
                left_btn_c,
                text_styling_c,
                MainMenuButtonId::SinglePlayer,
                p,
            );
        });
        node((c_half, c_blue), p, |p| {
            text("This is the right pane!", text_box, text_styling_c, p);
        });
    });
}

#[derive(Component, Debug)]
pub struct MenuMarker;

pub fn handle_menu_interactions(
    ui_entities: Query<(&MainMenuButtonId, &Interaction), Changed<Interaction>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for (id, inter) in &ui_entities {
        match (id, inter) {
            (MainMenuButtonId::SinglePlayer, Interaction::Pressed) => {
                println!("Single player button pressed!!!");
                next_state.set(GameState::InGame);
            }
            _ => {}
        }
    }
}

pub fn remove_menu(mut commands: Commands, menu_entities: Query<Entity, With<MenuMarker>>) {
    for e in &mut menu_entities.iter() {
        commands.entity(e).despawn_recursive();
    }
}

// ----- Classes (they're really just callback functions that modify bundles / text styles, but it's useful to think of them as .css classes) -----
pub fn c_root(b: &mut NodeBundle) {
    b.style.width = Val::Percent(100.);
    b.style.height = Val::Percent(100.)
}

pub fn c_half(b: &mut NodeBundle) {
    let s = &mut b.style;
    s.width = Val::Percent(50.);
    s.height = Val::Percent(100.);
    s.flex_direction = FlexDirection::Column;
    s.justify_content = JustifyContent::Center;
    s.align_items = AlignItems::Center;
    s.padding = UiRect::all(Val::Px(10.));
}

pub fn c_black(b: &mut NodeBundle) {
    b.background_color = BLACK.into();
}

pub fn c_blue(b: &mut NodeBundle) {
    b.background_color = Color::rgb_u8(125, 164, 212).into();
}

pub fn text_box(_a: &AssetServer, b: &mut TextBundle) {
    b.style.margin = UiRect::all(Val::Px(10.));
}

pub fn left_btn_c(assets: &AssetServer, b: &mut ButtonBundle) {
    let s = &mut b.style;
    s.width = Val::Px(128.);
    s.height = Val::Px(24.);
    s.justify_content = JustifyContent::Center;
    s.align_items = AlignItems::Center;
    b.background_color = Color::rgb_u8(66, 135, 245).into();
    // b.image = assets.load("button.png").into();
}

pub fn text_styling_c(assets: &AssetServer, s: &mut TextStyle) {
    s.font = assets.load("fonts/space_invaders.ttf").into();
    s.font_size = 16.;
    s.color = Color::WHITE.into();
}
