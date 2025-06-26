from http.server import HTTPServer, SimpleHTTPRequestHandler

class CustomHTTPRequestHandler(SimpleHTTPRequestHandler):
    def end_headers(self):
        # Add the Cross-Origin-Resource-Policy header
        self.send_header("Cross-Origin-Resource-Policy", "same-origin");
        self.send_header("Cross-Origin-Opener-Policy","same-origin");
        self.send_header("Cross-Origin-Embedder-Policy", "require-corp");
        super().end_headers()

if __name__ == "__main__":
    port = 8123
    server_address = ("", port)
    httpd = HTTPServer(server_address, CustomHTTPRequestHandler)
    print(f"Serving on port {port}")
    httpd.serve_forever()