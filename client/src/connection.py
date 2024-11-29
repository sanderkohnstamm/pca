import grpc
from concurrent import futures
import time
import counter_pb2
import counter_pb2_grpc

# Import the generated classes

class CounterClient:
    def __init__(self, host='localhost', port=50051):
        self.channel = grpc.insecure_channel(f'{host}:{port}')
        self.stub = counter_pb2_grpc.CounterServiceStub(self.channel)

    def ping(self, id):
        request = counter_pb2.ProtoPing(id=id)
        return self.stub.Ping(request)

    def update_counter_with(self, id, count):
        request = counter_pb2.ProtoCount(id=id, count=count)
        return self.stub.UpdateCounterWith(request)
    
    def send_text(self, id, text):
        request = counter_pb2.ProtoText(id=id, text=text)
        return self.stub.SendText(request)

def main():
    client = CounterClient()
    
    # Test ping
    response = client.ping(id="test_id")
    print("Ping response:", response)

    # Test update counter
    for i in range(10):
        response = client.update_counter_with(id="test_id", count=i)
        print(f"UpdateCounterWith response for count {i}:", response)
        time.sleep(1)  # Sleep for 1 second between updates
    print("UpdateCounterWith response:", response)

    # Test send text   
    for i in range(10):
        response = client.send_text(id="test_id", text=f"Hello, World! {i}")
        print(f"SendText response for text {i}:", response)
        time.sleep(1)



if __name__ == '__main__':
    main()