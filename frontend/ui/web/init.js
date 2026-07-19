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

function prettyFontName(filename) {
  // 1. Remove the file extension (e.g., .ttf, .otf, .woff2)
  let name = filename.replace(/\.[^/.]+$/, "");

  // 2. Replace hyphens and underscores with spaces
  name = name.replace(/[-_]/g, " ");

  // 3. Insert spaces before capital letters (for PascalCase/CamelCase) 
  // but avoid splitting consecutive capitals (like "TTF" or "SVG")
  name = name.replace(/([a-z])([A-Z])/g, '$1 $2');
  name = name.replace(/([A-Z])([A-Z][a-z])/g, '$1 $2');

  // 4. Clean up any accidental double spaces and trim edges
  return name.replace(/\s+/g, ' ').trim();
}

function pretty(url) {
    const filename = url.split('/').pop();
    if (filename.includes("canvaskit")) return "Flutter engine";
    if (filename.includes("ttf"))       return `${prettyFontName(filename)}`;
    if (filename.includes("rust"))      return "Rust code";
    if (filename.includes("dart"))      return "Application";
    return "User interface";
}

function updateProgressBar(downloadIndex, currentProgress) {
    const totalProgress = (downloadIndex + currentProgress) / 4;
    const fill = document.querySelector(".progress-bar-fill");
    if (fill) fill.style.width = (totalProgress * 100) + "%";
}

async function download(url, downloadIndex) {
    const htmltext = document.querySelector(".loading-text");
	const htmltitle = document.querySelector(".loading-title");
    updateProgressBar(downloadIndex, 0);

    let response;
    try {
        response = await fetch(url);
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

    let total = parseInt(response.headers.get("content-length"), 10);
	if (url.includes("canvaskit")) {
		total = 10*1024*1024;
	}
	htmltitle.textContent = "The legs complain, the clock insists, the road continues."
    htmltext.textContent = `${pretty(url)}: ${formatBytes(total)} (please wait)`;

    const reader = response.body.getReader();
    let loaded = 0;

    while (true) {
        const { done, value } = await reader.read();
        if (done) break;
        loaded += value.byteLength;
		if (total) {
            const ratio = Math.min(loaded / total, 1);   // cap at 1 (decompressed > compressed)
            updateProgressBar(downloadIndex, ratio);
			while (loaded > total) {
				total = 2*total;
			}
            const pct = (100 * loaded / total).toFixed(0);
			htmltitle.textContent = `Only ${((total-loaded)/1000).toFixed(0)} km to the next coffee.`
            htmltext.textContent = `${pretty(url)} (${pct} %)`;
        } else {
			htmltitle.textContent = "It’s not far. It just takes a long time."
            htmltext.textContent = `${pretty(url)} (${formatBytes(loaded)})`;
        }
    }
	htmltitle.textContent = "Somewhere ahead there is a bakery."
    updateProgressBar(downloadIndex, 1);
    console.log(`Fetched ${pretty(url)}: ${formatBytes(loaded)}`);
}

(async () => {
    try {
        await download("main.dart.js",                                                                                        0);
        // await download("https://www.gstatic.com/flutter-canvaskit/a8bfdfc394deaed5c57bd45a64ac4294dc976a72/canvaskit.wasm", 1);
		await download("https://www.gstatic.com/flutter-canvaskit/c416acfeb8126e097f758c664aaa3da929e27da0/canvaskit.wasm", 1);
        await download("pkg/rust_lib_ui_bg.wasm",                                                                       3);
		await download("assets/fonts/LibertinusSans-Regular.ttf",                                                      3);
        await download("assets/fonts/LibertinusSans-Bold.ttf",                                                         3);
        await download("assets/fonts/LibertinusSans-Italic.ttf",                                                       3);

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
