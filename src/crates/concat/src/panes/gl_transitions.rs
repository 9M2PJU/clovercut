// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Clovercut contributors

//! Fetches community transition metadata and installs requested shaders off the UI.
use crate::studio::Studio;
use concat_effects::Catalogue;
use concat_host::gl_transitions::Entry;
use std::collections::HashMap;

#[derive(Default)]
pub struct State {
    pub entries: Vec<Entry>,
    loading: bool,
    action: u64,
    pending: HashMap<String, Pending>,
}
#[derive(Default)]
struct Pending {
    preview: Option<u64>,
    apply: Option<(u64, Vec<String>, u64)>,
}

pub fn start(studio: &mut Studio) {
    if !studio.prefs.gl_transitions_library
        || studio.gl_transitions.loading
        || !studio.gl_transitions.entries.is_empty()
    {
        return;
    }
    studio.gl_transitions.loading = true;
    let cache = studio.host.dirs.data.join("gl-transitions");
    let spawned = std::thread::Builder::new()
        .name("GL catalogue".into())
        .spawn(move || {
            let result = concat_host::gl_transitions::catalogue(&cache);
            crate::host::on_ui(move |studio, _, _| {
                studio.gl_transitions.loading = false;
                match result {
                    Ok(entries) => studio.gl_transitions.entries = entries,
                    Err(error) => {
                        log::warn!("GL catalogue: {error}");
                        if studio.prefs.gl_transitions_library {
                            studio.notify(
                                &crate::i18n::tf("libraries.glCatalogueFailed", &[&error]),
                                true,
                            );
                        }
                    }
                }
            });
        });
    if let Err(error) = spawned {
        studio.gl_transitions.loading = false;
        studio.notify(&error.to_string(), true);
    }
}

pub fn apply(studio: &mut Studio, id: &str) {
    studio.gl_transitions.action = studio.gl_transitions.action.wrapping_add(1);
    if Catalogue::builtin().get(id).is_some() {
        studio.apply_transition(id);
        return;
    }
    download(studio, id, None, Some(crate::host::project_epoch()));
}

pub fn download(studio: &mut Studio, id: &str, preview: Option<u64>, apply: Option<u64>) {
    if !studio.prefs.gl_transitions_library {
        return;
    }
    let apply = apply.map(|epoch| {
        (
            epoch,
            studio.selection.clone(),
            studio.gl_transitions.action,
        )
    });
    if let Some(pending) = studio.gl_transitions.pending.get_mut(id) {
        if preview.is_some() {
            pending.preview = preview;
        }
        if apply.is_some() {
            pending.apply = apply;
        }
        return;
    }
    if preview.is_some() && apply.is_none() && studio.gl_transitions.pending.len() >= 2 {
        return;
    }
    let Some(entry) = studio
        .gl_transitions
        .entries
        .iter()
        .find(|entry| entry.id() == id)
        .cloned()
    else {
        return;
    };
    let effects = Studio::looks_dir(&studio.host.dirs);
    let id = id.to_owned();
    let applying = apply.is_some();
    studio
        .gl_transitions
        .pending
        .insert(id.clone(), Pending { preview, apply });
    if applying {
        studio.notify(&crate::i18n::t("libraries.glDownloading"), false);
    }
    let failure_id = id.clone();
    let spawned = std::thread::Builder::new()
        .name("GL transition".into())
        .spawn(move || {
            let result = concat_host::gl_transitions::install(&entry, &effects);
            crate::host::on_ui(move |studio, _, _| {
                let pending = studio
                    .gl_transitions
                    .pending
                    .remove(&id)
                    .unwrap_or_default();
                if let Err(error) = result {
                    log::warn!("GL transition {id}: {error}");
                    if pending.apply.is_some() {
                        studio.notify(
                            &crate::i18n::tf("libraries.glDownloadFailed", &[&error]),
                            true,
                        );
                    }
                    return;
                }
                studio.reload_packages(false);
                if !studio.prefs.gl_transitions_library {
                    return;
                }
                if let Some((epoch, selection, action)) = pending.apply
                    && epoch == crate::host::project_epoch()
                    && selection == studio.selection
                    && action == studio.gl_transitions.action
                {
                    studio.apply_transition(&id);
                }
                if let Some(generation) = pending.preview {
                    super::transition_preview::resume(studio, &id, generation);
                }
            });
        });
    if let Err(error) = spawned {
        studio.gl_transitions.pending.remove(&failure_id);
        studio.notify(&error.to_string(), true);
    }
}
