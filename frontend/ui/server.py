from http.server import HTTPServer, SimpleHTTPRequestHandler
import ssl
import sys;

class CustomHTTPRequestHandler(SimpleHTTPRequestHandler):
    def end_headers(self):
        # Add the Cross-Origin-Resource-Policy header
        # self.send_header("Cross-Origin-Resource-Policy", "cross-origin");
        self.send_header("Cross-Origin-Opener-Policy","same-origin");
        self.send_header("Cross-Origin-Embedder-Policy", "require-corp");
        super().end_headers()
        
def main():
    port = 8123
    domain = "vps-e637d6c5.vps.ovh.net";
    mode="https"
    # mode = "http"
    # domain = "localhost";
    if len(sys.argv) >= 2:
        mode=sys.argv[1];
    if len(sys.argv) >= 3:
        domain=sys.argv[2];
    #server_address = (domain, port)
    server_address = ("", port)

    context=None;
    if mode == "https":
        certfile=f"/tmp/{domain:s}.cert";
        keyfile=f"/tmp/{domain:s}.key";
        print("  domain:",domain);
        print(" keyfile:",keyfile);
        print("certfile:",certfile);
        context = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
        context.load_cert_chain(certfile=certfile, keyfile=keyfile)
        context.check_hostname = False
    
    with HTTPServer(server_address, CustomHTTPRequestHandler) as httpd:
        if mode=="https":
            assert(context);
            httpd.socket = context.wrap_socket(httpd.socket, server_side=True)
        print(f"serving {mode}://{domain}:{port}")
        httpd.serve_forever()

if __name__ == "__main__":
    main();
