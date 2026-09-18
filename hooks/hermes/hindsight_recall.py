"""Read-only Hindsight transport. Only trusted host settings choose the URL/bank.

The short-lived worker puts a hard timeout around DNS, headers and body reads.
Credentials travel over stdin, never argv, disk, logs or model context.
"""
from __future__ import annotations

import ipaddress
import json
from pathlib import Path
import subprocess
import sys
from urllib.error import HTTPError
from urllib.parse import quote, urlsplit
from urllib.request import HTTPRedirectHandler, ProxyHandler, Request, build_opener

MAX_RESPONSE_BYTES = 1_048_576


def endpoint(base_url: str, bank: str) -> tuple[str, bool]:
    if not isinstance(base_url, str) or not isinstance(bank, str) or not bank.strip():
        raise ValueError("Hindsight URL and bank are required")
    parts = urlsplit(base_url)
    if (parts.scheme not in ("http", "https") or not parts.hostname
            or parts.username is not None or parts.password is not None or parts.query or parts.fragment
            or any(ord(char) < 33 for char in base_url)
            or bank in (".", "..") or len(bank) > 200):
        raise ValueError("invalid Hindsight endpoint")
    try:
        loopback = ipaddress.ip_address(parts.hostname).is_loopback
    except ValueError:
        # Only literal loopback addresses avoid DNS rebinding for plain HTTP.
        loopback = False
    if parts.scheme == "http" and not loopback:
        raise ValueError("plain HTTP is allowed only for a literal loopback address")
    return base_url.rstrip("/") + "/v1/default/banks/" + quote(bank, safe="") + "/memories/recall", loopback


class NoRedirect(HTTPRedirectHandler):
    def redirect_request(self, req, fp, code, msg, headers, newurl):
        return None


def request(payload: dict, timeout: float) -> list[dict]:
    """Run one bounded real HTTP recall. Used by the isolated worker only."""
    url, _ = endpoint(payload["base_url"], payload["bank"])
    headers = {"Content-Type": "application/json", "Accept": "application/json"}
    token = payload.get("token", "")
    if token:
        if not isinstance(token, str) or any(ord(c) < 32 for c in token):
            raise ValueError("invalid credential")
        headers["Authorization"] = "Bearer " + token
    req = Request(url, data=json.dumps(payload["body"], allow_nan=False).encode(),
                  headers=headers, method="POST")
    # No redirects and no ambient proxies: neither can redirect credentials.
    with build_opener(ProxyHandler({}), NoRedirect()).open(req, timeout=timeout) as response:
        raw = response.read(MAX_RESPONSE_BYTES + 1)
    if len(raw) > MAX_RESPONSE_BYTES:
        raise ValueError("response exceeds limit")
    data = json.loads(raw)
    if not isinstance(data, dict) or not isinstance(data.get("results"), list):
        raise ValueError("malformed recall response")
    results = data["results"]
    if any(not isinstance(item, dict) or not isinstance(item.get("id"), str)
           or not isinstance(item.get("text"), str) or not item["id"]
           or (item.get("tags") is not None and (not isinstance(item["tags"], list)
               or any(not isinstance(tag, str) for tag in item["tags"]))) for item in results):
        raise ValueError("malformed memory")
    return results


def recall(base_url: str, bank: str, token: str, body: dict, timeout: float) -> tuple[list[dict], str]:
    if timeout <= 0:
        return [], "timeout"
    payload = {"base_url": base_url, "bank": bank, "token": token, "body": body, "timeout": timeout}
    try:
        process = subprocess.run(
            [sys.executable, str(Path(__file__).resolve())],
            input=json.dumps(payload, allow_nan=False), capture_output=True, text=True,
            timeout=timeout, check=False,
        )
        if process.returncode or len(process.stdout) > MAX_RESPONSE_BYTES * 8:
            return [], "backend_error"
        data = json.loads(process.stdout)
        return data["results"], data["status"]
    except subprocess.TimeoutExpired:
        return [], "timeout"
    except (OSError, ValueError, KeyError, TypeError):
        return [], "backend_error"


def main() -> None:
    try:
        raw = sys.stdin.buffer.read(65_537)
        if len(raw) > 65_536:
            raise ValueError("request exceeds limit")
        payload = json.loads(raw)
        results = request(payload, payload["timeout"])
        output = {"results": results, "status": "ok"}
    except HTTPError as error:
        output = {"results": [], "status": "unauthorized" if error.code in (401, 403) else "backend_error"}
    except Exception:
        # This is a process boundary: do not echo URLs, response bodies or secrets.
        output = {"results": [], "status": "backend_error"}
    print(json.dumps(output, ensure_ascii=True, allow_nan=False))


if __name__ == "__main__":
    main()
