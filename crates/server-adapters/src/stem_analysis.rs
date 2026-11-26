//! Stem analysis jobs (ADR-015): the external program and the queue that runs one job at a
//! time and imports what it wrote.

use crate::stem_package::{StemStore, read_structure};
use std::{
    collections::HashMap,
    ffi::OsString,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
    process::{ChildStdout, Command, Stdio},
    sync::{Arc, Mutex, PoisonError, mpsc},
    thread,
    time::Duration,
};
use veltrunox_core::RecordingId;
use veltrunox_server_app::{
    MediaCatalog, PlaybackBroadcast, PlaybackService, RecordingRepository, StemRepository,
    StemSeparator, StemService,
};

/// The configured program: `PROGRAM INPUT OUTPUT.zip TITLE` splits, `PROGRAM --structure
/// OUTPUT.json --stem NAME=FILE ...` finds the structure; `progress N` lines on its standard
/// output.
pub struct ProgramSeparator(PathBuf);

impl ProgramSeparator {
    pub fn new(program: &Path) -> Self {
        Self(program.to_owned())
    }
}

impl StemSeparator for ProgramSeparator {
    fn separate(
        &self,
        audio: &Path,
        title: &str,
        package: &Path,
        report: &dyn Fn(u8),
    ) -> Result<(), String> {
        let mut command = Command::new(&self.0);
        command.arg(audio).arg(package).arg(title);
        self.run(command, report)
    }

    fn find_structure(
        &self,
        stems: &[(&str, &Path)],
        output: &Path,
        report: &dyn Fn(u8),
    ) -> Result<(), String> {
        let mut command = Command::new(&self.0);
        command.arg("--structure").arg(output);
        for (name, path) in stems {
            let mut stem = OsString::from(format!("{name}="));
            stem.push(path);
            command.arg("--stem").arg(stem);
        }
        self.run(command, report)
    }
}

impl ProgramSeparator {
    fn run(&self, mut command: Command, report: &dyn Fn(u8)) -> Result<(), String> {
        let mut child = command
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .spawn()
            .map_err(|error| format!("cannot start {}: {error}", self.0.display()))?;
        let progress = child.stdout.take().map(progress_of);
        // The program's end decides, not the end of its output: that can stay open after it,
        // held by something the program started.
        let status = loop {
            if let Some(status) = child.try_wait().map_err(|error| error.to_string())? {
                break status;
            }
            match progress.as_ref().map(|lines| lines.recv_timeout(POLL)) {
                Some(Ok(percent)) => report(percent),
                Some(Err(mpsc::RecvTimeoutError::Timeout)) => {}
                Some(Err(mpsc::RecvTimeoutError::Disconnected)) | None => thread::sleep(POLL),
            }
        };
        status
            .success()
            .then_some(())
            .ok_or_else(|| format!("the program ended with {status}"))
    }
}

/// How often the end of the program is looked for.
const POLL: Duration = Duration::from_millis(200);

/// The percentages in the program's output, read by a thread of its own.
fn progress_of(output: ChildStdout) -> mpsc::Receiver<u8> {
    let (sender, receiver) = mpsc::channel();
    thread::spawn(move || {
        BufReader::new(output)
            .lines()
            .map_while(Result::ok)
            .filter_map(|line| line.strip_prefix("progress ")?.trim().parse::<u8>().ok())
            // Once nobody listens the job is over.
            .try_for_each(|percent| sender.send(percent.min(100)))
    });
    receiver
}

/// What a job does for its recording.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StemTask {
    /// Splits the recording into stems.
    Split,
    /// Creates regions from the structure found in the stems.
    Regions,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Job {
    pub task: StemTask,
    pub state: JobState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JobState {
    Queued,
    Running(u8),
    /// The code of the reason, for the interface.
    Failed(&'static str),
}

/// What a job needs from the server.
pub(crate) struct JobContext {
    pub recordings: Arc<dyn RecordingRepository>,
    pub media: Arc<dyn MediaCatalog>,
    pub stem_repository: Arc<dyn StemRepository>,
    pub stems: Arc<StemService>,
    pub store: Arc<StemStore>,
    pub playback: Arc<PlaybackService>,
    pub broadcast: Arc<dyn PlaybackBroadcast>,
}

type States = Arc<Mutex<HashMap<RecordingId, Job>>>;

pub struct StemJobs {
    queue: mpsc::Sender<(RecordingId, StemTask)>,
    states: States,
    broadcast: Arc<dyn PlaybackBroadcast>,
}

impl StemJobs {
    pub(crate) fn start(separator: Arc<dyn StemSeparator>, context: JobContext) -> Self {
        let (queue, waiting) = mpsc::channel::<(RecordingId, StemTask)>();
        let states = States::default();
        let broadcast = context.broadcast.clone();
        let shared = states.clone();
        thread::spawn(move || {
            for (id, task) in waiting {
                let running = |percent| Job {
                    task,
                    state: JobState::Running(percent),
                };
                set(&shared, &context, id, Some(running(0)));
                let report = |percent| set(&shared, &context, id, Some(running(percent)));
                let outcome = match task {
                    StemTask::Split => split(&*separator, &context, id, &report),
                    StemTask::Regions => create_regions(&*separator, &context, id, &report),
                };
                let failed = outcome.err().map(|code| Job {
                    task,
                    state: JobState::Failed(code),
                });
                set(&shared, &context, id, failed);
            }
        });
        Self {
            queue,
            states,
            broadcast,
        }
    }

    /// Queues the job unless one for the recording waits or runs already.
    pub fn enqueue(&self, id: RecordingId, task: StemTask) {
        {
            let mut states = lock(&self.states);
            if matches!(
                states.get(&id),
                Some(Job {
                    state: JobState::Queued | JobState::Running(_),
                    ..
                })
            ) {
                return;
            }
            let queued = Job {
                task,
                state: JobState::Queued,
            };
            states.insert(id, queued);
        }
        self.broadcast.library_changed();
        // The thread only ends with the server.
        let _ = self.queue.send((id, task));
    }

    pub fn state(&self, id: RecordingId) -> Option<Job> {
        lock(&self.states).get(&id).cloned()
    }
}

fn lock(states: &States) -> std::sync::MutexGuard<'_, HashMap<RecordingId, Job>> {
    states.lock().unwrap_or_else(PoisonError::into_inner)
}

fn set(states: &States, context: &JobContext, id: RecordingId, state: Option<Job>) {
    match state {
        Some(state) => lock(states).insert(id, state),
        None => lock(states).remove(&id),
    };
    context.broadcast.library_changed();
}

fn failed(code: &'static str) -> impl Fn(String) -> &'static str {
    move |detail| {
        eprintln!("stem job: {detail}");
        code
    }
}

fn split(
    separator: &dyn StemSeparator,
    context: &JobContext,
    id: RecordingId,
    report: &dyn Fn(u8),
) -> Result<(), &'static str> {
    let audio = context
        .media
        .media_path(id)
        .map_err(|error| failed("storage_failed")(error.to_string()))?
        .ok_or("audio_file_missing")?;
    let title = context
        .recordings
        .list()
        .map_err(|error| failed("storage_failed")(error.to_string()))?
        .into_iter()
        .find(|recording| recording.id == id)
        .ok_or("recording_unknown")?
        .title;
    let output = context.store.temporary_file("zip");
    separator
        .separate(&audio, title.as_str(), output.path(), report)
        .map_err(failed("stems_analysis_failed"))?;
    let package = context
        .store
        .read(output.path())
        .map_err(|error| failed("stems_package_invalid")(error.to_string()))?;
    let installed = {
        let _guard = context.store.lock();
        let set = context
            .store
            .keep(&package)
            .map_err(|error| failed("storage_failed")(error.to_string()))?;
        context
            .stems
            .install(&package.content_hash, &set, &package.structure)
    };
    let _ = context.store.sweep(context.stem_repository.as_ref());
    let installed =
        installed.map_err(|error| failed("stems_analysis_failed")(error.to_string()))?;
    context
        .playback
        .reload_mix(installed.recording.id)
        .map_err(|error| failed("storage_failed")(error.to_string()))
}

fn create_regions(
    separator: &dyn StemSeparator,
    context: &JobContext,
    id: RecordingId,
    report: &dyn Fn(u8),
) -> Result<(), &'static str> {
    let set = context
        .stems
        .stem_set(id)
        .map_err(|error| failed("storage_failed")(error.to_string()))?;
    let stems: Vec<(&str, &Path)> = set
        .stems
        .iter()
        .map(|stem| (stem.name.as_str(), stem.path.as_path()))
        .collect();
    let output = context.store.temporary_file("json");
    separator
        .find_structure(&stems, output.path(), report)
        .map_err(failed("stems_analysis_failed"))?;
    let analysis = std::fs::read_to_string(output.path())
        .map_err(|error| failed("stems_analysis_failed")(error.to_string()))?;
    let structure = read_structure(&analysis)
        .map_err(|error| failed("stems_analysis_failed")(error.to_string()))?;
    context
        .stems
        .add_structure(id, &structure, &analysis)
        .map(|_| ())
        .map_err(|error| failed("stems_analysis_failed")(error.to_string()))
}

#[cfg(test)]
#[cfg(unix)]
mod tests {
    use super::*;
    use std::{os::unix::fs::PermissionsExt, time::Instant};

    fn script(name: &str, body: &str) -> PathBuf {
        let path = std::env::temp_dir().join(format!("veltrunox-{name}-{}", std::process::id()));
        std::fs::write(&path, format!("#!/bin/sh\n{body}\n")).unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
        path
    }

    fn run(program: &Path) -> Result<(), String> {
        let unused = Path::new("unused");
        ProgramSeparator::new(program).separate(unused, "title", unused, &|_| {})
    }

    #[test]
    fn the_end_of_the_program_counts_even_if_its_output_stays_open() {
        let program = script("lingering", "sleep 20 &\necho progress 95");
        let started = Instant::now();
        let outcome = run(&program);
        std::fs::remove_file(program).unwrap();
        assert_eq!(outcome, Ok(()));
        assert!(started.elapsed() < Duration::from_secs(10));
    }

    #[test]
    fn a_failing_program_is_an_error() {
        let program = script("failing", "exit 3");
        let outcome = run(&program);
        std::fs::remove_file(program).unwrap();
        assert!(outcome.is_err());
    }
}
