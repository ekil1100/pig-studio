use shared_kernel::{AppError, AppResult};
use std::{
    collections::BTreeMap,
    io::{BufRead, BufReader},
    path::Path,
    process::{Command, Stdio},
    sync::mpsc,
    thread,
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProcessOutput {
    pub stdout_lines: Vec<String>,
    pub stderr_lines: Vec<String>,
    pub exit_code: Option<i32>,
}

pub trait StreamingProcessSink {
    fn stdout_line(&mut self, line: String) -> AppResult<()>;
    fn stderr_line(&mut self, line: String) -> AppResult<()>;
}

pub trait ProcessRunner {
    fn run(
        &self,
        program: &Path,
        args: &[String],
        cwd: &Path,
        env: &BTreeMap<String, String>,
    ) -> AppResult<ProcessOutput>;

    fn run_streaming(
        &self,
        program: &Path,
        args: &[String],
        cwd: &Path,
        env: &BTreeMap<String, String>,
        sink: &mut dyn StreamingProcessSink,
    ) -> AppResult<Option<i32>>;
}

#[derive(Debug, Default, Clone, Copy)]
pub struct StdProcessRunner;

impl ProcessRunner for StdProcessRunner {
    fn run(
        &self,
        program: &Path,
        args: &[String],
        cwd: &Path,
        env: &BTreeMap<String, String>,
    ) -> AppResult<ProcessOutput> {
        let output = Command::new(program)
            .args(args)
            .current_dir(cwd)
            .envs(env)
            .output()
            .map_err(|error| AppError::External(error.to_string()))?;

        Ok(ProcessOutput {
            stdout_lines: String::from_utf8_lossy(&output.stdout)
                .lines()
                .map(ToOwned::to_owned)
                .collect(),
            stderr_lines: String::from_utf8_lossy(&output.stderr)
                .lines()
                .map(ToOwned::to_owned)
                .collect(),
            exit_code: output.status.code(),
        })
    }

    fn run_streaming(
        &self,
        program: &Path,
        args: &[String],
        cwd: &Path,
        env: &BTreeMap<String, String>,
        sink: &mut dyn StreamingProcessSink,
    ) -> AppResult<Option<i32>> {
        let mut child = Command::new(program)
            .args(args)
            .current_dir(cwd)
            .envs(env)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|error| AppError::External(error.to_string()))?;

        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| AppError::External("failed to capture stdout".into()))?;
        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| AppError::External("failed to capture stderr".into()))?;

        enum StreamMessage {
            Stdout(String),
            Stderr(String),
        }

        let (sender, receiver) = mpsc::channel();
        let stdout_sender = sender.clone();
        let stdout_reader = thread::spawn(move || -> Result<(), String> {
            let reader = BufReader::new(stdout);
            for line in reader.lines() {
                let line = line.map_err(|error| error.to_string())?;
                stdout_sender
                    .send(StreamMessage::Stdout(line))
                    .map_err(|error| error.to_string())?;
            }
            Ok(())
        });
        let stderr_reader = thread::spawn(move || -> Result<(), String> {
            let reader = BufReader::new(stderr);
            for line in reader.lines() {
                let line = line.map_err(|error| error.to_string())?;
                sender
                    .send(StreamMessage::Stderr(line))
                    .map_err(|error| error.to_string())?;
            }
            Ok(())
        });

        for message in receiver {
            match message {
                StreamMessage::Stdout(line) => sink.stdout_line(line)?,
                StreamMessage::Stderr(line) => sink.stderr_line(line)?,
            }
        }

        match stdout_reader.join() {
            Ok(Ok(())) => {}
            Ok(Err(error)) => return Err(AppError::External(error)),
            Err(_) => return Err(AppError::External("stdout reader thread panicked".into())),
        }
        match stderr_reader.join() {
            Ok(Ok(())) => {}
            Ok(Err(error)) => return Err(AppError::External(error)),
            Err(_) => return Err(AppError::External("stderr reader thread panicked".into())),
        }

        let status = child
            .wait()
            .map_err(|error| AppError::External(error.to_string()))?;
        Ok(status.code())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        path::PathBuf,
        time::{Duration, Instant},
    };

    struct RecordingSink {
        started_at: Instant,
        stdout_lines: Vec<String>,
        stderr_lines: Vec<String>,
        first_stdout_at: Option<Duration>,
    }

    impl RecordingSink {
        fn new(started_at: Instant) -> Self {
            Self {
                started_at,
                stdout_lines: Vec::new(),
                stderr_lines: Vec::new(),
                first_stdout_at: None,
            }
        }
    }

    impl StreamingProcessSink for RecordingSink {
        fn stdout_line(&mut self, line: String) -> AppResult<()> {
            if self.first_stdout_at.is_none() {
                self.first_stdout_at = Some(self.started_at.elapsed());
            }
            self.stdout_lines.push(line);
            Ok(())
        }

        fn stderr_line(&mut self, line: String) -> AppResult<()> {
            self.stderr_lines.push(line);
            Ok(())
        }
    }

    fn shell_command(script: &str) -> (PathBuf, Vec<String>) {
        #[cfg(windows)]
        {
            (PathBuf::from("cmd"), vec!["/C".into(), script.into()])
        }

        #[cfg(not(windows))]
        {
            (PathBuf::from("/bin/sh"), vec!["-c".into(), script.into()])
        }
    }

    fn delayed_stdout_command() -> (PathBuf, Vec<String>) {
        #[cfg(windows)]
        {
            shell_command("echo first && ping 127.0.0.1 -n 2 >nul && echo second")
        }

        #[cfg(not(windows))]
        {
            shell_command("printf 'first\\n'; sleep 0.4; printf 'second\\n'")
        }
    }

    fn stdout_stderr_command() -> (PathBuf, Vec<String>) {
        #[cfg(windows)]
        {
            shell_command("echo out && echo err 1>&2")
        }

        #[cfg(not(windows))]
        {
            shell_command("printf 'out\\n'; printf 'err\\n' 1>&2")
        }
    }

    #[test]
    fn run_streaming_emits_stdout_before_process_exit() {
        let runner = StdProcessRunner;
        let cwd = std::env::current_dir().expect("cwd");
        let env = BTreeMap::new();
        let (program, args) = delayed_stdout_command();
        let started_at = Instant::now();
        let mut sink = RecordingSink::new(started_at);

        let exit_code = runner
            .run_streaming(&program, &args, &cwd, &env, &mut sink)
            .expect("streaming run");
        let total_elapsed = started_at.elapsed();

        assert_eq!(exit_code, Some(0));
        assert_eq!(sink.stdout_lines, vec!["first", "second"]);
        assert!(total_elapsed >= Duration::from_millis(350));
        assert!(
            sink.first_stdout_at.expect("first stdout timestamp") < Duration::from_millis(250),
            "stdout was not emitted until after the process had nearly finished"
        );
    }

    #[test]
    fn run_streaming_captures_stdout_and_stderr() {
        let runner = StdProcessRunner;
        let cwd = std::env::current_dir().expect("cwd");
        let env = BTreeMap::new();
        let (program, args) = stdout_stderr_command();
        let mut sink = RecordingSink::new(Instant::now());

        let exit_code = runner
            .run_streaming(&program, &args, &cwd, &env, &mut sink)
            .expect("streaming run");

        assert_eq!(exit_code, Some(0));
        assert_eq!(sink.stdout_lines, vec!["out"]);
        assert_eq!(sink.stderr_lines, vec!["err"]);
    }
}
