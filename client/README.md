Client side


generate protos
python -m grpc_tools.protoc -I. --python_out=. --grpc_python_out=. counter.proto