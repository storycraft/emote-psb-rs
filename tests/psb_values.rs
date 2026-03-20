use std::collections::HashMap;
use std::io::Cursor;

use emote_psb::{
    psb::{read::PsbFile, table::StringTable, write::PsbWriter},
    value::{
        PsbValue,
        de::Deserializer,
        number::PsbNumber,
        ser::{Buffer, serialize},
    },
};
use serde::Deserialize;
use smol_str::SmolStr;

/// Performs a full PSB file round-trip: serialize with `PsbWriter`, then
/// deserialize with `PsbFile`.
fn psb_roundtrip(value: &PsbValue) -> PsbValue {
    let mut buf = Cursor::new(Vec::new());
    let writer = PsbWriter::new(2, false, value, &mut buf).unwrap();
    writer.finish().unwrap();
    buf.set_position(0);
    let mut psb = PsbFile::open(buf).unwrap();
    psb.deserialize_root::<PsbValue>().unwrap()
}

/// Performs a serde-level round-trip: serialize with `value::ser::serialize`,
/// then deserialize with `value::de::Deserializer` — without writing a full
/// PSB file header.
fn serde_roundtrip(value: &PsbValue) -> PsbValue {
    let mut buf = Buffer::new();
    serialize(value, &mut buf).unwrap();

    let mut names_table = StringTable::new();
    for name in buf.names() {
        names_table.push_str(name);
    }
    let mut strings_table = StringTable::new();
    for s in buf.strings() {
        strings_table.push_str(s);
    }

    let mut bytes = Vec::new();
    buf.write(&mut bytes).unwrap();

    let cursor = Cursor::new(bytes);
    let mut de = Deserializer::new(&names_table, &strings_table, cursor);
    PsbValue::deserialize(&mut de).unwrap()
}

// ---------------------------------------------------------------------------
// PSB file serialization / deserialization round-trip tests
// ---------------------------------------------------------------------------

#[test]
fn psb_null_roundtrip() {
    assert_eq!(psb_roundtrip(&PsbValue::Null), PsbValue::Null);
}

#[test]
fn psb_bool_true_roundtrip() {
    assert_eq!(psb_roundtrip(&PsbValue::Bool(true)), PsbValue::Bool(true));
}

#[test]
fn psb_bool_false_roundtrip() {
    assert_eq!(psb_roundtrip(&PsbValue::Bool(false)), PsbValue::Bool(false));
}

#[test]
fn psb_integer_roundtrip() {
    for v in [
        0i64,
        1,
        -1,
        127,
        -128,
        255,
        -12322,
        i32::MAX as i64,
        i32::MIN as i64,
    ] {
        let val = PsbValue::Number(PsbNumber::Integer(v));
        assert_eq!(psb_roundtrip(&val), val, "failed for integer {v}");
    }
}

#[test]
fn psb_float_zero_roundtrip() {
    let val = PsbValue::Number(PsbNumber::Float(0.0f32));
    assert_eq!(psb_roundtrip(&val), val);
}

#[test]
fn psb_float_nonzero_roundtrip() {
    for v in [1.0f32, -1.0, 122.0, f32::MIN, f32::MAX] {
        let val = PsbValue::Number(PsbNumber::Float(v));
        assert_eq!(psb_roundtrip(&val), val, "failed for float {v}");
    }
}

#[test]
fn psb_double_roundtrip() {
    for v in [0.0f64, 1.0, -1.0, 122.0, f64::MIN, f64::MAX] {
        let val = PsbValue::Number(PsbNumber::Double(v));
        assert_eq!(psb_roundtrip(&val), val, "failed for double {v}");
    }
}

#[test]
fn psb_string_roundtrip() {
    let val = PsbValue::String("hello, world".into());
    assert_eq!(psb_roundtrip(&val), val);
}

#[test]
fn psb_resource_roundtrip() {
    let val = PsbValue::Resource(42);
    assert_eq!(psb_roundtrip(&val), val);
}

#[test]
fn psb_extra_resource_roundtrip() {
    let val = PsbValue::ExtraResource(7);
    assert_eq!(psb_roundtrip(&val), val);
}

#[test]
fn psb_list_roundtrip() {
    let val = PsbValue::List(vec![
        PsbValue::Number(PsbNumber::Integer(12)),
        PsbValue::Number(PsbNumber::Integer(157)),
        PsbValue::Bool(true),
        PsbValue::Null,
    ]);
    assert_eq!(psb_roundtrip(&val), val);
}

#[test]
fn psb_nested_list_roundtrip() {
    let val = PsbValue::List(vec![
        PsbValue::List(vec![
            PsbValue::Number(PsbNumber::Integer(1)),
            PsbValue::Number(PsbNumber::Integer(2)),
        ]),
        PsbValue::List(vec![PsbValue::Bool(false)]),
    ]);
    assert_eq!(psb_roundtrip(&val), val);
}

#[test]
fn psb_object_roundtrip() {
    let mut map = HashMap::new();
    map.insert(
        SmolStr::new("alpha"),
        PsbValue::Number(PsbNumber::Integer(1)),
    );
    map.insert(SmolStr::new("beta"), PsbValue::Bool(false));
    map.insert(SmolStr::new("gamma"), PsbValue::Null);
    let val = PsbValue::Object(map);
    assert_eq!(psb_roundtrip(&val), val);
}

#[test]
fn psb_nested_object_roundtrip() {
    let mut inner = HashMap::new();
    inner.insert(SmolStr::new("x"), PsbValue::Number(PsbNumber::Integer(10)));
    inner.insert(SmolStr::new("y"), PsbValue::Number(PsbNumber::Integer(20)));

    let mut outer = HashMap::new();
    outer.insert(SmolStr::new("nested"), PsbValue::Object(inner));
    outer.insert(
        SmolStr::new("count"),
        PsbValue::Number(PsbNumber::Integer(2)),
    );

    let val = PsbValue::Object(outer);
    assert_eq!(psb_roundtrip(&val), val);
}

#[test]
fn psb_compiler_number_roundtrip() {
    let val = PsbValue::CompilerNumber;
    assert_eq!(psb_roundtrip(&val), val);
}

#[test]
fn psb_compiler_string_roundtrip() {
    let val = PsbValue::CompilerString;
    assert_eq!(psb_roundtrip(&val), val);
}

#[test]
fn psb_compiler_resource_roundtrip() {
    let val = PsbValue::CompilerResource;
    assert_eq!(psb_roundtrip(&val), val);
}

#[test]
fn psb_compiler_decimal_roundtrip() {
    let val = PsbValue::CompilerDecimal;
    assert_eq!(psb_roundtrip(&val), val);
}

#[test]
fn psb_compiler_array_roundtrip() {
    let val = PsbValue::CompilerArray;
    assert_eq!(psb_roundtrip(&val), val);
}

#[test]
fn psb_compiler_bool_roundtrip() {
    let val = PsbValue::CompilerBool;
    assert_eq!(psb_roundtrip(&val), val);
}

#[test]
fn psb_compiler_binary_tree_roundtrip() {
    let val = PsbValue::CompilerBinaryTree;
    assert_eq!(psb_roundtrip(&val), val);
}

// ---------------------------------------------------------------------------
// Serde serializer / deserializer tests for each PsbValue variant
// ---------------------------------------------------------------------------

#[test]
fn serde_null() {
    assert_eq!(serde_roundtrip(&PsbValue::Null), PsbValue::Null);
}

#[test]
fn serde_bool_true() {
    assert_eq!(serde_roundtrip(&PsbValue::Bool(true)), PsbValue::Bool(true));
}

#[test]
fn serde_bool_false() {
    assert_eq!(
        serde_roundtrip(&PsbValue::Bool(false)),
        PsbValue::Bool(false)
    );
}

#[test]
fn serde_integer_zero() {
    let val = PsbValue::Number(PsbNumber::Integer(0));
    assert_eq!(serde_roundtrip(&val), val);
}

#[test]
fn serde_integer_positive() {
    let val = PsbValue::Number(PsbNumber::Integer(12345));
    assert_eq!(serde_roundtrip(&val), val);
}

#[test]
fn serde_integer_negative() {
    let val = PsbValue::Number(PsbNumber::Integer(-12322));
    assert_eq!(serde_roundtrip(&val), val);
}

#[test]
fn serde_integer_large() {
    let val = PsbValue::Number(PsbNumber::Integer(i32::MAX as i64));
    assert_eq!(serde_roundtrip(&val), val);
}

#[test]
fn serde_float_zero() {
    let val = PsbValue::Number(PsbNumber::Float(0.0f32));
    assert_eq!(serde_roundtrip(&val), val);
}

#[test]
fn serde_float_nonzero() {
    let val = PsbValue::Number(PsbNumber::Float(122.0f32));
    assert_eq!(serde_roundtrip(&val), val);
}

#[test]
fn serde_double() {
    let val = PsbValue::Number(PsbNumber::Double(122.0f64));
    assert_eq!(serde_roundtrip(&val), val);
}

#[test]
fn serde_string() {
    let val = PsbValue::String("test string".into());
    assert_eq!(serde_roundtrip(&val), val);
}

#[test]
fn serde_resource() {
    let val = PsbValue::Resource(42);
    assert_eq!(serde_roundtrip(&val), val);
}

#[test]
fn serde_extra_resource() {
    let val = PsbValue::ExtraResource(7);
    assert_eq!(serde_roundtrip(&val), val);
}

#[test]
fn serde_list() {
    let val = PsbValue::List(vec![
        PsbValue::Number(PsbNumber::Integer(12)),
        PsbValue::Number(PsbNumber::Integer(157)),
    ]);
    assert_eq!(serde_roundtrip(&val), val);
}

#[test]
fn serde_object() {
    let mut map = HashMap::new();
    map.insert(
        SmolStr::new("key1"),
        PsbValue::Number(PsbNumber::Integer(1)),
    );
    map.insert(
        SmolStr::new("key2"),
        PsbValue::Number(PsbNumber::Integer(2)),
    );
    let val = PsbValue::Object(map);
    assert_eq!(serde_roundtrip(&val), val);
}

#[test]
fn serde_compiler_number() {
    let val = PsbValue::CompilerNumber;
    assert_eq!(serde_roundtrip(&val), val);
}

#[test]
fn serde_compiler_string() {
    let val = PsbValue::CompilerString;
    assert_eq!(serde_roundtrip(&val), val);
}

#[test]
fn serde_compiler_resource() {
    let val = PsbValue::CompilerResource;
    assert_eq!(serde_roundtrip(&val), val);
}

#[test]
fn serde_compiler_decimal() {
    let val = PsbValue::CompilerDecimal;
    assert_eq!(serde_roundtrip(&val), val);
}

#[test]
fn serde_compiler_array() {
    let val = PsbValue::CompilerArray;
    assert_eq!(serde_roundtrip(&val), val);
}

#[test]
fn serde_compiler_bool() {
    let val = PsbValue::CompilerBool;
    assert_eq!(serde_roundtrip(&val), val);
}

#[test]
fn serde_compiler_binary_tree() {
    let val = PsbValue::CompilerBinaryTree;
    assert_eq!(serde_roundtrip(&val), val);
}

// ---------------------------------------------------------------------------
// Object key sorting tests
// ---------------------------------------------------------------------------

/// After serialization, the names table must be sorted in alphabetical order.
#[test]
fn object_keys_sorted_in_names_table() {
    let mut map = HashMap::new();
    map.insert(SmolStr::new("zebra"), PsbValue::Null);
    map.insert(SmolStr::new("apple"), PsbValue::Null);
    map.insert(SmolStr::new("mango"), PsbValue::Null);
    map.insert(SmolStr::new("cherry"), PsbValue::Null);

    let val = PsbValue::Object(map);
    let mut buf = Buffer::new();
    serialize(&val, &mut buf).unwrap();

    let names: Vec<&str> = buf.names().iter().map(SmolStr::as_str).collect();
    let mut expected = names.clone();
    expected.sort_unstable();
    assert_eq!(names, expected, "object keys are not sorted alphabetically");
}

/// Nested object keys from all levels must all be sorted alphabetically.
#[test]
fn nested_object_keys_sorted_in_names_table() {
    let mut inner = HashMap::new();
    inner.insert(SmolStr::new("zeta"), PsbValue::Null);
    inner.insert(SmolStr::new("alpha"), PsbValue::Null);

    let mut outer = HashMap::new();
    outer.insert(SmolStr::new("outer_b"), PsbValue::Object(inner));
    outer.insert(SmolStr::new("outer_a"), PsbValue::Null);

    let val = PsbValue::Object(outer);
    let mut buf = Buffer::new();
    serialize(&val, &mut buf).unwrap();

    let names: Vec<&str> = buf.names().iter().map(SmolStr::as_str).collect();
    let mut expected = names.clone();
    expected.sort_unstable();
    assert_eq!(
        names, expected,
        "nested object keys are not sorted alphabetically"
    );
}

/// Keys inserted in reverse alphabetical order must still be recovered
/// correctly after a full PSB round-trip.
#[test]
fn object_keys_all_preserved_after_roundtrip() {
    let mut map = HashMap::new();
    map.insert(
        SmolStr::new("zebra"),
        PsbValue::Number(PsbNumber::Integer(3)),
    );
    map.insert(
        SmolStr::new("apple"),
        PsbValue::Number(PsbNumber::Integer(1)),
    );
    map.insert(
        SmolStr::new("mango"),
        PsbValue::Number(PsbNumber::Integer(2)),
    );

    let val = PsbValue::Object(map.clone());
    let result = psb_roundtrip(&val);

    let PsbValue::Object(result_map) = result else {
        panic!("expected Object variant after round-trip");
    };
    assert_eq!(result_map.len(), map.len());
    for (key, expected_value) in &map {
        assert_eq!(
            result_map.get(key),
            Some(expected_value),
            "key '{key}' missing or has wrong value after round-trip"
        );
    }
}

/// Ensure that a PSB object with many keys round-trips correctly and that the
/// serialized names table is still alphabetically sorted.
#[test]
fn object_many_keys_sorted_and_preserved() {
    let keys = ["omega", "beta", "delta", "alpha", "gamma", "epsilon"];
    let mut map = HashMap::new();
    for (i, k) in keys.iter().enumerate() {
        map.insert(
            SmolStr::new(*k),
            PsbValue::Number(PsbNumber::Integer(i as i64)),
        );
    }

    let val = PsbValue::Object(map.clone());

    // Check names table is sorted after serialization
    let mut buf = Buffer::new();
    serialize(&val, &mut buf).unwrap();
    let names: Vec<&str> = buf.names().iter().map(SmolStr::as_str).collect();
    let mut expected_sorted = names.clone();
    expected_sorted.sort_unstable();
    assert_eq!(names, expected_sorted, "names table is not sorted");

    // Check all keys and values survive the full round-trip
    let result = psb_roundtrip(&val);
    let PsbValue::Object(result_map) = result else {
        panic!("expected Object variant after round-trip");
    };
    for (key, expected_value) in &map {
        assert_eq!(
            result_map.get(key),
            Some(expected_value),
            "key '{key}' missing or wrong after round-trip"
        );
    }
}
