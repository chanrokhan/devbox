const invoke = window.__TAURI__.core.invoke;

const toolMeta = {
  time: ["时间转换", "Unix 时间戳、UTC、本地时间互转"],
  json: ["JSON 工具", "格式化、压缩、校验 JSON 内容"],
  codec: ["编码转换", "Base64、URL、Hex 编解码"],
  crypto: ["加密摘要", "哈希摘要与 AES-GCM 加解密"],
  random: ["随机生成", "Key、密码、UUID 与随机字节"],
};

let activeTool = "time";
let jsonParsed = null;
let jsonView = "source";

const $ = (selector) => document.querySelector(selector);
const $$ = (selector) => Array.from(document.querySelectorAll(selector));

function setText(selector, value) {
  $(selector).textContent = value;
}

function setValue(selector, value) {
  $(selector).value = value;
}

function getValue(selector) {
  return $(selector).value;
}

function formatLocalInput(date) {
  const pad = (value) => String(value).padStart(2, "0");

  return [
    date.getFullYear(),
    "-",
    pad(date.getMonth() + 1),
    "-",
    pad(date.getDate()),
    " ",
    pad(date.getHours()),
    ":",
    pad(date.getMinutes()),
    ":",
    pad(date.getSeconds()),
  ].join("");
}

function metric(label, value) {
  const item = document.createElement("div");
  item.className = "metric";
  item.innerHTML = `<span></span><code></code>`;
  item.querySelector("span").textContent = label;
  item.querySelector("code").textContent = value;
  return item;
}

async function refreshClock() {
  const snapshot = await invoke("clock_snapshot");
  $("#clock-metrics").replaceChildren(
    metric("本地时间", snapshot.local),
    metric("UTC 时间", snapshot.utc),
    metric("Unix 秒", String(snapshot.unix_seconds)),
    metric("Unix 毫秒", String(snapshot.unix_millis)),
  );
}

async function refreshRandomPresets() {
  const [aes128, aes256, nonce] = await Promise.all([
    invoke("random_hex_bytes", { length: "16" }),
    invoke("random_hex_bytes", { length: "32" }),
    invoke("random_hex_bytes", { length: "12" }),
  ]);

  $("#random-presets").replaceChildren(
    metric("AES-128 Key", aes128),
    metric("AES-256 Key", aes256),
    metric("GCM Nonce", nonce),
  );
}

function selectTool(tool) {
  activeTool = tool;
  const [title, subtitle] = toolMeta[tool];
  setText("#page-title", title);
  setText("#page-subtitle", subtitle);

  $$(".nav-item").forEach((item) => {
    item.classList.toggle("active", item.dataset.tool === tool);
  });

  $$(".tool-panel").forEach((panel) => {
    panel.classList.toggle("active", panel.id === `tool-${tool}`);
  });
}

function renderJsonSource(source) {
  $("#json-source").replaceChildren(...tokenizeJson(source).map((token) => {
    const span = document.createElement("span");
    span.className = token.className;
    span.textContent = token.text;
    return span;
  }));
}

function tokenizeJson(source) {
  const tokens = [];
  let index = 0;

  while (index < source.length) {
    const current = source[index];

    if (current === "\"") {
      const start = index;
      index += 1;
      while (index < source.length) {
        if (source[index] === "\\") {
          index += 2;
          continue;
        }
        if (source[index] === "\"") {
          index += 1;
          break;
        }
        index += 1;
      }
      const text = source.slice(start, index);
      let next = index;
      while (/\s/.test(source[next] || "")) next += 1;
      tokens.push({ className: source[next] === ":" ? "json-key" : "json-string", text });
    } else if (/[0-9-]/.test(current)) {
      const start = index;
      index += 1;
      while (/[0-9.eE+-]/.test(source[index] || "")) index += 1;
      tokens.push({ className: "json-number", text: source.slice(start, index) });
    } else if (source.startsWith("true", index)) {
      tokens.push({ className: "json-bool", text: "true" });
      index += 4;
    } else if (source.startsWith("false", index)) {
      tokens.push({ className: "json-bool", text: "false" });
      index += 5;
    } else if (source.startsWith("null", index)) {
      tokens.push({ className: "json-null", text: "null" });
      index += 4;
    } else if ("{}[]:,".includes(current)) {
      tokens.push({ className: "json-punctuation", text: current });
      index += 1;
    } else {
      tokens.push({ className: "json-space", text: current });
      index += 1;
    }
  }

  return tokens;
}

function renderJsonTree(value, label = null) {
  if (Array.isArray(value)) {
    const details = document.createElement("details");
    details.className = "tree-node";
    details.open = true;
    details.append(summary(label, "[", `${value.length} items`, "]"));
    const children = document.createElement("div");
    children.className = "tree-children";
    value.forEach((child, index) => children.append(renderJsonTree(child, `[${index}]`)));
    details.append(children);
    return details;
  }

  if (value && typeof value === "object") {
    const keys = Object.keys(value);
    const details = document.createElement("details");
    details.className = "tree-node";
    details.open = true;
    details.append(summary(label, "{", `${keys.length} keys`, "}"));
    const children = document.createElement("div");
    children.className = "tree-children";
    keys.forEach((key) => children.append(renderJsonTree(value[key], key)));
    details.append(children);
    return details;
  }

  const leaf = document.createElement("div");
  leaf.className = "tree-leaf";
  appendLabel(leaf, label);
  const primitive = document.createElement("span");
  primitive.className = primitiveClass(value);
  primitive.textContent = primitiveText(value);
  leaf.append(primitive);
  return leaf;
}

function summary(label, openMark, meta, closeMark) {
  const node = document.createElement("summary");
  appendLabel(node, label);
  node.append(span("json-punctuation", openMark), span("json-meta", meta), span("json-punctuation", closeMark));
  return node;
}

function appendLabel(parent, label) {
  if (label === null) return;
  parent.append(span("json-key", `"${label}"`), span("json-punctuation", ": "));
}

function span(className, text) {
  const node = document.createElement("span");
  node.className = className;
  node.textContent = text;
  return node;
}

function primitiveClass(value) {
  if (value === null) return "json-null";
  if (typeof value === "string") return "json-string";
  if (typeof value === "number") return "json-number";
  if (typeof value === "boolean") return "json-bool";
  return "json-punctuation";
}

function primitiveText(value) {
  return typeof value === "string" ? JSON.stringify(value) : String(value);
}

function setJsonView(view) {
  jsonView = view;
  $$(".segment").forEach((button) => {
    button.classList.toggle("active", button.dataset.jsonView === view);
  });
  $("#json-source").classList.toggle("hidden", view !== "source");
  $("#json-tree").classList.toggle("hidden", view !== "tree");
}

async function processJson(mode) {
  const result = await invoke("process_json", { input: getValue("#json-input"), mode });
  setText("#json-status", result.status);
  jsonParsed = result.parsed;
  renderJsonSource(result.output);
  $("#json-tree").replaceChildren(
    jsonParsed ? renderJsonTree(jsonParsed) : span("json-null", result.status),
  );
  if (mode === "validate") setJsonView("source");
}

function activeOutputElement() {
  if (activeTool === "time") return $("#time-output");
  const panel = $(`#tool-${activeTool}`);
  return panel.querySelector(".output-target") || panel.querySelector("textarea, pre");
}

async function copyActiveOutput() {
  const target = activeOutputElement();
  const value = "value" in target ? target.value : target.textContent;
  await navigator.clipboard.writeText(value || "");
}

async function init() {
  const now = new Date();
  setValue("#timestamp-input", String(Math.floor(now.getTime() / 1000)));
  setValue("#datetime-input", formatLocalInput(now));
  setValue("#json-input", "{\"name\":\"DevToolbox\",\"features\":[\"json\",\"base64\",\"aes\"],\"config\":{\"theme\":\"native\",\"offline\":true},\"local\":true}");
  setValue("#codec-input", "Hello, DevToolbox");
  setValue("#crypto-input", "payload");

  const [aesKey, nonce, initialRandom] = await Promise.all([
    invoke("random_hex_bytes", { length: "32" }),
    invoke("random_hex_bytes", { length: "12" }),
    invoke("generate_random", { kind: "password", length: "32" }),
  ]);
  setValue("#aes-key", aesKey);
  setValue("#aes-nonce", nonce);
  setValue("#random-output", initialRandom);

  await Promise.all([
    refreshClock(),
    refreshRandomPresets(),
    processJson("pretty"),
    invoke("time_from_timestamp", { input: getValue("#timestamp-input") }).then((value) => setText("#time-output", value)),
  ]);

  setInterval(refreshClock, 1000);
}

$$(".nav-item").forEach((item) => {
  item.addEventListener("click", () => selectTool(item.dataset.tool));
});

$$("[data-json-view]").forEach((button) => {
  button.addEventListener("click", () => setJsonView(button.dataset.jsonView));
});

document.addEventListener("click", async (event) => {
  const button = event.target.closest("button");
  if (!button) return;

  if (button.dataset.action === "timestamp-convert") {
    setText("#time-output", await invoke("time_from_timestamp", { input: getValue("#timestamp-input") }));
  }

  if (button.dataset.action === "timestamp-now-sec") {
    const value = String(Math.floor(Date.now() / 1000));
    setValue("#timestamp-input", value);
    setText("#time-output", await invoke("time_from_timestamp", { input: value }));
  }

  if (button.dataset.action === "timestamp-now-ms") {
    const value = String(Date.now());
    setValue("#timestamp-input", value);
    setText("#time-output", await invoke("time_from_timestamp", { input: value }));
  }

  if (button.dataset.action === "datetime-convert") {
    setText("#time-output", await invoke("timestamp_from_local", { input: getValue("#datetime-input") }));
  }

  if (button.dataset.action === "datetime-now") {
    const value = formatLocalInput(new Date());
    setValue("#datetime-input", value);
    setText("#time-output", await invoke("timestamp_from_local", { input: value }));
  }

  if (button.dataset.action === "json-pretty") await processJson("pretty");
  if (button.dataset.action === "json-minify") await processJson("minify");
  if (button.dataset.action === "json-validate") await processJson("validate");

  if (button.dataset.codec) {
    setValue("#codec-output", await invoke("codec_transform", { input: getValue("#codec-input"), mode: button.dataset.codec }));
  }

  if (button.dataset.hash) {
    setValue("#crypto-output", await invoke("hash_text", { input: getValue("#crypto-input"), algorithm: button.dataset.hash }));
  }

  if (button.dataset.action === "aes-key") setValue("#aes-key", await invoke("random_hex_bytes", { length: "32" }));
  if (button.dataset.action === "aes-nonce") setValue("#aes-nonce", await invoke("random_hex_bytes", { length: "12" }));

  if (button.dataset.aes) {
    setValue("#crypto-output", await invoke("aes_gcm_transform", {
      input: getValue("#crypto-input"),
      keyHex: getValue("#aes-key"),
      nonceHex: getValue("#aes-nonce"),
      mode: button.dataset.aes,
    }));
  }

  if (button.dataset.random) {
    setValue("#random-output", await invoke("generate_random", {
      kind: button.dataset.random,
      length: getValue("#random-length"),
    }));
  }
});

$("#copy-output").addEventListener("click", copyActiveOutput);

init().catch((error) => {
  console.error(error);
  document.body.insertAdjacentHTML("afterbegin", `<div class="status-pill">${String(error)}</div>`);
});
