use bevy::{prelude::*, window::PrimaryWindow};

use super::{AssetHandles, AtlasIndexable, Spawnable};

#[derive(Component, Default)]
pub struct PlayerShip {
    delta_x: f32,
}

impl AtlasIndexable for PlayerShip {
    const SPRITE_INDEX: usize = 13;
}

impl PlayerShip {
    const ACCELERATION: f32 = 70.0; // pixels per second per second
    const MAX_VELOCITY: f32 = 1500.0; // pixels per second

    fn fire_laser(commands: &mut Commands, player_pos: Vec3, atlas_handle: Handle<TextureAtlas>) {
        Laser::spawn(
            Vec3::new(player_pos.x, player_pos.y + 24.0, 0.0),
            atlas_handle,
            commands,
        );
    }

    fn accelerate(&mut self, dt: f32, multiplier: f32) {
        self.delta_x += PlayerShip::ACCELERATION * dt * multiplier;
    }

    fn apply_delta_x(&mut self, pos: &mut Vec3, window_width: f32) {
        self.delta_x = self
            .delta_x
            .clamp(-PlayerShip::MAX_VELOCITY, PlayerShip::MAX_VELOCITY);
        pos.x += self.delta_x; // change position
        pos.x = pos.x.clamp(-window_width / 2.0, window_width / 2.0);
        self.delta_x *= 0.8;
    }

    pub fn kbd_movement_sys(
        keyboard_input: Res<Input<KeyCode>>,
        time: Res<Time>,
        mut player_ships: Query<(&mut PlayerShip, &mut Transform)>,
        windows: Query<&Window, With<PrimaryWindow>>,
    ) {
        let dt = time.delta_seconds();

        let Ok(window) = windows.get_single() else {
            return;
        };

        let Ok((mut player, mut trans)) = player_ships.get_single_mut() else {
            return;
        };
        if keyboard_input.pressed(KeyCode::Left) || keyboard_input.pressed(KeyCode::A) {
            player.accelerate(dt, -1.)
        }

        if keyboard_input.pressed(KeyCode::Right) || keyboard_input.pressed(KeyCode::D) {
            player.accelerate(dt, 1.)
        }

        player.apply_delta_x(&mut trans.translation, window.width());
    }

    pub fn gamepad_movement_sys(
        gamepads: Res<Gamepads>,
        axes: Res<Axis<GamepadAxis>>,
        button_inputs: Res<Input<GamepadButton>>,
        time: Res<Time>,
        mut player_ships: Query<(&mut PlayerShip, &mut Transform)>,
        windows: Query<&Window, With<PrimaryWindow>>,
    ) {
        let dt = time.delta_seconds();

        let Ok(window) = windows.get_single() else {
            return;
        };

        let Ok((mut player, mut trans)) = player_ships.get_single_mut() else {
            return;
        };

        let Some(gamepad) = gamepads.iter().nth(0) else {
            return;
        };

        let dpad_left =
            button_inputs.pressed(GamepadButton::new(gamepad, GamepadButtonType::DPadLeft));
        let dpad_right =
            button_inputs.pressed(GamepadButton::new(gamepad, GamepadButtonType::DPadRight));

        let left_stick_x = axes
            .get(GamepadAxis::new(gamepad, GamepadAxisType::LeftStickX))
            .unwrap();

        if dpad_left {
            player.accelerate(dt, -1.)
        } else if dpad_right {
            player.accelerate(dt, 1.)
        } else if left_stick_x < -0.01 || left_stick_x > 0.01 {
            player.accelerate(dt, left_stick_x)
        }

        player.apply_delta_x(&mut trans.translation, window.width());
    }

    pub fn firing_sys(
        keyboard_input: Res<Input<KeyCode>>,
        button_inputs: Res<Input<GamepadButton>>,
        gamepads: Res<Gamepads>,
        mut commands: Commands,
        player_ships: Query<(&mut Transform, &Handle<TextureAtlas>), With<PlayerShip>>,
        lasers: Query<(), With<Laser>>,
        asset_handles: Res<AssetHandles>,
    ) {
        for (trans, atlas_handle) in player_ships.iter() {
            let gamepad_fired = if let Some(gp) = gamepads.iter().nth(0) {
                let trigger_down = button_inputs
                    .just_pressed(GamepadButton::new(gp, GamepadButtonType::RightTrigger2));

                let button_down =
                    button_inputs.just_pressed(GamepadButton::new(gp, GamepadButtonType::South));

                trigger_down || button_down
            } else {
                false
            };

            let kbd_fired = keyboard_input.just_pressed(KeyCode::Space)
                || keyboard_input.just_pressed(KeyCode::Return);

            if kbd_fired || gamepad_fired {
                if lasers.iter().count() > 0 {
                    return;
                }
                PlayerShip::fire_laser(&mut commands, trans.translation, atlas_handle.clone());
                commands.spawn(AudioBundle {
                    source: asset_handles.shoot_sound.clone(),
                    ..default()
                });
            }
        }
    }
}

#[derive(Component, Default)]
pub struct Laser;

impl AtlasIndexable for Laser {
    const SPRITE_INDEX: usize = 10;
}

impl Laser {
    const VELOCITY: f32 = 480.0; // pixels per second

    /// Update the laser's position according to the frame delta time
    fn update_position(&mut self, dt: f32, pos: &mut Vec3) {
        pos.y += Laser::VELOCITY * dt;
    }

    /// Despawn the laser if it goes off the top of the screen
    fn needs_despawn(&self, pos: &mut Vec3, window_height: f32) -> bool {
        pos.y > window_height / 2.0
    }

    pub fn movement_sys(
        time: Res<Time>,
        mut lasers: Query<(Entity, &mut Laser, &mut Transform)>,
        mut commands: Commands,
        windows: Query<&Window, With<PrimaryWindow>>,
    ) {
        let Ok(window) = windows.get_single() else {
            return;
        };
        let dt = time.delta_seconds();
        for (entity, mut laser, mut trans) in lasers.iter_mut() {
            let pos = &mut trans.translation;
            laser.update_position(dt, pos);
            if laser.needs_despawn(pos, window.height()) {
                commands.entity(entity).despawn();
            }
        }
    }
}
