function formatBytes(n) {
    if (n < 1024) {
        return `${n.toFixed(1)} bytes`;
    }
    n = n / 1024;
    if (n < 1024) {
        return `${n.toFixed(1)} kb`;
    }
    n = n / 1024;
    if (n < 1024) {
        return `${n.toFixed(1)} Mb`;
    }
}

function percent(n, total) {
    const s = (100 * n / total).toFixed(0);
    return `${s} %`;
}

function pretty(url) {
    var filename = url.split('/').pop()
    if (filename.includes("canvaskit")) {
        return "framework";
    }
    if (filename.includes("rust")) {
        return "algorithms";
    }
    return "user interface";
}

function openAndSend(request, url, retry) {
    var fullurl = url;
    if (retry) {
        var timestamp = Math.floor(Date.now() / 1000);
        fullurl = url + "?" + timestamp;
    }
    request.open("GET", fullurl, true);
    request.send()
}

async function download(url) {
    return new Promise((resolve, reject) => {
        const request = new XMLHttpRequest();
        request.responseType = 'blob';
        const htmltext = document.querySelector(".loading-text");

        tryfetch = async function (url) {
            console.log("start fetch", url);
            try {
                const response = await fetch(url, { cache: "reload" });
                if (!response.ok) {
                    msg = `Fetching ${pretty(url)}: ${response.status}`;
                    console.error(msg);
                    htmltext.textContent = msg;
                    reject(new Error(msg));
                }
                const total = parseInt(response.headers.get("content-length"),10);
                f = formatBytes(total);
                msg = `Fetching ${pretty(url)}: ${f} (please wait)`;
                console.log(msg);
                htmltext.textContent = msg;

                const reader = response.body.getReader();    
                let loaded = 0;
                while (true) {
                    const {done, value} = await reader.read();
                    if (done) break;
                    loaded += value.byteLength;
                    f = formatBytes(loaded);
                    p = percent(loaded, total);
                    msg = `Fetch ${pretty(url)}: ${f} [${p}]`;
                    htmltext.textContent = msg;
                  }
                f = formatBytes(loaded);
                msg = `Fetched ${loaded} bytes`;
                //await new Promise(r => setTimeout(r, 5000));
                console.log(msg);
                htmltext.textContent = msg;
                resolve(response);
            } catch (error) {
                msg = `Fetching ${pretty(url)} failed: ${error.message}`;
                console.error(msg);
                htmltext.textContent = msg;
                reject(new Error(msg));
            }
        };

        // Add a progress event listener to track download progress
        request.onprogress = function (event) {
            if (event.lengthComputable) {
                f = formatBytes(event.loaded);
                p = percent(event.loaded, event.total);
                msg = `Load ${pretty(url)}: ${f} [${p}]`;
                htmltext.textContent = msg;
                console.log(`${url}: ${msg}`);
            } else {
                console.log(`Downloaded ${event.loaded} bytes (total size unknown)`);
            }
        };

        // Resolve the promise when the request is complete
        request.onload = function () {
            if (request.status === 200 || request.status === 206) {
                console.log(`Download complete: ${url}`);
                resolve(request.response);
            } else {
                tryfetch(url);
            }
        };

        // Reject the promise on network errors
        request.onerror = function () {
            tryfetch(url);
        };

        openAndSend(request, url, false);
    });
}

// Example usage
(async () => {
    try {
        await download("https://www.gstatic.com/flutter-canvaskit/a8bfdfc394deaed5c57bd45a64ac4294dc976a72/canvaskit.wasm");
        await download("pkg/rust_lib_ui_bg.wasm");
        await download("main.dart.js");
        const htmltext = document.querySelector(".loading-text");
        htmltext.textContent = `starting app...`;
        console.log("start app");
        const script = document.createElement("script");
        script.src = "flutter_bootstrap.js";
        document.body.appendChild(script);

    } catch (error) {
        console.error(error);
        //window.location.reload(true);
    }
})();

