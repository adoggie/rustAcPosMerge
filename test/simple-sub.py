import zmq

context = zmq.Context()
subscriber = context.socket(zmq.SUB)
subscriber.connect("tcp://127.0.0.1:15556")

# 订阅所有消息
subscriber.subscribe(b"")

while True:
    message = subscriber.recv()
    print("Received message:", len(message),message)
