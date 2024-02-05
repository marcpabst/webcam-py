use webcam_py::prelude::*;

fn main() {
    let caps = CameraCaps {
        width: 1280,
        height: 720,
        framerate_numerator: 30,
        framerate_denominator: 1,
        format: "NV12".to_string(),
    };
    let recorder = start_recording(caps, "output.mp4".to_string()).unwrap();
    std::thread::sleep(std::time::Duration::from_secs(5));
    stop_recording(&recorder);
}
