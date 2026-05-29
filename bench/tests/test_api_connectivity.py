"""Tests for bench/run.py API connectivity."""

import io
import os
import sys
import unittest
from unittest.mock import patch, MagicMock

sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

from run import send_generate_request, main


class TestSendGenerateRequest(unittest.TestCase):
    def setUp(self):
        self.api_key = "test-key-123"
        self.target_url = "http://localhost:8080"
        self.expected_url = "http://localhost:8080/api/generate"

    @patch("urllib.request.urlopen")
    def test_sends_post_to_correct_url(self, mock_urlopen):
        mock_response = MagicMock()
        mock_response.status = 200
        mock_response.read.return_value = b'{"ok": true}'
        mock_response.__enter__.return_value = mock_response
        mock_urlopen.return_value = mock_response

        send_generate_request(self.api_key, self.target_url)

        called_url = mock_urlopen.call_args[0][0].full_url
        self.assertEqual(called_url, self.expected_url)

    @patch("urllib.request.urlopen")
    def test_sends_empty_json_body(self, mock_urlopen):
        mock_response = MagicMock()
        mock_response.status = 200
        mock_response.read.return_value = b'{"ok": true}'
        mock_response.__enter__.return_value = mock_response
        mock_urlopen.return_value = mock_response

        send_generate_request(self.api_key, self.target_url)

        called_request = mock_urlopen.call_args[0][0]
        data = called_request.data
        self.assertEqual(data, b"{}")

    @patch("urllib.request.urlopen")
    def test_sends_x_api_key_header(self, mock_urlopen):
        mock_response = MagicMock()
        mock_response.status = 200
        mock_response.read.return_value = b'{"ok": true}'
        mock_response.__enter__.return_value = mock_response
        mock_urlopen.return_value = mock_response

        send_generate_request(self.api_key, self.target_url)

        called_request = mock_urlopen.call_args[0][0]
        headers = dict(called_request.headers)
        self.assertIn("X-api-key", headers)
        self.assertEqual(headers["X-api-key"], self.api_key)

    @patch("urllib.request.urlopen")
    def test_sets_content_type_header(self, mock_urlopen):
        mock_response = MagicMock()
        mock_response.status = 200
        mock_response.read.return_value = b'{"ok": true}'
        mock_response.__enter__.return_value = mock_response
        mock_urlopen.return_value = mock_response

        send_generate_request(self.api_key, self.target_url)

        called_request = mock_urlopen.call_args[0][0]
        headers = dict(called_request.headers)
        self.assertIn("Content-type", headers)
        self.assertEqual(headers["Content-type"], "application/json")

    @patch("urllib.request.urlopen")
    def test_returns_status_body_elapsed(self, mock_urlopen):
        mock_response = MagicMock()
        mock_response.status = 200
        mock_response.read.return_value = b'{"ok": true}'
        mock_response.__enter__.return_value = mock_response
        mock_urlopen.return_value = mock_response

        status, body, elapsed_ms = send_generate_request(self.api_key, self.target_url)

        self.assertEqual(status, 200)
        self.assertEqual(body, '{"ok": true}')
        self.assertIsInstance(elapsed_ms, float)
        self.assertGreaterEqual(elapsed_ms, 0)

    @patch("urllib.request.urlopen")
    def test_uses_default_target_url(self, mock_urlopen):
        mock_response = MagicMock()
        mock_response.status = 200
        mock_response.read.return_value = b'{}'
        mock_response.__enter__.return_value = mock_response
        mock_urlopen.return_value = mock_response

        send_generate_request(self.api_key)

        called_url = mock_urlopen.call_args[0][0].full_url
        self.assertEqual(called_url, "http://localhost:8080/api/generate")

    @patch("urllib.request.urlopen")
    def test_handles_401_error(self, mock_urlopen):
        from urllib.error import HTTPError

        mock_urlopen.side_effect = HTTPError(
            "http://localhost:8080/api/generate",
            401,
            "Unauthorized",
            {"X-API-Key": "invalid"},
            io.BytesIO(b'{"error": "unauthorized"}'),
        )

        with self.assertRaises(HTTPError):
            send_generate_request("bad-key", self.target_url)

    @patch("urllib.request.urlopen")
    def test_handles_500_error(self, mock_urlopen):
        from urllib.error import HTTPError

        mock_urlopen.side_effect = HTTPError(
            "http://localhost:8080/api/generate",
            500,
            "Server Error",
            {},
            io.BytesIO(b'internal error'),
        )

        with self.assertRaises(HTTPError):
            send_generate_request(self.api_key, self.target_url)

    @patch("urllib.request.urlopen")
    def test_handles_connection_refused(self, mock_urlopen):
        from urllib.error import URLError

        mock_urlopen.side_effect = URLError("Connection refused")

        with self.assertRaises(URLError):
            send_generate_request(self.api_key, "http://localhost:9999")


class TestMainCLI(unittest.TestCase):
    @patch("run.send_generate_request")
    @patch("builtins.print")
    def test_main_with_api_key_calls_send(self, mock_print, mock_send):
        mock_send.return_value = (200, '{"ok": true}', 42.5)

        testargs = ["run.py", "--api-key", "test-key"]
        with patch.object(sys, "argv", testargs):
            main()

        mock_send.assert_called_once_with("test-key", "http://localhost:8080")

    @patch("run.send_generate_request")
    @patch("builtins.print")
    def test_main_with_target_url(self, mock_print, mock_send):
        mock_send.return_value = (200, '{"ok": true}', 42.5)

        testargs = ["run.py", "--api-key", "test-key", "--target-url", "http://other:8080"]
        with patch.object(sys, "argv", testargs):
            main()

        mock_send.assert_called_once_with("test-key", "http://other:8080")

    @patch("run.send_generate_request")
    @patch("builtins.print")
    def test_main_prints_response_details(self, mock_print, mock_send):
        mock_send.return_value = (200, '{"ok": true}', 42.5)

        testargs = ["run.py", "--api-key", "test-key"]
        with patch.object(sys, "argv", testargs):
            main()

        printed_texts = [call[0][0] for call in mock_print.call_args_list]
        combined = " ".join(printed_texts)
        self.assertIn("200", combined)
        self.assertIn("ok", combined)
        self.assertIn("42.5", combined)

    @patch("run.send_generate_request")
    def test_main_exits_1_on_connection_error(self, mock_send):
        from urllib.error import URLError

        mock_send.side_effect = URLError("Connection refused")

        testargs = ["run.py", "--api-key", "test-key"]
        with patch.object(sys, "argv", testargs):
            with self.assertRaises(SystemExit) as ctx:
                main()

        self.assertEqual(ctx.exception.code, 1)

    @patch("run.send_generate_request")
    def test_main_exits_1_on_auth_error(self, mock_send):
        from urllib.error import HTTPError

        mock_send.side_effect = HTTPError(
            "url", 401, "Unauthorized", {}, io.BytesIO(b'{"error": "unauthorized"}'),
        )

        testargs = ["run.py", "--api-key", "bad-key"]
        with patch.object(sys, "argv", testargs):
            with self.assertRaises(SystemExit) as ctx:
                main()

        self.assertEqual(ctx.exception.code, 1)

    @patch("run.send_generate_request")
    def test_main_exits_1_on_server_error(self, mock_send):
        from urllib.error import HTTPError

        mock_send.side_effect = HTTPError(
            "url", 500, "Server Error", {}, io.BytesIO(b'internal error'),
        )

        testargs = ["run.py", "--api-key", "test-key"]
        with patch.object(sys, "argv", testargs):
            with self.assertRaises(SystemExit) as ctx:
                main()

        self.assertEqual(ctx.exception.code, 1)

    @patch("run.send_generate_request")
    @patch("builtins.print")
    def test_main_prints_elapsed_in_ms(self, mock_print, mock_send):
        mock_send.return_value = (200, '{"ok": true}', 1234.56)

        testargs = ["run.py", "--api-key", "test-key"]
        with patch.object(sys, "argv", testargs):
            main()

        printed_texts = [call[0][0] for call in mock_print.call_args_list]
        combined = " ".join(printed_texts)
        self.assertIn("1234.56", combined)


if __name__ == "__main__":
    unittest.main()
