const http = require ('http');
const util = require("./../util.js");

http.get({
    hostname: '127.0.0.1',
    port: 8000,
    path: '/scripts/simple.sh',
    agent: false,
    headers:{
        'random-header': 'ignore'
    },
    }, (res) => {
        let correct = true;
        correct = correct & util.compareStatus(res, 200, "OK");
        correct = correct & util.compareHeaders({"content-type": "text/plain; charset=utf-8", "Content-length": "16"}, res.headers);
        res.on ('data', (data) => {
            if (data.toString().trim() !== "Packet received") {
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