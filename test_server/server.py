from flask import Flask, request

IP = "0.0.0.0"
PORT = "15000"


app = Flask(__name__)

@app.route('/gettest')
def route1():
    print("route1 called")
    return 'Hello, World!'

@app.route('/posttest', methods=["POST"])
def route2():
    print("route2 called")
    print(request.json)
    return 'Hello, World!'

app.run(host=IP, port=PORT)
