"""Tests for bench/run.py data seeding."""

import io
import json
import os
import sys
import unittest
from unittest.mock import patch, MagicMock

sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

from run import (
    create_client,
    create_api_key,
    create_blueprint,
    _build_blueprint_body,
    _run_seed,
    _parse_args,
)


class TestCreateClient(unittest.TestCase):
    @patch("urllib.request.urlopen")
    def test_creates_client_with_correct_name(self, mock_urlopen):
        mock_response = MagicMock()
        mock_response.status = 200
        mock_response.read.return_value = b'{"id": "abc-123", "name": "bench-scratch"}'
        mock_response.__enter__.return_value = mock_response
        mock_urlopen.return_value = mock_response

        client_id = create_client("supersecret", "http://localhost:8080")

        self.assertEqual(client_id, "abc-123")

        called_request = mock_urlopen.call_args[0][0]
        self.assertEqual(called_request.full_url, "http://localhost:8080/api/clients")
        self.assertEqual(called_request.method, "POST")
        self.assertEqual(json.loads(called_request.data), {"name": "bench-scratch"})

    @patch("urllib.request.urlopen")
    def test_sends_api_key_header(self, mock_urlopen):
        mock_response = MagicMock()
        mock_response.status = 200
        mock_response.read.return_value = b'{"id": "abc", "name": "bench-scratch"}'
        mock_response.__enter__.return_value = mock_response
        mock_urlopen.return_value = mock_response

        create_client("my-key", "http://localhost:8080")

        called_request = mock_urlopen.call_args[0][0]
        headers = dict(called_request.headers)
        self.assertEqual(headers["X-api-key"], "my-key")

    @patch("urllib.request.urlopen")
    def test_uses_default_target_url(self, mock_urlopen):
        mock_response = MagicMock()
        mock_response.status = 200
        mock_response.read.return_value = b'{"id": "abc", "name": "bench-scratch"}'
        mock_response.__enter__.return_value = mock_response
        mock_urlopen.return_value = mock_response

        create_client("key")

        called_request = mock_urlopen.call_args[0][0]
        self.assertEqual(called_request.full_url, "http://localhost:8080/api/clients")

    @patch("urllib.request.urlopen")
    def test_raises_on_http_error(self, mock_urlopen):
        from urllib.error import HTTPError

        mock_urlopen.side_effect = HTTPError(
            "url", 401, "Unauthorized", {}, io.BytesIO(b'{"error": "unauthorized"}'),
        )

        with self.assertRaises(HTTPError):
            create_client("bad-key")


class TestCreateApiKey(unittest.TestCase):
    @patch("urllib.request.urlopen")
    def test_creates_key_with_generate_permission(self, mock_urlopen):
        mock_response = MagicMock()
        mock_response.status = 200
        mock_response.read.return_value = b'{"id": "key-1", "key": "arche_k_test123", "name": "benchmark-key", "permissions": ["generate"]}'
        mock_response.__enter__.return_value = mock_response
        mock_urlopen.return_value = mock_response

        result = create_api_key("client-abc", "supersecret", "http://localhost:8080")

        self.assertEqual(result, "arche_k_test123")

        called_request = mock_urlopen.call_args[0][0]
        self.assertEqual(
            called_request.full_url,
            "http://localhost:8080/api/clients/client-abc/keys",
        )
        self.assertEqual(called_request.method, "POST")
        self.assertEqual(
            json.loads(called_request.data),
            {"name": "benchmark-key", "permissions": ["generate"]},
        )

    @patch("urllib.request.urlopen")
    def test_sends_api_key_header(self, mock_urlopen):
        mock_response = MagicMock()
        mock_response.status = 200
        mock_response.read.return_value = (
            b'{"id": "k1", "key": "arche_k_x", "name": "benchmark-key", "permissions": ["generate"]}'
        )
        mock_response.__enter__.return_value = mock_response
        mock_urlopen.return_value = mock_response

        create_api_key("client-abc", "my-key", "http://localhost:8080")

        called_request = mock_urlopen.call_args[0][0]
        headers = dict(called_request.headers)
        self.assertEqual(headers["X-api-key"], "my-key")

    @patch("urllib.request.urlopen")
    def test_uses_default_target_url(self, mock_urlopen):
        mock_response = MagicMock()
        mock_response.status = 200
        mock_response.read.return_value = (
            b'{"id": "k1", "key": "arche_k_x", "name": "benchmark-key", "permissions": ["generate"]}'
        )
        mock_response.__enter__.return_value = mock_response
        mock_urlopen.return_value = mock_response

        create_api_key("client-abc", "key")

        called_request = mock_urlopen.call_args[0][0]
        self.assertEqual(
            called_request.full_url,
            "http://localhost:8080/api/clients/client-abc/keys",
        )

    @patch("urllib.request.urlopen")
    def test_raises_on_http_error(self, mock_urlopen):
        from urllib.error import HTTPError

        mock_urlopen.side_effect = HTTPError(
            "url", 403, "Forbidden", {}, io.BytesIO(b'{"error": "forbidden"}'),
        )

        with self.assertRaises(HTTPError):
            create_api_key("client-abc", "bad-key")


def _expected_attr(name):
    return {
        "description": f"Attribute {name}",
        "valueType": "range",
        "min": 0.0,
        "max": 100.0,
        "distribution": {"type": "uniform"},
    }


def _make_blueprint_body(name):
    return {
        "name": name,
        "archetype": "item",
        "weight": 1.0,
        "attributes": {
            "attr_0": _expected_attr("attr_0"),
            "attr_1": _expected_attr("attr_1"),
            "attr_2": _expected_attr("attr_2"),
        },
        "attributeOrder": ["attr_0", "attr_1", "attr_2"],
        "affixes": {
            "minPrefixes": 0,
            "maxPrefixes": 0,
            "minSuffixes": 0,
            "maxSuffixes": 0,
            "prefixes": [],
            "suffixes": [],
        },
    }


class TestBuildBlueprintBody(unittest.TestCase):
    def test_builds_expected_body(self):
        body = _build_blueprint_body("Blueprint-0000")
        self.assertEqual(body, _make_blueprint_body("Blueprint-0000"))

    def test_builds_body_with_different_name(self):
        body = _build_blueprint_body("MyBlueprint")
        self.assertEqual(body["name"], "MyBlueprint")

    def test_builds_three_attributes(self):
        body = _build_blueprint_body("x")
        self.assertEqual(len(body["attributes"]), 3)
        self.assertEqual(
            body["attributeOrder"], ["attr_0", "attr_1", "attr_2"],
        )


class TestCreateBlueprint(unittest.TestCase):
    @patch("urllib.request.urlopen")
    def test_creates_blueprint_with_correct_payload(self, mock_urlopen):
        mock_response = MagicMock()
        mock_response.status = 200
        mock_response.read.return_value = b'{"id": "bp-1", "name": "Blueprint-0000"}'
        mock_response.__enter__.return_value = mock_response
        mock_urlopen.return_value = mock_response

        result = create_blueprint(
            "client-abc", "supersecret", "Blueprint-0000", "http://localhost:8080"
        )

        self.assertEqual(result, "bp-1")

        called_request = mock_urlopen.call_args[0][0]
        self.assertEqual(
            called_request.full_url,
            "http://localhost:8080/api/blueprints?client_id=client-abc",
        )
        self.assertEqual(called_request.method, "POST")
        self.assertEqual(
            json.loads(called_request.data), _make_blueprint_body("Blueprint-0000"),
        )

    @patch("urllib.request.urlopen")
    def test_creates_blueprint_with_given_name(self, mock_urlopen):
        mock_response = MagicMock()
        mock_response.status = 200
        mock_response.read.return_value = b'{"id": "bp-x", "name": "x"}'
        mock_response.__enter__.return_value = mock_response
        mock_urlopen.return_value = mock_response

        create_blueprint("c", "k", "Blueprint-0005", "http://localhost:8080")

        called_request = mock_urlopen.call_args[0][0]
        self.assertEqual(
            json.loads(called_request.data)["name"], "Blueprint-0005",
        )

    @patch("urllib.request.urlopen")
    def test_uses_default_target_url(self, mock_urlopen):
        mock_response = MagicMock()
        mock_response.status = 200
        mock_response.read.return_value = b'{"id": "bp-1", "name": "x"}'
        mock_response.__enter__.return_value = mock_response
        mock_urlopen.return_value = mock_response

        create_blueprint("client-abc", "key", "Blueprint-0000")

        called_request = mock_urlopen.call_args[0][0]
        self.assertEqual(
            called_request.full_url,
            "http://localhost:8080/api/blueprints?client_id=client-abc",
        )

    @patch("urllib.request.urlopen")
    def test_raises_on_http_error(self, mock_urlopen):
        from urllib.error import HTTPError

        mock_urlopen.side_effect = HTTPError(
            "url", 500, "Server Error", {}, io.BytesIO(b"error"),
        )

        with self.assertRaises(HTTPError):
            create_blueprint("client-abc", "bad-key", "Blueprint-0000")


def _make_mock_response(status, body_bytes):
    mock_response = MagicMock()
    mock_response.status = status
    mock_response.read.return_value = body_bytes
    mock_response.__enter__.return_value = mock_response
    return mock_response


class TestRunSeed(unittest.TestCase):
    def _mock_urlopen_sequence(self, mock_urlopen, responses):
        mock_urlopen.side_effect = responses

    def test_creates_client_api_key_and_10_blueprints(self):
        client_resp = _make_mock_response(
            200, b'{"id": "client-uuid", "name": "bench-scratch"}'
        )
        key_resp = _make_mock_response(
            200,
            b'{"id": "key-1", "key": "arche_k_secret", "name": "benchmark-key", "permissions": ["generate"]}',
        )
        bp_resp = _make_mock_response(200, b'{"id": "bp-x", "name": "x"}')

        responses = [client_resp, key_resp] + [bp_resp] * 10

        with patch("urllib.request.urlopen", side_effect=responses):
            with patch("builtins.print") as mock_print:
                _run_seed("super-key", "http://localhost:8080")

        printed_texts = [call[0][0] for call in mock_print.call_args_list]
        combined = " ".join(printed_texts)
        self.assertIn("client-uuid", combined)
        self.assertIn("arche_k_secret", combined)

    def test_exits_1_on_create_client_error(self):
        from urllib.error import HTTPError

        with patch(
            "urllib.request.urlopen",
            side_effect=HTTPError(
                "url", 401, "Unauthorized", {}, io.BytesIO(b'{}'),
            ),
        ):
            with self.assertRaises(SystemExit) as ctx:
                _run_seed("bad-key", "http://localhost:8080")

        self.assertEqual(ctx.exception.code, 1)

    def test_exits_1_on_create_blueprint_error(self):
        from urllib.error import HTTPError

        client_resp = _make_mock_response(
            200, b'{"id": "client-uuid", "name": "bench-scratch"}'
        )
        key_resp = _make_mock_response(
            200,
            b'{"id": "key-1", "key": "arche_k_secret", "name": "benchmark-key", "permissions": ["generate"]}',
        )
        http_error = HTTPError(
            "url", 500, "Server Error", {}, io.BytesIO(b"error"),
        )

        with patch(
            "urllib.request.urlopen",
            side_effect=[client_resp, key_resp, http_error],
        ):
            with self.assertRaises(SystemExit) as ctx:
                _run_seed("super-key", "http://localhost:8080")

        self.assertEqual(ctx.exception.code, 1)


class TestParseArgs(unittest.TestCase):
    def test_api_key_and_seed(self):
        args = _parse_args(["--api-key", "super-key", "--seed"])
        self.assertEqual(args.api_key, "super-key")
        self.assertTrue(args.seed)

    def test_seed_and_target_url(self):
        args = _parse_args([
            "--api-key", "super-key", "--seed",
            "--target-url", "http://other:8080",
        ])
        self.assertEqual(args.api_key, "super-key")
        self.assertEqual(args.target_url, "http://other:8080")
        self.assertTrue(args.seed)

    def test_without_seed(self):
        args = _parse_args(["--api-key", "key"])
        self.assertFalse(args.seed)

    def test_default_target_url(self):
        args = _parse_args(["--api-key", "key", "--seed"])
        self.assertEqual(args.target_url, "http://localhost:8080")


if __name__ == "__main__":
    unittest.main()
