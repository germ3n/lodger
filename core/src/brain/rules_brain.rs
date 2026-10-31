//! The rule brain: answers the menu with fixed weights when there is no model, when the queue
//! is full, or when `brain.force_rules` is on. It is also the baseline the model is judged
//! against. The weights favour `wait` early and the small things before the noticed ones.

use crate::events::Event;
use crate::model::Rng;

/// Picks one of the menu, `wait` among them.
pub fn choose<'a>(menu: &[&'a Event], stage: u8, rng: &mut Rng) -> &'a Event {
    choose_wanting(menu, stage, "", rng)
}

/// [`choose`] with the want leaning the weights: `one thing of theirs` doubles the take and
/// what follows it, `to be looked at` the figures, `a place of its own` the building and the
/// path, as `docs/entity.md`'s What it wants has it.
pub fn choose_wanting<'a>(menu: &[&'a Event], stage: u8, want: &str, rng: &mut Rng) -> &'a Event {
    choose_with(menu, stage, want, &|_| None, rng)
}

/// [`choose_wanting`] with how the target takes each kind of event leaning it too (the
/// review's N2): `notice(effect)` is the share of that kind they noticed, and an event's
/// weight is times 0.6 for one they never notice up to 1.4 for one they always do, so it
/// does more of what reaches them and still tries the rest. `wait` is not leaned.
pub fn choose_with<'a>(
    menu: &[&'a Event],
    stage: u8,
    want: &str,
    notice: &dyn Fn(&str) -> Option<f64>,
    rng: &mut Rng,
) -> &'a Event {
    let weights: Vec<f64> = menu
        .iter()
        .map(|event| {
            let base = if event.id == "wait" {
                match stage {
                    1 => 3.0,
                    2 => 2.0,
                    _ => 1.0,
                }
            } else if event.small {
                2.0
            } else {
                match stage {
                    1 | 2 => 0.5,
                    _ => 1.5,
                }
            };
            let leaning = match want {
                "one thing of theirs" => matches!(
                    event.id,
                    "take" | "place_taken" | "return_taken" | "item_on_table"
                ),
                "to be looked at" => {
                    event.id.contains("figure")
                        || matches!(
                            event.id,
                            "behind_you" | "on_the_roof" | "across_water" | "in_doorway" | "stare"
                        )
                }
                "a place of its own" => matches!(
                    event.id,
                    "sign" | "cairn" | "hole" | "light_path" | "path_worn" | "grave"
                ),
                _ => false,
            };
            let wanted = if leaning { base * 2.0 } else { base };
            let effect = event.effect();
            let noticed = if effect.is_empty() {
                None
            } else {
                notice(effect)
            };

            noticed.map_or(wanted, |rate| wanted * (0.6 + 0.8 * rate.clamp(0.0, 1.0)))
        })
        .collect();
    let total: f64 = weights.iter().sum();
    let mut draw = rng.next_f64() * total;

    for (event, weight) in menu.iter().zip(weights) {
        if draw < weight {
            return event;
        }
        draw -= weight;
    }

    menu[menu.len() - 1]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::events::find;

    #[test]
    fn early_stages_wait_more_and_later_ones_act_more() {
        let menu = vec![
            find("wait").unwrap(),
            find("open_door").unwrap(),
            find("take").unwrap(),
        ];
        let mut rng = Rng::new(5);
        let count = |stage: u8, rng: &mut Rng| {
            let mut waits = 0;
            let mut takes = 0;
            for _ in 0..3000 {
                match choose(&menu, stage, rng).id {
                    "wait" => waits += 1,
                    "take" => takes += 1,
                    _ => {}
                }
            }
            (waits, takes)
        };

        let (waits_early, takes_early) = count(1, &mut rng);
        let (waits_late, takes_late) = count(4, &mut rng);
        assert!(waits_early > waits_late, "{waits_early} vs {waits_late}");
        assert!(takes_late > takes_early, "{takes_late} vs {takes_early}");
    }

    #[test]
    fn what_they_notice_is_chosen_more_and_what_they_ignore_less() {
        let menu = vec![find("knock").unwrap(), find("sign").unwrap()];
        assert_eq!(menu[0].effect(), "sounds");
        assert_eq!(menu[1].effect(), "words");
        let count = |notice: &dyn Fn(&str) -> Option<f64>| {
            let mut rng = Rng::new(9);
            (0..4000)
                .filter(|_| choose_with(&menu, 4, "", notice, &mut rng).id == "sign")
                .count()
        };

        let even = count(&|_| None);
        let leaned = count(&|effect| Some(if effect == "words" { 1.0 } else { 0.0 }));
        assert!(leaned > even + 400, "{leaned} vs {even}");
        let other_way = count(&|effect| Some(if effect == "words" { 0.0 } else { 1.0 }));
        assert!(other_way < even - 400, "{other_way} vs {even}");
        assert!(other_way > 0, "still tried");
    }
}
