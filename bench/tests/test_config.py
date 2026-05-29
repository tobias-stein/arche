"""Tests for bench/run.py config loading."""

import os
import sys
import tempfile
import unittest

sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

from run import load_config, _cross_product


def _write_yaml(content):
    f = tempfile.NamedTemporaryFile(mode="w", suffix=".yaml", delete=False)
    f.write(content)
    f.close()
    return f.name


class TestConfigLoader(unittest.TestCase):
    def test_3x3x3_produces_27_scenarios(self):
        path = _write_yaml("""dimensions:
  blueprints: [10, 100, 1000]
  affixes: [0, 2, 4]
  attributes: [3, 10, 25]
""")
        try:
            scenarios = load_config(path)
            self.assertEqual(len(scenarios), 27)
        finally:
            os.unlink(path)

    def test_2x2x2_produces_8_scenarios(self):
        path = _write_yaml("""dimensions:
  blueprints: [10, 100]
  affixes: [0, 2]
  attributes: [3, 10]
""")
        try:
            scenarios = load_config(path)
            self.assertEqual(len(scenarios), 8)
        finally:
            os.unlink(path)

    def test_1x1x1_produces_1_scenario(self):
        path = _write_yaml("""dimensions:
  blueprints: [10]
  affixes: [0]
  attributes: [3]
""")
        try:
            scenarios = load_config(path)
            self.assertEqual(len(scenarios), 1)
        finally:
            os.unlink(path)

    def test_scenario_keys_use_count_suffix(self):
        path = _write_yaml("""dimensions:
  blueprints: [10]
  affixes: [0]
  attributes: [3]
""")
        try:
            scenarios = load_config(path)
            s = scenarios[0]
            self.assertEqual(
                set(s.keys()), {"blueprint_count", "affix_count", "attribute_count"}
            )
        finally:
            os.unlink(path)

    def test_scenario_values_match_input(self):
        path = _write_yaml("""dimensions:
  blueprints: [10]
  affixes: [0]
  attributes: [3]
""")
        try:
            scenarios = load_config(path)
            s = scenarios[0]
            self.assertEqual(s["blueprint_count"], 10)
            self.assertEqual(s["affix_count"], 0)
            self.assertEqual(s["attribute_count"], 3)
        finally:
            os.unlink(path)

    def test_cross_product_order(self):
        d = {"a": [1, 2], "b": [3, 4]}
        result = _cross_product(d)
        self.assertEqual(len(result), 4)
        self.assertEqual(result, [
            {"a_count": 1, "b_count": 3},
            {"a_count": 1, "b_count": 4},
            {"a_count": 2, "b_count": 3},
            {"a_count": 2, "b_count": 4},
        ])

    def test_cross_product_single_dim(self):
        d = {"x": [42]}
        result = _cross_product(d)
        self.assertEqual(len(result), 1)
        self.assertEqual(result[0], {"x_count": 42})

    def test_missing_file_raises_error(self):
        with self.assertRaises(ValueError) as ctx:
            load_config("/tmp/nonexistent_bench_config.yaml")
        self.assertIn("Config file not found", str(ctx.exception))

    def test_missing_dimensions_key(self):
        path = _write_yaml("""not_dimensions:
  blueprints: [10]
""")
        try:
            with self.assertRaises(ValueError) as ctx:
                load_config(path)
            self.assertIn("Missing 'dimensions:'", str(ctx.exception))
        finally:
            os.unlink(path)

    def test_empty_dimensions(self):
        path = _write_yaml("""dimensions:
""")
        try:
            with self.assertRaises(ValueError) as ctx:
                load_config(path)
            self.assertIn("No dimension entries", str(ctx.exception))
        finally:
            os.unlink(path)

    def test_empty_list(self):
        path = _write_yaml("""dimensions:
  blueprints: []
""")
        try:
            with self.assertRaises(ValueError) as ctx:
                load_config(path)
            self.assertIn("empty list", str(ctx.exception))
        finally:
            os.unlink(path)

    def test_malformed_entry(self):
        path = _write_yaml("""dimensions:
  blueprints: [10, 100]
  affixes invalid
""")
        try:
            with self.assertRaises(ValueError) as ctx:
                load_config(path)
            self.assertIn("expected indented dimension entry", str(ctx.exception))
        finally:
            os.unlink(path)

    def test_comments_are_ignored(self):
        path = _write_yaml("""# This is a comment
dimensions:
  blueprints: [10]  # inline comment
  affixes: [0]
  attributes: [3]
""")
        try:
            scenarios = load_config(path)
            self.assertEqual(len(scenarios), 1)
        finally:
            os.unlink(path)

    def test_two_dimensions_only(self):
        path = _write_yaml("""dimensions:
  blueprints: [10, 100]
  affixes: [0, 2]
""")
        try:
            scenarios = load_config(path)
            self.assertEqual(len(scenarios), 4)
            self.assertEqual(set(scenarios[0].keys()), {"blueprint_count", "affix_count"})
        finally:
            os.unlink(path)


if __name__ == "__main__":
    unittest.main()
