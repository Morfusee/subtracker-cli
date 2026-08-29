use std::{collections::HashMap, error::Error, io::stdout, sync::Arc, time::Duration};

use chrono::Utc;
use crossterm::event::{Event, EventStream};
use futures_util::StreamExt;
use ratatui::{Terminal, backend::CrosstermBackend};
use tokio::time::{Instant, MissedTickBehavior, interval_at};

use subtracker_cli::{
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
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let _guard = TerminalGuard::enter(CrosstermOps)?;

    let backend = CrosstermBackend::new(stdout());
    let mut terminal = Terminal::new(backend)?;

    let providers = production_providers()?;
    let (mut runtime, mut refresh_results) = RuntimeController::new(providers);
    let mut events = EventStream::new();

    let refresh_period = Duration::from_secs(60);
    let mut refresh_timer = interval_at(Instant::now() + refresh_period, refresh_period);
    refresh_timer.set_missed_tick_behavior(MissedTickBehavior::Skip);

    let animation_period = Duration::from_millis(250);
    let mut animation_timer = interval_at(Instant::now() + animation_period, animation_period);
    animation_timer.set_missed_tick_behavior(MissedTickBehavior::Skip);

    let theme = Theme::detect();
    let mut spinner_frame = 0u8;

    runtime.request_refresh();

    loop {
        terminal.draw(|frame| {
            ui::render(frame, runtime.app(), Utc::now(), spinner_frame, theme);
        })?;

        tokio::select! {
            maybe_event = events.next() => {
                match maybe_event {
                    Some(Ok(Event::Key(key))) => {
                        match action_for_key(key) {
                            Action::Quit => break,
                            Action::Refresh => runtime.request_refresh(),
                            Action::Ignore => {}
                        }
                    }
                    Some(Ok(Event::Resize(_, _))) => {}
                    Some(Ok(_)) => {}
                    Some(Err(error)) => return Err(error.into()),
                    None => break,
                }
            }

            _ = refresh_timer.tick() => {
                runtime.request_refresh();
            }

            _ = animation_timer.tick() => {
                spinner_frame = spinner_frame.wrapping_add(1);
            }

            refresh = refresh_results.recv() => {
                if let Some(refresh) = refresh {
                    runtime.apply_refresh_result(refresh);
                }
            }
        }
    }

    Ok(())
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
