use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use crossbeam::channel::Sender;
use polling::Poller;

use crate::event::Event;
use crate::message::Message;

const PUBLISHER_STREAM_POLL_KEY: usize = 0;
const POLL_TIMEOUT_MS: u64 = 300;

pub struct PublisherHandler {
    handler_thread: Option<JoinHandle<()>>,
    terminate: Arc<Mutex<bool>>,
}

impl PublisherHandler {
    pub fn new(stream: TcpStream, event_sender: Sender<Event>) -> Self {
        let terminate = Arc::new(Mutex::new(false));
        Self {
            handler_thread: Some(Self::start_handler_thread(
                stream,
                event_sender,
                terminate.clone(),
            )),
            terminate,
        }
    }

    fn start_handler_thread(
        stream: TcpStream,
        event_sender: Sender<Event>,
        terminate: Arc<Mutex<bool>>,
    ) -> JoinHandle<()> {
        thread::spawn(move || Self::handle_publisher(stream, event_sender, terminate))
    }

    fn handle_publisher(
        mut stream: TcpStream,
        event_sender: Sender<Event>,
        terminate: Arc<Mutex<bool>>,
    ) {
        log::info!("Handling publisher: [{}]", stream.peer_addr().unwrap());

        // Setup polling.
        let poller = match Poller::new() {
            Ok(poller) => poller,
            Err(e) => {
                log::error!(
                    "Failed to create a Poller for publisher [{}]: [{}]",
                    stream.peer_addr().unwrap(),
                    e
                );
                return;
            }
        };

        // Add the publisher stream to the poller.
        if let Err(e) = poller.add(&stream, polling::Event::readable(PUBLISHER_STREAM_POLL_KEY)) {
            log::error!(
                "Failed to add the publisher stream [{}] to the poller: [{}]",
                stream.peer_addr().unwrap(),
                e
            );
            return;
        }

        // Receive messages from the publisher.
        log::info!("Receiving messages from: [{}]", stream.peer_addr().unwrap());
        let mut poll_events: Vec<polling::Event> = Vec::new();
        while !(*terminate.lock().unwrap()) {
            // Modify the poller's interest in the publisher's stream.
            // This is required to receive multiple read events on macOS.
            if let Err(e) =
                poller.modify(&stream, polling::Event::readable(PUBLISHER_STREAM_POLL_KEY))
            {
                log::error!(
                    "Failed to modify the publisher stream [{}] to the poller: [{}]",
                    stream.peer_addr().unwrap(),
                    e
                );
                break;
            }

            // Clear all previous poll events.
            poll_events.clear();

            // Wait for at least one I/O event or a timeout.
            let poll_events_number = match poller.wait(
                &mut poll_events,
                Some(Duration::from_millis(POLL_TIMEOUT_MS)),
            ) {
                Ok(number) => number,
                Err(e) => {
                    log::error!(
                        "Failed polling for events from [{}]: [{}]",
                        stream.peer_addr().unwrap(),
                        e,
                    );
                    continue;
                }
            };

            // Check if timeout has been reached.
            if 0 == poll_events_number {
                continue;
            }

            // Receive a message from the publisher.
            let message = match Message::read(&mut stream) {
                Ok(message) => message,
                Err(e) => {
                    log::error!(
                        "Error receiving message from [{}]: [{}]",
                        stream.peer_addr().unwrap(),
                        e,
                    );
                    break;
                }
            };

            // Send a Publish event.
            if let Err(e) = event_sender.send(Event::Publish(message)) {
                log::error!(
                    "Failed sending Publish event from [{}]: [{}]",
                    stream.peer_addr().unwrap(),
                    e,
                )
            }
        }
    }
}

impl Drop for PublisherHandler {
    fn drop(&mut self) {
        if let Some(thread) = self.handler_thread.take() {
            // Indicate the handler thread that it should terminate.
            *self.terminate.lock().unwrap() = true;

            // Join the handler thread.
            thread.join().unwrap();
        }
    }
}
