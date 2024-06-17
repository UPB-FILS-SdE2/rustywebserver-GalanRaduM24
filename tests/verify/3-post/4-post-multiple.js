const http = require ('http');
const os = require ('os');
const util = require("../util.js");

const text = 'super cool message'+os.EOL+'super cool person'

const req = http.request({
    hostname: '127.0.0.1',
    port: 8000,
    path: '/scripts/test/multiplelines.sh',
    agent: false,
    method: 'POST',
    headers:{
        'Content-Type': 'text/plain; charset=utf-8',
        'Content-length': text.length,
        }
    }, (res) => {
        let correct = true;
        correct = correct & util.compareStatus(res, 200, "OK");
        correct = correct & util.compareHeaders({"content-type": "text/html"}, res.headers);
        res.on ('data', (data) => {
            if (data.toString().trim() !== "Message super cool message received from super cool person") {
                console.error("Wrong message");
                correct = false;
            }
        });
        res.on("end", () => {
            if (!correct) {
                process.exit(1);
            }
        });
});
req.write (text);
req.end ();