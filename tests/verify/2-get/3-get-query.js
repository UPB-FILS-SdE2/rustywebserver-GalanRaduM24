const http = require ('http');
const util = require("./../util.js");

http.get({
    hostname: '127.0.0.1',
    port: 8000,
    path: '/scripts/print_query_string.sh?subject=server&value=this+is+a+text',
    agent: false,
    headers:{},
    }, (res) => {
        let correct = true;
        correct = correct & util.compareStatus(res, 200, "OK");
        correct = correct & util.compareHeaders({"content-type": "text/plain"}, res.headers);
        res.on ('data', (data) => {
            if (data.toString().trim().split(/\r?\n/).sort().join("\n") !== "Query_subject=server\nQuery_value=this+is+a+text") {
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