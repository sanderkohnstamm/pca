import cv2
import asyncio
import time
from detector import Detector
from dataloader import DataLoader

async def main():
    # Initialize the components
    detector = Detector('onnx_models/yolov8n.onnx')
    dataloader = DataLoader(0)
    asyncio.create_task(dataloader.start())

    frame_count = 0
    start_time = time.time()

    async for frame in dataloader:
        frame_count += 1
        elapsed_time = time.time() - start_time
        if elapsed_time > 1.0:
            fps = frame_count / elapsed_time
            print(f"FPS: {fps:.2f}")
            frame_count = 0
            start_time = time.time()

        boxes, scores, class_ids = detector(frame)
        # print(f"Detected {len(boxes)} objects")
        # print(f"Boxes: {boxes}")
        print(f"Class IDs: {class_ids}")
        output_image = detector.draw_detections(frame)
        cv2.imshow('Object Detection', output_image)
        if cv2.waitKey(1) & 0xFF == ord('q'):
            break

    dataloader.cap.release()
    cv2.destroyAllWindows()

if __name__ == "__main__":
    try:
        asyncio.run(main())
    except KeyboardInterrupt:
        print("Program interrupted by user")