#coding:utf-8

import os,os.path,sys,time,datetime,traceback,json
import struct

import zmq
import fire
import base64
import json
from zmqex import init_keepalive
from payload import *

MX_PUB_ADDR="tcp://127.0.0.1:15555"
MX_SUB_ADDR="tcp://127.0.0.1:15556"

def do_sub(sub_addr=MX_SUB_ADDR):
    ctx = zmq.Context()
    sock = ctx.socket(zmq.SUB)
    init_keepalive(sock)
    sock.setsockopt(zmq.SUBSCRIBE, b'')  # 订阅所有品种
    sock.connect(sub_addr)
    while True:
        m = sock.recv()
        print(str(datetime.datetime.now()),m)

def do_pub(text,pub_addr=MX_PUB_ADDR,times=1000):
    ctx = zmq.Context()
    sock = ctx.socket(zmq.PUB)
    init_keepalive(sock)
    sock.connect(pub_addr)
    time.sleep(.5)  # 必须等一下，zmq bug
    for n in range(times):
        t = f"{n}.{text}"
        sock.send(t.encode())
        print( str(datetime.datetime.now()).split(".")[0] + " msg sent:",t)
        # sock.close()
        time.sleep(1)

    sock.close()

# 发布策略仓位信号
def pub_ps(pub_addr=MX_PUB_ADDR,times=1000):
    ctx = zmq.Context()
    sock = ctx.socket(zmq.PUB)
    init_keepalive(sock)
    sock.connect(pub_addr)
    time.sleep(.5)  # 必须等一下，zmq bug
    for n in range(times):
        ps = 1.23
        p = NetworkPayload()
        p.head = '0001'.ljust(40,'\0').encode()
        p.body = 'MA'.ljust(8,'\0').encode() + 'pyelf-001-st'.ljust(95,'\0').encode() +\
            struct.pack('!d',ps)
        t = p.marshall()
        sock.send(t)
        print( str(datetime.datetime.now()).split(".")[0] + " msg sent:",t)
        # sock.close()
        time.sleep(1)

    sock.close()

if __name__ == '__main__':
    fire.Fire()