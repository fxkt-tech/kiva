/* Track only the WASM response, retaining streaming compilation and backpressure. */
(() => {
  "use strict";
  window.kivaLoader = Object.freeze({
    async load(init, source) {
      const startup = window.kivaStartup;
      startup.progress("download", 0, 0);
      let reader;
      try {
        const response = await fetch(source);
        if (!response.ok) throw new Error(`WASM download returned ${response.status}`);
        // Fetch exposes decoded chunks. Encoded Content-Length counts other bytes.
        const encoding = response.headers.get("content-encoding");
        const length = response.headers.get("content-length");
        const size = length && /^\d+$/.test(length) ? Number(length) : 0;
        // CORS can hide Content-Encoding while still exposing Content-Length.
        const unencoded = encoding ? encoding.toLowerCase() === "identity" : response.type !== "cors";
        const total = unencoded && Number.isSafeInteger(size) ? size : 0;
        let module = response;
        if (response.body) {
          let loaded = 0;
          reader = response.body.getReader();
          startup.progress("download", loaded, total);
          const body = new ReadableStream({
            async pull(controller) {
              try {
                const { done, value } = await reader.read();
                if (done) {
                  if (total > loaded) throw new Error("WASM response ended before Content-Length");
                  startup.progress("initialize", 0, 0);
                  controller.close();
                } else {
                  loaded += value.byteLength;
                  startup.progress("download", loaded, total);
                  controller.enqueue(value);
                }
              } catch (error) {
                startup.fail("下载中断，请重新加载后重试。");
                controller.error(error);
              }
            },
            cancel(reason) { return reader.cancel(reason); },
          });
          module = new Response(body, {
            headers: response.headers, status: response.status, statusText: response.statusText,
          });
        }
        // Passing a Response preserves wasm-bindgen's streaming/fallback behavior.
        const wasm = await init({ module_or_path: module });
        startup.progress("initialize", 0, 0);
        return wasm;
      } catch (error) {
        startup.fail("游戏加载失败，请重新加载后重试。");
        // Initialization can reject before consuming the Response. Stop that
        // download without letting a cancellation error replace the cause.
        reader?.cancel(error).catch(() => {});
        window.dispatchEvent(new CustomEvent("TrunkApplicationError", { detail: { reason: error } }));
        throw error;
      }
    },
  });
})();
