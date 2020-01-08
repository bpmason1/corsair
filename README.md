# corsair
CORS-ignoring proxy

### run the proxy
`corsair --listen-ip=127.0.0.1:8000 --proxy-ip=127.0.0.1:8080 --permissive`

The `--permissive` flag indicates that all CORS requests should be accepted for proxying.
The flag is required for now because I have not yet implemented logic to configure CORS rules.

