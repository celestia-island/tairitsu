import assert from "node:assert/strict";
import { test } from "node:test";

import {
  ApiError,
  configureErrorHandler,
  requestJson,
  upload,
} from "../dist/index.js";

/** In-memory transport stub recording calls. */
function stubTransport(result) {
  const calls = [];
  const transport = {
    async httpProxy(method, path, body, headers) {
      calls.push(`${method} ${path} ${JSON.stringify(headers ?? {})} ${JSON.stringify(body)}`);
      return result;
    },
    async upload(path, _body, headers) {
      calls.push(`UPLOAD ${path} ${JSON.stringify(headers ?? {})}`);
      return result;
    },
    async download(path, _options) {
      calls.push(`DOWNLOAD ${path}`);
      throw new Error("download stub not wired here");
    },
  };
  return { transport, calls };
}

test("requestJson forwards method/body/headers and unwraps the body", async () => {
  const { transport, calls } = stubTransport({ status: 200, body: { ok: 1 } });
  const out = await requestJson(transport, () => "zh", "/api/x", {
    method: "POST",
    body: { a: 1 },
  });
  assert.deepEqual(out, { ok: 1 });
  assert.equal(calls.length, 1);
  assert.match(calls[0] ?? "", /^POST \/api\/x .*"Accept-Language":"zh".*"a":1/);
});

test("requestJson throws ApiError on non-2xx and reports via the hook", async () => {
  const seen = [];
  configureErrorHandler((err, path, method) => seen.push(`${method} ${path}`));
  const { transport } = stubTransport({ status: 404, body: "nope" });
  await assert.rejects(
    () => requestJson(transport, () => "en", "/api/missing"),
    (e) => e instanceof ApiError && e.status === 404 && e.body === "nope",
  );
  assert.deepEqual(seen, ["GET /api/missing"]);
});

test("silent option suppresses the error hook but still throws", async () => {
  const seen = [];
  configureErrorHandler((err) => seen.push(String(err.status)));
  const { transport } = stubTransport({ status: 500, body: null });
  await assert.rejects(() =>
    requestJson(transport, () => "en", "/api/boom", { silent: true }),
  );
  assert.deepEqual(seen, []);
});

test("upload requires FormData and surfaces non-2xx as ApiError", async () => {
  const { transport, calls } = stubTransport({ status: 200, body: null });
  await assert.rejects(
    () =>
      upload(transport, () => "en", "/up", { body: "not-formdata" }),
    /FormData/,
  );
  const fd = new FormData();
  fd.set("f", "v");
  await upload(transport, () => "en", "/up", { body: fd });
  assert.match(calls[0] ?? "", /^UPLOAD \/up/);
});

test("ApiError message carries status and body", () => {
  const e = new ApiError(502, "bad gateway");
  assert.equal(e.status, 502);
  assert.match(e.message, /502.*bad gateway/);
});
