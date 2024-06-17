let fs = require("fs");
let path = require("path");

function toLowerKeys(obj) {
    return Object.keys(obj).reduce((accumulator, key) => {
        accumulator[key.toLowerCase()] = obj[key];
        return accumulator;
    }, {});
}

function compareFile(filename, data) {
    let file_data = fs.readFileSync(filename);
    if (!file_data.equals(data)) {
        console.error(`Downloaded file ${filename} is different from original`);
        return false;
    }
    return true;
}

function compareHeaders(headers, data) {
    let correct = true;
    let dataheaders = toLowerKeys(data);
    for (let header in headers) {
        if (headers[header] !== dataheaders[header.toLocaleLowerCase()]) {
            console.error(`Header ${header} differs: Yours: "${dataheaders[header.toLocaleLowerCase()]}", correct: "${headers[header]}"`);
            correct = false;
        }
    }
    return correct;
}

function filename(filename) {
    return path.join("public", filename);
}

function compareStatus (res, code, text) {
    let correct = true;
    if (res.statusCode !== code) {
        console.error(`Incorrect status code, got ${res.statusCode}, expected ${code}`);
        correct = false;
    }

    if (res.statusMessage !== text) {
        console.error(`Incorrect status text, got ${res.statusMessage}, expected ${text}`);
        correct = false;
    }
    return correct;
}

module.exports.compareFile = compareFile;
module.exports.compareHeaders = compareHeaders;
module.exports.compareStatus = compareStatus;
module.exports.filename = filename;
