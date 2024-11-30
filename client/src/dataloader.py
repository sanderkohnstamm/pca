import time
import asyncio

try:
    from picamera2 import Picamera2
    IS_RPI = True
except ImportError:
    import cv2
    IS_RPI = False

class DataLoader:
    def __init__(self, stream_url=0, use_rpi_cam=IS_RPI):
        self.use_rpi_cam = use_rpi_cam
        if self.use_rpi_cam:
            self.picam = Picamera2()
            config = self.picam.create_preview_configuration()
            self.picam.configure(config)
            self.picam.start()
            time.sleep(2)  # Allow the camera to warm up
        else:
            self.cap = cv2.VideoCapture(stream_url)
            self.input_width = int(self.cap.get(cv2.CAP_PROP_FRAME_WIDTH))
            self.input_height = int(self.cap.get(cv2.CAP_PROP_FRAME_HEIGHT))
            print(f"Initialized DataLoader with stream URL: {stream_url}")
            print(f"Input width: {self.input_width}, Input height: {self.input_height}")

        self.frame = None
        self.event = asyncio.Event()

    async def start(self):
        if self.use_rpi_cam:
            while True:
                self.frame = self.picam.capture_array()
                self.event.set()  # Signal that a new frame is ready
                await asyncio.sleep(0.001)  # Avoid busy waiting
        else:
            while True:
                ret, frame = self.cap.read()
                if ret and frame is not None and frame.size > 0:
                    self.frame = frame
                    self.event.set()  # Signal that a new frame is ready
                    await asyncio.sleep(0.001)  # Avoid busy waiting
                else:
                    print("Failed to capture frame")
                    await asyncio.sleep(0.001)  # Avoid busy waiting

    def __aiter__(self):
        return self

    async def __anext__(self):
        await self.event.wait()  # Wait until a new frame is ready
        self.event.clear()  # Clear the event for the next frame
        return self.frame

async def run():
    print('Starting DataLoader...')
    data_loader = DataLoader(0, use_rpi_cam=IS_RPI)  # Set use_rpi_cam based on the platform
    asyncio.create_task(data_loader.start())

    frame_count = 0
    start_time = time.time()

    async for frame in data_loader:
        frame_count += 1
        elapsed_time = time.time() - start_time
        if elapsed_time > 5.0:
            fps = frame_count / elapsed_time
            print(f"FPS: {fps:.2f}")
            frame_count = 0
            start_time = time.time()

if __name__ == "__main__":
    asyncio.run(run())