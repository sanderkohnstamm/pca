import logging
from connection import DetectionClient
import socket
import cv2
import asyncio
import time
from detector import Detector
from dataloader import DataLoader

ID = "MacBook"
LOCAL_IP = socket.gethostbyname(socket.gethostname())
HOST = "localhost"
PORT = 50051


async def main():
    # Initialize the components
    detector = Detector('onnx_models/yolov8n_with_metadata.onnx')
    dataloader = DataLoader(0)
    client = DetectionClient(host=HOST, port=PORT, id=ID, own_ip=LOCAL_IP)
    asyncio.create_task(dataloader.start())

    frame_count = 0
    start_time = time.time()

    # Spawn ping loop
    async def ping_loop():
        while True:
            await asyncio.sleep(1)
            client.ping(id=ID)

    asyncio.create_task(ping_loop())

    async for frame in dataloader:
        frame_count += 1
        elapsed_time = time.time() - start_time
        if elapsed_time > 5.0:
            fps = frame_count / elapsed_time
            logging.info(f"FPS: {fps:.2f}")
            client.frame_rate = fps
            frame_count = 0
            start_time = time.time()

        # Perform object detection and send the detections to the server if connected
        if client.connected:
            boxes, scores, class_names = detector(frame)
            client(boxes, scores, class_names) 

            


        # print(f"Scores: {scores}")

    dataloader.cap.release()
    cv2.destroyAllWindows()

if __name__ == "__main__":
    try:
        asyncio.run(main())
    except KeyboardInterrupt:
        print("Program interrupted by user")