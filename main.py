import a2s
import logging

logging.basicConfig(
    format="%(asctime)s %(message)s",
    level=logging.INFO,
)

HOST = "127.0.0.1"
PORT = 7100

info = a2s.info((HOST, PORT))
print(info)
