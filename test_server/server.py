from flask import Flask, request
import random
import sys

IP = "0.0.0.0"
PORT = "15000"


app = Flask(__name__)

statuses = [200, 404, 201, 500, 302]

@app.route('/gettest')
def route1():
    print("route1 called")
    print(request.json)
    print(request.headers)
    return 'Hello, World!'

@app.route('/posttest', methods=["POST", "PUT", "PATCH", "DELETE"])
def route2():
    print("route2 called")
    print(request.json)
    print(request.headers)
    return 'Hello, World!', random.choice(statuses)

ssl_context = None
if len(sys.argv) == 2 and sys.argv[1] == "true":
    ssl_context=("cert.pem", "key.pem")
app.run(host=IP, port=PORT, ssl_context=ssl_context)
