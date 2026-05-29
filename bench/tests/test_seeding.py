"""Tests for bench/run.py data seeding."""

import io
import json
import os
import sys
import unittest
from unittest.mock import patch, MagicMock

sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

from run import create_client, create_api_key, create_blueprint, _run_seed, main


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


class TestRunSeed(unittest.TestCase):
    @patch("run.create_blueprint")
    @patch("run.create_api_key")
    @patch("run.create_client")
    @patch("builtins.print")
    def test_creates_client_api_key_and_10_blueprints(
        self, mock_print, mock_create_client, mock_create_api_key, mock_create_blueprint
    ):
        mock_create_client.return_value = "client-uuid"
        mock_create_api_key.return_value = "arche_k_secret"

        _run_seed("super-key", "http://localhost:8080")

        mock_create_client.assert_called_once_with("super-key", "http://localhost:8080")
        mock_create_api_key.assert_called_once_with(
            "client-uuid", "super-key", "http://localhost:8080"
        )
        self.assertEqual(mock_create_blueprint.call_count, 10)

        expected_names = [f"Blueprint-{i:04d}" for i in range(10)]
        for i, call_args in enumerate(mock_create_blueprint.call_args_list):
            args, _ = call_args
            self.assertEqual(args[0], "client-uuid")
            self.assertEqual(args[1], "arche_k_secret")
            self.assertEqual(args[2], expected_names[i])
            self.assertEqual(args[3], "http://localhost:8080")

    @patch("run.create_blueprint")
    @patch("run.create_api_key")
    @patch("run.create_client")
    @patch("builtins.print")
    def test_prints_client_id_and_api_key(
        self, mock_print, mock_create_client, mock_create_api_key, mock_create_blueprint
    ):
        mock_create_client.return_value = "client-uuid"
        mock_create_api_key.return_value = "arche_k_secret"

        _run_seed("super-key", "http://localhost:8080")

        printed_texts = [call[0][0] for call in mock_print.call_args_list]
        combined = " ".join(printed_texts)
        self.assertIn("client-uuid", combined)
        self.assertIn("arche_k_secret", combined)

    @patch("run.create_client")
    @patch("builtins.print")
    def test_exits_1_on_create_client_error(
        self, mock_print, mock_create_client
    ):
        from urllib.error import HTTPError

        mock_create_client.side_effect = HTTPError(
            "url", 401, "Unauthorized", {}, io.BytesIO(b'{}'),
        )

        with self.assertRaises(SystemExit) as ctx:
            _run_seed("bad-key", "http://localhost:8080")

        self.assertEqual(ctx.exception.code, 1)

    @patch("run.create_blueprint")
    @patch("run.create_api_key")
    @patch("run.create_client")
    @patch("builtins.print")
    def test_exits_1_on_create_blueprint_error(
        self, mock_print, mock_create_client, mock_create_api_key, mock_create_blueprint
    ):
        from urllib.error import HTTPError

        mock_create_client.return_value = "client-uuid"
        mock_create_api_key.return_value = "arche_k_secret"
        mock_create_blueprint.side_effect = HTTPError(
            "url", 500, "Server Error", {}, io.BytesIO(b"error"),
        )

        with self.assertRaises(SystemExit) as ctx:
            _run_seed("super-key", "http://localhost:8080")

        self.assertEqual(ctx.exception.code, 1)

    @patch("run.create_blueprint")
    @patch("run.create_api_key")
    @patch("run.create_client")
    @patch("builtins.print")
    def test_uses_scoped_api_key_for_blueprints(
        self, mock_print, mock_create_client, mock_create_api_key, mock_create_blueprint
    ):
        mock_create_client.return_value = "client-uuid"
        mock_create_api_key.return_value = "arche_k_scoped"

        _run_seed("super-key", "http://localhost:8080")

        for call_args, _ in mock_create_blueprint.call_args_list:
            self.assertEqual(call_args[1], "arche_k_scoped")


class TestMainCLIWithSeed(unittest.TestCase):
    @patch("run._run_seed")
    @patch("builtins.print")
    def test_main_with_api_key_and_seed_calls_run_seed(self, mock_print, mock_run_seed):
        testargs = ["run.py", "--api-key", "super-key", "--seed"]
        with patch.object(sys, "argv", testargs):
            main()

        mock_run_seed.assert_called_once_with("super-key", "http://localhost:8080")

    @patch("run._run_seed")
    @patch("builtins.print")
    def test_main_with_seed_and_target_url(self, mock_print, mock_run_seed):
        testargs = [
            "run.py", "--api-key", "super-key", "--seed",
            "--target-url", "http://other:8080",
        ]
        with patch.object(sys, "argv", testargs):
            main()

        mock_run_seed.assert_called_once_with("super-key", "http://other:8080")

    @patch("run._run_api_connectivity")
    @patch("builtins.print")
    def test_main_without_seed_calls_connectivity(
        self, mock_print, mock_connectivity
    ):
        testargs = ["run.py", "--api-key", "key"]
        with patch.object(sys, "argv", testargs):
            main()

        mock_connectivity.assert_called_once_with("key", "http://localhost:8080")


if __name__ == "__main__":
    unittest.main()
