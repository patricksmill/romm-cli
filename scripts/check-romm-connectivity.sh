#!/usr/bin/env bash
# Diagnose why a browser might reach RomM while curl / romm-cli fail.
# Usage:
#   ./scripts/check-romm-connectivity.sh https://romm.example.com
#   API_BASE_URL=https://romm.example.com ./scripts/check-romm-connectivity.sh
set -u

raw="${1:-${API_BASE_URL:-}}"
if [[ -z "$raw" ]]; then
	echo "Usage: $0 <url>"
	echo "  Example: $0 https://romm.example.com"
	echo "  Or:      API_BASE_URL=https://romm.example.com $0"
	exit 1
fi

# Normalize to site origin (no /api): same idea as romm-cli's API_BASE_URL
trim_slash="${raw%/}"
if [[ "$trim_slash" =~ /api$ ]]; then
	base="${trim_slash%/api}"
else
	base="$trim_slash"
fi

openapi_url="${base}/openapi.json"

if [[ "$base" =~ ^https?://([^/:]+) ]]; then
	host="${BASH_REMATCH[1]}"
else
	host="(could not parse host)"
fi

echo "== RomM connectivity check =="
echo "Origin:  $base"
echo "Host:    $host"
echo "Probe:   $openapi_url"
echo ""

echo "=== 1) DNS ==="
if command -v dig &>/dev/null; then
	dig +short "$host" A || true
	dig +short "$host" AAAA || true
else
	getent hosts "$host" 2>/dev/null || true
fi
echo ""

echo "=== 2) HTTPS on TCP (IPv4) — same stack as romm-cli / default curl ==="
if curl -4 -sS -o /dev/null -w "HTTP %{http_code} in %{time_total}s\n" --connect-timeout 10 --max-time 25 -I "$openapi_url" 2>&1; then
	:
else
	echo "(failed — see verbose run below)"
	curl -4 -vI --connect-timeout 10 --max-time 25 "$openapi_url" 2>&1 || true
fi
echo ""

echo "=== 3) HTTPS on TCP (IPv6, if you have AAAA) ==="
if curl -6 -sS -o /dev/null -w "HTTP %{http_code} in %{time_total}s\n" --connect-timeout 10 --max-time 25 -I "$openapi_url" 2>/dev/null; then
	:
else
	echo "(failed — common on macOS if this machine has no working IPv6 route to Cloudflare)"
fi
echo ""

# Apple /usr/bin/curl often lacks HTTP/3; Homebrew curl lists HTTP3 in `curl --version`.
pick_curl_http3() {
	local c
	for c in /opt/homebrew/opt/curl/bin/curl /usr/local/opt/curl/bin/curl "$(command -v curl 2>/dev/null)"; do
		[[ -n "$c" && -x "$c" ]] || continue
		if "$c" --version 2>/dev/null | grep -q HTTP3; then
			printf '%s' "$c"
			return 0
		fi
	done
	return 1
}

echo "=== 4) HTTP/3 (QUIC) — many browsers use this instead of TCP :443 ==="
if CURL3="$(pick_curl_http3)" && [[ -n "$CURL3" ]]; then
	echo "Using: $CURL3"
	if "$CURL3" --http3 -sS -o /dev/null -w "HTTP %{http_code} in %{time_total}s\n" --connect-timeout 10 --max-time 25 -I "$openapi_url" 2>&1; then
		:
	else
		echo "(failed — verbose:)"
		"$CURL3" --http3 -v -o /dev/null --connect-timeout 10 --max-time 25 "$openapi_url" 2>&1 || true
	fi
else
	echo "No curl with HTTP/3 on this system."
	echo "Install:  brew install curl"
	echo "Retry:    /opt/homebrew/opt/curl/bin/curl --http3 -vI '$openapi_url'"
	echo "          (Intel Mac: /usr/local/opt/curl/bin/curl if installed via Homebrew)"
fi
echo ""

echo "=== How to read results ==="
echo "- (2) is what romm-cli uses (TCP TLS to port 443)."
echo "- If (2) fails but (4) succeeds: browser may be on HTTP/3 only; fix TCP :443 path (Cloudflare, origin, firewall)."
echo "- If (2) and (4) both fail: broader network or TLS issue."
echo "- If (2) succeeds: HTTPS from the CLI works; check API_BASE_URL and auth in romm-cli."
