from connection import DetectionClient
import cv2
import asyncio
import time
from detector import Detector
from dataloader import DataLoader



full_class_names_list = ['person', 'bicycle', 'car', 'motorcycle', 'airplane', 'bus', 'train', 'truck', 'boat', 'traffic light',
               'fire hydrant', 'stop sign', 'parking meter', 'bench', 'bird', 'cat', 'dog', 'horse', 'sheep', 'cow',
               'elephant', 'bear', 'zebra', 'giraffe', 'backpack', 'umbrella', 'handbag', 'tie', 'suitcase', 'frisbee',
               'skis', 'snowboard', 'sports ball', 'kite', 'baseball bat', 'baseball glove', 'skateboard', 'surfboard',
               'tennis racket', 'bottle', 'wine glass', 'cup', 'fork', 'knife', 'spoon', 'bowl', 'banana', 'apple',
               'sandwich', 'orange', 'broccoli', 'carrot', 'hot dog', 'pizza', 'donut', 'cake', 'chair', 'couch',
               'potted plant', 'bed', 'dining table', 'toilet', 'tv', 'laptop', 'mouse', 'remote', 'keyboard',
               'cell phone', 'microwave', 'oven', 'toaster', 'sink', 'refrigerator', 'book', 'clock', 'vase',
               'scissors', 'teddy bear', 'hair drier', 'toothbrush']


ID = "test_id"


async def main():
    # Initialize the components
    detector = Detector('onnx_models/yolov8n.onnx', full_class_names_list=full_class_names_list)
    dataloader = DataLoader(0)
    client = DetectionClient()
    asyncio.create_task(dataloader.start())

    frame_count = 0
    start_time = time.time()

    # Spawn ping loop
    async def ping_loop():
        while True:
            await asyncio.sleep(1)
            client.ping(id="test_id")
    asyncio.create_task(ping_loop())

    has_sent_error = False
    async for frame in dataloader:
        frame_count += 1
        elapsed_time = time.time() - start_time
        if elapsed_time > 1.0:
            fps = frame_count / elapsed_time
            print(f"FPS: {fps:.2f}")
            frame_count = 0
            start_time = time.time()

        boxes, scores, class_names = detector(frame)
        print(f"Class IDs: {class_names}")
        print(f"Bboxes: {boxes}")
        try:
            client(boxes, scores, class_names) 
            has_sent_error = False
        except Exception as e:
            if not has_sent_error:
                has_sent_error = True
                print("Failed to send detections")
                print(f"Error: {e}")

        # print(f"Scores: {scores}")

    dataloader.cap.release()
    cv2.destroyAllWindows()

if __name__ == "__main__":
    try:
        asyncio.run(main())
    except KeyboardInterrupt:
        print("Program interrupted by user")