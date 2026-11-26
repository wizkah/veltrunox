//! Stem analysis jobs (ADR-015): the external program and the queue that runs one job at a
//! time and imports what it wrote.

use crate::stem_package::StemStore;
use std::{
    collections::HashMap,
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

/// The configured program: `PROGRAM INPUT OUTPUT.zip TITLE`, with `progress N` lines on its
/// standard output.
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
        let mut child = Command::new(&self.0)
            .arg(audio)
            .arg(package)
            .arg(title)
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

type States = Arc<Mutex<HashMap<RecordingId, JobState>>>;

pub struct StemJobs {
    queue: mpsc::Sender<RecordingId>,
    states: States,
    broadcast: Arc<dyn PlaybackBroadcast>,
}

impl StemJobs {
    pub(crate) fn start(separator: Arc<dyn StemSeparator>, context: JobContext) -> Self {
        let (queue, waiting) = mpsc::channel::<RecordingId>();
        let states = States::default();
        let broadcast = context.broadcast.clone();
        let shared = states.clone();
        thread::spawn(move || {
            for id in waiting {
                set(&shared, &context, id, Some(JobState::Running(0)));
                let report = |percent| {
                    set(&shared, &context, id, Some(JobState::Running(percent)));
                };
                let outcome = run(&*separator, &context, id, &report);
                set(&shared, &context, id, outcome.err().map(JobState::Failed));
            }
        });
        Self {
            queue,
            states,
            broadcast,
        }
    }

    /// Queues the recording unless it waits or runs already.
    pub fn enqueue(&self, id: RecordingId) {
        {
            let mut states = lock(&self.states);
            if matches!(
                states.get(&id),
                Some(JobState::Queued | JobState::Running(_))
            ) {
                return;
            }
            states.insert(id, JobState::Queued);
        }
        self.broadcast.library_changed();
        // The thread only ends with the server.
        let _ = self.queue.send(id);
    }

    pub fn state(&self, id: RecordingId) -> Option<JobState> {
        lock(&self.states).get(&id).cloned()
    }
}

fn lock(states: &States) -> std::sync::MutexGuard<'_, HashMap<RecordingId, JobState>> {
    states.lock().unwrap_or_else(PoisonError::into_inner)
}

fn set(states: &States, context: &JobContext, id: RecordingId, state: Option<JobState>) {
    match state {
        Some(state) => lock(states).insert(id, state),
        None => lock(states).remove(&id),
    };
    context.broadcast.library_changed();
}

fn run(
    separator: &dyn StemSeparator,
    context: &JobContext,
    id: RecordingId,
    report: &dyn Fn(u8),
) -> Result<(), &'static str> {
    let failed = |code: &'static str| {
        move |detail: String| {
            eprintln!("stem analysis of recording {id:?}: {detail}");
            code
        }
    };
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

#[cfg(all(test, unix))]
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
