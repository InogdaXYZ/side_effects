use bevy::prelude::*;

use crate::{GameState, Medicine, MedicineEffect, Medicines, MyFonts};
/***************************************/
/*    ##     ## ##     ## ########     */
/*    ##     ## ##     ## ##     ##    */
/*    ##     ## ##     ## ##     ##    */
/*    ######### ##     ## ##     ##    */
/*    ##     ## ##     ## ##     ##    */
/*    ##     ## ##     ## ##     ##    */
/*    ##     ##  #######  ########     */
/***************************************/

#[derive(Component)]
#[allow(clippy::upper_case_acronyms)]
pub struct HUD;

#[derive(Component)]
pub struct ExperimentButton(ExperimentAction);

enum ExperimentAction {
    Conduct,
    Finish,
}
#[derive(Component)]
struct ExperimentButtonCaption;

pub fn setup(mut commands: Commands, fonts: Res<MyFonts>, medicines: Res<Medicines>) {
    let medicine_card = |parent: &mut ChildBuilder, medicine_index: usize, medicine: &Medicine| {
        // Medicine card
        parent
            .spawn(NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Column,
                    gap: Size::all(Val::Px(10.)),
                    padding: UiRect::all(Val::Px(20.)),
                    ..Default::default()
                },
                background_color: Color::SALMON.into(),
                ..Default::default()
            })
            .with_children(|parent| {
                // Title
                parent.spawn(TextBundle::from_section(
                    "Medicine ".to_string() + &medicine.name,
                    TextStyle {
                        font: fonts.fira_sans_regular.clone_weak(),
                        font_size: 40.,
                        color: Color::BLACK,
                    },
                ));

                parent.spawn(TextBundle::from_section(
                    "Effects:",
                    TextStyle {
                        font: fonts.fira_sans_regular.clone_weak(),
                        font_size: 30.,
                        color: Color::BLACK,
                    },
                ));

                // Effects
                for (effect, value) in &[
                    (MedicineEffect::Appetite, medicine.appetite),
                    (MedicineEffect::Smell, medicine.smell),
                    (MedicineEffect::Fear, medicine.fear),
                ] {
                    parent
                        .spawn(NodeBundle {
                            style: Style {
                                flex_direction: FlexDirection::Row,
                                justify_content: JustifyContent::SpaceBetween,
                                gap: Size::all(Val::Px(20.)),
                                ..Default::default()
                            },
                            ..Default::default()
                        })
                        .with_children(|parent| {
                            parent.spawn(TextBundle::from_section(
                                effect.title(),
                                TextStyle {
                                    font: fonts.fira_sans_regular.clone_weak(),
                                    font_size: 20.,
                                    color: Color::BLACK,
                                },
                            ));

                            parent
                                .spawn(NodeBundle {
                                    style: Style {
                                        flex_direction: FlexDirection::Row,
                                        ..Default::default()
                                    },
                                    ..Default::default()
                                })
                                .with_children(|parent| {
                                    for choice in &[-1, 0, 1] {
                                        parent
                                            .spawn((
                                                MedicineEffectButton {
                                                    medicine_index,
                                                    effect: *effect,
                                                    value: *choice,
                                                },
                                                ButtonBundle {
                                                    style: Style {
                                                        size: Size::all(Val::Px(20.)),
                                                        justify_content: JustifyContent::Center,
                                                        ..Default::default()
                                                    },
                                                    background_color: if value == choice {
                                                        // Selected
                                                        Color::YELLOW.into()
                                                    } else {
                                                        Color::GRAY.into()
                                                    },
                                                    ..Default::default()
                                                },
                                            ))
                                            .with_children(|parent| {
                                                parent.spawn(
                                                    TextBundle::from_section(
                                                        match choice {
                                                            0 => "=",
                                                            c if c < &0 => "↓",
                                                            _ => "↑",
                                                        },
                                                        TextStyle {
                                                            font: fonts
                                                                .fira_sans_regular
                                                                .clone_weak(),
                                                            font_size: 20.0,
                                                            color: Color::BLACK,
                                                        },
                                                    ) // Set the alignment of the Text
                                                    .with_text_alignment(TextAlignment::Center),
                                                );
                                            });
                                    }
                                });
                        });
                }

                // Included in experiment
                parent
                    .spawn(NodeBundle {
                        style: Style {
                            flex_direction: FlexDirection::Row,
                            justify_content: JustifyContent::SpaceBetween,
                            gap: Size::all(Val::Px(20.)),
                            ..Default::default()
                        },
                        ..Default::default()
                    })
                    .with_children(|parent| {
                        parent.spawn(TextBundle::from_section(
                            "Include in test",
                            TextStyle {
                                font: fonts.fira_sans_regular.clone_weak(),
                                font_size: 20.,
                                color: Color::BLACK,
                            },
                        ));

                        parent
                            .spawn(NodeBundle {
                                style: Style {
                                    flex_direction: FlexDirection::Row,
                                    ..Default::default()
                                },
                                ..Default::default()
                            })
                            .with_children(|parent| {
                                for value in &[false, true] {
                                    parent
                                        .spawn((
                                            TestMedicineButton(medicine_index, *value),
                                            ButtonBundle {
                                                style: Style {
                                                    justify_content: JustifyContent::Center,
                                                    padding: UiRect::horizontal(Val::Px(4.)),

                                                    ..Default::default()
                                                },

                                                background_color: if value
                                                    == &medicine.in_experiment
                                                {
                                                    // Selected
                                                    Color::YELLOW.into()
                                                } else {
                                                    Color::GRAY.into()
                                                },
                                                ..Default::default()
                                            },
                                        ))
                                        .with_children(|parent| {
                                            parent.spawn(
                                                TextBundle::from_section(
                                                    if *value { "yes" } else { "no" },
                                                    TextStyle {
                                                        font: fonts.fira_sans_regular.clone_weak(),
                                                        font_size: 20.0,
                                                        color: Color::BLACK,
                                                    },
                                                ) // Set the alignment of the Text
                                                .with_text_alignment(TextAlignment::Center),
                                            );
                                        });
                                }
                            });
                    });
            });
    };

    commands
        .spawn((
            HUD,
            NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    position: UiRect {
                        top: Val::Percent(0.),
                        left: Val::Auto,
                        right: Val::Auto,
                        ..Default::default()
                    },
                    gap: Size::all(Val::Px(20.)),
                    padding: UiRect::all(Val::Px(40.)),
                    ..Default::default()
                },
                ..Default::default()
            },
        ))
        .with_children(|parent| {
            /************************************************************************************/
            /*    ##     ## ######## ########  ####  ######  #### ##    ## ########  ######     */
            /*    ###   ### ##       ##     ##  ##  ##    ##  ##  ###   ## ##       ##    ##    */
            /*    #### #### ##       ##     ##  ##  ##        ##  ####  ## ##       ##          */
            /*    ## ### ## ######   ##     ##  ##  ##        ##  ## ## ## ######    ######     */
            /*    ##     ## ##       ##     ##  ##  ##        ##  ##  #### ##             ##    */
            /*    ##     ## ##       ##     ##  ##  ##    ##  ##  ##   ### ##       ##    ##    */
            /*    ##     ## ######## ########  ####  ######  #### ##    ## ########  ######     */
            /************************************************************************************/
            parent
                .spawn(NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::Row,
                        gap: Size::all(Val::Px(20.)),
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .with_children(|parent| {
                    for (medicine_index, medicine) in medicines.0.iter().enumerate() {
                        medicine_card(parent, medicine_index, medicine);
                    }
                });



            /****************************************************************************************************/
            /*    ########  ##          ###    ##    ##          ##     ######  ########  #######  ########     */
            /*    ##     ## ##         ## ##    ##  ##          ##     ##    ##    ##    ##     ## ##     ##    */
            /*    ##     ## ##        ##   ##    ####          ##      ##          ##    ##     ## ##     ##    */
            /*    ########  ##       ##     ##    ##          ##        ######     ##    ##     ## ########     */
            /*    ##        ##       #########    ##         ##              ##    ##    ##     ## ##           */
            /*    ##        ##       ##     ##    ##        ##         ##    ##    ##    ##     ## ##           */
            /*    ##        ######## ##     ##    ##       ##           ######     ##     #######  ##           */
            /****************************************************************************************************/
            parent
                .spawn(NodeBundle {
                    style: Style {
                        flex_direction: FlexDirection::Column,
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .with_children(|parent| {
                    parent
                        .spawn((
                            ExperimentButton(ExperimentAction::Conduct),
                            ButtonBundle {
                                background_color: Color::DARK_GRAY.into(),
                                style: Style{
                                    padding: UiRect::all(Val::Px(10.)),
                                    ..Default::default()
                                },
                                ..Default::default()
                            },
                        ))
                        .with_children(|parent| {
                            parent.spawn((
                                ExperimentButtonCaption,
                                TextBundle::from_section(
                                    "Conduct experiment",
                                    TextStyle {
                                        font: fonts.fira_sans_regular.clone_weak(),
                                        font_size: 30.0,
                                        color: Color::WHITE,
                                    },
                                ) // Set the alignment of the Text
                                .with_text_alignment(TextAlignment::Center),
                            ));
                        });

                    parent
                        .spawn((
                            ExperimentButton(ExperimentAction::Finish),
                            ButtonBundle {
                                background_color: Color::DARK_GRAY.into(),
                                style: Style{
                                    padding: UiRect::all(Val::Px(10.)),
                                    ..Default::default()
                                },
                                visibility: Visibility::Hidden,
                                ..Default::default()
                            },
                        ))
                        .with_children(|parent| {
                            parent.spawn((
                                ExperimentButtonCaption,
                                TextBundle::from_section(
                                    "Finish experiment",
                                    TextStyle {
                                        font: fonts.fira_sans_regular.clone_weak(),
                                        font_size: 30.0,
                                        color: Color::WHITE,
                                    },
                                )
                                .with_text_alignment(TextAlignment::Center),
                            ));
                        });
                });
        });
}

#[derive(Component, Debug, PartialEq, Eq)]
pub struct MedicineEffectButton {
    medicine_index: usize,
    effect: MedicineEffect,
    value: i32,
}

pub fn medicine_property_button(
    interaction_query: Query<(&MedicineEffectButton, &Interaction), Changed<Interaction>>,
    mut medicines: ResMut<Medicines>,
    mut buttons: Query<(&mut BackgroundColor, &MedicineEffectButton)>,
) {
    for (this_button, interaction) in interaction_query.iter() {
        match interaction {
            Interaction::Clicked => {
                medicines.0[this_button.medicine_index]
                    .set_effect(&this_button.effect, this_button.value);

                for (mut background, button) in buttons.iter_mut() {
                    if button == this_button {
                        *background = Color::YELLOW.into()
                    } else if button.medicine_index == this_button.medicine_index
                        && button.effect == this_button.effect
                    {
                        *background = Color::GRAY.into()
                    }
                }
            }
            Interaction::Hovered => {}
            Interaction::None => {}
        }
    }
}

#[derive(Component, Debug, PartialEq, Eq)]
pub struct TestMedicineButton(usize, bool);

pub fn test_medicine_button(
    interaction_query: Query<(&TestMedicineButton, &Interaction), Changed<Interaction>>,
    mut medicines: ResMut<Medicines>,
    mut buttons: Query<(&mut BackgroundColor, &TestMedicineButton)>,
) {
    for (this_button, interaction) in interaction_query.iter() {
        match interaction {
            Interaction::Clicked => {
                medicines.0[this_button.0].in_experiment = this_button.1;

                for (mut background, button) in buttons.iter_mut() {
                    if button == this_button {
                        *background = Color::YELLOW.into()
                    } else if button.0 == this_button.0 {
                        *background = Color::GRAY.into()
                    }
                }
            }
            Interaction::Hovered => {}
            Interaction::None => {}
        }
    }
}

pub fn experiment_button(
    mut interaction_query: Query<(Entity, &ExperimentButton, &Interaction), Changed<Interaction>>,
    mut buttons: Query<(Entity, &mut Visibility), With<ExperimentButton>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for (this, button, interaction) in interaction_query.iter_mut() {
        match interaction {
            Interaction::Clicked => {
                for (entity, mut visibility) in buttons.iter_mut() {
                    *visibility = if entity == this {
                        Visibility::Hidden
                    } else {
                        Visibility::Visible
                    }
                }
                match button.0 {
                    ExperimentAction::Conduct => next_state.set(GameState::Experimenting),
                    ExperimentAction::Finish => next_state.set(GameState::Planning),
                }
            }
            Interaction::Hovered => {}
            Interaction::None => {}
        }
    }
}
