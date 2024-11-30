import time
import cv2
import numpy as np
import onnxruntime

from utils import  multiclass_nms

full_class_names_list = ['person', 'bicycle', 'car', 'motorcycle', 'airplane', 'bus', 'train', 'truck', 'boat', 'traffic light',
               'fire hydrant', 'stop sign', 'parking meter', 'bench', 'bird', 'cat', 'dog', 'horse', 'sheep', 'cow',
               'elephant', 'bear', 'zebra', 'giraffe', 'backpack', 'umbrella', 'handbag', 'tie', 'suitcase', 'frisbee',
               'skis', 'snowboard', 'sports ball', 'kite', 'baseball bat', 'baseball glove', 'skateboard', 'surfboard',
               'tennis racket', 'bottle', 'wine glass', 'cup', 'fork', 'knife', 'spoon', 'bowl', 'banana', 'apple',
               'sandwich', 'orange', 'broccoli', 'carrot', 'hot dog', 'pizza', 'donut', 'cake', 'chair', 'couch',
               'potted plant', 'bed', 'dining table', 'toilet', 'tv', 'laptop', 'mouse', 'remote', 'keyboard',
               'cell phone', 'microwave', 'oven', 'toaster', 'sink', 'refrigerator', 'book', 'clock', 'vase',
               'scissors', 'teddy bear', 'hair drier', 'toothbrush']

class Detector:
    def __init__(self, path, conf_thres=0.7, iou_thres=0.5, full_class_names_list=full_class_names_list):
        print(conf_thres)
        self.conf_threshold = conf_thres
        self.iou_threshold = iou_thres

        self.full_class_names_list = full_class_names_list
        self.image_shape = None
        # Initialize model
        self.initialize_model(path)

    def __call__(self, image):
        if not self.image_shape or self.image_shape != image.shape:
            print("Image shape: ", image.shape)
            self.image_shape = image.shape

        return self.detect_objects(image)

    def initialize_model(self, path):
        self.session = onnxruntime.InferenceSession(path,
                                                    providers=onnxruntime.get_available_providers())
        # Get model info
        self.get_input_details()
        self.get_output_details()


    def detect_objects(self, image):
        input_tensor = self.prepare_input(image)

        # Perform inference on the image
        outputs = self.inference(input_tensor)

        self.boxes, self.scores, self.class_names = self.process_output(outputs)

        return self.boxes, self.scores, self.class_names

    def get_class_names(self, class_ids):
        print(class_ids)
        return [self.full_class_names_list[class_id] for class_id in class_ids]

    def prepare_input(self, image):
        self.img_height, self.img_width = image.shape[:2]

        input_img = cv2.cvtColor(image, cv2.COLOR_BGR2RGB)

        # Resize input image
        input_img = cv2.resize(input_img, (self.input_width, self.input_height))

        # Scale input pixel values to 0 to 1
        input_img = input_img / 255.0
        input_img = input_img.transpose(2, 0, 1)
        input_tensor = input_img[np.newaxis, :, :, :].astype(np.float32)

        return input_tensor


    def inference(self, input_tensor):
        start = time.perf_counter()
        outputs = self.session.run(self.output_names, {self.input_names[0]: input_tensor})

        # print(f"Inference time: {(time.perf_counter() - start)*1000:.2f} ms")
        return outputs

    def process_output(self, output):
        predictions = np.squeeze(output[0]).T
        # Filter out object confidence scores below threshold
        scores = np.max(predictions[:, 4:], axis=1)
        predictions = predictions[scores > self.conf_threshold, :]
        scores = scores[scores > self.conf_threshold]

        if len(scores) == 0:
            return [], [], []

        # Get the class with the highest confidence
        class_ids = np.argmax(predictions[:, 4:], axis=1)

        # Get bounding boxes for each object
        boxes = self.extract_boxes(predictions)
        # Apply non-maxima suppression to suppress weak, overlapping bounding boxes
        # indices = nms(boxes, scores, self.iou_threshold)
        indices = multiclass_nms(boxes, scores, class_ids, self.iou_threshold)

        class_names = self.get_class_names(class_ids[indices])

        return boxes[indices], scores[indices], class_names

    def extract_boxes(self, predictions):
        # Extract boxes from predictions
        boxes = predictions[:, :4]

        return boxes


    def get_input_details(self):
        model_inputs = self.session.get_inputs()
        self.input_names = [model_inputs[i].name for i in range(len(model_inputs))]

        self.input_shape = model_inputs[0].shape
        self.input_height = self.input_shape[2]
        self.input_width = self.input_shape[3]

    def get_output_details(self):
        model_outputs = self.session.get_outputs()
        self.output_names = [model_outputs[i].name for i in range(len(model_outputs))]




def draw_boxes(image, boxes, scores, class_ids):
    for box, score, class_id in zip(boxes, scores, class_ids):
        if class_id == 'person':
            center_x, center_y, w, h = box
            x1 = int((center_x - w / 2) * image.shape[1])
            y1 = int((center_y - h / 2) * image.shape[0])
            x2 = int((center_x + w / 2) * image.shape[1])
            y2 = int((center_y + h / 2) * image.shape[0])
            cv2.rectangle(image, (x1, y1), (x2, y2), (0, 255, 0), 2)
            cv2.putText(image, f'{class_id} {score:.2f}', (x1, y1 - 10), cv2.FONT_HERSHEY_SIMPLEX, 0.9, (0, 255, 0), 2)

if __name__ == '__main__':
    # Load the detector
    detector = Detector('onnx_models/yolov8n.onnx')

    # Load the image
    image = cv2.imread('image.png')

    # Perform object detection
    boxes, scores, class_ids = detector(image)  
    draw_boxes(image, boxes, scores, class_ids)

    print(f"Boxes: {boxes}")
    print(f"Scores: {scores}")
    print(f"Class IDs: {class_ids}")

    # Display the image
    cv2.imshow('B', image)
    cv2.waitKey(0)
    cv2.destroyAllWindows()