//! The worker thread the model runs on, and the queue in front of it.
//!
//! One thread, started with a loader that must succeed before the first job is taken. Jobs carry
//! an optional key: when the worker takes a job whose key has a newer job waiting, it answers
//! [`SUPERSEDED`] without scoring, so a mod that asks again before its last question was answered
//! only pays for the newest. A job already being scored runs to the end. The queue holds
//! [`MAX_QUEUED`]; past that `submit` refuses, so the caller can fall back at once.
//!
//! From `~/engine/game/base/src/ai/queue.rs`, with the names changed.

use super::{Answer, Question};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

pub const MAX_QUEUED: usize = 64;
pub const SUPERSEDED: &str = "superseded";
const SHUT_DOWN: &str = "the model has been shut down";

/// What the worker runs: llama.cpp in the build, a fake in tests. Scoring is what the lodger
/// decides with; writing is the few words of a sign or a chat line (`brain.text = generate`),
/// which a scorer may not support.
pub trait Scorer {
    fn score(&mut self, situation: &str, questions: &[Question]) -> Result<Vec<Answer>, String>;

    /// Continues `prompt` within `leash`: at most its tokens of letters and spaces (and digits
    /// when it allows them), stopping at a line's end. The default cannot write.
    fn generate(&mut self, prompt: &str, leash: Leash) -> Result<String, String> {
        let _ = (prompt, leash);
        Err("this model does not write".to_string())
    }
}

/// How far a written line may run: at most `max_tokens` tokens and `words` words (the decode
/// stops before a token that would start one more), and digits only when `digits` (a line
/// counting what the player killed). Letters and spaces always.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Leash {
    pub max_tokens: usize,
    pub words: usize,
    pub digits: bool,
}

pub struct Reply {
    pub id: u64,
    /// The answers of a scoring job; an empty list for a writing job.
    pub result: Result<Vec<Answer>, String>,
    /// The text of a writing job.
    pub text: Option<String>,
}

pub struct Job {
    pub id: u64,
    pub key: Option<String>,
    /// The situation to score against, or the prompt to continue.
    pub situation: String,
    pub questions: Vec<Question>,
    /// Some for a writing job: how far the line may run.
    pub generate: Option<Leash>,
    pub reply: Sender<Reply>,
}

struct Shared {
    latest: Mutex<HashMap<String, u64>>,
    queued: AtomicUsize,
    closing: AtomicBool,
}

pub struct Worker {
    tx: Mutex<Option<Sender<Job>>>,
    handle: Mutex<Option<JoinHandle<()>>>,
    shared: Arc<Shared>,
}

impl Worker {
    /// Starts the thread and runs `make` on it; answers once the scorer is made, or with its
    /// error, so a model that fails to load is the caller's error and the thread does not stay.
    pub fn start<S, F>(make: F) -> Result<Self, String>
    where
        S: Scorer,
        F: FnOnce() -> Result<S, String> + Send + 'static,
    {
        let (tx, rx) = channel::<Job>();
        let (ready_tx, ready_rx) = channel::<Result<(), String>>();
        let shared = Arc::new(Shared {
            latest: Mutex::new(HashMap::new()),
            queued: AtomicUsize::new(0),
            closing: AtomicBool::new(false),
        });
        let thread_shared = Arc::clone(&shared);
        let handle = std::thread::Builder::new()
            .name("lodger-model".to_string())
            .spawn(move || {
                let mut scorer = match make() {
                    Ok(scorer) => {
                        let _ = ready_tx.send(Ok(()));
                        scorer
                    }
                    Err(err) => {
                        let _ = ready_tx.send(Err(err));

                        return;
                    }
                };

                run(&mut scorer, rx, &thread_shared);
            })
            .map_err(|err| err.to_string())?;

        match ready_rx.recv() {
            Ok(Ok(())) => Ok(Self {
                tx: Mutex::new(Some(tx)),
                handle: Mutex::new(Some(handle)),
                shared,
            }),
            Ok(Err(err)) => {
                let _ = handle.join();

                Err(err)
            }
            Err(_) => {
                let _ = handle.join();

                Err("the model worker stopped while loading".to_string())
            }
        }
    }

    pub fn submit(&self, job: Job) -> Result<(), String> {
        if self.shared.queued.load(Ordering::SeqCst) >= MAX_QUEUED {
            return Err("the model queue is full".to_string());
        }

        let guard = self.tx.lock().unwrap_or_else(|err| err.into_inner());
        let tx = guard.as_ref().ok_or_else(|| SHUT_DOWN.to_string())?;

        if let Some(key) = &job.key {
            self.shared
                .latest
                .lock()
                .unwrap_or_else(|err| err.into_inner())
                .insert(key.clone(), job.id);
        }

        self.shared.queued.fetch_add(1, Ordering::SeqCst);

        tx.send(job).map_err(|_| {
            self.shared.queued.fetch_sub(1, Ordering::SeqCst);

            "the model worker has stopped".to_string()
        })
    }

    pub fn queued(&self) -> usize {
        self.shared.queued.load(Ordering::SeqCst)
    }

    /// Finishes the running job, answers the queued ones with the shut-down error, joins the
    /// thread. The scorer is dropped on its own thread, which llama.cpp needs.
    pub fn stop(&self) {
        self.shared.closing.store(true, Ordering::SeqCst);
        drop(self.tx.lock().unwrap_or_else(|err| err.into_inner()).take());

        if let Some(handle) = self
            .handle
            .lock()
            .unwrap_or_else(|err| err.into_inner())
            .take()
        {
            let _ = handle.join();
        }
    }
}

fn run<S: Scorer>(scorer: &mut S, rx: Receiver<Job>, shared: &Shared) {
    for job in rx {
        shared.queued.fetch_sub(1, Ordering::SeqCst);

        let stale = job.key.as_ref().is_some_and(|key| {
            let latest = shared.latest.lock().unwrap_or_else(|err| err.into_inner());

            latest.get(key) != Some(&job.id)
        });
        let mut text = None;
        let result = if shared.closing.load(Ordering::SeqCst) {
            Err(SHUT_DOWN.to_string())
        } else if stale {
            Err(SUPERSEDED.to_string())
        } else if let Some(leash) = job.generate {
            scorer.generate(&job.situation, leash).map(|written| {
                text = Some(written);
                Vec::new()
            })
        } else {
            scorer.score(&job.situation, &job.questions)
        };

        if let Some(key) = &job.key {
            let mut latest = shared.latest.lock().unwrap_or_else(|err| err.into_inner());

            if latest.get(key) == Some(&job.id) {
                latest.remove(key);
            }
        }

        // The asker may be gone (its world closed); that is fine.
        let _ = job.reply.send(Reply {
            id: job.id,
            result,
            text,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicBool;
    use std::time::Duration;

    const WAIT: Duration = Duration::from_secs(5);

    struct Fake {
        seen: Arc<Mutex<Vec<String>>>,
        gate: Option<Arc<(Mutex<bool>, std::sync::Condvar)>>,
    }

    impl Scorer for Fake {
        fn score(
            &mut self,
            situation: &str,
            questions: &[Question],
        ) -> Result<Vec<Answer>, String> {
            if let Some(gate) = &self.gate {
                let (open, signal) = &**gate;
                let mut open = open.lock().unwrap();

                while !*open {
                    open = signal.wait(open).unwrap();
                }
            }

            self.seen.lock().unwrap().push(situation.to_string());

            Ok(questions
                .iter()
                .map(|question| Answer {
                    choice: question.choices[0].clone(),
                    confidence: 0.5,
                })
                .collect())
        }
    }

    fn fake() -> (Fake, Arc<Mutex<Vec<String>>>) {
        let seen = Arc::new(Mutex::new(Vec::new()));

        (
            Fake {
                seen: Arc::clone(&seen),
                gate: None,
            },
            seen,
        )
    }

    fn question() -> Vec<Question> {
        vec![Question {
            text: "Move?".to_string(),
            choices: vec!["stay".to_string(), "go".to_string()],
        }]
    }

    fn job(id: u64, key: Option<&str>, situation: &str, reply: &Sender<Reply>) -> Job {
        Job {
            generate: None,
            id,
            key: key.map(str::to_string),
            situation: situation.to_string(),
            questions: question(),
            reply: reply.clone(),
        }
    }

    fn gate() -> Arc<(Mutex<bool>, std::sync::Condvar)> {
        Arc::new((Mutex::new(false), std::sync::Condvar::new()))
    }

    fn open(gate: &Arc<(Mutex<bool>, std::sync::Condvar)>) {
        *gate.0.lock().unwrap() = true;
        gate.1.notify_all();
    }

    #[test]
    fn answers_come_back_on_the_reply_channel_and_do_not_block_the_submitter() {
        let (scorer, _) = fake();
        let worker = Worker::start(move || Ok(scorer)).unwrap();
        let (tx, rx) = channel();

        worker.submit(job(7, None, "close", &tx)).unwrap();
        let reply = rx.recv_timeout(WAIT).unwrap();

        assert_eq!(reply.id, 7);
        assert_eq!(reply.result.unwrap()[0].choice, "stay");
        worker.stop();
    }

    #[test]
    fn a_loading_failure_is_returned_from_start() {
        struct Never;

        impl Scorer for Never {
            fn score(&mut self, _: &str, _: &[Question]) -> Result<Vec<Answer>, String> {
                unreachable!()
            }
        }

        let err = Worker::start(|| -> Result<Never, String> { Err("no model".to_string()) })
            .err()
            .unwrap();

        assert_eq!(err, "no model");
    }

    #[test]
    fn requests_with_the_same_key_keep_only_the_newest() {
        let (mut scorer, seen) = fake();
        let blocked = gate();
        scorer.gate = Some(Arc::clone(&blocked));
        let worker = Worker::start(move || Ok(scorer)).unwrap();
        let (tx, rx) = channel();

        // The first request is picked up and held at the gate; the next two queue behind it.
        worker.submit(job(1, Some("plan"), "first", &tx)).unwrap();
        std::thread::sleep(Duration::from_millis(50));
        worker.submit(job(2, Some("plan"), "second", &tx)).unwrap();
        worker.submit(job(3, Some("plan"), "third", &tx)).unwrap();
        open(&blocked);

        let mut replies: Vec<Reply> = (0..3).map(|_| rx.recv_timeout(WAIT).unwrap()).collect();
        replies.sort_by_key(|reply| reply.id);

        assert!(replies[0].result.is_ok(), "the running one finishes");
        assert_eq!(
            replies[1].result.as_ref().err().map(String::as_str),
            Some(SUPERSEDED)
        );
        assert!(replies[2].result.is_ok(), "the newest one runs");
        assert_eq!(
            *seen.lock().unwrap(),
            vec!["first".to_string(), "third".to_string()]
        );
        worker.stop();
    }

    #[test]
    fn different_keys_and_keyless_requests_all_run_in_order() {
        let (scorer, seen) = fake();
        let worker = Worker::start(move || Ok(scorer)).unwrap();
        let (tx, rx) = channel();

        worker.submit(job(1, Some("a"), "one", &tx)).unwrap();
        worker.submit(job(2, Some("b"), "two", &tx)).unwrap();
        worker.submit(job(3, None, "three", &tx)).unwrap();
        worker.submit(job(4, None, "four", &tx)).unwrap();

        for _ in 0..4 {
            assert!(rx.recv_timeout(WAIT).unwrap().result.is_ok());
        }

        assert_eq!(
            *seen.lock().unwrap(),
            ["one", "two", "three", "four"]
                .into_iter()
                .map(String::from)
                .collect::<Vec<_>>()
        );
        worker.stop();
    }

    #[test]
    fn the_queue_is_bounded() {
        let (mut scorer, _) = fake();
        let blocked = gate();
        scorer.gate = Some(Arc::clone(&blocked));
        let worker = Worker::start(move || Ok(scorer)).unwrap();
        let (tx, _rx) = channel();

        worker.submit(job(0, None, "held", &tx)).unwrap();
        std::thread::sleep(Duration::from_millis(50));

        for id in 1..=MAX_QUEUED as u64 {
            worker.submit(job(id, None, "queued", &tx)).unwrap();
        }

        let err = worker.submit(job(999, None, "overflow", &tx)).unwrap_err();

        assert!(err.contains("full"), "{err}");
        assert_eq!(worker.queued(), MAX_QUEUED);
        open(&blocked);
        worker.stop();
    }

    #[test]
    fn stop_finishes_the_running_request_and_refuses_the_rest() {
        let (mut scorer, seen) = fake();
        let blocked = gate();
        scorer.gate = Some(Arc::clone(&blocked));
        let worker = Arc::new(Worker::start(move || Ok(scorer)).unwrap());
        let (tx, rx) = channel();

        worker.submit(job(1, None, "running", &tx)).unwrap();
        std::thread::sleep(Duration::from_millis(50));
        worker.submit(job(2, None, "waiting", &tx)).unwrap();
        worker.submit(job(3, None, "also waiting", &tx)).unwrap();

        let stopping = Arc::clone(&worker);
        let stopper = std::thread::spawn(move || stopping.stop());
        std::thread::sleep(Duration::from_millis(50));
        open(&blocked);
        stopper.join().unwrap();

        assert!(
            rx.recv_timeout(WAIT).unwrap().result.is_ok(),
            "the running one finishes"
        );

        for _ in 0..2 {
            let reply = rx.recv_timeout(WAIT).unwrap();

            assert!(reply.result.unwrap_err().contains("shut down"));
        }

        assert_eq!(
            *seen.lock().unwrap(),
            vec!["running".to_string()],
            "nothing queued ran"
        );

        let err = worker.submit(job(4, None, "late", &tx)).unwrap_err();

        assert!(err.contains("shut down"), "{err}");
    }

    #[test]
    fn the_scorer_is_dropped_on_the_worker_thread_when_it_stops() {
        struct Tracked(Arc<AtomicBool>, std::thread::ThreadId);

        impl Scorer for Tracked {
            fn score(&mut self, _: &str, _: &[Question]) -> Result<Vec<Answer>, String> {
                Ok(Vec::new())
            }
        }

        impl Drop for Tracked {
            fn drop(&mut self) {
                // Freed where it was built, never on the caller's thread.
                self.0
                    .store(std::thread::current().id() == self.1, Ordering::SeqCst);
            }
        }

        let freed_in_place = Arc::new(AtomicBool::new(false));
        let flag = Arc::clone(&freed_in_place);
        let worker = Worker::start(move || Ok(Tracked(flag, std::thread::current().id()))).unwrap();
        worker.stop();

        assert!(freed_in_place.load(Ordering::SeqCst));
    }
}
