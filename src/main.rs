use std::{collections::HashMap, error::Error, io::stdout, sync::Arc, time::Duration};

use chrono::Utc;
use crossterm::event::{Event, EventStream};
use futures_util::StreamExt;
use ratatui::{Terminal, backend::CrosstermBackend};
use tokio::time::{Instant, MissedTickBehavior, interval_at};

use subtracker::{
    app::UpdateAction,
    event::{Action, action_for_key},
    model::ProviderId,
    providers::{
        UsageProvider, antigravity::AntigravityProvider, codex::CodexProvider,
        opencode::OpenCodeProvider, process::SystemProcessRunner,
    },
    refresh::ProviderRegistry,
    runtime::RuntimeController,
    terminal::{CrosstermOps, TerminalGuard},
    ui::{self, theme::Theme},
    updater::{UpdateChecker, install_update, open_release_notes},
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RunOutcome {
    Quit,
    InstallUpdate,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    if run_tui().await? == RunOutcome::InstallUpdate && !install_update()? {
        return Err("cargo install subtracker failed".into());
    }
    Ok(())
}

async fn run_tui() -> Result<RunOutcome, Box<dyn Error>> {
    let _guard = TerminalGuard::enter(CrosstermOps)?;

    let backend = CrosstermBackend::new(stdout());
    let mut terminal = Terminal::new(backend)?;

    let providers = production_providers()?;
    let update_checker = UpdateChecker::production()?;
    let (mut runtime, mut refresh_results, mut update_results) =
        RuntimeController::new(providers, update_checker);
    let mut events = EventStream::new();

    let refresh_period = Duration::from_secs(60);
    let mut refresh_timer = interval_at(Instant::now() + refresh_period, refresh_period);
    refresh_timer.set_missed_tick_behavior(MissedTickBehavior::Skip);

    let animation_period = Duration::from_millis(250);
    let mut animation_timer = interval_at(Instant::now() + animation_period, animation_period);
    animation_timer.set_missed_tick_behavior(MissedTickBehavior::Skip);

    let update_period = Duration::from_secs(15 * 60);
    let mut update_timer = interval_at(Instant::now() + update_period, update_period);
    update_timer.set_missed_tick_behavior(MissedTickBehavior::Skip);

    let theme = Theme::detect();
    let mut spinner_frame = 0u8;

    runtime.request_refresh();
    runtime.request_update_check();

    let outcome = loop {
        terminal.draw(|frame| {
            ui::render(frame, runtime.app(), Utc::now(), spinner_frame, theme);
        })?;

        tokio::select! {
            maybe_event = events.next() => {
                match maybe_event {
                    Some(Ok(Event::Key(key))) => {
                        match action_for_key(key, runtime.app().is_update_modal_open()) {
                            Action::Quit => break RunOutcome::Quit,
                            Action::Refresh => runtime.request_refresh(),
                            Action::NextProvider => runtime.next_provider(),
                            Action::PreviousProvider => runtime.prev_provider(),
                            Action::ToggleCollapse => runtime.toggle_focused_collapse(),
                            Action::OpenUpdateModal => runtime.open_update_modal(),
                            Action::NextUpdateAction => runtime.next_update_action(),
                            Action::PreviousUpdateAction => runtime.previous_update_action(),
                            Action::CloseUpdateModal => runtime.close_update_modal(),
                            Action::ConfirmUpdateAction => match runtime.app().selected_update_action() {
                                UpdateAction::UpdateNow => break RunOutcome::InstallUpdate,
                                UpdateAction::ViewReleaseNotes => {
                                    let url = runtime
                                        .app()
                                        .available_update()
                                        .expect("update modal requires an available update")
                                        .release_url
                                        .clone();
                                    if open_release_notes(&url).is_ok() {
                                        runtime.close_update_modal();
                                    }
                                }
                                UpdateAction::RemindLater => runtime.dismiss_update(),
                            },
                            Action::Ignore => {}
                        }
                    }
                    Some(Ok(Event::Resize(_, _))) => {}
                    Some(Ok(_)) => {}
                    Some(Err(error)) => return Err(error.into()),
                    None => break RunOutcome::Quit,
                }
            }

            _ = refresh_timer.tick() => {
                runtime.request_refresh();
            }

            _ = animation_timer.tick() => {
                spinner_frame = spinner_frame.wrapping_add(1);
            }

            _ = update_timer.tick() => {
                runtime.request_update_check();
            }

            refresh = refresh_results.recv() => {
                if let Some(refresh) = refresh {
                    runtime.apply_refresh_result(refresh);
                }
            }

            update = update_results.recv() => {
                if let Some(update) = update {
                    runtime.apply_update_check_result(update);
                }
            }
        }
    };

    Ok(outcome)
}

fn production_providers() -> Result<ProviderRegistry, Box<dyn Error>> {
    let process_runner = Arc::new(SystemProcessRunner);
    let timeout = Duration::from_secs(15);

    let codex: Arc<dyn UsageProvider> = Arc::new(CodexProvider::production()?);
    let opencode: Arc<dyn UsageProvider> = Arc::new(OpenCodeProvider::production()?);
    let antigravity: Arc<dyn UsageProvider> =
        Arc::new(AntigravityProvider::new(process_runner, timeout));

    let providers: HashMap<ProviderId, Arc<dyn UsageProvider>> = [
        (ProviderId::Codex, codex),
        (ProviderId::OpenCode, opencode),
        (ProviderId::Antigravity, antigravity),
    ]
    .into_iter()
    .collect();

    Ok(providers)
}
