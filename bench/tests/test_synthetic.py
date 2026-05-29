"""Tests for bench/synthetic.py deterministic synthetic data generator."""

import os
import sys
import unittest

sys.path.insert(0, os.path.join(os.path.dirname(__file__), ".."))

from synthetic import (
    _scenario_seed,
    _make_rng,
    _pick_attribute_type,
    _pick_distribution,
    VALUE_TYPES,
    DISTRIBUTIONS,
    generate_global_meta_attributes,
    generate_blueprint_body,
)


class TestScenarioSeed(unittest.TestCase):
    def test_same_scenario_same_seed(self):
        s1 = {"blueprint_count": 10, "affix_count": 2, "attribute_count": 3}
        s2 = {"blueprint_count": 10, "affix_count": 2, "attribute_count": 3}
        self.assertEqual(_scenario_seed(s1), _scenario_seed(s2))

    def test_different_scenario_different_seed(self):
        s1 = {"blueprint_count": 10, "affix_count": 2, "attribute_count": 3}
        s2 = {"blueprint_count": 100, "affix_count": 2, "attribute_count": 3}
        self.assertNotEqual(_scenario_seed(s1), _scenario_seed(s2))

    def test_seed_is_integer(self):
        s = {"blueprint_count": 10, "affix_count": 0, "attribute_count": 3}
        self.assertIsInstance(_scenario_seed(s), int)


class TestMakeRng(unittest.TestCase):
    def test_same_params_produces_same_rng_sequence(self):
        s = {"blueprint_count": 10, "affix_count": 0, "attribute_count": 3}
        r1 = _make_rng(s, 0, 0)
        r2 = _make_rng(s, 0, 0)
        self.assertEqual(r1.random(), r2.random())

    def test_different_blueprint_different_sequence(self):
        s = {"blueprint_count": 10, "affix_count": 0, "attribute_count": 3}
        r1 = _make_rng(s, 0, 0)
        r2 = _make_rng(s, 1, 0)
        self.assertNotEqual(r1.random(), r2.random())

    def test_different_attr_different_sequence(self):
        s = {"blueprint_count": 10, "affix_count": 0, "attribute_count": 3}
        r1 = _make_rng(s, 0, 0)
        r2 = _make_rng(s, 0, 1)
        self.assertNotEqual(r1.random(), r2.random())


class TestPickAttributeType(unittest.TestCase):
    def test_returns_valid_type(self):
        s = {"blueprint_count": 10, "affix_count": 0, "attribute_count": 100}
        rng = _make_rng(s, 0, 0)
        for _ in range(100):
            self.assertIn(_pick_attribute_type(rng), VALUE_TYPES)

    def test_all_types_are_possible(self):
        seen = set()
        for i in range(200):
            s = {"blueprint_count": 10, "affix_count": 0, "attribute_count": 100}
            rng = _make_rng(s, 0, i)
            seen.add(_pick_attribute_type(rng))
        self.assertEqual(seen, set(VALUE_TYPES))


class TestPickDistribution(unittest.TestCase):
    def test_returns_valid_distribution(self):
        s = {"blueprint_count": 10, "affix_count": 0, "attribute_count": 100}
        rng = _make_rng(s, 0, 0)
        for _ in range(100):
            self.assertIn(_pick_distribution(rng), DISTRIBUTIONS)

    def test_all_distributions_are_possible(self):
        seen = set()
        for i in range(200):
            s = {"blueprint_count": 10, "affix_count": 0, "attribute_count": 100}
            rng = _make_rng(s, 0, i)
            seen.add(_pick_distribution(rng))
        self.assertEqual(seen, set(DISTRIBUTIONS))


_DEFAULT_SCENARIO = {"blueprint_count": 10, "affix_count": 0, "attribute_count": 3}


class TestGenerateBlueprintBody(unittest.TestCase):
    def test_same_scenario_identical_output(self):
        body1 = generate_blueprint_body("Blueprint-0000", 3, _DEFAULT_SCENARIO, 0)
        body2 = generate_blueprint_body("Blueprint-0000", 3, _DEFAULT_SCENARIO, 0)
        self.assertEqual(body1, body2)

    def test_different_scenario_different_output(self):
        s1 = {"blueprint_count": 10, "affix_count": 0, "attribute_count": 3}
        s2 = {"blueprint_count": 100, "affix_count": 0, "attribute_count": 3}
        body1 = generate_blueprint_body("Blueprint-0000", 3, s1, 0)
        body2 = generate_blueprint_body("Blueprint-0000", 3, s2, 0)
        self.assertNotEqual(body1, body2)

    def test_blueprint_name_format(self):
        body = generate_blueprint_body("Blueprint-0000", 3, _DEFAULT_SCENARIO, 0)
        self.assertEqual(body["name"], "Blueprint-0000")

    def test_correct_attribute_count(self):
        for count in [1, 3, 10, 25]:
            body = generate_blueprint_body("test", count, _DEFAULT_SCENARIO, 0)
            self.assertEqual(len(body["attributes"]), count)
            self.assertEqual(len(body["attributeOrder"]), count)

    def test_has_required_structure(self):
        body = generate_blueprint_body("test", 3, _DEFAULT_SCENARIO, 0)
        self.assertIn("name", body)
        self.assertIn("archetype", body)
        self.assertIn("weight", body)
        self.assertIn("attributes", body)
        self.assertIn("attributeOrder", body)
        self.assertIn("affixes", body)
        self.assertEqual(body["archetype"], "item")
        self.assertEqual(body["weight"], 1.0)

    def test_all_attribute_types_appear_across_blueprints(self):
        seen_types = set()
        for bp_idx in range(10):
            body = generate_blueprint_body(f"Blueprint-{bp_idx:04d}", 10,
                                            _DEFAULT_SCENARIO, bp_idx)
            for attr in body["attributes"].values():
                if "$ref_id" in attr:
                    seen_types.add("$ref_id")
                else:
                    seen_types.add(attr["valueType"])
        for t in VALUE_TYPES:
            self.assertIn(t, seen_types)

    def test_all_distributions_appear_among_range_attributes(self):
        seen_dists = set()
        for bp_idx in range(20):
            body = generate_blueprint_body(f"Blueprint-{bp_idx:04d}", 10,
                                            _DEFAULT_SCENARIO, bp_idx)
            for attr in body["attributes"].values():
                if attr.get("valueType") == "range":
                    seen_dists.add(attr["distribution"]["type"])
        for d in DISTRIBUTIONS:
            self.assertIn(d, seen_dists)

    def test_some_attributes_use_global_ref(self):
        gma_ids = ["aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa",
                    "bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb"]
        total = 0
        ref_count = 0
        for bp_idx in range(10):
            body = generate_blueprint_body(f"Blueprint-{bp_idx:04d}", 10,
                                            _DEFAULT_SCENARIO, bp_idx,
                                            global_meta_ids=gma_ids)
            for attr in body["attributes"].values():
                total += 1
                if "$ref_id" in attr:
                    ref_count += 1
        self.assertGreater(ref_count, 0)
        self.assertLess(ref_count, total)

    def test_attribute_names_in_order(self):
        body = generate_blueprint_body("test", 5, _DEFAULT_SCENARIO, 0)
        self.assertEqual(body["attributeOrder"], ["attr_0", "attr_1", "attr_2", "attr_3", "attr_4"])

    def test_different_blueprint_idx_different_body(self):
        body1 = generate_blueprint_body("test", 3, _DEFAULT_SCENARIO, 0)
        body2 = generate_blueprint_body("test", 3, _DEFAULT_SCENARIO, 1)
        self.assertNotEqual(body1, body2)


class TestGenerateGlobalMetaAttributes(unittest.TestCase):
    def test_returns_list(self):
        attrs = generate_global_meta_attributes(_DEFAULT_SCENARIO)
        self.assertIsInstance(attrs, list)

    def test_each_has_name(self):
        attrs = generate_global_meta_attributes(_DEFAULT_SCENARIO)
        for attr in attrs:
            self.assertIn("name", attr)
            self.assertTrue(attr["name"].startswith("bench_global_meta_"))
            self.assertIn("valueType", attr)
            self.assertIn("description", attr)

    def test_deterministic(self):
        a1 = generate_global_meta_attributes(_DEFAULT_SCENARIO)
        a2 = generate_global_meta_attributes(_DEFAULT_SCENARIO)
        self.assertEqual(a1, a2)

    def test_different_scenario_different(self):
        s1 = {"blueprint_count": 10, "affix_count": 0, "attribute_count": 3}
        s2 = {"blueprint_count": 100, "affix_count": 0, "attribute_count": 3}
        a1 = generate_global_meta_attributes(s1)
        a2 = generate_global_meta_attributes(s2)
        self.assertNotEqual(a1, a2)

    def test_count_scales_with_attribute_count(self):
        small = generate_global_meta_attributes({"blueprint_count": 10, "affix_count": 0, "attribute_count": 3})
        large = generate_global_meta_attributes({"blueprint_count": 10, "affix_count": 0, "attribute_count": 25})
        self.assertLessEqual(len(small), len(large))
        self.assertGreater(len(large), 0)

    def test_ref_ids_match_global_meta_ids(self):
        gma_ids = ["aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa",
                    "bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb",
                    "cccccccc-cccc-cccc-cccc-cccccccccccc"]
        body = generate_blueprint_body("test", 10, _DEFAULT_SCENARIO, 0,
                                        global_meta_ids=gma_ids)
        for attr in body["attributes"].values():
            if "$ref_id" in attr:
                self.assertIn(attr["$ref_id"], gma_ids)


if __name__ == "__main__":
    unittest.main()
