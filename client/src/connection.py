import grpc
from concurrent import futures
import time

# Import the generated classes
from generated_protos import detector_pb2
from generated_protos import detector_pb2_grpc


class DetectionClient:
    def __init__(self, host='localhost', port=50051, id='test_id'):
        self.id = id
        self.channel = grpc.insecure_channel(f'{host}:{port}')
        self.stub = detector_pb2_grpc.DetectorServiceStub(self.channel)

    def __call__(self, boxes, scores, class_names):
        return self.send_detections(generate_proto_detections(self.id, boxes, scores, class_names))

    def ping(self, id):
        request = detector_pb2.ProtoPing(id=id)
        return self.stub.Ping(request)

    def send_detections(self, request):
        return self.stub.SendDetections(request)



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