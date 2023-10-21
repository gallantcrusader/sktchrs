use chrono::prelude::*;

pub struct Clock {
    pub command: String,
}

impl Clock {
    pub fn new() -> Self {
        Clock {
            command: String::new(),
        }
    }

    pub fn update(&mut self) {
        let t: DateTime<Local> = Local::now();
        let time = t.format("%H:%M");
        let date = t.format("%a %d. %b");

        self.command = format!("--set calendar icon=\"{}\" label=\"{}\"", date, time);
    }
}
