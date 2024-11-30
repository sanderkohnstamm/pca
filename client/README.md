Client side


generate protos
python -m grpc_tools.protoc -I. --python_out=. --grpc_python_out=. detector.proto


make a mediamtx server that exposes the stream at <local_ip>:8889/cam