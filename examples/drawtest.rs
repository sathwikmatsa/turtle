extern crate piston_window;

extern crate turtleide;

use canvas::{TurtleCanvas, Command, Radians};

fn main() {
    let canvas = TurtleCanvas::new();

    let dashed = false;
    for i in 0..361 {
        if dashed {
            canvas.apply(Command::Pen {
                enabled: i % 2 == 0,
            });
        }
        canvas.apply(Command::Move {
            distance: 2.,
        });
        canvas.apply(Command::Rotate {
            angle: Radians(1f64.to_radians()),
            clockwise: true,
        });
    }
}

mod canvas {
    use std::thread;
    use std::process;
    use std::time::{Instant, Duration};
    use std::sync::{Mutex, MutexGuard, Arc, TryLockError};
    use std::ops::{Add, Sub, Mul, Div, Rem, Neg};
    use std::f64::consts::PI;

    use piston_window::*;

    use turtleide::Speed;

    #[derive(Default, Clone, Copy, Debug, PartialOrd, PartialEq)]
    pub struct Radians(pub f64);

    impl Radians {
        pub fn cos(self) -> f64 {
            self.0.cos()
        }

        pub fn sin(self) -> f64 {
            self.0.sin()
        }
    }

    impl Add for Radians {
        type Output = Self;

        fn add(self, other: Self) -> Self {
            Radians(self.0 + other.0)
        }
    }

    impl Sub for Radians {
        type Output = Self;

        fn sub(self, other: Self) -> Self {
            Radians(self.0 - other.0)
        }
    }

    impl Mul<f64> for Radians {
        type Output = Self;

        fn mul(self, other: f64) -> Self {
            Radians(self.0 * other)
        }
    }

    impl Div<f64> for Radians {
        type Output = Self;

        fn div(self, other: f64) -> Self {
            Radians(self.0 / other)
        }
    }

    impl Rem<f64> for Radians {
        type Output = Self;

        fn rem(self, other: f64) -> Self {
            Radians(self.0 % other)
        }
    }

    impl Neg for Radians {
        type Output = Self;

        fn neg(self) -> Self {
            Radians(-self.0)
        }
    }

    trait ToCanvasCoordinates : Copy {
        fn to_canvas_coords(self, center: [f64; 2]) -> Self;
    }

    impl ToCanvasCoordinates for [f64; 2] {
        /// Transforms the given local coordinate into a point that can be drawn on the canvas.
        ///
        /// Takes into account the direction of the axis and center when converting
        /// `local` from cartesian coordinates.
        ///
        /// Origin in window is the top left corner and the y-axis goes down instead of up.
        fn to_canvas_coords(self, center: [f64; 2]) -> [f64; 2] {
            [center[0] + self[0], center[1] - self[1]]
        }
    }

    trait AsMillis {
        fn as_millis(&self) -> u64;
    }

    impl AsMillis for Duration {
        fn as_millis(&self) -> u64 {
            self.as_secs() * 1000 + (self.subsec_nanos() / 1_000_000) as u64
        }
    }

    pub enum Command {
        Move {
            distance: f64,
        },
        Rotate {
            angle: Radians,
            clockwise: bool,
        },
        Pen {
            enabled: bool,
        },
    }

    #[derive(Debug, Clone, Copy)]
    pub enum Color {
        Black,
        White,
        Transparent,
    }

    impl From<Color> for types::Color {
        fn from(color: Color) -> Self {
            match color {
                Color::Black => [0., 0., 0., 255.],
                Color::White => [255., 255., 255., 255.],
                Color::Transparent => [0., 0., 0., 0.],
            }
        }
    }

    #[derive(Debug, Clone)]
    struct Path {
        start: [f64; 2],
        end: [f64; 2],
        pen: Pen,
    }

    type Response = Result<(), ()>;

    struct TurtleState {
        pub position: [f64; 2],
        pub heading: Radians,
        pub speed: Speed,
    }

    impl Default for TurtleState {
        fn default() -> TurtleState {
            TurtleState {
                position: Default::default(),
                heading: Radians(90f64.to_radians()),
                speed: Speed::Instant,
            }
        }
    }

    struct DrawingState {
        pen: Pen,
        //pub background: Color,
    }

    #[derive(Debug, Clone)]
    struct Pen {
        pub enabled: bool,
        pub thickness: f64,
        pub color: Color,
    }

    struct Animation {
        kind: AnimationKind,
        speed: Speed,
        start: Instant,
    }

    enum AnimationKind {
        Move {
            path: Path,
        },
        Rotation {
            target_angle: Radians,
            clockwise: bool,
        },
    }

    pub struct TurtleCanvas {
        thread_handle: Option<thread::JoinHandle<()>>,
        command: Arc<Mutex<Option<Command>>>,
    }

    impl TurtleCanvas {
        pub fn new() -> TurtleCanvas {
            let command = Arc::new(Mutex::new(None));

            let command_queue = command.clone();
            let handle = thread::spawn(move || {
                let mut window: PistonWindow = WindowSettings::new(
                    "Turtle IDE", [800, 600]
                ).exit_on_esc(true).build().unwrap();
                let mut paths: Vec<Path> = Vec::new();
                let mut animation: Option<Animation> = None;
                let mut turtle = TurtleState::default();
                let mut drawing = DrawingState {
                    pen: Pen {
                        enabled: true,
                        thickness: 1.,
                        color: Color::Black,
                    },
                };
                // Keeps the lock until the animation is complete
                let mut lock: Option<MutexGuard<_>> = None;
                let mut pending_command: Option<Command> = None;

                while let Some(e) = window.next() {
                    match command_queue.try_lock() {
                        Ok(mut command_lock) => {
                            pending_command = command_lock.take();
                            if pending_command.is_some() {
                                // Start the animation and block until it is complete
                                println!("acquired", );
                                lock = Some(command_lock);
                            }
                        },
                        Ok(_) |
                        Err(TryLockError::WouldBlock) => {}, // Do nothing
                        Err(TryLockError::Poisoned(..)) => break, // Quit
                    }

                    if let Some(command) = pending_command.take() {
                        if animation.is_some() {
                            unreachable!("The main thread did not wait for the animation to complete before sending another command")
                        }
                        match command {
                            Command::Move {distance} => {
                                if distance != 0. {
                                    let start = turtle.position;
                                    let x = distance * turtle.heading.cos();
                                    let y = distance * turtle.heading.sin();
                                    let end = math::add(start, [x, y]);
                                    animation = Some(Animation {
                                        kind: AnimationKind::Move {
                                            path: Path {
                                                start, end,
                                                pen: drawing.pen.clone(),
                                            },
                                        },
                                        speed: turtle.speed,
                                        start: Instant::now(),
                                    });
                                }
                            },
                            Command::Rotate {angle, clockwise} => {
                                if angle != Radians(0.) {
                                    let target_angle = turtle.heading + if clockwise {
                                        -angle
                                    }
                                    else {
                                        angle
                                    };
                                    assert!(target_angle != turtle.heading);
                                    animation = Some(Animation {
                                        kind: AnimationKind::Rotation {
                                            target_angle,
                                            clockwise,
                                        },
                                        speed: turtle.speed,
                                        start: Instant::now(),
                                    });
                                }
                            },
                            Command::Pen {enabled} => {
                                drawing.pen.enabled = enabled;
                                println!("released", );
                                lock.take();
                            },
                        }
                    }

                    window.draw_2d(&e, |c, g| {
                        clear([1.0; 4], g);

                        let view = c.get_view_size();
                        let width = view[0] as f64;
                        let height = view[1] as f64;
                        let center = [width * 0.5, height * 0.5];

                        let mut animation_complete = false;
                        if let Some(Animation {ref kind, ref speed, ref start}) = animation {
                            let elapsed = start.elapsed().as_millis() as f64;

                            match *kind {
                                AnimationKind::Move {ref path} => {
                                    // This code finds the point on the line between start and
                                    // end that we are supposed to be at right now using some
                                    // vector math.
                                    //
                                    // The basic idea is to find the vector that represents the
                                    // line between start and end, normalize it into a direction,
                                    // and then scale that vector to be the size that it should be
                                    // at the elapsed time based on the speed.
                                    let &Path {start, end, ref pen} = path;
                                    let speed = speed.to_absolute(); // px per second
                                    let path_line = math::sub(start, end);
                                    let path_length = math::square_len(path_line).sqrt();
                                    let direction = math::mul_scalar(path_line, 1./path_length);
                                    debug_assert_eq!((math::square_len(direction).sqrt() * 1000.).round(), 1000.);
                                    // (px / sec) * (sec / 1000ms) * ms => px
                                    let offset = math::mul_scalar(direction, speed / 1000. * elapsed);
                                    let offset_length = math::square_len(offset).sqrt();

                                    let current = if speed.is_infinite() || offset_length >= path_length {
                                        paths.push(path.clone());
                                        animation_complete = true;
                                        end
                                    }
                                    else {
                                        math::add(start, offset)
                                    };

                                    turtle.position = current;

                                    let &Pen {thickness, color, enabled} = pen;
                                    if enabled {
                                        let start = start.to_canvas_coords(center);
                                        let current = current.to_canvas_coords(center);

                                        line(color.into(), thickness,
                                            [start[0], start[1], current[0], current[1]],
                                            c.transform, g);
                                    }
                                },
                                AnimationKind::Rotation {target_angle, clockwise} => {
                                    // TODO: Use the returned value in Radians and impl Div for Radians
                                    let speed = Radians(speed.to_rotation());
                                    let target_angle = target_angle % (2.*PI);

                                    let heading = turtle.heading;
                                    let current = (heading + speed / 1000. * elapsed) % (2.*PI);
                                    let current = if speed.0.is_infinite() || current > target_angle {
                                        animation_complete = true;
                                        target_angle
                                    }
                                    else {
                                        let current = if clockwise {
                                            Radians(2.*PI) - current
                                        } else { current };

                                        current
                                    };
                                    assert!(!current.0.is_nan(), "heading became NaN");
                                    turtle.heading = current;
                                },
                            }
                        }
                        if animation_complete {
                            animation = None;
                            println!("released", );
                            lock.take();
                        }

                        for path in &paths {
                            if !path.pen.enabled {
                                continue;
                            }

                            let Path {start, end, ref pen} = *path;
                            let start = start.to_canvas_coords(center);
                            let end = end.to_canvas_coords(center);

                            line(pen.color.into(), pen.thickness,
                                [start[0], start[1], end[0], end[1]],
                                c.transform, g);
                        }

                        // Draw the turtle's shell
                        let cos = turtle.heading.cos();
                        let sin = turtle.heading.sin();
                        let turtle_x = turtle.position[0];
                        let turtle_y = turtle.position[1];
                        let shell: Vec<_> = [
                            [0., 15.],
                            [10., 0.],
                            [0., -15.],
                        ].into_iter().map(|pt| {
                            // Rotate each point by the heading and add the current turtle position
                            let x = cos * pt[0] - sin * pt[1] + turtle_x;
                            let y = sin * pt[0] + cos * pt[1] + turtle_y;
                            [x, y].to_canvas_coords(center)
                        }).collect();

                        polygon(Color::White.into(), &shell, c.transform, g);
                        for i in 0..shell.len() {
                            let start = shell[i];
                            let end = shell[(i + 1) % shell.len()];

                            line(Color::Black.into(), 1.,
                                [start[0], start[1], end[0], end[1]],
                                c.transform, g);
                        }
                    });
                }
            });

            Self {
                thread_handle: Some(handle),
                command,
            }
        }

        pub fn apply(&self, command: Command) {
            self.command.lock().map(|mut cmd| {
                println!("{:?}", cmd.is_some());
                *cmd = Some(command);
                println!("placed command", );
            }).map_err(|_| {
                // The connection has been closed so the window was closed
                // or an error occurred on that thread
                process::exit(0);
            });
        }
    }

    impl Drop for TurtleCanvas {
        fn drop(&mut self) {
            if let Some(handle) = self.thread_handle.take() {
                handle.join().unwrap();
            }
        }
    }
}
