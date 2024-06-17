
const http = require ('http');
const util = require("../util.js");

let req = http.request({
    hostname: '127.0.0.1',
    port: 8000,
    path: '/scripts/print_method.sh',
    agent: false,
    method: "POST",
    headers:{},
    }, (res) => {
        let correct = true;
        correct = correct & util.compareStatus(res, 200, "OK");
        correct = correct & util.compareHeaders({"content-type": "text/plain"}, res.headers);
        res.on ('data', (data) => {
            if (data.toString().trim() !== "POST") {
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

req.write("text");
req.end();