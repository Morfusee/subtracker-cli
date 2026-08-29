use std::io::{self, stdout};

use crossterm::{
    cursor::Show,
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};

pub trait TerminalOps {
    fn enter(&mut self) -> io::Result<()>;
    fn restore(&mut self) -> io::Result<()>;
}

#[derive(Default)]
pub struct CrosstermOps;

impl TerminalOps for CrosstermOps {
    fn enter(&mut self) -> io::Result<()> {
        enable_raw_mode()?;
        execute!(stdout(), EnterAlternateScreen)?;
        Ok(())
    }

    fn restore(&mut self) -> io::Result<()> {
        let _ = disable_raw_mode();
        execute!(stdout(), LeaveAlternateScreen, Show)?;
        Ok(())
    }
}

pub struct TerminalGuard<T: TerminalOps> {
    ops: Option<T>,
}

impl<T: TerminalOps> TerminalGuard<T> {
    pub fn enter(mut ops: T) -> io::Result<Self> {
        ops.enter()?;
        Ok(Self { ops: Some(ops) })
    }
}

impl<T: TerminalOps> Drop for TerminalGuard<T> {
    fn drop(&mut self) {
        if let Some(ops) = self.ops.as_mut() {
            let _ = ops.restore();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    };

    struct RecordingOps {
        restored: Arc<AtomicBool>,
    }

    impl TerminalOps for RecordingOps {
        fn enter(&mut self) -> std::io::Result<()> {
            Ok(())
        }

        fn restore(&mut self) -> std::io::Result<()> {
            self.restored.store(true, Ordering::SeqCst);
            Ok(())
        }
    }

    #[test]
    fn dropping_guard_restores_terminal() {
        let restored = Arc::new(AtomicBool::new(false));

        {
            let ops = RecordingOps {
                restored: restored.clone(),
            };
            let _guard = TerminalGuard::enter(ops).unwrap();
        }

        assert!(restored.load(Ordering::SeqCst));
    }
}
