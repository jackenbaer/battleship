
from sseclient import SSEClient


import argparse
parser = argparse.ArgumentParser()
parser.add_argument("-g", required=True)
parser.add_argument("-p", required=True)
args = parser.parse_args()

messages = SSEClient(f'http://127.0.0.1:8080/events/{args.g}/{args.p}')
for msg in messages:
    print(msg)