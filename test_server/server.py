from flask import Flask, request
import random

IP = "0.0.0.0"
PORT = "15000"


app = Flask(__name__)

statuses = [200, 404, 201, 500, 302]

@app.route('/gettest')
def route1():
    print("route1 called")
    return 'Hello, World!'

@app.route('/posttest', methods=["POST"])
def route2():
    print("route2 called")
    print(request.json)
    return 'Hello, World!', random.choice(statuses)

app.run(host=IP, port=PORT)
