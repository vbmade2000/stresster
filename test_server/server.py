from flask import Flask

IP = "0.0.0.0"
PORT = "15000"


app = Flask(__name__)

@app.route('/')
def route1():
    print("route1 called")
    return 'Hello, World!'

app.run(host=IP, port=PORT)
