use std::sync::Arc;
use std::sync::mpsc::Sender;
use std::time::{Duration, Instant};

use lords_sim::{Command, Simulation, State};

use crate::clock::Clock;
use crate::snapshot::Snapshot;
use crate::speed::Speed;

/// A request from the [`Runtime`](crate::Runtime) handle to the simulation thread.
#[derive(Debug)]
pub(crate) enum Request {
    Pause,
    Resume,
    SetSpeed(Speed),
    Command(Command),
    Capture(Sender<Arc<State>>),
}

/// How long the ticks due at once may take together, and how many ticks pass between
/// looking at the time.
const BUDGET: Duration = Duration::from_millis(12);
const BUDGET_CHECK: u32 = 32;

pub(crate) struct Engine {
    sim: Simulation,
    clock: Clock,
    speed: Speed,
}

impl Engine {
    /// Returns an engine for `sim` whose first tick is due one tick interval after `now`,
    /// or once it is resumed if it starts `paused`.
    pub(crate) fn new(sim: Simulation, speed: Speed, paused: bool, now: Instant) -> Self {
        let interval = speed.tick_interval();
        let clock = if paused {
            Clock::paused(interval)
        } else {
            Clock::starting_at(now, interval)
        };

        Self { sim, clock, speed }
    }

    pub(crate) fn handle(&mut self, request: Request, now: Instant) {
        match request {
            Request::Pause => self.clock.pause(),
            Request::Resume => self.clock.resume(now),
            Request::SetSpeed(speed) => {
                self.speed = speed;
                self.clock.set_interval(speed.tick_interval(), now);
            }
            Request::Command(command) => self.sim.apply(command),
            Request::Capture(captured) => {
                let _ = captured.send(Arc::new(self.sim.state().clone()));
            }
        }
    }

    /// Runs the ticks that are due at `now`, if there are any, and returns whether it did.
    ///
    /// The clock is paused if something happens in them that the player should not miss.
    pub(crate) fn advance(&mut self, now: Instant) -> bool {
        let due = self.clock.take_due_ticks(now);
        let started = Instant::now();
        for tick in 0..due {
            self.sim.tick();
            if self.sim.take_alert() {
                self.clock.pause();
                break;
            }
            // Ticks that do not fit in the time there is for them are let go of, so that
            // the simulation falls behind its speed rather than stops answering.
            if tick % BUDGET_CHECK == BUDGET_CHECK - 1 && started.elapsed() > BUDGET {
                break;
            }
        }
        due > 0
    }

    /// Returns the moment the next tick is due, or `None` while paused.
    pub(crate) fn deadline(&self) -> Option<Instant> {
        self.clock.deadline()
    }

    pub(crate) fn tick_interval(&self) -> Duration {
        self.clock.interval()
    }

    pub(crate) fn snapshot(&self) -> Snapshot {
        Snapshot {
            view: self.sim.view(),
            paused: self.clock.is_paused(),
            speed: self.speed,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc;

    use lords_sim::{Name, Sex, Timestamp};

    use super::*;
    use crate::speed::BASE_INTERVAL;

    const INTERVAL: Duration = BASE_INTERVAL;

    fn engine(start: Instant) -> Engine {
        Engine::new(
            Simulation::new(Name::default(), Sex::Female),
            Speed::BASE,
            false,
            start,
        )
    }

    fn after_ticks(ticks: u64) -> Timestamp {
        Timestamp::from_seconds(ticks / lords_sim::time::TICKS_PER_SECOND)
    }

    #[test]
    fn due_ticks_advance_simulation_time() {
        let start = Instant::now();
        let mut engine = engine(start);

        assert!(!engine.advance(start));
        assert!(engine.advance(start + INTERVAL));
        assert!(engine.advance(start + INTERVAL * 2));

        assert_eq!(engine.snapshot().view.time, after_ticks(2));
    }

    #[test]
    fn ticks_missed_are_made_up_for() {
        let start = Instant::now();
        let mut engine = engine(start);

        assert!(engine.advance(start + INTERVAL * 3));

        assert_eq!(engine.snapshot().view.time, after_ticks(3));
    }

    #[test]
    fn clock_is_paused_when_the_player_must_not_miss_what_happens() {
        let start = Instant::now();
        let mut engine = engine(start);
        let mut now = start;

        // Left alone, the player is parched within two days, which is not to be missed.
        while !engine.snapshot().paused {
            now += INTERVAL * 60;
            engine.advance(now);
            assert!(engine.snapshot().view.time < after_ticks(2 * 86_400));
        }

        let parched = lords_sim::body::Ailment::Parched;
        let view = engine.snapshot().view;
        assert!(view.player.body().ailments().contains(&parched));
        assert!(!engine.advance(now + INTERVAL * 60));
    }

    #[test]
    fn pausing_stops_time_until_resumed() {
        let start = Instant::now();
        let mut engine = engine(start);

        engine.handle(Request::Pause, start);
        assert!(!engine.advance(start + INTERVAL * 5));
        assert!(engine.snapshot().paused);
        assert_eq!(engine.snapshot().view.time, Timestamp::EPOCH);

        engine.handle(Request::Resume, start + INTERVAL * 5);
        assert!(engine.advance(start + INTERVAL * 6));
        assert!(!engine.snapshot().paused);
        assert_eq!(engine.snapshot().view.time, after_ticks(1));
    }

    #[test]
    fn engine_started_paused_keeps_its_speed_for_later() {
        let start = Instant::now();
        let fast = Speed::new(10);
        let interval = fast.tick_interval();
        let mut engine = Engine::new(
            Simulation::new(Name::default(), Sex::Female),
            fast,
            true,
            start,
        );

        assert!(engine.snapshot().paused);
        assert_eq!(engine.snapshot().speed, fast);
        assert!(!engine.advance(start + interval * 5));

        engine.handle(Request::Resume, start + interval * 5);
        assert!(engine.advance(start + interval * 6));
        assert_eq!(engine.snapshot().view.time, after_ticks(1));
    }

    #[test]
    fn commands_applied_while_paused() {
        let start = Instant::now();
        let mut engine = engine(start);
        engine.advance(start + INTERVAL);
        engine.handle(Request::Pause, start + INTERVAL);

        engine.handle(Request::Command(Command::Reset), start + INTERVAL);

        assert!(engine.snapshot().paused);
        assert_eq!(engine.snapshot().view.time, Timestamp::EPOCH);
    }

    #[test]
    fn capture_holds_the_state_as_it_was_when_asked_for() {
        let start = Instant::now();
        let mut engine = engine(start);
        engine.advance(start + INTERVAL);
        let (sender, captured) = mpsc::channel();

        engine.handle(Request::Capture(sender), start + INTERVAL);
        engine.advance(start + INTERVAL * 2);

        let state = captured.try_recv().expect("the state was not captured");
        assert_eq!(state.time, after_ticks(1));
        assert_eq!(engine.snapshot().view.time, after_ticks(2));
    }

    #[test]
    fn capture_nobody_waits_for_is_dropped() {
        let start = Instant::now();
        let mut engine = engine(start);
        let (sender, captured) = mpsc::channel();
        drop(captured);

        engine.handle(Request::Capture(sender), start);

        assert!(engine.advance(start + INTERVAL));
    }

    #[test]
    fn requests_do_not_move_deadline() {
        let start = Instant::now();
        let mut engine = engine(start);

        engine.handle(Request::Command(Command::Reset), start + INTERVAL / 2);
        engine.handle(Request::Resume, start + INTERVAL / 2);

        assert_eq!(engine.deadline(), Some(start + INTERVAL));
    }

    #[test]
    fn speed_change_is_published_and_shortens_ticks() {
        let start = Instant::now();
        let mut engine = engine(start);
        let fast = Speed::new(10);
        let interval = fast.tick_interval();

        engine.handle(Request::SetSpeed(fast), start);

        assert_eq!(engine.snapshot().speed, fast);
        assert!(engine.advance(start + interval));
        assert!(engine.advance(start + interval * 2));
        assert_eq!(engine.snapshot().view.time, after_ticks(2));
    }
}
