from concurrent import futures
import time
import grpc
import logging

# Import the generated classes
from generated_protos import detector_pb2
from generated_protos import detector_pb2_grpc

logging.basicConfig(level=logging.INFO)

class DetectionClient:
    def __init__(self, host='localhost', port=50051, id='test_id', own_ip='localhost'):
        self.host = host
        self.port = port
        self.id = id
        self.own_ip = own_ip
        self.frame_rate = 0
        self.connected = False
        self.connect()

    def connect(self):
        try:
            self.channel = grpc.insecure_channel(f'{self.host}:{self.port}')
            self.stub = detector_pb2_grpc.DetectorServiceStub(self.channel)
            self.connected = True
            logging.info("Connected to server")
        except Exception as e:
            logging.error(f"Failed to connect: {e}")
            self.connected = False

    def reconnect(self):
        while not self.connected:
            logging.info("Attempting to reconnect...")
            self.connect()
            if not self.connected:
                time.sleep(5)  # Wait before trying to reconnect

    def __call__(self, boxes, scores, class_names):
        try:
            response = self.send_detections(generate_proto_detections(self.id, boxes, scores, class_names))
            self.handle_set_connected()
            return response
        except Exception as e:
            self.handle_disconnect(e)

    def ping(self, id):
        try:
            logging.info(f"Pinging server with id: {id}")
            request = detector_pb2.ProtoPing(id=id, ip=self.own_ip, frame_rate=self.frame_rate) 
            response = self.stub.Ping(request)
            self.handle_set_connected()
            return response
        except Exception as e:
            self.handle_disconnect(e)

    def send_detections(self, request):
        try:
            response = self.stub.SendDetections(request)
            self.handle_set_connected()
            return response
        except Exception as e:
            self.handle_disconnect(e)

    def handle_disconnect(self, error):
        if self.connected:
            logging.error(f"Disconnected from server: {error}")
            self.connected = False
        self.reconnect()

    def handle_set_connected(self):
        if not self.connected:
            logging.info("Reconnected to server")
            self.connected = True

def generate_proto_detections(id, boxes, scores, class_names):
    detections = []
    for box, score, class_name in zip(boxes, scores, class_names):
        detection = detector_pb2.ProtoDetection(
            class_name=class_name,
            score=score,
            bounding_box=detector_pb2.ProtoBoundingBox(center_x=box[0], center_y=box[1], width=box[2], height=box[3])
        )
        detections.append(detection)
    
    proto_detections = detector_pb2.ProtoDetections(id=id, detections=detections)
    return proto_detections

def main():
    client = DetectionClient()
    client.ping(id="test_id")
    boxes = [[0, 0, 100, 100], [200, 200, 300, 300]]
    scores = [0.9, 0.8]
    class_names = ["person", "car"]

    client(boxes, scores, class_names)

if __name__ == "__main__":
    main()