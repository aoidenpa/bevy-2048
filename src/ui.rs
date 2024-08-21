use bevy::prelude::*;

use crate::{AppState, HighScore, InitSet, NewGameEvent, PieceFont, Score, TitleFont};

pub struct GameUiPlugin;

impl Plugin for GameUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, create_ui.after(InitSet))
            .add_systems(Update, new_game_system)
            .add_systems(OnEnter(AppState::GameOver), create_game_over)
            .add_systems(OnExit(AppState::GameOver), remove_game_over)
            .add_systems(Update, update_score_ui.run_if(resource_changed::<Score>));
    }
}

fn create_ui(mut commands: Commands, font: Res<PieceFont>, title_font: Res<TitleFont>) {
    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::End,
                justify_content: JustifyContent::Center,
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn(ButtonBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        height: Val::Px(65.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    border_color: BorderColor(Color::NONE),
                    background_color: Color::NONE.into(),
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        "New Game (R)",
                        TextStyle {
                            font: font.0.clone_weak(),
                            font_size: 50.0,
                            color: Color::srgb(0.75, 0.75, 0.75),
                        },
                    ));
                });
        });
    //score
    let score_entity = commands
        .spawn(TextBundle {
            text: Text::from_sections([
                TextSection::new(
                    "Score: ",
                    TextStyle {
                        font: font.0.clone_weak(),
                        font_size: 50.0,
                        color: Color::srgb(0.6, 0.6, 0.6),
                    },
                ),
                TextSection::new(
                    "0",
                    TextStyle {
                        font: font.0.clone_weak(),
                        font_size: 50.0,
                        color: Color::srgb(0.3, 0.3, 0.3),
                    },
                ),
            ]),
            style: Style {
                position_type: PositionType::Absolute,
                right: Val::Px(40.0),
                bottom: Val::Percent(5.0),
                ..default()
            },
            ..default()
        })
        .id();
    let high_score_entity = commands
        .spawn(TextBundle {
            text: Text::from_sections([
                TextSection::new(
                    "High: ",
                    TextStyle {
                        font: font.0.clone_weak(),
                        font_size: 50.0,
                        color: Color::srgb(0.6, 0.6, 0.6),
                    },
                ),
                TextSection::new(
                    "0",
                    TextStyle {
                        font: font.0.clone_weak(),
                        font_size: 50.0,
                        color: Color::srgb(0.3, 0.3, 0.3),
                    },
                ),
            ]),
            style: Style {
                position_type: PositionType::Absolute,
                right: Val::Px(40.0),
                bottom: Val::Percent(2.0),
                /* left: Val::Percent(75.0),
                top: Val::Percent(96.0), */
                ..default()
            },
            ..default()
        })
        .id();
    commands.insert_resource(ScoreUi {
        cur: score_entity,
        high: high_score_entity,
    });
    commands.spawn(TextBundle {
        style: Style {
            justify_self: JustifySelf::Center,
            top: Val::Percent(2.0),
            ..default()
        },
        text: Text::from_section(
            "2048",
            TextStyle {
                font: title_font.0.clone_weak(),
                font_size: 120.0,
                color: Color::srgb(0.5, 0.1, 0.4),
            },
        ),
        ..default()
    });
}
fn new_game_system(
    mut interaction_query: Query<(&Interaction, &Children), (Changed<Interaction>, With<Button>)>,
    mut text_query: Query<&mut Text>,
    mut new_game_event: EventWriter<NewGameEvent>,
) {
    for (interaction, children) in &mut interaction_query {
        let mut text = text_query.get_mut(children[0]).unwrap();
        match *interaction {
            Interaction::Pressed => {
                new_game_event.send(NewGameEvent);
            }
            Interaction::Hovered => {
                text.sections[0].style.color = Color::srgb(0.3, 0.3, 0.3);
            }
            Interaction::None => {
                text.sections[0].style.color = Color::srgb(0.75, 0.75, 0.75);
            }
        }
    }
}

fn create_game_over(mut commands: Commands, font: Res<PieceFont>) {
    commands.spawn((
        TextBundle {
            style: Style {
                align_self: AlignSelf::Center,
                justify_self: JustifySelf::Center,
                ..default()
            },
            text: Text::from_section(
                "GAME OVER",
                TextStyle {
                    font: font.0.clone_weak(),
                    font_size: 200.0,
                    color: Color::srgb(0.1, 0.1, 0.1),
                },
            ),
            ..default()
        },
        GameOverUi,
    ));
}
fn remove_game_over(mut commands: Commands, query: Query<Entity, With<GameOverUi>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn update_score_ui(
    score: Res<Score>,
    score_ui: Res<ScoreUi>,
    high_score: Res<HighScore>,
    mut query: Query<&mut Text>,
) {
    if let Ok(mut text) = query.get_mut(score_ui.cur) {
        text.sections[1].value = score.0.to_string();
    }
    if let Ok(mut text) = query.get_mut(score_ui.high) {
        text.sections[1].value = high_score.0.to_string();
    }
}

#[derive(Resource)]
struct ScoreUi {
    cur: Entity,
    high: Entity,
}

#[derive(Component)]
struct GameOverUi;
