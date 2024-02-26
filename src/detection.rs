use ndarray::{s, Array4, Axis};
use opencv::{
    core::{Size, CV_32F, CV_32FC3},
    imgproc::{self, resize},
    prelude::*,
};
use ort::{inputs, Error, Session, SessionOutputs};

const MODEL_PATH: &str = "onnx_models/yolov8m.onnx";
#[rustfmt::skip]
const YOLOV8_CLASS_LABELS: [&str; 80] = [
    "person", "bicycle", "car", "motorcycle", "airplane", "bus", "train", "truck", "boat", "traffic light",
	"fire hydrant", "stop sign", "parking meter", "bench", "bird", "cat", "dog", "horse", "sheep", "cow", "elephant",
	"bear", "zebra", "giraffe", "backpack", "umbrella", "handbag", "tie", "suitcase", "frisbee", "skis", "snowboard",
	"sports ball", "kite", "baseball bat", "baseball glove", "skateboard", "surfboard", "tennis racket", "bottle",
	"wine glass", "cup", "fork", "knife", "spoon", "bowl", "banana", "apple", "sandwich", "orange", "broccoli",
	"carrot", "hot dog", "pizza", "donut", "cake", "chair", "couch", "potted plant", "bed", "dining table", "toilet",
	"tv", "laptop", "mouse", "remote", "keyboard", "cell phone", "microwave", "oven", "toaster", "sink", "refrigerator",
	"book", "clock", "vase", "scissors", "teddy bear", "hair drier", "toothbrush"
];

pub struct Detector {
    session: Session,
}

impl Detector {
    pub fn new() -> Result<Self, ort::Error> {
        ort::init()
            .with_execution_providers([ort::CUDAExecutionProvider::default().build()])
            .commit()?;

        let session = Session::builder()?.with_model_from_file(MODEL_PATH)?;

        Ok(Detector { session })
    }

    pub fn detect(&self, frame: Mat) -> Result<Vec<(BoundingBox, &str, f32)>, ort::Error> {
        // Your detection logic here, adapted from your main program
        // For example, resize the image, process it, and run the detection model

        // Placeholder return to match expected function signature
        // Assume `frame` is your input image as a Mat object
        // Preprocess the frame: resize, normalize, etc.
        let processed_frame = self.preprocess_frame(&frame).unwrap();

        // Convert processed_frame into a tensor format expected by ORT
        // For simplicity, let's assume `to_tensor` returns an ndarray::Array4<f32>
        // which represents data in [Batch, Channel, Height, Width] format
        let tensor = self.to_tensor(&processed_frame)?;

        // Use the `inputs!` macro to prepare the input for the session
        // Assuming `tensor` is now in the correct shape and data type
        let inputs = inputs![tensor]?;

        // Run YOLOv8 inference
        let outputs: SessionOutputs = self.session.run(inputs)?;

        // Process outputs to extract bounding boxes, labels, and scores
        let detections = self.process_outputs(outputs).unwrap();

        Ok(detections)
    }

    fn preprocess_frame(&self, frame: &Mat) -> Result<Mat, anyhow::Error> {
        let target_size = Size::new(640, 640); // Example: YOLOv8's expected input size
        let mut resized = Mat::default();
        resize(
            frame,
            &mut resized,
            target_size,
            0.0,
            0.0,
            imgproc::INTER_LINEAR,
        )?;

        // Convert BGR to RGB if necessary (assuming the model expects RGB)
        let mut rgb = Mat::default();
        imgproc::cvt_color(&resized, &mut rgb, imgproc::COLOR_BGR2RGB, 0)?;

        // Normalize the pixel values to [0, 1] or another range if required
        let mut normalized = Mat::default();
        rgb.convert_to(&mut normalized, CV_32FC3, 1.0 / 255.0, 0.0)?;

        Ok(normalized)
    }

    fn to_tensor(&self, processed_frame: &Mat) -> Result<Array4<f32>, ort::Error> {
        let height = processed_frame.rows() as usize;
        let width = processed_frame.cols() as usize;
        let channels = 3; // Assuming the frame is RGB

        let data = unsafe {
            // Ensure `processed_frame` is of type CV_32FC3 and continuous
            if !processed_frame.is_continuous() {
                // Handle the error appropriately
                panic!("Frame is not continuous.");
            }

            // Calculate the total number of `f32` elements in the CV_32FC3 matrix
            let num_elements = height * width * channels * 3; // Three times for three channels

            // Access the raw data of the matrix
            // The data type here is assumed to be `Vec<u8>` because OpenCV Mat data is stored as bytes.
            // Each f32 value is 4 bytes, so we need to work with the raw byte data.
            let data_ptr = processed_frame.data();

            // Interpret the raw byte data as f32 values. This requires dividing the total byte length by 4 (size of f32).
            let f32_slice = std::slice::from_raw_parts(data_ptr as *const f32, num_elements);

            f32_slice.to_vec()
        };

        // Create an ndarray from the raw data slice with the correct shape
        let tensor = Array4::from_shape_vec((1, channels, height, width), data.to_vec()).unwrap();

        // Depending on your model's expected input format, you might need to permute the axes.
        // This example assumes that the model expects the format to be (Batch, Height, Width, Channels),
        // which is common for models trained with TensorFlow. Adjust the permuted_axes call as needed for your model.
        let tensor = tensor.permuted_axes([0, 2, 3, 1]);

        Ok(tensor)
    }

    fn process_outputs(
        &self,
        outputs: SessionOutputs,
    ) -> Result<Vec<(BoundingBox, &str, f32)>, anyhow::Error> {
        // Placeholder: Parse the outputs to extract bounding boxes, labels, and scores.
        // This will involve interpreting the raw output tensors from the model,
        // applying confidence thresholds, and performing non-maximum suppression.
        let output = outputs["output0"]
            .extract_tensor::<f32>()
            .unwrap()
            .view()
            .t()
            .into_owned();
        // Example output structure for illustration only
        let detections = vec![(
            BoundingBox {
                x1: 0.1,
                y1: 0.2,
                x2: 0.3,
                y2: 0.4,
            },
            "person",
            0.95,
        )];

        Ok(detections)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct BoundingBox {
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
}

fn intersection(box1: &BoundingBox, box2: &BoundingBox) -> f32 {
    (box1.x2.min(box2.x2) - box1.x1.max(box2.x1)) * (box1.y2.min(box2.y2) - box1.y1.max(box2.y1))
}

fn union(box1: &BoundingBox, box2: &BoundingBox) -> f32 {
    ((box1.x2 - box1.x1) * (box1.y2 - box1.y1)) + ((box2.x2 - box2.x1) * (box2.y2 - box2.y1))
        - intersection(box1, box2)
}
