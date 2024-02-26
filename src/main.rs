mod detection;

use anyhow::Result;
use detection::Detector;
use opencv::{highgui, prelude::*, videoio};

fn main() -> Result<()> {
    let mut cam = videoio::VideoCapture::new(0, videoio::CAP_ANY)?;
    let opened = videoio::VideoCapture::is_opened(&cam)?;
    if !opened {
        return Err(anyhow::anyhow!("Unable to open default camera."));
    }

    cam.set(videoio::CAP_PROP_FRAME_WIDTH, 640.0)?;
    cam.set(videoio::CAP_PROP_FRAME_HEIGHT, 480.0)?;

    highgui::named_window("Webcam", highgui::WINDOW_NORMAL)?;

    let detector = Detector::new()?;

    loop {
        let mut frame = Mat::default();
        if !cam.read(&mut frame)? {
            break;
        }

        if frame.size()?.width > 0 {
            // Convert the `frame` into a format that your detection model expects
            // This might involve converting `frame` to a `DynamicImage`, then:

            let detections = detector.detect(frame.clone())?;

            // Display the processed frame
            highgui::imshow("Webcam", &frame)?;
        }

        if highgui::wait_key(10)? == 27 {
            break; // Exit on ESC
        }
    }

    Ok(())
}
