import zmq
import time

context = zmq.Context()
publisher = context.socket(zmq.PUB)
publisher.bind("tcp://127.0.0.1:15555")

while True:
    current_time = time.ctime()
    message = current_time.encode()
    publisher.send(message)
    time.sleep(1)
