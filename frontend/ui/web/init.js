function formatBytes(n) {
    if (n < 1024) {
        return `${n.toFixed(1)} bytes`;
    }
    n = n / 1024;
    if (n < 1024) {
        return `${n.toFixed(1)} kb`;
    }
    n = n / 1024;
    return `${n.toFixed(1)} Mb`;
}

function percent(n, total) {
    return `${(100 * n / total).toFixed(0)} %`;
}

function pretty(url) {
    const filename = url.split('/').pop();
    if (filename.includes("canvaskit")) return "Flutter engine";
    if (filename.includes("ttf"))       return "Libertinus fonts";
    if (filename.includes("rust"))      return "Rust code";
    if (filename.includes("dart"))      return "Application";
    return "User interface";
}

function updateProgressBar(downloadIndex, currentProgress) {
    const totalProgress = (downloadIndex + currentProgress) / 6;
    const fill = document.querySelector(".progress-bar-fill");
    if (fill) fill.style.width = (totalProgress * 100) + "%";
}

async function download(url, downloadIndex) {
    const htmltext = document.querySelector(".loading-text");
    updateProgressBar(downloadIndex, 0);

    let response;
    try {
        response = await fetch(url, { cache: "reload" });
    } catch (error) {
        const msg = `Network error fetching ${pretty(url)}: ${error.message}`;
        console.error(msg);
        htmltext.textContent = msg;
        throw error;
    }

    if (!response.ok) {
        const msg = `Failed to fetch ${pretty(url)}: HTTP ${response.status}`;
        console.error(msg);
        htmltext.textContent = msg;
        throw new Error(msg);
    }

    const total = parseInt(response.headers.get("content-length"), 10);
    htmltext.textContent = `Fetching ${pretty(url)}: ${formatBytes(total)} (please wait)`;

    const reader = response.body.getReader();
    let loaded = 0;

    while (true) {
        const { done, value } = await reader.read();
        if (done) break;
        loaded += value.byteLength;
		if (total) {
            const ratio = Math.min(loaded / total, 1);   // cap at 1 (decompressed > compressed)
            updateProgressBar(downloadIndex, ratio);
            const pct = Math.min(100, (100 * loaded / total).toFixed(0));
            htmltext.textContent = `Fetching ${pretty(url)}: ${formatBytes(loaded)} / ~${formatBytes(total)} [${pct} %]`;
        } else {
            htmltext.textContent = `Fetching ${pretty(url)}: ${formatBytes(loaded)}`;
        }
    }

    updateProgressBar(downloadIndex, 1);
    console.log(`Fetched ${pretty(url)}: ${formatBytes(loaded)}`);
}

(async () => {
    try {
        await download("main.dart.js",                                                                                        0);
        await download("https://www.gstatic.com/flutter-canvaskit/a8bfdfc394deaed5c57bd45a64ac4294dc976a72/canvaskit.wasm", 1);
        await download("pkg/rust_lib_ui_bg.wasm",                                                                            2);
        await download("assets/fonts/LibertinusSerif-Regular.ttf",                                                      3);
        await download("assets/fonts/LibertinusSerif-Bold.ttf",                                                         4);
        await download("assets/fonts/LibertinusSerif-Italic.ttf",                                                       5);

        // All done
        updateProgressBar(6, 0);
        document.querySelector(".loading-text").textContent = "Starting WPX…";
        console.log("Starting app");
        const script = document.createElement("script");
        script.src = "flutter_bootstrap.js";
        document.body.appendChild(script);
    } catch (error) {
        console.error("Download sequence failed:", error);
    }
})();
