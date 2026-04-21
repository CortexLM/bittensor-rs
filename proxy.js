const http = require("http");

const TARGET_HOST = "95.133.252.108";
const TARGET_PORT = 8000;
const PROXY_PORT = process.env.PORT || 4000;
const DEBUG = process.env.DEBUG !== "0";

const colors = {
  reset: "\x1b[0m", dim: "\x1b[2m", cyan: "\x1b[36m", green: "\x1b[32m",
  yellow: "\x1b[33m", red: "\x1b[31m", magenta: "\x1b[35m", blue: "\x1b[34m",
};

let reqCounter = 0;

function log(id, color, tag, msg) {
  if (!DEBUG) return;
  const ts = new Date().toISOString().slice(11, 23);
  console.log(`${colors.dim}${ts}${colors.reset} ${color}[${tag}]${colors.reset} #${id} ${msg}`);
}

function truncate(str, max = 300) {
  if (!str) return "";
  return str.length > max ? str.slice(0, max) + "..." : str;
}

function makeProxyRequest(options, body) {
  return new Promise((resolve, reject) => {
    const req = http.request(options, resolve);
    req.on("error", reject);
    req.on("timeout", () => req.destroy(new Error("upstream timeout")));
    if (body.length > 0) req.write(body);
    req.end();
  });
}

function readFullBody(stream) {
  return new Promise((resolve) => {
    const chunks = [];
    stream.on("data", (c) => chunks.push(c));
    stream.on("end", () => resolve(Buffer.concat(chunks)));
  });
}

const server = http.createServer((clientReq, clientRes) => {
  const id = ++reqCounter;
  const startTime = Date.now();

  log(id, colors.cyan, "REQ", `${clientReq.method} ${clientReq.url}`);

  const chunks = [];
  clientReq.on("data", (chunk) => chunks.push(chunk));

  clientReq.on("end", async () => {
    const rawBody = Buffer.concat(chunks);
    let reqBody = rawBody;

    // ===== INJECT stream_options.include_usage for chat/completions =====
    if (rawBody.length > 0 && clientReq.url.includes("/chat/completions")) {
      try {
        const parsed = JSON.parse(rawBody.toString());

        // Force include_usage so upstream returns real token counts in the stream
        if (parsed.stream) {
          if (!parsed.stream_options) parsed.stream_options = {};
          parsed.stream_options.include_usage = true;
          reqBody = Buffer.from(JSON.stringify(parsed));
          log(id, colors.green, "FIX", "injected stream_options.include_usage=true");
        }

        log(id, colors.yellow, "BODY", `model=${parsed.model || "?"} stream=${parsed.stream ?? "?"} max_tokens=${parsed.max_tokens} messages=${parsed.messages?.length ?? "?"}`);

        // Clamp max_tokens if input tokens would overflow
        // We'll know real tokens from the stream usage, but we can preemptively check
        // by tokenizing the input via /tokenize
        if (parsed.messages && parsed.max_tokens) {
          try {
            const inputText = parsed.messages.map(m => {
              const c = typeof m.content === "string" ? m.content : JSON.stringify(m.content);
              return `${m.role}: ${c}`;
            }).join("\n");

            const tokBody = Buffer.from(JSON.stringify({ prompt: inputText }));
            const tokRes = await new Promise((resolve, reject) => {
              const tokReq = http.request({
                hostname: TARGET_HOST, port: TARGET_PORT, path: "/tokenize",
                method: "POST",
                headers: { "content-type": "application/json", "content-length": tokBody.length },
                timeout: 8000,
              }, resolve);
              tokReq.on("error", reject);
              tokReq.on("timeout", () => { tokReq.destroy(); reject(new Error("tok timeout")); });
              tokReq.write(tokBody);
              tokReq.end();
            });

            const tokData = await readFullBody(tokRes);
            const tokParsed = JSON.parse(tokData.toString());
            const inputTokens = tokParsed.count || 0;
            const maxModelLen = tokParsed.max_model_len || 202752;
            const total = inputTokens + (parsed.max_tokens || 32768);

            log(id, colors.blue, "TOKENS", `input=${inputTokens} max_tokens=${parsed.max_tokens} total=${total} limit=${maxModelLen} ${total > maxModelLen ? "OVERFLOW!" : "OK"}`);

            if (total > maxModelLen) {
              const newMax = Math.max(1024, maxModelLen - inputTokens - 100);
              log(id, colors.yellow, "CLAMP", `max_tokens ${parsed.max_tokens} -> ${newMax}`);
              parsed.max_tokens = newMax;
              // Re-inject stream_options after clamping
              if (!parsed.stream_options) parsed.stream_options = {};
              parsed.stream_options.include_usage = true;
              reqBody = Buffer.from(JSON.stringify(parsed));
            }
          } catch (e) {
            log(id, colors.red, "TOKENIZE-ERR", e.message);
          }
        }
      } catch (e) {
        log(id, colors.red, "PARSE-ERR", e.message);
      }
    }

    // ===== Forward request to upstream =====
    const proxyHeaders = { ...clientReq.headers };
    proxyHeaders.host = `${TARGET_HOST}:${TARGET_PORT}`;
    proxyHeaders["content-length"] = reqBody.length;

    const options = {
      hostname: TARGET_HOST, port: TARGET_PORT,
      path: clientReq.url, method: clientReq.method,
      headers: proxyHeaders, timeout: 120000,
    };

    try {
      const proxyRes = await makeProxyRequest(options, reqBody);
      const elapsed = Date.now() - startTime;
      const sc = proxyRes.statusCode;
      const statusColor = sc < 300 ? colors.green : sc < 400 ? colors.yellow : colors.red;
      log(id, statusColor, "RES", `${sc} ${proxyRes.statusMessage} (${elapsed}ms)`);

      // ===== 400 INTERCEPTOR =====
      if (sc === 400) {
        const errBody = await readFullBody(proxyRes);
        const errText = errBody.toString();
        log(id, colors.red, "400", truncate(errText, 500));
        if (!clientRes.headersSent) clientRes.writeHead(sc, proxyRes.headers);
        clientRes.write(errBody);
        clientRes.end();
        return;
      }

      const isStream = proxyRes.headers["content-type"]?.includes("text/event-stream") ||
                        (proxyRes.headers["transfer-encoding"] === "chunked" && sc >= 200 && sc < 300);

      if (!clientRes.headersSent) clientRes.writeHead(sc, proxyRes.headers);

      if (isStream) {
        let chunkCount = 0, totalBytes = 0, firstToken = null;
        let streamInputTokens = 0, streamOutputTokens = 0;

        proxyRes.on("data", (chunk) => {
          chunkCount++;
          totalBytes += chunk.length;
          if (chunkCount === 1) {
            firstToken = Date.now() - startTime;
            log(id, colors.magenta, "STREAM", `first chunk at ${firstToken}ms`);
          }

          // Parse SSE for usage data
          const text = chunk.toString();
          for (const line of text.split("\n")) {
            if (!line.startsWith("data: ")) continue;
            const data = line.slice(6).trim();
            if (data === "[DONE]") continue;
            try {
              const ev = JSON.parse(data);
              if (ev.usage) {
                streamInputTokens = ev.usage.prompt_tokens || 0;
                streamOutputTokens = ev.usage.completion_tokens || 0;
              }
            } catch {}
          }

          if (chunkCount <= 2 || text.includes("[DONE]")) {
            log(id, colors.magenta, "STREAM", truncate(text.replace(/\n/g, "\\n"), 200));
          }
          clientRes.write(chunk);
        });

        proxyRes.on("end", () => {
          const total = Date.now() - startTime;
          if (streamInputTokens > 0 || streamOutputTokens > 0) {
            log(id, colors.blue, "USAGE", `prompt=${streamInputTokens} completion=${streamOutputTokens} total=${streamInputTokens + streamOutputTokens}`);
          }
          log(id, colors.green, "DONE", `stream: ${chunkCount} chunks, ${totalBytes}b, ${total}ms, TTFT=${firstToken || "?"}ms`);
          clientRes.end();
        });
      } else {
        const resChunks = [];
        proxyRes.on("data", (chunk) => { resChunks.push(chunk); clientRes.write(chunk); });
        proxyRes.on("end", () => {
          if (DEBUG) {
            try {
              const p = JSON.parse(Buffer.concat(resChunks).toString());
              if (p.usage) log(id, colors.blue, "USAGE", `prompt=${p.usage.prompt_tokens} completion=${p.usage.completion_tokens} total=${p.usage.total_tokens}`);
            } catch {}
          }
          log(id, colors.green, "DONE", `${Date.now() - startTime}ms`);
          clientRes.end();
        });
      }
    } catch (err) {
      log(id, colors.red, "ERR", `${err.code || err.message}`);
      if (!clientRes.headersSent) clientRes.writeHead(502, { "Content-Type": "application/json" });
      clientRes.end(JSON.stringify({ error: { message: `Proxy error: ${err.message}`, code: err.code } }));
    }
  });

  clientReq.on("error", (err) => log(id, colors.red, "CLIENT-ERR", err.message));
});

server.listen(PROXY_PORT, () => {
  console.log(`\n${"=".repeat(60)}`);
  console.log(`  LLM Proxy (BYOK fix)`);
  console.log(`  Listening on    : http://0.0.0.0:${PROXY_PORT}`);
  console.log(`  Forwarding to   : http://${TARGET_HOST}:${TARGET_PORT}`);
  console.log(`  Key fix         : stream_options.include_usage=true injected`);
  console.log(`${"=".repeat(60)}\n`);
});

server.on("error", (err) => console.error("Server error:", err));
process.on("SIGINT", () => { console.log("\nShutting down..."); server.close(() => process.exit(0)); });
