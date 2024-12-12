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
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            align_items: AlignItems::End,
            justify_content: JustifyContent::Center,
            ..default()
        }).with_children(|parent| {
            parent.spawn((
		    Button,
		    Node {
			width: Val::Percent(100.0),
                        height: Val::Px(100.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
		    },
                    BorderColor(Color::NONE),
                    BackgroundColor(Color::NONE),
                )).with_children(|parent| {
                    parent.spawn((
			Text::new("New Game (R)"),
			TextFont {
			    font: font.0.clone_weak(),
                            font_size: 64.0,
			    ..default()
			},
			TextColor(Color::srgb(0.75, 0.75, 0.75)),
                    ));
                });
	    parent.spawn((
		Text::new("Score: "),
		TextFont {
		    font: title_font.0.clone_weak(),
                    font_size: 32.0,
		    ..default()
		},
		TextColor(Color::srgb(0.6, 0.6, 0.6)),
		Node {
		    position_type: PositionType::Absolute,
		    left: Val::Px(10.0),
		    top: Val::Px(10.0),
                    ..default()
		}
	    )).with_child((
		TextSpan::new("0"),
		TextColor(Color::srgb(0.6, 0.6, 0.6)),
		TextFont {
		    font: title_font.0.clone_weak(),
                    font_size: 32.0,
		    ..default()
		},
		ScoreMarker,
	    ));
	    parent.spawn((
		Text::new("High: "),
		TextFont {
		    font: title_font.0.clone_weak(),
                    font_size: 32.0,
		    ..default()
		},
		TextColor(Color::srgb(0.6, 0.6, 0.6)),
		Node {
		    position_type: PositionType::Absolute,
		    left: Val::Px(10.0),
		    top: Val::Px(60.0),
                    ..default()
		}
	    )).with_child((
		TextSpan::new("0"),
		TextColor(Color::srgb(0.6, 0.6, 0.6)),
		TextFont {
		    font: title_font.0.clone_weak(),
                    font_size: 32.0,
		    ..default()
		},
		HighScoreMarker,
	    ));
        });
    commands.spawn((
	Text::new("2048"),
	Node {
	    position_type: PositionType::Absolute,
	    justify_self: JustifySelf::Center,
            top: Val::Px(90.0),
            ..default()
	},
	TextFont {
            font: title_font.0.clone_weak(),
            font_size: 80.0,
	    ..default()
	},
	TextColor(Color::srgb(0.5, 0.1, 0.4)),
    ));
}
fn new_game_system(
    mut interaction_query: Query<(&Interaction, &Children), (Changed<Interaction>, With<Button>)>,
    mut text_query: Query<&mut TextColor>,
    mut new_game_event: EventWriter<NewGameEvent>,
) {
    for (interaction, children) in &mut interaction_query {
        let mut text_color = text_query.get_mut(children[0]).unwrap();
        match *interaction {
            Interaction::Pressed => {
                new_game_event.send(NewGameEvent);
            }
            Interaction::Hovered => {
                text_color.0 = Color::srgb(0.3, 0.3, 0.3);
            }
            Interaction::None => {
                text_color.0 = Color::srgb(0.75, 0.75, 0.75);
            }
        }
    }
}

fn create_game_over(mut commands: Commands, font: Res<PieceFont>) {
    commands.spawn((
	Text::new("GAME OVER"),
	TextFont {
	    font: font.0.clone_weak(),
            font_size: 200.0,
	    ..default()
	},
	TextColor(Color::srgb(0.1, 0.1, 0.1)),
	Node {
	    align_self: AlignSelf::Center,
            justify_self: JustifySelf::Center,
            ..default()
	},
        GameOverMarker,
    ));
}
fn remove_game_over(mut commands: Commands, query: Query<Entity, With<GameOverMarker>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

fn update_score_ui(
    score: Res<Score>,
    high_score: Res<HighScore>,
    mut score_query: Query<&mut TextSpan, (With<ScoreMarker>, Without<HighScoreMarker>)>,
    mut high_query: Query<&mut TextSpan, With<HighScoreMarker>>,
) {
    if let Ok(mut text) = score_query.get_single_mut() {
        text.0 = score.0.to_string();
    }
    if let Ok(mut text) = high_query.get_single_mut() {
        text.0 = high_score.0.to_string();
    }
}

#[derive(Component)]
struct GameOverMarker;
#[derive(Component)]
struct ScoreMarker;
#[derive(Component)]
struct HighScoreMarker;
