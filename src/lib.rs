use pyo3::prelude::*;
extern crate gstreamer as gst;
use std::sync::{atomic::AtomicBool, Arc};

use gst::prelude::*;

fn record(
    filename: &str,
    caps: &CameraCaps,
    is_recording: Arc<AtomicBool>,
    stop_flag: Arc<AtomicBool>,
) {
    // Initialize GStreamer
    gst::init().unwrap();

    // Create the elements
    let source = gst::ElementFactory::make("autovideosrc").build().unwrap();
    let caps_filter = gst::ElementFactory::make("capsfilter").build().unwrap();
    let tee = gst::ElementFactory::make("tee").build().unwrap();
    // let display_queue = gst::ElementFactory::make("queue").build().unwrap();
    // let autovideoconvert = gst::ElementFactory::make("autovideoconvert")
    //     .build()
    //     .unwrap();
    let autovideosink = gst::ElementFactory::make("autovideosink").build().unwrap();
    let encoder = gst::ElementFactory::make("vtenc_h264").build().unwrap();
    let parser = gst::ElementFactory::make("h264parse").build().unwrap();
    let queue = gst::ElementFactory::make("queue").build().unwrap();
    let muxer = gst::ElementFactory::make("mpegtsmux").build().unwrap();
    let sink = gst::ElementFactory::make("filesink").build().unwrap();

    // Set properties
    caps_filter.set_property(
        "caps",
        &gst::Caps::builder("video/x-raw")
            .field("width", caps.width)
            .field("height", caps.height)
            .field(
                "framerate",
                &gst::Fraction::new(caps.framerate_numerator, caps.framerate_denominator),
            )
            .build(),
    );
    sink.set_property("location", filename.to_value());

    // Create the empty pipeline
    let pipeline = gst::Pipeline::default();

    // Build the pipeline
    pipeline
        .add_many(&[
            &source,
            //&caps_filter,
            //&tee,
            // &display_queue,
            // &autovideoconvert,
            &autovideosink,
            // &encoder,
            // &parser,
            // &queue,
            // &muxer,
            // &sink,
        ])
        .unwrap();

    // link the source to the caps filter and the tee
    //gst::Element::link_many(&[&source, &caps_filter, &tee]);

    // link source to autovideosink
    source.link(&autovideosink).unwrap();

    // // link the display pipeline
    // gst::Element::link_many(&[
    //     &source,
    //     &caps_filter,
    //     // &display_queue,
    //     // &autovideoconvert,
    //     &autovideosink,
    // ])
    // .unwrap();

    // link the recording pipeline
    //gst::Element::link_many(&[&tee, &encoder, &parser, &queue, &muxer, &sink]).unwrap();

    // Start playing
    pipeline.set_state(gst::State::Playing).unwrap();

    // Wait until error or EOS
    let bus = pipeline.bus().unwrap();
    for msg in bus.iter_timed(gst::ClockTime::NONE) {
        match msg.view() {
            gst::MessageView::Eos(..) => break,
            gst::MessageView::Error(err) => {
                println!(
                    "Error from {:?}: {}",
                    err.src().map(|s| s.path_string()),
                    err.error()
                );
                println!("Debug info: {:?}", err.debug());
                break;
            }
            gst::MessageView::StateChanged(s) => {
                println!(
                    "State change for {:?}: from {:?} to {:?} (pending: {:?})",
                    s.src().map(|s| s.path_string()),
                    s.old(),
                    s.current(),
                    s.pending()
                );

                match s.current() {
                    gst::State::Playing => {
                        // chexk if state change pertains to whole pipeline
                        if s.src().map(|s| s.path_string()).unwrap() == "/GstPipeline:pipeline0" {
                            println!("Recording started (messsage from outside)");
                            is_recording.store(true, std::sync::atomic::Ordering::Relaxed);
                        }
                    }
                    _ => {}
                }
            }
            _ => {
                println!("Other message: {:?}", msg);
                // check if recording is still required
                if stop_flag.load(std::sync::atomic::Ordering::Relaxed) {
                    break;
                }
            }
        }
    }

    // Clean up
    pipeline.set_state(gst::State::Null).unwrap();
}

#[pyclass(name = "Recorder")]
pub struct Recorder {
    is_recording: Arc<AtomicBool>,
    stop_flag: Arc<AtomicBool>,
}

#[derive(Debug, Clone)]
#[pyclass(name = "CameraCaps")]
pub struct CameraCaps {
    pub width: i32,
    pub height: i32,
    pub framerate_numerator: i32,
    pub framerate_denominator: i32,
    pub format: String,
}

#[pymethods]
impl CameraCaps {
    #[new]
    fn __new__(
        width: i32,
        height: i32,
        framerate_numerator: i32,
        framerate_denominator: i32,
        format: String,
    ) -> Self {
        CameraCaps {
            width,
            height,
            framerate_numerator,
            framerate_denominator,
            format,
        }
    }
}

#[pyfunction]
fn start_recording(caps: CameraCaps, filename: String) -> PyResult<Recorder> {
    // run record in a new thread
    let stop_flag = Arc::new(AtomicBool::new(false));
    let is_recording = Arc::new(AtomicBool::new(false));
    let stop_flag_clone = stop_flag.clone();
    let is_recording_clone = is_recording.clone();

    std::thread::spawn(move || {
        let caps = caps.clone();
        record(&filename, &caps, is_recording_clone, stop_flag_clone);
    });

    // wait for recording to start
    while !is_recording.load(std::sync::atomic::Ordering::Relaxed) {
        std::thread::sleep(std::time::Duration::from_millis(10));
    }

    Ok(Recorder {
        is_recording,
        stop_flag,
    })
}

#[pyfunction]
fn stop_recording(recorder: &Recorder) {
    recorder
        .stop_flag
        .store(false, std::sync::atomic::Ordering::Relaxed);
}

/// A Python module implemented in Rust.
#[pymodule]
fn webcam_py(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(start_recording, m)?)?;
    m.add_function(wrap_pyfunction!(stop_recording, m)?)?;
    m.add_class::<Recorder>()?;
    m.add_class::<CameraCaps>()?;
    Ok(())
}
