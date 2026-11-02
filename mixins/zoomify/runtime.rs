// Smooth Zoom for Horizon. Hold
// (or toggle) C to zoom in to the initial zoom, scroll to zoom further in or
// out in steps, with eased transitions and slower turning while zoomed.
// This file is the module the bundle adds to the game (`crate::game::zoomify`).
use super::Game;
use std::cell::RefCell;
use std::time::Instant;

pub const ID: &str = "zoomify";
pub const NAME: &str = "Smooth Zoom";
pub const VERSION: &str = "1.0.0";
pub const DESCRIPTION: &str = "Hold C to zoom, scroll to zoom further, smooth transitions and slower turning while zoomed.";
const KEY: &str = "zoom";
const DEFAULT_KEY: &str = "key.keyboard.c";

const INITIAL: [f32; 6] = [2.0, 3.0, 4.0, 5.0, 6.0, 8.0];
const TIMES: [f32; 6] = [0.0, 0.25, 0.5, 0.75, 1.0, 1.5];
const MAX_ZOOM: [f32; 4] = [10.0, 20.0, 50.0, 100.0];
const SENSITIVITY: [u32; 6] = [0, 25, 50, 75, 100, 150];
const TRANSITIONS: [&str; 4] = ["Ease Out Exp", "Ease Out Sine", "Linear", "Instant"];

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Settings {
    pub initial: f32,
    pub zoom_in_time: f32,
    pub zoom_out_time: f32,
    /// Index in `TRANSITIONS`.
    pub transition: usize,
    pub scroll: bool,
    pub max_zoom: f32,
    /// Percent: 100 turns as much slower as the view is zoomed.
    pub sensitivity: u32,
    pub toggle: bool,
    pub retain_steps: bool,
}

impl Settings {
    fn read(game: &Game<'_>) -> Self {
        let get = |k: &str| game.native_setting(ID, k);
        let f = |k: &str, d: f32| get(k).and_then(|v| v.parse().ok()).unwrap_or(d);
        let flag = |k: &str, d: bool| get(k).map_or(d, |v| v == "true");
        Self {
            initial: f("initial", 4.0),
            zoom_in_time: f("zoom_in_time", 1.0),
            zoom_out_time: f("zoom_out_time", 0.5),
            transition: get("transition").and_then(|v| v.parse().ok()).filter(|&v| v < TRANSITIONS.len()).unwrap_or(0),
            scroll: flag("scroll", true),
            max_zoom: f("max_zoom", 50.0),
            sensitivity: get("sensitivity").and_then(|v| v.parse().ok()).unwrap_or(100),
            toggle: flag("toggle", false),
            retain_steps: flag("retain_steps", false),
        }
    }
    fn write(&self, game: &Game<'_>) {
        let b = |v: bool| if v { "true" } else { "false" }.to_owned();
        for (k, v) in [
            ("initial", self.initial.to_string()),
            ("zoom_in_time", self.zoom_in_time.to_string()),
            ("zoom_out_time", self.zoom_out_time.to_string()),
            ("transition", self.transition.to_string()),
            ("scroll", b(self.scroll)),
            ("max_zoom", self.max_zoom.to_string()),
            ("sensitivity", self.sensitivity.to_string()),
            ("toggle", b(self.toggle)),
            ("retain_steps", b(self.retain_steps)),
        ] {
            game.native_set_setting(ID, k, &v);
        }
    }
}

/// The zoom moving from one level to another over a duration.
#[derive(Clone, Copy)]
struct Motion {
    from: f32,
    to: f32,
    start: Instant,
    seconds: f32,
}

struct State {
    settings: Option<Settings>,
    zooming: bool,
    /// Scroll steps from the initial zoom (each one ×1.25).
    steps: i32,
    motion: Motion,
}

thread_local! {
    static STATE: RefCell<State> = RefCell::new(State {
        settings: None,
        zooming: false,
        steps: 0,
        motion: Motion { from: 1.0, to: 1.0, start: Instant::now(), seconds: 0.0 },
    });
}

fn with<R>(f: impl FnOnce(&mut State) -> R) -> R {
    STATE.with(|s| f(&mut s.borrow_mut()))
}

fn settings(game: &Game<'_>) -> Settings {
    if let Some(s) = with(|s| s.settings) {
        return s;
    }
    let s = Settings::read(game);
    with(|st| st.settings = Some(s));
    s
}

fn ease(transition: usize, t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    match transition {
        0 => {
            if t >= 1.0 {
                1.0
            } else {
                1.0 - 2f32.powf(-10.0 * t)
            }
        }
        1 => (t * std::f32::consts::FRAC_PI_2).sin(),
        2 => t,
        _ => 1.0,
    }
}

impl Motion {
    /// The zoom now, eased between the levels (in log space, so each
    /// doubling takes as long).
    fn now(&self, transition: usize) -> f32 {
        if self.seconds <= 0.0 || transition == 3 {
            return self.to;
        }
        let t = self.start.elapsed().as_secs_f32() / self.seconds;
        let k = ease(transition, t);
        (self.from.ln() + (self.to.ln() - self.from.ln()) * k).exp()
    }
}

/// Starts moving to the zoom the key and the steps ask for.
fn retarget(s: &mut State, opts: Settings) {
    let target = if s.zooming {
        (opts.initial * 1.25f32.powi(s.steps)).clamp(1.0, opts.max_zoom)
    } else {
        1.0
    };
    let current = s.motion.now(opts.transition);
    if (target - s.motion.to).abs() < 1e-4 {
        return;
    }
    let seconds = if target > current {
        opts.zoom_in_time
    } else {
        opts.zoom_out_time
    };
    s.motion = Motion {
        from: current,
        to: target,
        start: Instant::now(),
        seconds,
    };
}

pub(super) fn labels(game: &mut Game<'_>) -> Vec<String> {
    let s = settings(game);
    let on = |b: bool| if b { "ON" } else { "OFF" };
    vec![
        format!("Initial Zoom: {}x", s.initial),
        format!("Zoom In Time: {}s", s.zoom_in_time),
        format!("Zoom Out Time: {}s", s.zoom_out_time),
        format!("Transition: {}", TRANSITIONS[s.transition]),
        format!("Scroll Zoom: {}", on(s.scroll)),
        format!("Maximum Zoom: {}x", s.max_zoom),
        format!("Relative Sensitivity: {}%", s.sensitivity),
        format!("Zoom Key: {}", if s.toggle { "Toggle" } else { "Hold" }),
        format!("Retain Zoom Steps: {}", on(s.retain_steps)),
    ]
}

pub(super) fn click(game: &mut Game<'_>, button: usize) {
    let mut s = settings(game);
    fn next<T: PartialEq + Copy>(list: &[T], v: T) -> T {
        let i = list.iter().position(|&x| x == v).map_or(0, |i| (i + 1) % list.len());
        list[i]
    }
    match button {
        0 => s.initial = next(&INITIAL, s.initial),
        1 => s.zoom_in_time = next(&TIMES, s.zoom_in_time),
        2 => s.zoom_out_time = next(&TIMES, s.zoom_out_time),
        3 => s.transition = (s.transition + 1) % TRANSITIONS.len(),
        4 => s.scroll = !s.scroll,
        5 => s.max_zoom = next(&MAX_ZOOM, s.max_zoom),
        6 => s.sensitivity = next(&SENSITIVITY, s.sensitivity),
        7 => s.toggle = !s.toggle,
        8 => s.retain_steps = !s.retain_steps,
        _ => {}
    }
    s.write(game);
    with(|st| st.settings = Some(s));
}

pub(super) fn key(game: &mut Game<'_>, key: &str, pressed: bool) {
    if !game.native_on(ID) || !game.native_window() {
        return;
    }
    if key != game.native_mod_key(ID, KEY, DEFAULT_KEY) {
        return;
    }
    let opts = settings(game);
    let playing = game.native_playing();
    with(|s| {
        let zoom = if opts.toggle {
            if !pressed || !playing {
                return;
            }
            !s.zooming
        } else {
            pressed && playing
        };
        if zoom == s.zooming {
            return;
        }
        s.zooming = zoom;
        if !zoom && !opts.retain_steps {
            s.steps = 0;
        }
        retarget(s, opts);
    });
}

/// Scroll zoom: true when the wheel was used for it.
pub(super) fn wheel(game: &mut Game<'_>, steps: f32) -> bool {
    if !game.native_on(ID) {
        return false;
    }
    let opts = settings(game);
    if !opts.scroll {
        return false;
    }
    with(|s| {
        if !s.zooming || steps == 0.0 {
            return false;
        }
        let d = if steps > 0.0 { 1 } else { -1 };
        let max_steps = (opts.max_zoom / opts.initial).ln() / 1.25f32.ln();
        let min_steps = (1.0 / opts.initial).ln() / 1.25f32.ln();
        s.steps = (s.steps + d).clamp(min_steps.floor() as i32, max_steps.ceil() as i32);
        // Scrolling follows quickly, whatever the key's transition.
        let quick = Settings {
            zoom_in_time: opts.zoom_in_time.min(0.15),
            zoom_out_time: opts.zoom_out_time.min(0.15),
            ..opts
        };
        retarget(s, quick);
        true
    })
}

/// The zoom now (1 when not zooming); none under the spyglass, which zooms
/// on its own.
pub(super) fn factor(game: &Game<'_>) -> f32 {
    if !game.native_on(ID) || game.scoping() {
        return 1.0;
    }
    let transition = with(|s| s.settings.map_or(0, |o| o.transition));
    with(|s| s.motion.now(transition)).max(1.0)
}

/// How much slower the mouse turns while zoomed.
pub(super) fn turn(game: &Game<'_>) -> f64 {
    let zoom = factor(game);
    if zoom <= 1.0 {
        return 1.0;
    }
    let share = with(|s| s.settings.map_or(100, |o| o.sensitivity)) as f32 / 100.0;
    let scale = 1.0 / zoom;
    f64::from(1.0 + (scale - 1.0) * share.min(1.5)).max(0.01)
}
