#coding:utf-8

import sys
"""

message specification:

[ver][reserved][type/filter][body]
   ver (1) : version ,  high/low (0.1 -> 0x01, 1.2 -> 0x12)
   reserved(3) ：
   type/filter(40) :  head part
   body(N) :    user-data

"""
CongoRiver = '1000'  #

class NetworkPayload(object):
    MIN_SIZE = 44
    VERSION = b'\x15'
    BODY_JSON = b'\0'
    BODY_PICKLE = b'\x01'
    def __init__(self):
        self.ver = NetworkPayload.VERSION
        self.encoding = NetworkPayload.BODY_JSON
        self.reserved = b'\0'*2
        self.head = b'\0'*40
        self.body = b''

    def marshall(self):
        # if sys.version_info.major == 2:
        return self.ver + self.encoding + self.reserved + self.head + self.body


    @staticmethod
    def parse(data):
        if len(data) < NetworkPayload.MIN_SIZE:
            return None
        packet = NetworkPayload()
        packet.ver = data[:1]
        packet.encoding = data[1:2]
        packet.reserved = data[2:4]
        packet.head = data[4:NetworkPayload.MIN_SIZE]
        packet.body = data[NetworkPayload.MIN_SIZE:]
        return packet

    @staticmethod
    def for_message(ver=b'',reserved=b'',head=b'',body=b'',encoding='json'):
        if sys.version_info.major == 3:
            if isinstance(body,str):
                body = body.encode()
            if isinstance(ver,str):
                ver = ver.encode()
            if isinstance(reserved,str):
                reserved = reserved.encode()
            if isinstance(head,str):
                head = head.encode()

        np = NetworkPayload()
        if encoding =='pickle':
            np.encoding = NetworkPayload.BODY_PICKLE

        if ver: np.ver = ver
        if reserved:
            np.reserved = reserved[:2] + b'\0'*(2-len(reserved))
        if head:
            np.head = head[:40] + b'\0'*(40-len(head))
        if body: np.body = body
        return np

def for_subscribe_address(topic):
    np = NetworkPayload()
    return np.ver + np.reserved + topic
