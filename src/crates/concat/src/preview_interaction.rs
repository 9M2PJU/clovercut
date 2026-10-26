// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Clovercut contributors
//! Resolve overlapping preview clicks without stealing a selected item's drag.

/// A press can drag the held selection but still click a different top layer.
#[derive(Debug, PartialEq, Eq)]
pub struct Pick<T> {
    /// The selected hit to drag, or the ordinary topmost hit.
    pub drag: T,
    /// Select this instead on release only when no drag occurred.
    pub click: Option<T>,
}

/// `hits` contains visible, unlocked hits in topmost-first order.
/// Modifier clicks keep normal topmost selection/toggle semantics.
pub fn pick<T>(hits: impl IntoIterator<Item = (T, bool)>, additive: bool) -> Option<Pick<T>> {
    let mut hits = hits.into_iter();
    let (top, selected) = hits.next()?;
    if !additive
        && !selected
        && let Some((held, _)) = hits.find(|(_, selected)| *selected)
    {
        return Some(Pick {
            drag: held,
            click: Some(top),
        });
    }
    Some(Pick {
        drag: top,
        click: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn an_overlapping_overlay_does_not_steal_the_selected_video_drag() {
        assert_eq!(
            pick([("title", false), ("video", true)], false),
            Some(Pick {
                drag: "video",
                click: Some("title")
            })
        );
    }
    #[test]
    fn an_explicit_click_can_still_pick_the_front_layer() {
        let pick = pick([("sticker", false), ("text", true)], false).unwrap();
        assert_eq!(pick.click, Some("sticker"));
        assert_eq!(pick.drag, "text");
    }
    #[test]
    fn modifiers_and_points_outside_the_selection_keep_ordinary_picking() {
        assert_eq!(
            pick([("top", false), ("held", true)], true),
            Some(Pick {
                drag: "top",
                click: None
            })
        );
        assert_eq!(
            pick([("outside", false)], false),
            Some(Pick {
                drag: "outside",
                click: None
            })
        );
        assert_eq!(pick::<&str>([], false), None);
    }
    #[test]
    fn an_already_selected_top_layer_preserves_the_group() {
        assert_eq!(
            pick([("selected-top", true), ("selected-bottom", true)], false),
            Some(Pick {
                drag: "selected-top",
                click: None
            })
        );
        assert_eq!(
            pick(
                [
                    ("other", false),
                    ("selected-top", true),
                    ("selected-bottom", true)
                ],
                false
            )
            .unwrap()
            .drag,
            "selected-top"
        );
    }
}
