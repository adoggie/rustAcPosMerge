#coding:utf-8

"""
https://gist.github.com/minrk/4667957
"""

import os
import string
import sys
import time
from random import randint
import zmq
import fire
from zmqex import init_keepalive

MX_SUB_ADDR = "tcp://127.0.0.1:15555"
MX_PUB_ADDR = "tcp://127.0.0.1:15556"



ctx = zmq.Context()

# https://blog.csdn.net/weixin_43214364/article/details/82811095

# xsub (bind ) , xpub(bind)
def run(sub_addr = MX_SUB_ADDR ,pub_addr=MX_PUB_ADDR,**kvs):
    xpub_url = pub_addr
    xsub_url = sub_addr

    print('xSub bind:', xsub_url,'waiting for  incoming..')
    print('xPub bind:', xpub_url,'waiting for  incoming..')


    xpub = ctx.socket(zmq.XPUB)
    init_keepalive(xpub)
    xpub.bind(xpub_url)
    xsub = ctx.socket(zmq.XSUB)
    init_keepalive(xsub)
    xsub.bind(xsub_url)

    poller = zmq.Poller()
    poller.register(xpub, zmq.POLLIN)
    poller.register(xsub, zmq.POLLIN)
    while True:
        events = dict(poller.poll(1000))
        if xpub in events:
            message = xpub.recv_multipart()
            print("[BROKER] xpub. subscription message: %r" % message[0])
            xsub.send_multipart(message)
        if xsub in events:
            message = xsub.recv_multipart()
            print("publishing message: %r" % message)
            xpub.send_multipart(message)

if __name__ == '__main__':
#     fire.Fire()
    run()