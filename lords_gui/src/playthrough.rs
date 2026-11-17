use std::mem;
use std::sync::Arc;
use std::sync::mpsc::{Receiver, TryRecvError};
use std::time::{Duration, Instant};

use lords_runtime::{Runtime, Snapshot, Speed};
use lords_save::{SaveKind, SaveRequest, SimulationId};
use lords_sim::{State, Timestamp};

pub struct Playthrough {
    runtime: Runtime,
    id: SimulationId,
    save_label: Option<String>,
    last_autosave: Instant,
    last_saved_time: Option<Timestamp>,
    pending: Vec<PendingSave>,
}

struct PendingSave {
    kind: SaveKind,
    label: String,
    speed: Speed,
    state: Receiver<Arc<State>>,
}

impl Playthrough {
    pub fn new(runtime: Runtime, id: SimulationId, save_label: Option<String>) -> Self {
        Self {
            runtime,
            id,
            save_label,
            last_autosave: Instant::now(),
            last_saved_time: None,
            pending: Vec::new(),
        }
    }

    pub fn runtime(&self) -> &Runtime {
        &self.runtime
    }

    pub fn save_label(&self) -> Option<&str> {
        self.save_label.as_deref()
    }

    pub fn save(&mut self, kind: SaveKind, label: String) {
        let snapshot = self.runtime.snapshot();
        self.save_as_of(&snapshot, kind, label);
    }

    /// Saves the game if it has been running for `interval` since the last autosave and
    /// has moved on since it was last saved.
    pub fn autosave(&mut self, snapshot: &Snapshot, interval: Option<Duration>, now: Instant) {
        let Some(interval) = interval else {
            return;
        };
        if snapshot.paused || now.saturating_duration_since(self.last_autosave) < interval {
            return;
        }

        self.last_autosave = now;
        if self.last_saved_time != Some(snapshot.view.time) {
            self.save_as_of(snapshot, SaveKind::Auto, String::new());
        }
    }

    /// Returns the saves whose state the simulation has handed over, ready to be written.
    pub fn take_captured(&mut self) -> Vec<SaveRequest> {
        let mut captured = Vec::new();

        for pending in mem::take(&mut self.pending) {
            match pending.state.try_recv() {
                Ok(state) => captured.push(self.request(pending, state)),
                Err(TryRecvError::Empty) => self.pending.push(pending),
                Err(TryRecvError::Disconnected) => log_lost_save(),
            }
        }
        captured
    }

    pub fn finish(&mut self) -> Vec<SaveRequest> {
        self.save(SaveKind::Exit, String::new());

        mem::take(&mut self.pending)
            .into_iter()
            .filter_map(|pending| match pending.state.recv() {
                Ok(state) => Some(self.request(pending, state)),
                Err(_) => {
                    log_lost_save();
                    None
                }
            })
            .collect()
    }

    fn save_as_of(&mut self, snapshot: &Snapshot, kind: SaveKind, label: String) {
        self.last_saved_time = Some(snapshot.view.time);
        if kind == SaveKind::Manual {
            self.save_label = Some(label.clone());
        }

        self.pending.push(PendingSave {
            kind,
            label,
            speed: snapshot.speed,
            state: self.runtime.capture(),
        });
    }

    fn request(&self, pending: PendingSave, state: Arc<State>) -> SaveRequest {
        SaveRequest {
            playthrough: self.id,
            kind: pending.kind,
            label: pending.label,
            speed: pending.speed.multiplier(),
            state,
        }
    }
}

fn log_lost_save() {
    log::error!("The simulation thread has stopped; the game was not saved");
}

#[cfg(test)]
mod tests {
    use std::thread;

    use lords_sim::{Name, Sex, Simulation};

    use super::*;

    const TIMEOUT: Duration = Duration::from_secs(10);
    const MINUTE: Option<Duration> = Some(Duration::from_secs(60));

    fn playthrough(paused: bool) -> Playthrough {
        let name = Name::new("Ada".to_owned(), Vec::new(), "Lord".to_owned());
        let runtime = Runtime::spawn_with(
            Simulation::new(name, Sex::Female),
            Speed::new(10),
            paused,
            || {},
        );

        Playthrough::new(runtime, SimulationId::generate(), None)
    }

    fn wait_until(mut condition: impl FnMut() -> bool) {
        let start = Instant::now();
        while !condition() {
            assert!(start.elapsed() < TIMEOUT, "waited too long");
            thread::sleep(Duration::from_millis(1));
        }
    }

    /// Returns every save asked for so far, once all of them have been captured.
    fn captured(game: &mut Playthrough) -> Vec<SaveRequest> {
        let mut captured = Vec::new();
        wait_until(|| {
            captured.extend(game.take_captured());
            game.pending.is_empty()
        });
        captured
    }

    fn running() -> Playthrough {
        let game = playthrough(false);
        wait_until(|| game.runtime.snapshot().view.time > Timestamp::EPOCH);
        game
    }

    #[test]
    fn save_made_by_hand_is_described_and_named_for_the_next() {
        let mut game = playthrough(true);
        assert_eq!(game.save_label(), None);

        game.save(SaveKind::Manual, "Camp".to_owned());
        game.save(SaveKind::Auto, String::new());
        let captured = captured(&mut game);

        assert_eq!(game.save_label(), Some("Camp"));
        assert_eq!(captured.len(), 2);
        assert_eq!(captured[0].kind, SaveKind::Manual);
        assert_eq!(captured[0].label, "Camp");
        assert_eq!(captured[0].speed, 10);
        assert_eq!(captured[0].playthrough, game.id);
        assert_eq!(captured[0].state.time, game.runtime.snapshot().view.time);
        assert_eq!(captured[1].kind, SaveKind::Auto);
    }

    #[test]
    fn game_is_autosaved_once_it_has_run_for_the_interval() {
        let mut game = running();
        let snapshot = game.runtime.snapshot();
        let after = |seconds| game.last_autosave + Duration::from_secs(seconds);
        let (minute_less_a_second, minute) = (after(59), after(60));

        game.autosave(&snapshot, MINUTE, minute_less_a_second);
        assert!(game.pending.is_empty());

        game.autosave(&snapshot, MINUTE, minute);
        let captured = captured(&mut game);
        assert_eq!(captured.len(), 1);
        assert_eq!(captured[0].kind, SaveKind::Auto);
        assert_eq!(game.save_label(), None);

        // Neither before another interval has passed, nor while the game has not moved on.
        game.autosave(&snapshot, MINUTE, minute + Duration::from_secs(59));
        game.autosave(&snapshot, MINUTE, minute + Duration::from_secs(60));
        assert!(game.pending.is_empty());
    }

    #[test]
    fn game_is_not_autosaved_while_paused_or_when_turned_off() {
        let much_later = Instant::now() + Duration::from_secs(3_600);

        let mut paused = playthrough(true);
        let snapshot = paused.runtime.snapshot();
        paused.autosave(&snapshot, MINUTE, much_later);
        assert!(paused.pending.is_empty());

        let mut turned_off = running();
        let snapshot = turned_off.runtime.snapshot();
        turned_off.autosave(&snapshot, None, much_later);
        assert!(turned_off.pending.is_empty());
    }

    #[test]
    fn finishing_saves_on_leaving_and_waits_for_every_save() {
        let mut game = playthrough(true);
        game.save(SaveKind::Manual, "Camp".to_owned());

        let kinds: Vec<SaveKind> = game
            .finish()
            .into_iter()
            .map(|request| request.kind)
            .collect();

        assert_eq!(kinds, [SaveKind::Manual, SaveKind::Exit]);
        assert!(game.pending.is_empty());
    }
}
