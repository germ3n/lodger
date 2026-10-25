//! From a decided event to the host's commands, resolved against the memory at the moment it
//! runs. Each action here is one of the general commands the host carries out; the event's
//! `fits` is the host's check when the command arrives, and `ready` is the mind's check that the
//! moment has come.

use crate::events::{Action, Event, Need};
use crate::memory::{Held, Memory};
use crate::model::Rng;
use craftkit::world::{BlockPos, Command, ItemStack, Now};

/// Whether the event's moment is here: its needs still hold.
pub fn ready(event: &Event, memory: &Memory, _config: &crate::config::Config) -> bool {
    let now = Now::default();
    event
        .needs
        .iter()
        .all(|need| *need == Need::Night || super::offer::meets(*need, memory, &now))
}

/// The place the event happens, from the memory, or none for a thing with no place.
pub fn place(event: &Event, memory: &Memory, rng: &mut Rng) -> Option<BlockPos> {
    let home = memory.home.as_ref();

    match event.action {
        Action::Wait
        | Action::Hint(_, _)
        | Action::RefuseRest
        | Action::Stare
        | Action::SoundAtTarget(_) => None,
        Action::ToggleDoor => home.and_then(|h| pick(&h.doors, rng)),
        Action::Sound(_) => home.map(|h| {
            // Another room: six blocks off the center, a random way.
            let angle = rng.next_f64() * std::f64::consts::TAU;
            [
                h.center[0] + (6.0 * angle.cos()).round() as i32,
                h.floor_y + 1,
                h.center[2] + (6.0 * angle.sin()).round() as i32,
            ]
        }),
        Action::ChestSounds | Action::ShuffleContainer | Action::SplitStack => {
            memory.used_container().map(|c| c.pos)
        }
        Action::Take => memory.favourite_in_container().map(|(c, _)| c.pos),
        Action::ReturnTaken => memory.held.as_ref().map(|h| h.from),
        Action::PlaceTaken => memory
            .place
            .or_else(|| home.and_then(|h| h.bed))
            .or_else(|| home.map(|h| h.center)),
        Action::MoveLight => home.and_then(|h| pick(&h.lights, rng)),
        Action::AddTorch => home.map(|h| {
            let dx = (rng.next_u64() % 7) as i32 - 3;
            let dz = (rng.next_u64() % 7) as i32 - 3;
            [h.center[0] + dx, h.floor_y, h.center[2] + dz]
        }),
    }
}

fn pick<T: Copy>(list: &[T], rng: &mut Rng) -> Option<T> {
    if list.is_empty() {
        None
    } else {
        Some(list[(rng.next_u64() % list.len() as u64) as usize])
    }
}

/// The commands for the event at `pos`, each with the ticks to wait before sending it (0 for
/// now). `ids` hands out command ids.
pub fn commands(
    event: &Event,
    pos: Option<BlockPos>,
    memory: &Memory,
    ids: &mut impl FnMut() -> u64,
    _now: &Now,
) -> Vec<(Command, u64)> {
    let target = memory.target.unwrap_or(0);
    let at = |p: BlockPos| [p[0] as f64 + 0.5, p[1] as f64 + 0.5, p[2] as f64 + 0.5];

    match event.action {
        Action::Wait => vec![],
        Action::ToggleDoor => pos
            .map(|p| vec![(Command::ToggleBlock { id: ids(), pos: p }, 0)])
            .unwrap_or_default(),
        Action::Sound(sound) => pos
            .map(|p| {
                vec![(
                    Command::PlaySound {
                        id: ids(),
                        player: target,
                        sound: sound.to_string(),
                        pos: at(p),
                        volume: 1.0,
                        pitch: 1.0,
                    },
                    0,
                )]
            })
            .unwrap_or_default(),
        Action::SoundAtTarget(sound) => vec![(
            Command::PlaySound {
                id: ids(),
                player: target,
                sound: sound.to_string(),
                pos: memory.last.pos,
                volume: 0.8,
                pitch: 1.0,
            },
            10,
        )],
        Action::ChestSounds => pos
            .map(|p| {
                vec![
                    (
                        Command::PlaySound {
                            id: ids(),
                            player: target,
                            sound: "minecraft:block.chest.open".to_string(),
                            pos: at(p),
                            volume: 0.7,
                            pitch: 1.0,
                        },
                        0,
                    ),
                    (
                        Command::PlaySound {
                            id: ids(),
                            player: target,
                            sound: "minecraft:block.chest.close".to_string(),
                            pos: at(p),
                            volume: 0.7,
                            pitch: 1.0,
                        },
                        30,
                    ),
                ]
            })
            .unwrap_or_default(),
        Action::MoveLight => pos
            .map(|p| {
                // A step of one to three blocks along one axis, same height.
                let steps = [
                    [2, 0, 0],
                    [-2, 0, 0],
                    [0, 0, 2],
                    [0, 0, -2],
                    [3, 0, 0],
                    [0, 0, -3],
                ];
                let step = steps[(p[0] + p[2]).rem_euclid(steps.len() as i32) as usize];
                vec![(
                    Command::MoveBlock {
                        id: ids(),
                        from: p,
                        to: [p[0] + step[0], p[1], p[2] + step[2]],
                    },
                    0,
                )]
            })
            .unwrap_or_default(),
        Action::AddTorch => pos
            .map(|p| {
                vec![(
                    Command::SetBlock {
                        id: ids(),
                        pos: [p[0], p[1] + 1, p[2]],
                        block: "minecraft:torch".to_string(),
                        state: String::new(),
                    },
                    0,
                )]
            })
            .unwrap_or_default(),
        Action::ShuffleContainer => match (pos, memory.used_container()) {
            (Some(p), Some(container)) if !container.contents.is_empty() => {
                let used: Vec<u32> = container.contents.iter().map(|s| s.slot).collect();
                let empty = (0..27u32).find(|slot| !used.contains(slot));
                match empty {
                    Some(to_slot) => {
                        let stack = &container.contents
                            [(p[0] + p[2]).rem_euclid(container.contents.len() as i32) as usize];
                        vec![(
                            Command::MoveItem {
                                id: ids(),
                                from: p,
                                from_slot: stack.slot,
                                to: p,
                                to_slot,
                                count: stack.count,
                            },
                            0,
                        )]
                    }
                    None => vec![],
                }
            }
            _ => vec![],
        },
        Action::SplitStack => match (pos, memory.used_container()) {
            (Some(p), Some(container)) => {
                let used: Vec<u32> = container.contents.iter().map(|s| s.slot).collect();
                let empty = (0..27u32).find(|slot| !used.contains(slot));
                match (container.contents.iter().find(|s| s.count >= 2), empty) {
                    (Some(stack), Some(to_slot)) => vec![(
                        Command::MoveItem {
                            id: ids(),
                            from: p,
                            from_slot: stack.slot,
                            to: p,
                            to_slot,
                            count: stack.count / 2,
                        },
                        0,
                    )],
                    _ => vec![],
                }
            }
            _ => vec![],
        },
        Action::Take => match memory.favourite_in_container() {
            Some((container, stack)) => vec![(
                Command::TakeItem {
                    id: ids(),
                    from: container.pos,
                    slot: stack.slot,
                    count: 1,
                },
                0,
            )],
            None => vec![],
        },
        Action::ReturnTaken => match &memory.held {
            Some(held) => vec![(
                Command::PutItem {
                    id: ids(),
                    into: held.from,
                    slot: Some(held.slot),
                    item: held.item.clone(),
                },
                0,
            )],
            None => vec![],
        },
        Action::PlaceTaken => match (&memory.held, pos) {
            (Some(held), Some(p)) => vec![(
                Command::DropItem {
                    id: ids(),
                    pos: [p[0] as f64 + 0.5, p[1] as f64 + 1.0, p[2] as f64 + 0.5],
                    item: held.item.clone(),
                },
                0,
            )],
            _ => vec![],
        },
        Action::Hint(kind, seconds) => vec![(
            Command::Hint {
                id: ids(),
                player: target,
                hint: kind.to_string(),
                pos: None,
                seconds,
                detail: String::new(),
            },
            0,
        )],
        Action::RefuseRest => vec![(
            Command::CancelSleep {
                id: ids(),
                player: target,
                seconds: 120.0,
            },
            0,
        )],
        Action::Stare => memory
            .last
            .creatures_near
            .iter()
            .take(8)
            .map(|entity| {
                (
                    Command::LookAt {
                        id: ids(),
                        entity: *entity,
                        at: memory.last.pos,
                        seconds: 20.0,
                    },
                    0,
                )
            })
            .collect(),
    }
}

/// The host took the favourite: the memory holds it now and the container's contents lose it.
pub fn took(memory: &mut Memory, pos: Option<BlockPos>) {
    let Some(pos) = pos else { return };
    let favourite = memory.favourites.first().map(|(id, _)| id.clone());
    let Some(container) = memory.containers.iter_mut().find(|c| c.pos == pos) else {
        return;
    };
    let Some(index) = container
        .contents
        .iter()
        .position(|s| Some(&s.id) == favourite.as_ref())
        .or_else(|| (!container.contents.is_empty()).then_some(0))
    else {
        return;
    };
    let stack = &mut container.contents[index];
    let held = Held {
        item: ItemStack {
            id: stack.id.clone(),
            count: 1,
            slot: 0,
        },
        from: pos,
        slot: stack.slot,
        taken_day: memory.done.last().map(|d| d.day).unwrap_or(0.0),
    };
    if stack.count <= 1 {
        container.contents.remove(index);
    } else {
        stack.count -= 1;
    }
    memory.held = Some(held);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::find;
    use crate::memory::{Container, Home};

    fn memory() -> Memory {
        let mut memory = Memory {
            target: Some(0),
            ..Memory::default()
        };
        memory.last.online = false;
        memory.home = Some(Home {
            bed: Some([0, 64, 0]),
            center: [0, 64, 0],
            floor_y: 64,
            doors: vec![[3, 64, 0]],
            lights: vec![[1, 65, 1]],
        });
        memory
            .favourites
            .push(("minecraft:iron_pickaxe".to_string(), 9));
        memory.containers.push(Container {
            pos: [2, 64, 0],
            opens: 5,
            last_opened_day: 0.0,
            contents: vec![
                ItemStack {
                    id: "minecraft:iron_pickaxe".to_string(),
                    count: 1,
                    slot: 4,
                },
                ItemStack {
                    id: "minecraft:cobblestone".to_string(),
                    count: 40,
                    slot: 5,
                },
            ],
        });
        memory
    }

    fn ids() -> impl FnMut() -> u64 {
        let mut next = 1;
        move || {
            next += 1;
            next
        }
    }

    #[test]
    fn a_door_toggles_a_chest_sounds_twice_and_a_take_removes_the_favourite() {
        let memory = memory();
        let mut rng = Rng::new(1);

        let door = find("open_door").unwrap();
        let p = place(door, &memory, &mut rng);
        assert_eq!(p, Some([3, 64, 0]));
        let made = commands(door, p, &memory, &mut ids(), &Now::default());
        assert!(matches!(
            made[0].0,
            Command::ToggleBlock {
                pos: [3, 64, 0],
                ..
            }
        ));

        let chest = find("open_chest").unwrap();
        let made = commands(
            chest,
            place(chest, &memory, &mut rng),
            &memory,
            &mut ids(),
            &Now::default(),
        );
        assert_eq!(made.len(), 2);
        assert_eq!(made[1].1, 30, "the close comes a second and a half later");

        let take = find("take").unwrap();
        let made = commands(
            take,
            place(take, &memory, &mut rng),
            &memory,
            &mut ids(),
            &Now::default(),
        );
        assert!(
            matches!(
                made[0].0,
                Command::TakeItem {
                    from: [2, 64, 0],
                    slot: 4,
                    count: 1,
                    ..
                }
            ),
            "{:?}",
            made[0].0
        );
    }

    #[test]
    fn taking_moves_the_item_from_the_container_to_the_hand_and_returning_puts_it_back() {
        let mut memory = memory();
        took(&mut memory, Some([2, 64, 0]));

        let held = memory.held.as_ref().unwrap();
        assert_eq!(held.item.id, "minecraft:iron_pickaxe");
        assert_eq!(held.slot, 4);
        assert!(memory.containers[0]
            .contents
            .iter()
            .all(|s| s.id != "minecraft:iron_pickaxe"));

        let back = find("return_taken").unwrap();
        let mut rng = Rng::new(1);
        let made = commands(
            back,
            place(back, &memory, &mut rng),
            &memory,
            &mut ids(),
            &Now::default(),
        );
        assert!(
            matches!(
                made[0].0,
                Command::PutItem {
                    into: [2, 64, 0],
                    slot: Some(4),
                    ..
                }
            ),
            "{:?}",
            made[0].0
        );
    }

    #[test]
    fn shuffling_moves_a_stack_to_an_empty_slot_and_splitting_halves_one() {
        let memory = memory();
        let mut rng = Rng::new(1);

        let shuffle = find("shuffle_chest").unwrap();
        let made = commands(
            shuffle,
            place(shuffle, &memory, &mut rng),
            &memory,
            &mut ids(),
            &Now::default(),
        );
        match &made[0].0 {
            Command::MoveItem {
                from, to, to_slot, ..
            } => {
                assert_eq!(from, to);
                assert!(![4, 5].contains(to_slot));
            }
            other => panic!("{other:?}"),
        }

        let split = find("stack_split").unwrap();
        let made = commands(
            split,
            place(split, &memory, &mut rng),
            &memory,
            &mut ids(),
            &Now::default(),
        );
        assert!(
            matches!(
                made[0].0,
                Command::MoveItem {
                    from_slot: 5,
                    count: 20,
                    ..
                }
            ),
            "{:?}",
            made[0].0
        );
    }
}
