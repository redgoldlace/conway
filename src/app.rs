use crate::{cell::Cell, world::World};
use crossterm::cursor::{DisableBlinking, Hide};
use crossterm::{
    cursor::{EnableBlinking, MoveTo, Show},
    event::{Event, KeyCode, KeyModifiers},
    execute,
    style::{PrintStyledContent, Stylize},
    terminal::{Clear, ClearType},
};
use std::time::Duration;
use std::{error::Error, io::Write};

pub trait Component {
    type State;
    type Error;

    fn display(&self, output: &mut impl Write) -> Result<(), Self::Error>;
    fn update(self, message: Option<Event>) -> Result<Self::State, Self::Error>;
}

pub struct App<'a, T> {
    options: Options<'a, T>,
    state: State,
}

pub struct Options<'a, T> {
    pub output: &'a mut T,
    pub tick_length: Duration,
}

pub enum State {
    Scale(Scale),
    Draw(Draw),
    Simulate(Simulate),
}

pub struct Scale {
    width: usize,
    height: usize,
    updated: bool,
}

pub struct Draw {
    x: usize,
    y: usize,
    world: World,
}

pub struct Simulate {
    generation: usize,
    world: World,
}

impl<'a, T> App<'a, T>
where
    T: Write,
{
    pub fn new(options: Options<'a, T>) -> Self {
        App {
            options,
            state: State::Scale(Scale {
                updated: true,
                width: 8,
                height: 8,
            }),
        }
    }

    pub fn run(self) -> Result<(), Box<dyn Error>> {
        // This is done to get around a weird issue relating to moved values (even though the moved fields are disjoint)
        let mut state = self.state;
        let options = self.options;

        crossterm::terminal::enable_raw_mode()?;
        execute!(options.output, Clear(ClearType::All), DisableBlinking, Hide)?;

        loop {
            state.display(options.output)?;
            let event = crossterm::event::poll(options.tick_length)?
                .then(|| crossterm::event::read().ok())
                .flatten();

            match state.update(event)? {
                Some(new_state) => state = new_state,
                None => {
                    execute!(options.output, EnableBlinking, Show)?;
                    crossterm::terminal::disable_raw_mode()?;
                    std::process::exit(0)
                }
            }
        }
    }
}

impl Component for State {
    // This is `Option<State>` to represent us receiving a `Ctrl` + `C` input and needing to exit.
    type State = Option<State>;
    type Error = Box<dyn Error>;

    fn display(&self, output: &mut impl Write) -> Result<(), Self::Error> {
        execute!(output, MoveTo(0, 0))?;

        match self {
            State::Scale(scale) => scale.display(output),
            State::Draw(draw) => draw.display(output),
            State::Simulate(simulate) => simulate.display(output),
        }?;

        writeln!(
            output,
            "{} + {}: Quit",
            "Ctrl".blue().bold(),
            "C".blue().bold()
        )?;

        output.flush()?;

        Ok(())
    }

    fn update(self, message: Option<Event>) -> Result<Option<State>, Self::Error> {
        if let Some(Event::Key(press)) = message {
            // Regardless of our current state, we need to handle a `Ctrl` + `C` and exit.
            let is_ctrl = press.modifiers.contains(KeyModifiers::CONTROL);
            let is_c = matches!(press.code, KeyCode::Char('c'));

            if is_ctrl && is_c {
                return Ok(None);
            }
        }

        match self {
            State::Scale(scale) => scale.update(message),
            State::Draw(draw) => draw.update(message),
            State::Simulate(simulate) => simulate.update(message),
        }
        .map(Some)
    }
}

impl Component for Scale {
    type State = State;
    type Error = Box<dyn Error>;

    fn display(&self, output: &mut impl Write) -> Result<(), Self::Error> {
        if self.updated {
            execute!(output, Clear(ClearType::FromCursorDown))?;

            for row_index in 0..self.height {
                for _ in 0..self.width {
                    write!(output, "{}", Cell::Dead.block())?;
                }

                if row_index + 1 < self.height {
                    write!(output, "\n")?;
                }
            }

            execute!(output, MoveTo(0, (self.height + 1) as u16),)?;
            writeln!(output, "Currently in {} mode", "Scale".bold().cyan(),)?;
            writeln!(
                output,
                "The grid is currently {} cell(s) wide and {} cell(s) high",
                self.width.to_string().bold(),
                self.height.to_string().bold(),
            )?;

            writeln!(output, "{}: Change grid size", "↑↓←→".blue().bold())?;
            writeln!(output, "{}: Start drawing", "Enter".blue().bold())?;
        }

        Ok(())
    }

    fn update(mut self, message: Option<Event>) -> Result<State, Self::Error> {
        let press = match message {
            Some(Event::Key(press)) => press,
            _ => return Ok(State::Scale(self)),
        };

        self.updated = matches!(
            press.code,
            KeyCode::Up | KeyCode::Down | KeyCode::Left | KeyCode::Right
        );

        if self.updated {
            match press.code {
                KeyCode::Up => self.height = (self.height - 1).max(1),
                KeyCode::Down => self.height += 1,
                KeyCode::Left => self.width = (self.width - 1).max(1),
                KeyCode::Right => self.width += 1,
                _ => unreachable!(),
            }
        }

        let state = match press.code {
            KeyCode::Enter => State::Draw(Draw {
                x: 0,
                y: 0,
                world: World::new(self.width, self.height),
            }),
            _ => State::Scale(self),
        };

        Ok(state)
    }
}

impl Component for Draw {
    type State = State;
    type Error = Box<dyn Error>;

    fn display(&self, output: &mut impl Write) -> Result<(), Self::Error> {
        writeln!(output, "{}", self.world)?;
        execute!(
            output,
            MoveTo(self.x as u16, self.y as u16),
            PrintStyledContent(match self.world.get((self.x, self.y)).unwrap() {
                Cell::Alive => "o".green(),
                Cell::Dead => "o".red(),
            }),
            MoveTo(0, (self.world.height() + 1) as u16),
            Clear(ClearType::FromCursorDown)
        )?;

        writeln!(output, "Currently in {} mode", "Drawing".bold().yellow())?;
        writeln!(output, "{}: Flip cell under cursor", "Space".blue().bold())?;
        writeln!(output, "{}: Move cursor", "↑↓←→".blue().bold())?;
        writeln!(output, "{}: Start simulating", "Enter".blue().bold())?;

        Ok(())
    }

    fn update(mut self, message: Option<Event>) -> Result<State, Self::Error> {
        let press = match message {
            Some(Event::Key(press)) => press,
            _ => return Ok(State::Draw(self)),
        };

        match press.code {
            KeyCode::Up => self.y = self.y.saturating_sub(1),
            KeyCode::Down => self.y = (self.y + 1).min(self.world.height() - 1),
            KeyCode::Left => self.x = self.x.saturating_sub(1),
            KeyCode::Right => self.x = (self.x + 1).min(self.world.width() - 1),
            KeyCode::Char(' ') => self.world.get_mut((self.x, self.y)).unwrap().flip(),
            _ => {}
        };

        let state = match press.code {
            KeyCode::Enter => State::Simulate(Simulate {
                generation: 0,
                world: self.world,
            }),
            _ => State::Draw(self),
        };

        Ok(state)
    }
}

impl Component for Simulate {
    type State = State;
    type Error = Box<dyn Error>;

    fn display(&self, output: &mut impl Write) -> Result<(), Self::Error> {
        writeln!(output, "{}", self.world)?;
        execute!(
            output,
            MoveTo(0, (self.world.height() + 1) as u16),
            Clear(ClearType::FromCursorDown)
        )?;

        writeln!(
            output,
            "Currently in {} mode",
            "Simulation".bold().magenta()
        )?;

        writeln!(
            output,
            "Currently at generation #{}",
            self.generation.to_string().bold()
        )?;

        Ok(())
    }

    fn update(mut self, _: Option<Event>) -> Result<State, Self::Error> {
        self.world = self.world.tick();
        self.generation += 1;

        Ok(State::Simulate(self))
    }
}
