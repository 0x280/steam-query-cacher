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

players = a2s.players((HOST, PORT))
print(players)

rules = a2s.rules((HOST, PORT))
print(rules)
