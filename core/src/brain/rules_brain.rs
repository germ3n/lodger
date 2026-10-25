//! The rule brain: answers the menu with fixed weights when there is no model, when the queue
//! is full, or when `brain.force_rules` is on. It is also the baseline the model is judged
//! against. The weights favour `wait` early and the small things before the noticed ones.

use crate::events::Event;
use crate::model::Rng;

/// Picks one of the menu, `wait` among them.
pub fn choose<'a>(menu: &[&'a Event], stage: u8, rng: &mut Rng) -> &'a Event {
    let weights: Vec<f64> = menu
        .iter()
        .map(|event| {
            if event.id == "wait" {
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
            }
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
}
