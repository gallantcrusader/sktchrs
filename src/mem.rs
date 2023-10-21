use std::io;

static TOPPROC: &str = "/bin/ps -Aceo pid,pmem,comm -r";
static FILTER_PATTERN: &str = "com.apple.";

fn get_top_process(command: &str, filter_pattern: &str) -> Result<String, io::Error> {
    let output = std::process::Command::new("sh")
        .arg("-c")
        .arg(command)
        .output()?;

    let output_str = String::from_utf8_lossy(&output.stdout);

    let top_process = output_str.lines().nth(2).unwrap_or("");
    let top_process = top_process
        .splitn(2, filter_pattern)
        .last()
        .unwrap_or(top_process);

    Ok(top_process.to_string())
}
