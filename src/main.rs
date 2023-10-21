mod cpu;
mod clock;
//mod mem; not done yet, would like to find a different implementation of these shenanigans

use std::env;
use std::process;
use std::sync::Mutex;
use std::sync::Arc;

use cpu::CPU;
use clock::Clock;

use lazy_static::lazy_static;

lazy_static!{
    static ref G_CPU: Arc<Mutex<Option<CPU>>> = Arc::new(Mutex::new(Some(CPU::new())));
    static ref G_CLOCK: Arc<Mutex<Option<Clock>>> = Arc::new(Mutex::new(Some(Clock::new())));
}


pub extern "C" fn handler(env: sketchybar_rs::Env) {
    // Environment variables passed from sketchybar can be accessed as seen below
    let name = env.get_v_for_c("NAME");
    let sender = env.get_v_for_c("SENDER");
    let info = env.get_v_for_c("INFO");
    let selected = env.get_v_for_c("SELECTED");

    if selected.len() > 0 {
        // Space items
        let width = if selected == "true" {
            "0"
        } else {
            "dynamic"
        };

        let command = format!(
            "--animate tanh 20 --set {} icon.highlight={} label.width={}",
            name,
            selected,
            width
        );
        println!("{}", name);
        sketchybar_rs::message(&command).unwrap();
    } else if sender == "front_app_switched" {
        // front_app item
        let command = format!("--set {} label=\"{}\"", name, info);
        sketchybar_rs::message(&command).unwrap();
    } else if sender == "routine" || sender == "forced" {
        // CPU and Clock routine updates
        let mut temp_cpu = G_CPU.lock().unwrap();
        let cpu = temp_cpu.as_mut().unwrap();
        let mut temp_clock = G_CLOCK.lock().unwrap();
        let clock = temp_clock.as_mut().unwrap();
        cpu.update();
        clock.update();

        if !cpu.command.is_empty() && !clock.command.is_empty() {
            let command = format!("{} {}", cpu.command, clock.command);
            let _ = sketchybar_rs::message(&command);
        }
        drop(temp_clock);
        drop(temp_cpu);
    }
}

fn main() {
    let mut args = env::args();
    let program_name = args.next().unwrap();

    if args.len() < 1 {
        eprintln!("Usage: {} <bootstrap name>", program_name);
        process::exit(1);
    }

    let bootstrap_name = args.next().unwrap();
    sketchybar_rs::server_begin(handler, &bootstrap_name);
    G_CPU.lock().unwrap().take();
    G_CLOCK.lock().unwrap().take();
}

