# HTTPS File Server

A secure HTTPS file server written in Rust that serves static files and provides a simple API endpoint.

## Quick Start

1. Generate a self-signed certificate (one-time setup):
```bash
openssl req -x509 -newkey rsa:4096 -nodes -keyout key.pem -out cert.pem -days 365 -subj '/CN=localhost'
```

2. Create a directory for your files:
```bash
mkdir public
echo "Hello World!" > public/index.html
```

3. Start the server:
```bash
./https_fileserve
```

4. Access the server at https://localhost:8443

## Features

- HTTPS support using rustls
- Static file serving from a configurable directory
- Custom route `/api/hello` as an example endpoint
- CORS support with configurable headers
- Security headers (COEP, COOP) for modern web security
- Command-line configuration options

## Prerequisites

- Rust and Cargo (latest stable version)
- TLS certificate and private key files (in PEM format)

## Installation

Clone the repository and build the project:

```bash
git clone <repository-url>
cd https_fileserve
cargo build --release
```

The binary will be available at `target/release/https_fileserve`

## Usage

Basic usage with default options:

```bash
./https_fileserve
```

This will:
- Serve files from `./public` directory
- Listen on all interfaces (0.0.0.0)
- Use port 8443
- Look for `cert.pem` and `key.pem` in the current directory

### Command Line Options

```
OPTIONS:
    -d, --directory <DIRECTORY>    Directory to serve files from [default: ./public]
    -c, --cert <CERT>             Path to TLS certificate file [default: cert.pem]
    -k, --key <KEY>              Path to TLS private key file [default: key.pem]
    -p, --port <PORT>            Port to listen on [default: 8443]
    -H, --host <HOST>            Interface to bind to [default: 0.0.0.0]
    -h, --help                   Print help information
```

### Examples

Serve files from a specific directory on port 9443:
```bash
./https_fileserve --directory /path/to/files --port 9443
```

Bind to a specific IP address:
```bash
./https_fileserve --host 192.168.1.100 --port 8443
```

Use custom certificate files:
```bash
./https_fileserve --cert /path/to/certificate.pem --key /path/to/private-key.pem
```

## Generating Self-Signed Certificates for Testing

For development and testing, you can generate a self-signed certificate using OpenSSL:

```bash
openssl req -x509 -newkey rsa:4096 -nodes -keyout key.pem -out cert.pem -days 365 -subj '/CN=localhost'
```

Note: Browsers will show a security warning when using self-signed certificates.

## Security Headers

The server automatically adds the following security headers to all responses:
- `Cross-Origin-Opener-Policy: same-origin`
- `Cross-Origin-Embedder-Policy: require-corp`
- `Access-Control-Allow-Headers: *`

## Accessing the Server

Once running, you can access the server using HTTPS:

```
https://hostname:8443/
https://hostname:8443/api/hello
```

Replace `hostname` with:
- `localhost` for local access
- Your machine's IP address or domain name for remote access

## example

```
rm document.pdf; 
cat /tmp/document.typst | curl -X POST -H "Content-Type: text/plain" --data-binary @- https://localhost:8123/api/typst --insecure -O -J
```
