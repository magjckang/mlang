const fs = require("fs/promises");
const http = require("http");
const path = require("path");
const util = require("util");
const zlib = require("zlib");

const gzip = util.promisify(zlib.gzip);

const MIME_TYPES = {
  ".html": "text/html; charset=utf-8",
  ".js": "application/javascript; charset=utf-8",
  ".css": "text/css; charset=utf-8",
  ".wasm": "application/wasm",
};

const { PORT = 80 } = process.env;
const [, , DIR = "."] = process.argv;

async function handleRequest(req, res) {
  const ext = path.extname(req.url);

  let filePath, contentType;

  if (ext) {
    filePath = req.url;
    contentType = MIME_TYPES[ext];
  } else {
    const { accept } = req.headers;
    if (accept && accept.split(",").includes("text/html")) {
      filePath = "index.html";
      contentType = MIME_TYPES[".html"];
    } else {
      notFound(res);
      return;
    }
  }

  filePath = path.join(DIR, filePath);

  let stats;

  try {
    stats = await fs.stat(filePath);
  } catch (err) {
    if (err.code === "ENOENT") {
      notFound(res);
      return;
    }
    throw err;
  }

  if (!stats.isFile()) {
    notFound(res);
    return;
  }

  const buffer = await fs.readFile(filePath);
  const gzipBuffer = await gzip(buffer);

  res.setHeader("Content-Encoding", "gzip");
  if (contentType) res.setHeader("Content-Type", contentType);
  res.setHeader("Content-Length", gzipBuffer.length);
  res.end(gzipBuffer);
}

function notFound(res) {
  res.statusCode = 404;
  res.end("Not Found");
}

http.createServer(handleRequest).listen(PORT);
